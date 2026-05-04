//! Dashboard launcher.
//!
//! ```text
//!   cargo run                              → real-time multi-instrument TUI (synthetic flow)
//!   cargo run -- --live coinbase           → live BTC-USD/ETH-USD/SOL-USD depth from Coinbase
//!   cargo run -- --no-tui                  → headless demo (text output, synthetic)
//!   cargo run -- --seed 1234               → deterministic dashboard from a seed (synthetic)
//! ```

use std::env;
use std::sync::mpsc;
use std::thread;

use nyquestro::book::Market;
use nyquestro::events::OrderEvent;
use nyquestro::feed::{run_coinbase, Bridge, CoinbaseConfig};
use nyquestro::simulator::{MarketSimulator, SimAction, SimConfig};
use nyquestro::telemetry::{spawn_writer, TelemetryEvent, TelemetryHandle};
use nyquestro::types::Symbol;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let no_tui = args.iter().any(|a| a == "--no-tui");
    let seed = parse_seed(&args).unwrap_or(0xC0FFEE);
    let live_venue = parse_live(&args);

    if let Some(venue) = live_venue.as_deref() {
        match venue {
            "coinbase" => return run_live_coinbase(),
            other => {
                eprintln!("unknown live venue: {other}; supported: coinbase");
                std::process::exit(2);
            }
        }
    }

    if no_tui {
        run_headless(seed)
    } else {
        nyquestro::ui::run(seed)
    }
}

fn parse_seed(args: &[String]) -> Option<u64> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--seed"
            && i + 1 < args.len()
            && let Ok(n) = args[i + 1].parse::<u64>()
        {
            return Some(n);
        }
        i += 1;
    }
    None
}

fn parse_live(args: &[String]) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--live" && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
        i += 1;
    }
    None
}

fn run_live_coinbase() -> Result<(), Box<dyn std::error::Error>> {
    let symbols = vec![
        (Symbol::from_const("BTC-USD"), 6_500_000u64),
        (Symbol::from_const("ETH-USD"), 300_000u64),
        (Symbol::from_const("SOL-USD"), 15_000u64),
    ];

    // Spawn the telemetry writer first so we can pass clones into both
    // the main App and the feed thread (so feed-side errors are
    // captured even before they reach the bridge).
    let telemetry = match spawn_writer() {
        Ok((handle, path)) => {
            eprintln!("telemetry → {}", path.display());
            handle
        }
        Err(e) => {
            eprintln!("telemetry disabled: {e}");
            TelemetryHandle::noop()
        }
    };

    // Channel from feed thread → main thread.
    let (action_tx, action_rx) = mpsc::channel();
    let symbols_for_bridge: Vec<Symbol> = symbols.iter().map(|(s, _)| *s).collect();
    let telemetry_for_feed = telemetry.clone();
    thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("failed to start tokio runtime: {e}");
                return;
            }
        };
        runtime.block_on(async move {
            let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(1024);
            let cfg = CoinbaseConfig::default();
            tokio::spawn(run_coinbase(cfg, event_tx));

            let mut bridge = Bridge::new(symbols_for_bridge);
            while let Some(event) = event_rx.recv().await {
                // Capture snapshots for the audit trail directly so the
                // raw_bids / raw_asks counts get recorded even if the
                // bridge truncates them downstream.
                if let nyquestro::feed::FeedEvent::Snapshot {
                    symbol,
                    bids,
                    asks,
                } = &event
                {
                    telemetry_for_feed.record(TelemetryEvent::Snapshot {
                        sym: symbol.to_string(),
                        raw_bids: bids.len(),
                        raw_asks: asks.len(),
                        capped: 50,
                    });
                }
                for feed_action in bridge.translate(event) {
                    if action_tx.send(feed_action).is_err() {
                        return;
                    }
                }
            }
        });
    });

    let app = nyquestro::ui::App::new_live(symbols, action_rx, telemetry);
    nyquestro::ui::run_with_app(app)
}

fn run_headless(seed: u64) -> Result<(), Box<dyn std::error::Error>> {
    let symbols: [(Symbol, u64); 3] = [
        (Symbol::from_const("AAPL"), 15_000),
        (Symbol::from_const("MSFT"), 30_000),
        (Symbol::from_const("NVDA"), 50_000),
    ];

    let mut market = Market::new();
    let mut sims: Vec<MarketSimulator> = symbols
        .iter()
        .enumerate()
        .map(|(i, (sym, fair))| {
            let cfg = SimConfig {
                symbol: *sym,
                fair_value_cents: *fair,
                ..SimConfig::default()
            };
            MarketSimulator::new(cfg, seed.wrapping_add((i as u64) * 0x100))
        })
        .collect();

    let mut totals = [(0u64, 0u64, 0u64); 3];
    println!("nyquestro headless mode · seed={seed}");
    println!("simulating 10 seconds across 3 symbols...");
    for _ in 0..200 {
        for (i, sim) in sims.iter_mut().enumerate() {
            for action in sim.step(0.05) {
                if let SimAction::Submit(o) = action
                    && let Ok(res) = market.submit_limit(o)
                {
                    totals[i].0 += 1;
                    totals[i].1 += res.fills.len() as u64;
                    totals[i].2 += res
                        .lifecycle
                        .iter()
                        .filter(|e| matches!(e, OrderEvent::Rejected { .. }))
                        .count() as u64;
                }
            }
        }
    }

    println!("\n── result ──────────────────────");
    for (i, (sym, _)) in symbols.iter().enumerate() {
        let book = market.book(*sym);
        let bid = book
            .and_then(|b| b.best_bid())
            .map(|(p, q)| format!("${:.2}×{}", p.to_dollars(), q.value()))
            .unwrap_or_else(|| "—".into());
        let ask = book
            .and_then(|b| b.best_ask())
            .map(|(p, q)| format!("${:.2}×{}", p.to_dollars(), q.value()))
            .unwrap_or_else(|| "—".into());
        let resting = book.map(|b| b.len()).unwrap_or(0);
        let mid = sims[i].mid_cents() as f64 / 100.0;
        println!(
            "{sym:<6} submitted {:>5}  filled {:>5}  rejected {}  resting {:>4}  bid {bid:<12} ask {ask:<12} mid ${mid:.2}",
            totals[i].0, totals[i].1, totals[i].2, resting,
        );
    }
    Ok(())
}
