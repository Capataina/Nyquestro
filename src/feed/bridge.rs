//! Translates Coinbase L2 [`FeedEvent`]s into the engine's
//! [`SimAction`] stream.
//!
//! Coinbase's `level2` channel is per-level, not per-order: it tells you
//! "the bid at $67000.50 is now 0.5 BTC", not "order ABC for 0.3 BTC was
//! cancelled and order DEF for 0.5 BTC was added". To feed our per-order
//! matching engine, we maintain a virtual `OrderID` per `(symbol, side,
//! price)` cell. Updates translate to "cancel old virtual id, submit new
//! virtual id with updated quantity"; clearing a level (qty = 0)
//! translates to a single cancel.
//!
//! Snapshots clear all virtual ids for the affected symbol and rebuild
//! from scratch.

use std::collections::HashMap;

use crate::feed::coinbase::FeedEvent;
use crate::order::Order;
use crate::simulator::SimAction;
use crate::types::{OrderID, Px, Qty, Side, Symbol, Ts};

/// One translated bridge output. `Action` carries an engine input
/// alongside the symbol-index hint so the dashboard can route to the
/// correct `SymbolState` without re-mapping. `Status` carries a connection
/// / subscribe / parse-error message so the dashboard can render a banner.
#[derive(Debug, Clone)]
pub enum FeedAction {
    Action { symbol_idx: usize, action: SimAction },
    Status(String),
}

pub struct Bridge {
    /// Symbol → its index in the dashboard's `App.symbols` vec.
    symbol_to_idx: HashMap<Symbol, usize>,
    /// `(Symbol, Side, Px)` → the synthetic `OrderID` representing that
    /// L2 cell on the engine's order book.
    level_id: HashMap<(Symbol, Side, Px), OrderID>,
    /// Monotonic `OrderID` allocator. We start high so the bridge's ids
    /// don't collide with any other consumer that might allocate from 1
    /// upward (synthetic simulators, tests).
    next_id: u64,
}

impl Bridge {
    pub fn new(symbols: Vec<Symbol>) -> Self {
        let symbol_to_idx = symbols
            .into_iter()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect();
        Bridge {
            symbol_to_idx,
            level_id: HashMap::new(),
            next_id: 1_000_000_000_000, // start high; well above synthetic-sim ids
        }
    }

    pub fn translate(&mut self, event: FeedEvent) -> Vec<FeedAction> {
        match event {
            FeedEvent::Snapshot {
                symbol,
                bids,
                asks,
            } => self.translate_snapshot(symbol, bids, asks),
            FeedEvent::Update {
                symbol,
                side,
                price,
                new_quantity,
            } => self.translate_update(symbol, side, price, new_quantity),
            FeedEvent::Status(s) => vec![FeedAction::Status(s)],
        }
    }

    fn translate_snapshot(
        &mut self,
        symbol: Symbol,
        bids: Vec<(Px, Qty)>,
        asks: Vec<(Px, Qty)>,
    ) -> Vec<FeedAction> {
        let idx = match self.symbol_to_idx.get(&symbol) {
            Some(i) => *i,
            None => return Vec::new(),
        };
        let mut actions = Vec::new();

        // Cancel everything we know about for this symbol.
        let stale_keys: Vec<_> = self
            .level_id
            .keys()
            .filter(|(s, _, _)| *s == symbol)
            .copied()
            .collect();
        for key in stale_keys {
            if let Some(id) = self.level_id.remove(&key) {
                actions.push(FeedAction::Action {
                    symbol_idx: idx,
                    action: SimAction::Cancel {
                        symbol,
                        order_id: id,
                    },
                });
            }
        }

        // Submit fresh virtual orders for every level in the snapshot.
        for (px, qty) in bids {
            if qty.is_zero() {
                continue;
            }
            if let Some(action) = self.submit_level(symbol, Side::Buy, px, qty) {
                actions.push(FeedAction::Action {
                    symbol_idx: idx,
                    action,
                });
            }
        }
        for (px, qty) in asks {
            if qty.is_zero() {
                continue;
            }
            if let Some(action) = self.submit_level(symbol, Side::Sell, px, qty) {
                actions.push(FeedAction::Action {
                    symbol_idx: idx,
                    action,
                });
            }
        }
        actions
    }

    fn translate_update(
        &mut self,
        symbol: Symbol,
        side: Side,
        price: Px,
        new_quantity: Qty,
    ) -> Vec<FeedAction> {
        let idx = match self.symbol_to_idx.get(&symbol) {
            Some(i) => *i,
            None => return Vec::new(),
        };
        let key = (symbol, side, price);
        let mut actions = Vec::new();

        // Cancel any existing virtual order at this level.
        if let Some(old_id) = self.level_id.remove(&key) {
            actions.push(FeedAction::Action {
                symbol_idx: idx,
                action: SimAction::Cancel {
                    symbol,
                    order_id: old_id,
                },
            });
        }

        // If new quantity is non-zero, submit a fresh virtual order.
        if !new_quantity.is_zero()
            && let Some(action) = self.submit_level(symbol, side, price, new_quantity)
        {
            actions.push(FeedAction::Action {
                symbol_idx: idx,
                action,
            });
        }
        actions
    }

    fn submit_level(
        &mut self,
        symbol: Symbol,
        side: Side,
        price: Px,
        qty: Qty,
    ) -> Option<SimAction> {
        let id = self.alloc_id();
        // Use real wall-clock time so the dashboard's trade tape renders
        // human-readable timestamps. Determinism doesn't apply here —
        // live mode is non-deterministic by construction (the venue
        // controls the message order).
        let ts = Ts::now();
        let order = Order::new(id, symbol, side, price, qty, ts).ok()?;
        self.level_id.insert((symbol, side, price), id);
        Some(SimAction::Submit(order))
    }

    fn alloc_id(&mut self) -> OrderID {
        let n = self.next_id;
        self.next_id = self.next_id.checked_add(1).unwrap_or(1);
        OrderID::new(n).expect("non-zero by construction")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::coinbase::FeedEvent;
    use crate::types::Symbol;

    fn px(c: u64) -> Px {
        Px::from_cents(c).unwrap()
    }
    fn qty(n: u32) -> Qty {
        Qty::new(n)
    }

    #[test]
    fn snapshot_routes_to_correct_idx() {
        let btc = Symbol::from_const("BTC-USD");
        let eth = Symbol::from_const("ETH-USD");
        let mut bridge = Bridge::new(vec![btc, eth]);
        let actions = bridge.translate(FeedEvent::Snapshot {
            symbol: eth,
            bids: vec![(px(300_000), qty(500))],
            asks: vec![(px(300_100), qty(500))],
        });
        assert!(!actions.is_empty());
        assert!(actions.iter().all(|a| matches!(a, FeedAction::Action { symbol_idx: 1, .. })));
    }

    #[test]
    fn update_with_nonzero_emits_cancel_then_submit() {
        let btc = Symbol::from_const("BTC-USD");
        let mut bridge = Bridge::new(vec![btc]);
        // Seed a level via snapshot.
        let _ = bridge.translate(FeedEvent::Snapshot {
            symbol: btc,
            bids: vec![(px(7_000_000), qty(100_000))],
            asks: vec![],
        });
        let actions = bridge.translate(FeedEvent::Update {
            symbol: btc,
            side: Side::Buy,
            price: px(7_000_000),
            new_quantity: qty(150_000),
        });
        // Cancel old + submit new = 2 actions.
        assert_eq!(actions.len(), 2);
        assert!(matches!(
            &actions[0],
            FeedAction::Action { action: SimAction::Cancel { .. }, .. }
        ));
        assert!(matches!(
            &actions[1],
            FeedAction::Action { action: SimAction::Submit(_), .. }
        ));
    }

    #[test]
    fn update_with_zero_emits_cancel_only() {
        let btc = Symbol::from_const("BTC-USD");
        let mut bridge = Bridge::new(vec![btc]);
        let _ = bridge.translate(FeedEvent::Snapshot {
            symbol: btc,
            bids: vec![(px(7_000_000), qty(100_000))],
            asks: vec![],
        });
        let actions = bridge.translate(FeedEvent::Update {
            symbol: btc,
            side: Side::Buy,
            price: px(7_000_000),
            new_quantity: Qty::ZERO,
        });
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            FeedAction::Action { action: SimAction::Cancel { .. }, .. }
        ));
    }

    #[test]
    fn unknown_symbol_drops_silently() {
        let btc = Symbol::from_const("BTC-USD");
        let mut bridge = Bridge::new(vec![btc]);
        let actions = bridge.translate(FeedEvent::Snapshot {
            symbol: Symbol::from_const("DOGE-USD"),
            bids: vec![(px(10), qty(1))],
            asks: vec![(px(11), qty(1))],
        });
        assert!(actions.is_empty());
    }
}
