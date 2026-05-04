//! Live market-data feed.
//!
//! [`coinbase`] connects to Coinbase Advanced Trade's public `level2`
//! WebSocket channel (no auth required) and emits parsed [`FeedEvent`]s.
//! [`bridge`] translates those events into the same `SimAction` stream
//! the synthetic simulator produces, so the dashboard's `App::dispatch`
//! handles both sources identically.
//!
//! The feed runs on its own tokio runtime in a separate OS thread; it
//! pushes `(symbol_idx, SimAction)` pairs through a `std::sync::mpsc`
//! channel into the main dashboard loop, which drains non-blockingly per
//! render frame.

pub mod bridge;
pub mod coinbase;

pub use bridge::{Bridge, FeedAction};
pub use coinbase::{run_coinbase, CoinbaseConfig, FeedEvent};

/// One Coinbase level-quantity unit corresponds to `1 / QTY_SCALE` of the
/// base asset. We use 1e6 so a Qty of 500_000 represents 0.5 BTC; this
/// keeps fractional crypto quantities in `u32` range while preserving
/// micro-unit precision (~$0.06 at $60k BTC).
pub const QTY_SCALE: f64 = 1_000_000.0;
