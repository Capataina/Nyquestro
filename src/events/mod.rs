//! Immutable event frames emitted by the matching engine.
//!
//! Every event type is `Copy`, allocation-free, and validated at construction
//! so downstream consumers (replay, fan-out, observability) can rely on the
//! invariants without re-checking. Reasons for rejection live in
//! [`OrderRejectionReason`] enums rather than `String` fields so the frames
//! stay `Copy`.

pub mod fill;
pub mod lifecycle;
pub mod quote;

pub use fill::FillEvent;
pub use lifecycle::{OrderEvent, OrderRejectionReason};
pub use quote::{QuoteEvent, QuoteSide};
