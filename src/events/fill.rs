//! `FillEvent` — emitted when two orders match.

use crate::errors::{NyquestroError, NyquestroResult};
use crate::types::{OrderID, Px, Qty, Symbol, Ts};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FillEvent {
    pub symbol: Symbol,
    pub buyer_order_id: OrderID,
    pub seller_order_id: OrderID,
    pub price: Px,
    pub quantity: Qty,
    pub timestamp: Ts,
}

impl FillEvent {
    /// Construct a fill. Rejects:
    /// - zero quantity
    /// - self-match (`buyer_order_id == seller_order_id`)
    pub fn new(
        symbol: Symbol,
        buyer_order_id: OrderID,
        seller_order_id: OrderID,
        price: Px,
        quantity: Qty,
        timestamp: Ts,
    ) -> NyquestroResult<Self> {
        if quantity.is_zero() {
            return Err(NyquestroError::InvalidQuantity);
        }
        if buyer_order_id == seller_order_id {
            return Err(NyquestroError::SelfMatch(buyer_order_id.value()));
        }
        Ok(FillEvent {
            symbol,
            buyer_order_id,
            seller_order_id,
            price,
            quantity,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYM: Symbol = Symbol::from_const("TEST");

    fn ts(n: u64) -> Ts {
        Ts::from_nanos(n)
    }

    #[test]
    fn rejects_zero_quantity() {
        let err = FillEvent::new(
            SYM,
            OrderID::new(1).unwrap(),
            OrderID::new(2).unwrap(),
            Px::from_cents(100).unwrap(),
            Qty::ZERO,
            ts(1),
        );
        assert!(matches!(err, Err(NyquestroError::InvalidQuantity)));
    }

    #[test]
    fn rejects_self_match() {
        let same = OrderID::new(1).unwrap();
        let err = FillEvent::new(SYM, same, same, Px::from_cents(100).unwrap(), Qty::new(5), ts(1));
        assert!(matches!(err, Err(NyquestroError::SelfMatch(1))));
    }

    #[test]
    fn accepts_valid_fill() {
        let f = FillEvent::new(
            SYM,
            OrderID::new(1).unwrap(),
            OrderID::new(2).unwrap(),
            Px::from_cents(150).unwrap(),
            Qty::new(7),
            ts(42),
        )
        .unwrap();
        assert_eq!(f.quantity, Qty::new(7));
        assert_eq!(f.price.cents(), 150);
        assert_eq!(f.symbol, SYM);
    }
}
