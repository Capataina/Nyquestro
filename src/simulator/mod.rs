//! Synthetic market data generator.
//!
//! Produces a realistic stream of orders the matching engine can chew on
//! while the dashboard observes. The flow shape follows the
//! Cont–Stoikov–Talreja stochastic LOB model:
//!
//! - Order arrivals are independent Poisson per side.
//! - Order sizes are log-normal, clipped to [1, 500].
//! - Order prices are drawn relative to a mean-reverting Ornstein–Uhlenbeck
//!   walk on the theoretical fair value, with intensity decreasing as
//!   `1 / (1 + ticks_from_top)^α`.
//! - Cancellations target a uniformly-chosen resting order.
//! - Aggressive vs passive split is ~20/80 by default.
//!
//! All randomness flows through a single `ChaCha8Rng`, so a fixed seed
//! produces a byte-identical event stream.

pub mod market;

pub use market::{MarketSimulator, SimAction, SimConfig};
