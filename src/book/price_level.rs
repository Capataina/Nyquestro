//! `PriceLevel` — FIFO queue of resting orders at a single price.
//!
//! `total_quantity` is maintained as a running sum across every mutation so
//! the order book can read top-of-book size in O(1) without iterating the
//! queue.

use std::collections::VecDeque;

use crate::errors::{NyquestroError, NyquestroResult};
use crate::order::Order;
use crate::types::{OrderID, Px, Qty};

#[derive(Debug, Clone)]
pub struct PriceLevel {
    price: Px,
    orders: VecDeque<Order>,
    total_quantity: Qty,
}

impl PriceLevel {
    pub fn new(price: Px) -> Self {
        PriceLevel {
            price,
            orders: VecDeque::new(),
            total_quantity: Qty::ZERO,
        }
    }

    #[inline]
    pub fn price(&self) -> Px {
        self.price
    }

    #[inline]
    pub fn total_quantity(&self) -> Qty {
        self.total_quantity
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.orders.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    /// Append an order to the back of the FIFO queue. The order's price
    /// must match the level.
    pub fn push_back(&mut self, order: Order) -> NyquestroResult<()> {
        if order.price() != self.price {
            return Err(NyquestroError::PriceLevelMismatch {
                expected_cents: self.price.cents(),
                actual_cents: order.price().cents(),
            });
        }
        self.total_quantity = self
            .total_quantity
            .checked_add(order.remaining())
            .ok_or(NyquestroError::QuantityOverflow)?;
        self.orders.push_back(order);
        Ok(())
    }

    /// Borrow the front order (oldest at this price).
    #[inline]
    pub fn front(&self) -> Option<&Order> {
        self.orders.front()
    }

    /// Borrow the front order mutably.
    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut Order> {
        self.orders.front_mut()
    }

    /// Decrement the level's running total by `executed`. Called by the
    /// matching engine after applying a fill to the front order so size
    /// queries reflect post-fill state.
    pub fn record_execution(&mut self, executed: Qty) -> NyquestroResult<()> {
        self.total_quantity = self
            .total_quantity
            .checked_sub(executed)
            .ok_or(NyquestroError::InvariantViolation(
                "PriceLevel total_quantity underflow",
            ))?;
        Ok(())
    }

    /// Remove and return the front order. Decrements `total_quantity` by
    /// the front order's remaining quantity at the time of removal.
    pub fn pop_front(&mut self) -> Option<Order> {
        let order = self.orders.pop_front()?;
        // Cannot fail: total_quantity ≥ remaining of front by invariant.
        if let Some(new_total) = self.total_quantity.checked_sub(order.remaining()) {
            self.total_quantity = new_total;
        }
        Some(order)
    }

    /// Remove the order with `id` from anywhere in the queue, returning it
    /// if present. O(n) — acceptable for cancellations which are rare
    /// relative to fills in the matching hot path.
    pub fn remove_by_id(&mut self, id: OrderID) -> Option<Order> {
        let pos = self.orders.iter().position(|o| o.id() == id)?;
        let order = self.orders.remove(pos)?;
        if let Some(new_total) = self.total_quantity.checked_sub(order.remaining()) {
            self.total_quantity = new_total;
        }
        Some(order)
    }

    /// Iterate orders in time-priority order (front to back).
    pub fn iter(&self) -> impl Iterator<Item = &Order> {
        self.orders.iter()
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Side, Symbol, Ts};

    const SYM: Symbol = Symbol::from_const("TEST");

    fn order(id: u64, price: u64, qty: u32, ts: u64) -> Order {
        Order::new(
            OrderID::new(id).unwrap(),
            SYM,
            Side::Buy,
            Px::from_cents(price).unwrap(),
            Qty::new(qty),
            Ts::from_nanos(ts),
        )
        .unwrap()
    }

    #[test]
    fn empty_level_has_zero_total() {
        let lvl = PriceLevel::new(Px::from_cents(100).unwrap());
        assert!(lvl.is_empty());
        assert_eq!(lvl.total_quantity(), Qty::ZERO);
    }

    #[test]
    fn push_back_maintains_total() {
        let mut lvl = PriceLevel::new(Px::from_cents(100).unwrap());
        lvl.push_back(order(1, 100, 5, 1)).unwrap();
        lvl.push_back(order(2, 100, 3, 2)).unwrap();
        assert_eq!(lvl.total_quantity(), Qty::new(8));
        assert_eq!(lvl.len(), 2);
    }

    #[test]
    fn push_back_rejects_wrong_price() {
        let mut lvl = PriceLevel::new(Px::from_cents(100).unwrap());
        let err = lvl.push_back(order(1, 200, 5, 1));
        assert!(matches!(
            err,
            Err(NyquestroError::PriceLevelMismatch { .. })
        ));
    }

    #[test]
    fn pop_front_returns_oldest_first() {
        let mut lvl = PriceLevel::new(Px::from_cents(100).unwrap());
        lvl.push_back(order(1, 100, 5, 1)).unwrap();
        lvl.push_back(order(2, 100, 3, 2)).unwrap();
        let first = lvl.pop_front().unwrap();
        assert_eq!(first.id().value(), 1);
        assert_eq!(lvl.total_quantity(), Qty::new(3));
    }

    #[test]
    fn record_execution_updates_total() {
        let mut lvl = PriceLevel::new(Px::from_cents(100).unwrap());
        lvl.push_back(order(1, 100, 5, 1)).unwrap();
        lvl.record_execution(Qty::new(2)).unwrap();
        assert_eq!(lvl.total_quantity(), Qty::new(3));
    }

    #[test]
    fn remove_by_id_returns_and_decrements() {
        let mut lvl = PriceLevel::new(Px::from_cents(100).unwrap());
        lvl.push_back(order(1, 100, 5, 1)).unwrap();
        lvl.push_back(order(2, 100, 3, 2)).unwrap();
        lvl.push_back(order(3, 100, 7, 3)).unwrap();
        let removed = lvl.remove_by_id(OrderID::new(2).unwrap()).unwrap();
        assert_eq!(removed.id().value(), 2);
        assert_eq!(lvl.total_quantity(), Qty::new(12));
        assert_eq!(lvl.len(), 2);
        // FIFO order preserved across removal.
        let rest: Vec<_> = lvl.iter().map(|o| o.id().value()).collect();
        assert_eq!(rest, vec![1, 3]);
    }

    #[test]
    fn remove_by_id_missing_returns_none() {
        let mut lvl = PriceLevel::new(Px::from_cents(100).unwrap());
        lvl.push_back(order(1, 100, 5, 1)).unwrap();
        assert!(lvl.remove_by_id(OrderID::new(99).unwrap()).is_none());
    }

    #[test]
    fn pop_front_empty_returns_none() {
        let mut lvl = PriceLevel::new(Px::from_cents(100).unwrap());
        assert!(lvl.pop_front().is_none());
    }
}
