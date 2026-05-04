//! Order entity.
//!
//! `Order` owns its own state machine. Fills and cancellations are checked
//! mutations: invalid inputs return classified errors and leave the order
//! unchanged. Status transitions are one-way; observing state never moves
//! the order.

use std::fmt;

use crate::errors::{NyquestroError, NyquestroResult};
use crate::types::{OrderID, Px, Qty, Side, Status, Symbol, Ts};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Order {
    id: OrderID,
    symbol: Symbol,
    side: Side,
    price: Px,
    quantity: Qty,
    remaining: Qty,
    timestamp: Ts,
    status: Status,
}

impl Order {
    /// Construct an order with a caller-supplied timestamp. Determinism in
    /// the matching engine depends on `Order` not consulting the wall clock
    /// itself — any test or replay can pin time exactly.
    pub fn new(
        id: OrderID,
        symbol: Symbol,
        side: Side,
        price: Px,
        quantity: Qty,
        timestamp: Ts,
    ) -> NyquestroResult<Self> {
        if quantity.is_zero() {
            return Err(NyquestroError::InvalidQuantity);
        }
        Ok(Order {
            id,
            symbol,
            side,
            price,
            quantity,
            remaining: quantity,
            timestamp,
            status: Status::Open,
        })
    }

    /// Convenience constructor for callers that don't care about
    /// determinism — uses `Ts::now()`.
    pub fn new_now(
        id: OrderID,
        symbol: Symbol,
        side: Side,
        price: Px,
        quantity: Qty,
    ) -> NyquestroResult<Self> {
        Self::new(id, symbol, side, price, quantity, Ts::now())
    }

    // ─── Accessors ─────────────────────────────────────────────────────────

    #[inline]
    pub fn id(&self) -> OrderID {
        self.id
    }

    #[inline]
    pub fn symbol(&self) -> Symbol {
        self.symbol
    }

    #[inline]
    pub fn side(&self) -> Side {
        self.side
    }

    #[inline]
    pub fn price(&self) -> Px {
        self.price
    }

    #[inline]
    pub fn quantity(&self) -> Qty {
        self.quantity
    }

    #[inline]
    pub fn remaining(&self) -> Qty {
        self.remaining
    }

    #[inline]
    pub fn filled(&self) -> Qty {
        // Safe: `quantity >= remaining` is an invariant the constructor and
        // `fill()` jointly maintain.
        Qty::new(self.quantity.value() - self.remaining.value())
    }

    #[inline]
    pub fn timestamp(&self) -> Ts {
        self.timestamp
    }

    #[inline]
    pub fn status(&self) -> Status {
        self.status
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    // ─── Mutations ─────────────────────────────────────────────────────────

    /// Apply a fill of `amount` units. Rejects (without mutation) when:
    /// - `amount` is zero
    /// - `amount` exceeds [`remaining`](Self::remaining)
    /// - the order is already terminal
    pub fn fill(&mut self, amount: Qty) -> NyquestroResult<()> {
        if self.status.is_terminal() {
            return Err(NyquestroError::OrderTerminal(self.id.value()));
        }
        if amount.is_zero() {
            return Err(NyquestroError::InvalidQuantity);
        }
        let new_remaining = self.remaining.checked_sub(amount).ok_or_else(|| {
            NyquestroError::OverFill {
                order_id: self.id.value(),
                fill: amount.value(),
                remaining: self.remaining.value(),
            }
        })?;

        let new_status = if new_remaining.is_zero() {
            Status::FullyFilled
        } else {
            Status::PartiallyFilled
        };
        self.transition_to(new_status)?;
        self.remaining = new_remaining;
        Ok(())
    }

    pub fn cancel(&mut self) -> NyquestroResult<()> {
        if self.status.is_terminal() {
            return Err(NyquestroError::OrderTerminal(self.id.value()));
        }
        self.transition_to(Status::Cancelled)?;
        Ok(())
    }

    fn transition_to(&mut self, next: Status) -> NyquestroResult<()> {
        if !self.status.can_transition_to(next) {
            return Err(NyquestroError::InvalidStatusTransition {
                order_id: self.id.value(),
                from: status_str(self.status),
                to: status_str(next),
            });
        }
        self.status = next;
        Ok(())
    }
}

fn status_str(s: Status) -> &'static str {
    match s {
        Status::Open => "OPEN",
        Status::PartiallyFilled => "PARTIAL",
        Status::FullyFilled => "FILLED",
        Status::Cancelled => "CANCELLED",
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Order{} {} {} {}@{} ({} remaining, {})",
            self.id, self.symbol, self.side, self.quantity, self.price, self.remaining, self.status
        )
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SYM: Symbol = Symbol::from_const("TEST");

    fn px(c: u64) -> Px {
        Px::from_cents(c).unwrap()
    }
    fn id(n: u64) -> OrderID {
        OrderID::new(n).unwrap()
    }
    fn qty(n: u32) -> Qty {
        Qty::new(n)
    }

    #[test]
    fn new_rejects_zero_quantity() {
        let err = Order::new(id(1), SYM, Side::Buy, px(100), Qty::ZERO, Ts::from_nanos(1));
        assert!(matches!(err, Err(NyquestroError::InvalidQuantity)));
    }

    #[test]
    fn fill_partial_then_full() {
        let mut o = Order::new(id(1), SYM, Side::Buy, px(100), qty(10), Ts::from_nanos(1)).unwrap();
        o.fill(qty(3)).unwrap();
        assert_eq!(o.remaining(), qty(7));
        assert_eq!(o.filled(), qty(3));
        assert_eq!(o.status(), Status::PartiallyFilled);

        o.fill(qty(7)).unwrap();
        assert_eq!(o.remaining(), Qty::ZERO);
        assert_eq!(o.status(), Status::FullyFilled);
    }

    #[test]
    fn fill_rejects_overfill_without_mutation() {
        let mut o = Order::new(id(1), SYM, Side::Buy, px(100), qty(10), Ts::from_nanos(1)).unwrap();
        let err = o.fill(qty(11));
        assert!(matches!(err, Err(NyquestroError::OverFill { .. })));
        // State preserved.
        assert_eq!(o.remaining(), qty(10));
        assert_eq!(o.status(), Status::Open);
    }

    #[test]
    fn fill_rejects_zero_amount() {
        let mut o = Order::new(id(1), SYM, Side::Buy, px(100), qty(10), Ts::from_nanos(1)).unwrap();
        assert!(matches!(o.fill(Qty::ZERO), Err(NyquestroError::InvalidQuantity)));
        assert_eq!(o.remaining(), qty(10));
    }

    #[test]
    fn fill_rejected_after_terminal() {
        let mut o = Order::new(id(1), SYM, Side::Buy, px(100), qty(10), Ts::from_nanos(1)).unwrap();
        o.fill(qty(10)).unwrap();
        assert_eq!(o.status(), Status::FullyFilled);
        let err = o.fill(qty(1));
        assert!(matches!(err, Err(NyquestroError::OrderTerminal(_))));
    }

    #[test]
    fn cancel_open_order() {
        let mut o = Order::new(id(1), SYM, Side::Buy, px(100), qty(10), Ts::from_nanos(1)).unwrap();
        o.cancel().unwrap();
        assert_eq!(o.status(), Status::Cancelled);
    }

    #[test]
    fn cancel_partial_order() {
        let mut o = Order::new(id(1), SYM, Side::Buy, px(100), qty(10), Ts::from_nanos(1)).unwrap();
        o.fill(qty(3)).unwrap();
        o.cancel().unwrap();
        assert_eq!(o.status(), Status::Cancelled);
        assert_eq!(o.remaining(), qty(7)); // remaining preserved
    }

    #[test]
    fn cancel_rejected_after_terminal() {
        let mut o = Order::new(id(1), SYM, Side::Buy, px(100), qty(10), Ts::from_nanos(1)).unwrap();
        o.fill(qty(10)).unwrap();
        assert!(matches!(o.cancel(), Err(NyquestroError::OrderTerminal(_))));
    }

    #[test]
    fn observing_state_does_not_move_order() {
        let o = Order::new(id(1), SYM, Side::Buy, px(100), qty(10), Ts::from_nanos(1)).unwrap();
        // All accessors take &self — no clone needed.
        let _ = o.id();
        let _ = o.side();
        let _ = o.price();
        let _ = o.quantity();
        let _ = o.remaining();
        let _ = o.timestamp();
        let _ = o.status();
        let _ = o.filled();
        // o is still usable here.
        assert_eq!(o.id(), id(1));
    }
}
