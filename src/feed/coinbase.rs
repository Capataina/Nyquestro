//! Coinbase Advanced Trade `level2` WebSocket client.
//!
//! Public channel — no authentication required. Connects to
//! `wss://advanced-trade-ws.coinbase.com`, subscribes to `level2` for the
//! configured products, and emits parsed [`FeedEvent`]s downstream.
//!
//! The client owns reconnection and exponential backoff; if the
//! connection drops, the loop sleeps `current_delay`, doubles it (capped
//! at 30s), and retries. A successful subscription resets the delay.
//!
//! Output channel is `tokio::sync::mpsc`. The bridge consumes events,
//! translates them to `SimAction`s, and forwards to the dashboard's
//! main-thread channel.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;

use crate::types::{Px, Qty, Side, Symbol};

const COINBASE_WS_URL: &str = "wss://advanced-trade-ws.coinbase.com";
const RECONNECT_BASE: Duration = Duration::from_millis(250);
const RECONNECT_MAX: Duration = Duration::from_secs(30);

/// Cap the number of levels per side delivered from a snapshot. Coinbase's
/// `level2` snapshot includes the entire visible book — for BTC-USD this
/// can be 25k+ levels per side. Forwarding all of them into our matching
/// engine pegs the main thread for tens of seconds and freezes the
/// dashboard. The dashboard only renders ~20 levels per side; capping at
/// 50 per side gives us headroom to absorb subsequent updates without
/// any visible truncation, while keeping the snapshot's bridge-action
/// count well within the per-frame dispatch budget.
const SNAPSHOT_LEVEL_CAP: usize = 50;

#[derive(Debug, Clone)]
pub struct CoinbaseConfig {
    /// Coinbase product ids to subscribe to, e.g. ["BTC-USD", "ETH-USD"].
    pub product_ids: Vec<String>,
}

impl Default for CoinbaseConfig {
    fn default() -> Self {
        CoinbaseConfig {
            product_ids: vec![
                "BTC-USD".to_string(),
                "ETH-USD".to_string(),
                "SOL-USD".to_string(),
            ],
        }
    }
}

/// Engine-shaped event produced by the WebSocket parser.
#[derive(Debug, Clone)]
pub enum FeedEvent {
    Snapshot {
        symbol: Symbol,
        bids: Vec<(Px, Qty)>,
        asks: Vec<(Px, Qty)>,
    },
    Update {
        symbol: Symbol,
        side: Side,
        price: Px,
        new_quantity: Qty,
    },
    Status(String),
}

// ─── Wire types ─────────────────────────────────────────────────────────────

/// Subscribe message sent on connect.
#[derive(Debug, Serialize)]
struct SubscribeMessage<'a> {
    #[serde(rename = "type")]
    msg_type: &'a str,
    product_ids: &'a [String],
    channel: &'a str,
}

/// Top-level message received from Coinbase. The `events` array carries
/// the per-product updates we care about; everything else (heartbeat,
/// subscriptions echo, etc.) is logged and ignored.
#[derive(Debug, Deserialize)]
struct ServerMessage {
    #[serde(default)]
    channel: String,
    #[serde(default)]
    events: Vec<EventEntry>,
}

#[derive(Debug, Deserialize)]
struct EventEntry {
    #[serde(rename = "type", default)]
    event_type: String,
    #[serde(default)]
    product_id: String,
    #[serde(default)]
    updates: Vec<UpdateEntry>,
}

#[derive(Debug, Deserialize)]
struct UpdateEntry {
    #[serde(default)]
    side: String,
    #[serde(default)]
    price_level: String,
    #[serde(default)]
    new_quantity: String,
}

// ─── Driver ─────────────────────────────────────────────────────────────────

/// Run the Coinbase WebSocket consumer to completion. The loop reconnects
/// indefinitely; the only way it exits is by the receiver dropping.
///
/// Status messages (connecting, subscribed, disconnected, parse errors)
/// are emitted as `FeedEvent::Status` so the dashboard can surface them
/// in a banner.
pub async fn run_coinbase(cfg: CoinbaseConfig, tx: mpsc::Sender<FeedEvent>) {
    let mut delay = RECONNECT_BASE;
    loop {
        let _ = tx
            .send(FeedEvent::Status(format!("connecting to {COINBASE_WS_URL}")))
            .await;

        match connect_and_pump(&cfg, &tx).await {
            Ok(()) => {
                let _ = tx
                    .send(FeedEvent::Status(
                        "feed closed cleanly; reconnecting".to_string(),
                    ))
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(FeedEvent::Status(format!(
                        "feed error: {e}; reconnecting in {}ms",
                        delay.as_millis()
                    )))
                    .await;
            }
        }

        sleep(delay).await;
        delay = (delay * 2).min(RECONNECT_MAX);
    }
}

async fn connect_and_pump(
    cfg: &CoinbaseConfig,
    tx: &mpsc::Sender<FeedEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (ws_stream, _resp) = tokio_tungstenite::connect_async(COINBASE_WS_URL).await?;
    let (mut sink, mut stream) = ws_stream.split();

    // Send one subscribe message per channel we care about. Coinbase
    // requires separate subscribes for level2 and ticker channels.
    let subscribe_l2 = SubscribeMessage {
        msg_type: "subscribe",
        product_ids: &cfg.product_ids,
        channel: "level2",
    };
    sink.send(Message::text(serde_json::to_string(&subscribe_l2)?))
        .await?;

    let _ = tx
        .send(FeedEvent::Status(format!(
            "subscribed: {} on level2",
            cfg.product_ids.join(", ")
        )))
        .await;

    while let Some(msg) = stream.next().await {
        let msg = msg?;
        match msg {
            Message::Text(text) => {
                if let Err(e) = handle_text(text.as_str(), tx).await {
                    let _ = tx
                        .send(FeedEvent::Status(format!("parse error: {e}")))
                        .await;
                }
            }
            Message::Binary(_) => { /* ignore */ }
            Message::Ping(payload) => {
                sink.send(Message::Pong(payload)).await?;
            }
            Message::Pong(_) => {}
            Message::Close(_) => break,
            Message::Frame(_) => {}
        }
    }
    Ok(())
}

async fn handle_text(
    text: &str,
    tx: &mpsc::Sender<FeedEvent>,
) -> Result<(), serde_json::Error> {
    let msg: ServerMessage = serde_json::from_str(text)?;
    if msg.channel != "l2_data" {
        // subscriptions echo, heartbeats, etc. — ignore
        return Ok(());
    }
    for event in msg.events {
        let symbol = match symbol_from_product_id(&event.product_id) {
            Some(s) => s,
            None => continue,
        };
        match event.event_type.as_str() {
            "snapshot" => {
                let mut bids = Vec::new();
                let mut asks = Vec::new();
                for u in &event.updates {
                    let qty = match parse_qty(&u.new_quantity) {
                        Some(q) if !q.is_zero() => q,
                        _ => continue,
                    };
                    let price = match parse_price(&u.price_level) {
                        Some(p) => p,
                        None => continue,
                    };
                    match u.side.as_str() {
                        "bid" => bids.push((price, qty)),
                        "offer" | "ask" => asks.push((price, qty)),
                        _ => {}
                    }
                }
                // Sort closest-to-touch first and cap so the bridge
                // doesn't dump tens of thousands of orders into the
                // matching engine on first connect.
                bids.sort_by_key(|(p, _)| std::cmp::Reverse(*p));
                asks.sort_by_key(|(p, _)| *p);
                bids.truncate(SNAPSHOT_LEVEL_CAP);
                asks.truncate(SNAPSHOT_LEVEL_CAP);
                let _ = tx
                    .send(FeedEvent::Status(format!(
                        "snapshot {symbol}: {} bids / {} asks (capped)",
                        bids.len(),
                        asks.len()
                    )))
                    .await;
                let _ = tx
                    .send(FeedEvent::Snapshot {
                        symbol,
                        bids,
                        asks,
                    })
                    .await;
            }
            "update" => {
                for u in &event.updates {
                    let side = match u.side.as_str() {
                        "bid" => Side::Buy,
                        "offer" | "ask" => Side::Sell,
                        _ => continue,
                    };
                    let price = match parse_price(&u.price_level) {
                        Some(p) => p,
                        None => continue,
                    };
                    // Quantity may be zero (level cleared). Use Qty::ZERO.
                    let qty = parse_qty(&u.new_quantity).unwrap_or(Qty::ZERO);
                    let _ = tx
                        .send(FeedEvent::Update {
                            symbol,
                            side,
                            price,
                            new_quantity: qty,
                        })
                        .await;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Map a Coinbase product id (e.g. "BTC-USD") to our 8-byte `Symbol`.
/// Returns `None` for product ids whose name overflows 8 bytes — we
/// fall back to the prefix in that case so the symbol still renders.
fn symbol_from_product_id(product_id: &str) -> Option<Symbol> {
    if product_id.is_empty() {
        return None;
    }
    let bytes = product_id.as_bytes();
    let len = bytes.len().min(8);
    let mut buf = [0u8; 8];
    buf[..len].copy_from_slice(&bytes[..len]);
    Some(Symbol::from_const_bytes(buf))
}

/// Parse Coinbase's price string (e.g. "67234.56") into our integer-cents
/// `Px`. Returns `None` for malformed input or zero/negative prices.
fn parse_price(s: &str) -> Option<Px> {
    let dollars: f64 = s.parse().ok()?;
    if !dollars.is_finite() || dollars <= 0.0 {
        return None;
    }
    let cents = (dollars * 100.0).round() as u64;
    Px::from_cents(cents.max(1)).ok()
}

/// Parse Coinbase's quantity string (e.g. "0.5") into our scaled-integer
/// `Qty`. Multiplies by [`QTY_SCALE`] so 0.5 BTC becomes Qty(500_000).
/// Returns `Qty::ZERO` for valid zero (level cleared) and `None` for
/// malformed input.
fn parse_qty(s: &str) -> Option<Qty> {
    let units: f64 = s.parse().ok()?;
    if !units.is_finite() || units < 0.0 {
        return None;
    }
    let scaled = (units * super::QTY_SCALE).round();
    if scaled > u32::MAX as f64 {
        return Some(Qty::new(u32::MAX));
    }
    Some(Qty::new(scaled.max(0.0) as u32))
}
