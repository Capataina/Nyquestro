//! `QuoteEvent` — emitted when top-of-book changes on either side.

use crate::errors::{NyquestroError, NyquestroResult};
use crate::types::{Px, Qty, Side, Symbol, Ts};

/// Which side of the book is being quoted, and whether the level was
/// strengthened, weakened, or fully cleared. Lets downstream consumers
/// disambiguate "best bid moved" from "size at best bid changed" without
/// keeping their own state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuoteSide {
    Bid,
    Ask,
}

impl From<Side> for QuoteSide {
    fn from(s: Side) -> Self {
        match s {
            Side::Buy => QuoteSide::Bid,
            Side::Sell => QuoteSide::Ask,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuoteEvent {
    pub symbol: Symbol,
    pub side: QuoteSide,
    pub price: Px,
    /// Aggregate displayed quantity at this price. May be zero when the
    /// level has been fully cleared.
    pub quantity: Qty,
    pub timestamp: Ts,
}

impl QuoteEvent {
    /// Quote with a non-zero displayed quantity (a level was set or
    /// adjusted but is still live).
    pub fn live(
        symbol: Symbol,
        side: QuoteSide,
        price: Px,
        quantity: Qty,
        timestamp: Ts,
    ) -> NyquestroResult<Self> {
        if quantity.is_zero() {
            return Err(NyquestroError::InvalidQuantity);
        }
        Ok(QuoteEvent {
            symbol,
            side,
            price,
            quantity,
            timestamp,
        })
    }

    /// Quote signalling that a level has been fully cleared (no resting
    /// quantity remaining at this price). Distinct from `live` because zero
    /// quantity is a meaningful signal here, not a validation failure.
    pub fn cleared(symbol: Symbol, side: QuoteSide, price: Px, timestamp: Ts) -> Self {
        QuoteEvent {
            symbol,
            side,
            price,
            quantity: Qty::ZERO,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYM: Symbol = Symbol::from_const("TEST");

    #[test]
    fn live_rejects_zero_quantity() {
        let err = QuoteEvent::live(
            SYM,
            QuoteSide::Bid,
            Px::from_cents(100).unwrap(),
            Qty::ZERO,
            Ts::from_nanos(1),
        );
        assert!(matches!(err, Err(NyquestroError::InvalidQuantity)));
    }

    #[test]
    fn cleared_carries_zero_quantity() {
        let q = QuoteEvent::cleared(SYM, QuoteSide::Ask, Px::from_cents(200).unwrap(), Ts::from_nanos(1));
        assert!(q.quantity.is_zero());
        assert_eq!(q.side, QuoteSide::Ask);
        assert_eq!(q.symbol, SYM);
    }

    #[test]
    fn side_conversion() {
        assert_eq!(QuoteSide::from(Side::Buy), QuoteSide::Bid);
        assert_eq!(QuoteSide::from(Side::Sell), QuoteSide::Ask);
    }
}
