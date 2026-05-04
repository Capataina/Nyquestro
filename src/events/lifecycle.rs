//! `OrderEvent` — lifecycle transitions for a single order.

use crate::errors::{NyquestroError, NyquestroResult};
use crate::types::{OrderID, Px, Qty, Side, Symbol, Ts};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderRejectionReason {
    InvalidQuantity,
    InvalidPrice,
    InvalidOrderId,
    SelfMatch,
    DuplicateOrderId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderEvent {
    /// New order accepted by the book.
    Placed {
        order_id: OrderID,
        symbol: Symbol,
        side: Side,
        price: Px,
        quantity: Qty,
        timestamp: Ts,
    },
    /// Order was either matched (`remaining > 0` if partial, `0` if full)
    /// or fully consumed by a counterparty fill.
    Filled {
        order_id: OrderID,
        symbol: Symbol,
        executed: Qty,
        remaining: Qty,
        timestamp: Ts,
    },
    /// Order cancelled before fully filling.
    Cancelled {
        order_id: OrderID,
        symbol: Symbol,
        remaining: Qty,
        timestamp: Ts,
    },
    /// Order rejected before reaching the book.
    Rejected {
        order_id: OrderID,
        symbol: Symbol,
        reason: OrderRejectionReason,
        timestamp: Ts,
    },
}

impl OrderEvent {
    pub fn placed(
        order_id: OrderID,
        symbol: Symbol,
        side: Side,
        price: Px,
        quantity: Qty,
        timestamp: Ts,
    ) -> NyquestroResult<Self> {
        if quantity.is_zero() {
            return Err(NyquestroError::InvalidQuantity);
        }
        Ok(OrderEvent::Placed {
            order_id,
            symbol,
            side,
            price,
            quantity,
            timestamp,
        })
    }

    pub fn filled(
        order_id: OrderID,
        symbol: Symbol,
        executed: Qty,
        remaining: Qty,
        timestamp: Ts,
    ) -> NyquestroResult<Self> {
        if executed.is_zero() {
            return Err(NyquestroError::InvalidQuantity);
        }
        Ok(OrderEvent::Filled {
            order_id,
            symbol,
            executed,
            remaining,
            timestamp,
        })
    }

    pub fn cancelled(order_id: OrderID, symbol: Symbol, remaining: Qty, timestamp: Ts) -> Self {
        OrderEvent::Cancelled {
            order_id,
            symbol,
            remaining,
            timestamp,
        }
    }

    pub fn rejected(
        order_id: OrderID,
        symbol: Symbol,
        reason: OrderRejectionReason,
        timestamp: Ts,
    ) -> Self {
        OrderEvent::Rejected {
            order_id,
            symbol,
            reason,
            timestamp,
        }
    }

    pub fn order_id(&self) -> OrderID {
        match self {
            OrderEvent::Placed { order_id, .. }
            | OrderEvent::Filled { order_id, .. }
            | OrderEvent::Cancelled { order_id, .. }
            | OrderEvent::Rejected { order_id, .. } => *order_id,
        }
    }

    pub fn symbol(&self) -> Symbol {
        match self {
            OrderEvent::Placed { symbol, .. }
            | OrderEvent::Filled { symbol, .. }
            | OrderEvent::Cancelled { symbol, .. }
            | OrderEvent::Rejected { symbol, .. } => *symbol,
        }
    }

    pub fn timestamp(&self) -> Ts {
        match self {
            OrderEvent::Placed { timestamp, .. }
            | OrderEvent::Filled { timestamp, .. }
            | OrderEvent::Cancelled { timestamp, .. }
            | OrderEvent::Rejected { timestamp, .. } => *timestamp,
        }
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
    fn placed_rejects_zero_quantity() {
        let err = OrderEvent::placed(
            OrderID::new(1).unwrap(),
            SYM,
            Side::Buy,
            Px::from_cents(100).unwrap(),
            Qty::ZERO,
            ts(1),
        );
        assert!(matches!(err, Err(NyquestroError::InvalidQuantity)));
    }

    #[test]
    fn filled_rejects_zero_executed() {
        let err = OrderEvent::filled(OrderID::new(1).unwrap(), SYM, Qty::ZERO, Qty::new(5), ts(1));
        assert!(matches!(err, Err(NyquestroError::InvalidQuantity)));
    }

    #[test]
    fn cancelled_is_infallible() {
        let e = OrderEvent::cancelled(OrderID::new(1).unwrap(), SYM, Qty::new(3), ts(1));
        assert!(matches!(e, OrderEvent::Cancelled { .. }));
        assert_eq!(e.order_id().value(), 1);
        assert_eq!(e.symbol(), SYM);
    }

    #[test]
    fn rejected_carries_reason() {
        let e = OrderEvent::rejected(
            OrderID::new(1).unwrap(),
            SYM,
            OrderRejectionReason::SelfMatch,
            ts(1),
        );
        if let OrderEvent::Rejected { reason, .. } = e {
            assert_eq!(reason, OrderRejectionReason::SelfMatch);
        } else {
            panic!("expected Rejected variant");
        }
    }

    #[test]
    fn accessor_methods_work_across_variants() {
        let p = OrderEvent::placed(
            OrderID::new(1).unwrap(),
            SYM,
            Side::Buy,
            Px::from_cents(100).unwrap(),
            Qty::new(5),
            ts(10),
        )
        .unwrap();
        assert_eq!(p.order_id().value(), 1);
        assert_eq!(p.timestamp(), ts(10));
        assert_eq!(p.symbol(), SYM);
    }
}
