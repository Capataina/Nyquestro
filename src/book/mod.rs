//! Order book and its building blocks.
//!
//! - [`PriceLevel`] — FIFO queue of resting orders at a single price.
//! - [`OrderBook`] — single-symbol bid/ask book with deterministic
//!   price-time matching.
//! - [`Market`] — multi-symbol wrapper holding one [`OrderBook`] per
//!   [`crate::types::Symbol`].

pub mod market;
pub mod order_book;
pub mod price_level;

pub use market::Market;
pub use order_book::{OrderBook, SubmitResult};
pub use price_level::PriceLevel;
