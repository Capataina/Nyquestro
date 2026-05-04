//! Smoke test for the Coinbase live feed without a TUI.
//!
//! Connects to Coinbase, prints the first ~30 events received, then
//! exits. Useful to confirm:
//! - the WebSocket connection establishes,
//! - the subscribe message is accepted,
//! - parsed `FeedEvent`s flow through, with sensible prices/quantities.
//!
//! Run with: `cargo run --release --example live_smoke`

use std::time::Duration;

use nyquestro::feed::{run_coinbase, Bridge, CoinbaseConfig, FeedEvent};
use nyquestro::simulator::SimAction;
use nyquestro::types::Symbol;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let cfg = CoinbaseConfig::default();
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<FeedEvent>(1024);

    println!("connecting to Coinbase Advanced Trade WebSocket...");
    println!("subscribing to: {}", cfg.product_ids.join(", "));
    println!();

    tokio::spawn(run_coinbase(cfg, event_tx));

    let mut bridge = Bridge::new(vec![
        Symbol::from_const("BTC-USD"),
        Symbol::from_const("ETH-USD"),
        Symbol::from_const("SOL-USD"),
    ]);

    let mut count = 0;
    let mut snapshot_count = 0;
    let mut update_count = 0;
    let mut submit_actions = 0u64;
    let mut cancel_actions = 0u64;

    loop {
        let event = match tokio::time::timeout(Duration::from_secs(15), event_rx.recv()).await
        {
            Ok(Some(e)) => e,
            Ok(None) => {
                println!("\nfeed channel closed");
                break;
            }
            Err(_) => {
                println!("\ntimeout waiting for events; exiting");
                break;
            }
        };

        match &event {
            FeedEvent::Status(s) => {
                println!("[status] {s}");
            }
            FeedEvent::Snapshot { symbol, bids, asks } => {
                snapshot_count += 1;
                println!(
                    "[snapshot] {symbol} · {} bids · {} asks · best bid {} · best ask {}",
                    bids.len(),
                    asks.len(),
                    bids.first()
                        .map(|(p, q)| format!("${:.2}×{}", p.to_dollars(), q.value()))
                        .unwrap_or_else(|| "—".into()),
                    asks.first()
                        .map(|(p, q)| format!("${:.2}×{}", p.to_dollars(), q.value()))
                        .unwrap_or_else(|| "—".into()),
                );
            }
            FeedEvent::Update {
                symbol,
                side,
                price,
                new_quantity,
            } => {
                update_count += 1;
                if update_count <= 10 {
                    println!(
                        "[update]   {symbol} {} @${:.2} → qty {}",
                        side,
                        price.to_dollars(),
                        new_quantity.value(),
                    );
                }
            }
        }

        for feed_action in bridge.translate(event) {
            match feed_action {
                nyquestro::feed::FeedAction::Action { action, .. } => match action {
                    SimAction::Submit(_) => submit_actions += 1,
                    SimAction::Cancel { .. } => cancel_actions += 1,
                    SimAction::CancelHint => {}
                },
                nyquestro::feed::FeedAction::Status(_) => {}
            }
        }

        count += 1;
        if count >= 60 {
            break;
        }
    }

    println!();
    println!("── summary ─────────────────────────");
    println!("events received    {count}");
    println!("snapshots          {snapshot_count}");
    println!("updates (first 10 shown above; total {update_count})");
    println!("bridge submits     {submit_actions}");
    println!("bridge cancels     {cancel_actions}");
    println!();
    if submit_actions > 0 {
        println!("✅ live feed and bridge are working");
    } else {
        println!("⚠ no bridge actions produced — feed may not have reached snapshot/update phase");
    }
}
