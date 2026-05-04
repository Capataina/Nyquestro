//! Minimal deterministic limit-order book.
//!
//! ## Design
//!
//! - One `OrderBook` is **single-symbol**. The book stores its own `Symbol`
//!   and rejects orders for any other symbol with `SymbolMismatch`. The
//!   `Market` wrapper holds one `OrderBook` per `Symbol`.
//! - Two `BTreeMap<Px, PriceLevel>` ladders, one per side. Bids are walked
//!   from highest price (`iter().next_back()`); asks from lowest
//!   (`iter().next()`).
//! - Within a price level, FIFO ordering is maintained by [`PriceLevel`]
//!   ([`VecDeque`] under the hood).
//! - Match price is the **resting** order's price: the incoming side gets
//!   price improvement when it crosses.
//! - **Quote events** are emitted only when the top-of-book price OR
//!   displayed quantity changes on the affected side. No spam on every
//!   fill.
//! - **Self-match policy:** match-time rejection. The aggressing order is
//!   wholly rejected; the resting order is untouched.
//! - **Determinism:** matching never consults the wall clock. Identical
//!   input sequences therefore produce byte-identical outputs.

use std::collections::BTreeMap;

use crate::book::price_level::PriceLevel;
use crate::errors::{NyquestroError, NyquestroResult};
use crate::events::{FillEvent, OrderEvent, OrderRejectionReason, QuoteEvent, QuoteSide};
use crate::order::Order;
use crate::types::{OrderID, Px, Qty, Side, Symbol, Ts};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SubmitResult {
    pub fills: Vec<FillEvent>,
    pub quotes: Vec<QuoteEvent>,
    pub lifecycle: Vec<OrderEvent>,
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    symbol: Symbol,
    bids: BTreeMap<Px, PriceLevel>,
    asks: BTreeMap<Px, PriceLevel>,
}

impl OrderBook {
    pub fn new(symbol: Symbol) -> Self {
        OrderBook {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn symbol(&self) -> Symbol {
        self.symbol
    }

    // ─── Inspection ────────────────────────────────────────────────────────

    pub fn best_bid(&self) -> Option<(Px, Qty)> {
        self.bids
            .iter()
            .next_back()
            .map(|(p, lvl)| (*p, lvl.total_quantity()))
    }

    pub fn best_ask(&self) -> Option<(Px, Qty)> {
        self.asks
            .iter()
            .next()
            .map(|(p, lvl)| (*p, lvl.total_quantity()))
    }

    pub fn len(&self) -> usize {
        self.bids.values().map(|l| l.len()).sum::<usize>()
            + self.asks.values().map(|l| l.len()).sum::<usize>()
    }

    pub fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }

    pub fn bid_levels(&self) -> impl DoubleEndedIterator<Item = (&Px, &PriceLevel)> {
        self.bids.iter().rev()
    }

    pub fn ask_levels(&self) -> impl Iterator<Item = (&Px, &PriceLevel)> {
        self.asks.iter()
    }

    /// Top-N bid levels by best price. Each entry is `(price, total_qty)`.
    pub fn top_n_bids(&self, n: usize) -> Vec<(Px, Qty)> {
        self.bid_levels()
            .take(n)
            .map(|(p, l)| (*p, l.total_quantity()))
            .collect()
    }

    pub fn top_n_asks(&self, n: usize) -> Vec<(Px, Qty)> {
        self.ask_levels()
            .take(n)
            .map(|(p, l)| (*p, l.total_quantity()))
            .collect()
    }

    /// Sum of displayed quantity across the top-N levels per side.
    pub fn depth(&self, n: usize) -> (Qty, Qty) {
        let bid_sum: u64 = self
            .bid_levels()
            .take(n)
            .map(|(_, l)| l.total_quantity().value() as u64)
            .sum();
        let ask_sum: u64 = self
            .ask_levels()
            .take(n)
            .map(|(_, l)| l.total_quantity().value() as u64)
            .sum();
        (
            Qty::new(bid_sum.min(u32::MAX as u64) as u32),
            Qty::new(ask_sum.min(u32::MAX as u64) as u32),
        )
    }

    /// Order Flow Imbalance over the top-N levels:
    /// `(bid_qty - ask_qty) / (bid_qty + ask_qty)`. Range `[-1, 1]`.
    /// Returns `0.0` when both sides are empty.
    pub fn ofi(&self, n: usize) -> f64 {
        let (bid, ask) = self.depth(n);
        let b = bid.value() as f64;
        let a = ask.value() as f64;
        let total = b + a;
        if total == 0.0 {
            0.0
        } else {
            (b - a) / total
        }
    }

    /// Microprice: volume-weighted mid using the *best* bid and ask.
    /// `(bid_qty * ask_px + ask_qty * bid_px) / (bid_qty + ask_qty)` in cents.
    /// Returns `None` when either side is empty.
    pub fn microprice(&self) -> Option<f64> {
        let (bp, bq) = self.best_bid()?;
        let (ap, aq) = self.best_ask()?;
        let bq = bq.value() as f64;
        let aq = aq.value() as f64;
        let total = bq + aq;
        if total == 0.0 {
            return None;
        }
        Some((bq * ap.cents() as f64 + aq * bp.cents() as f64) / total)
    }

    /// Spread in cents (best ask - best bid). Returns `None` when either
    /// side is empty.
    pub fn spread_cents(&self) -> Option<u64> {
        let (bp, _) = self.best_bid()?;
        let (ap, _) = self.best_ask()?;
        Some(ap.cents().saturating_sub(bp.cents()))
    }

    /// Number of distinct price levels per side.
    pub fn level_counts(&self) -> (usize, usize) {
        (self.bids.len(), self.asks.len())
    }

    // ─── Submission ────────────────────────────────────────────────────────

    pub fn submit_limit(&mut self, mut order: Order) -> NyquestroResult<SubmitResult> {
        if order.symbol() != self.symbol {
            return Err(NyquestroError::SymbolMismatch {
                expected: self.symbol.as_u64(),
                actual: order.symbol().as_u64(),
            });
        }

        let mut result = SubmitResult::default();
        let pre_same_side_top = self.top_of(order.side());
        let pre_opposite_top = self.top_of(order.side().opposite());

        let aggressor_id = order.id();
        let aggressor_symbol = order.symbol();
        let mut self_match_detected = false;

        loop {
            if !order.is_active() || order.remaining().is_zero() {
                break;
            }
            let opposite_top = match self.top_of(order.side().opposite()) {
                Some(t) => t,
                None => break,
            };
            if !crosses(order.side(), order.price(), opposite_top.0) {
                break;
            }

            let opposite_best_px = opposite_top.0;
            let opposite_levels = self.book_mut(order.side().opposite());
            let level = opposite_levels
                .get_mut(&opposite_best_px)
                .expect("level must exist; we just probed it");
            let resting_id = level.front().expect("non-empty").id();
            if resting_id == aggressor_id {
                self_match_detected = true;
                break;
            }

            let (resting_id_for_event, resting_ts, resting_done, trade_qty) = {
                let resting = level.front_mut().expect("non-empty");
                let trade = Qty::new(
                    order
                        .remaining()
                        .value()
                        .min(resting.remaining().value()),
                );
                resting.fill(trade)?;
                (
                    resting.id(),
                    resting.timestamp(),
                    resting.status().is_terminal(),
                    trade,
                )
            };
            level.record_execution(trade_qty)?;
            order.fill(trade_qty)?;

            let (buyer_id, seller_id) = match order.side() {
                Side::Buy => (aggressor_id, resting_id_for_event),
                Side::Sell => (resting_id_for_event, aggressor_id),
            };
            let fill = FillEvent::new(
                aggressor_symbol,
                buyer_id,
                seller_id,
                opposite_best_px,
                trade_qty,
                resting_ts,
            )?;
            result.fills.push(fill);

            result.lifecycle.push(OrderEvent::filled(
                aggressor_id,
                aggressor_symbol,
                trade_qty,
                order.remaining(),
                resting_ts,
            )?);

            if resting_done {
                let popped = level.pop_front().expect("front must exist");
                result.lifecycle.push(OrderEvent::filled(
                    popped.id(),
                    aggressor_symbol,
                    trade_qty,
                    Qty::ZERO,
                    resting_ts,
                )?);
            }

            if level.is_empty() {
                opposite_levels.remove(&opposite_best_px);
            }
        }

        if self_match_detected {
            result.lifecycle.push(OrderEvent::rejected(
                aggressor_id,
                aggressor_symbol,
                OrderRejectionReason::SelfMatch,
                order.timestamp(),
            ));
            self.emit_quote_if_changed(
                order.side().opposite(),
                pre_opposite_top,
                order.timestamp(),
                &mut result.quotes,
            );
            return Ok(result);
        }

        if order.remaining().value() > 0 && order.is_active() {
            let same_side = self.book_mut(order.side());
            let level = same_side
                .entry(order.price())
                .or_insert_with(|| PriceLevel::new(order.price()));
            level.push_back(order)?;

            result.lifecycle.push(OrderEvent::placed(
                aggressor_id,
                aggressor_symbol,
                order.side(),
                order.price(),
                order.quantity(),
                order.timestamp(),
            )?);
        }

        self.emit_quote_if_changed(
            order.side(),
            pre_same_side_top,
            order.timestamp(),
            &mut result.quotes,
        );
        self.emit_quote_if_changed(
            order.side().opposite(),
            pre_opposite_top,
            order.timestamp(),
            &mut result.quotes,
        );

        Ok(result)
    }

    pub fn cancel(&mut self, id: OrderID, ts: Ts) -> NyquestroResult<OrderEvent> {
        let symbol = self.symbol;
        for side in [Side::Buy, Side::Sell] {
            let book = self.book_mut(side);
            let prices: Vec<Px> = book.keys().copied().collect();
            for price in prices {
                let level = book
                    .get_mut(&price)
                    .expect("collected from existing keys");
                if let Some(removed) = level.remove_by_id(id) {
                    let remaining = removed.remaining();
                    if level.is_empty() {
                        book.remove(&price);
                    }
                    return Ok(OrderEvent::cancelled(id, symbol, remaining, ts));
                }
            }
        }
        Err(NyquestroError::OrderNotFound(id.value()))
    }

    fn book_mut(&mut self, side: Side) -> &mut BTreeMap<Px, PriceLevel> {
        match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        }
    }

    fn top_of(&self, side: Side) -> Option<(Px, Qty)> {
        match side {
            Side::Buy => self.best_bid(),
            Side::Sell => self.best_ask(),
        }
    }

    fn emit_quote_if_changed(
        &self,
        side: Side,
        before: Option<(Px, Qty)>,
        ts: Ts,
        out: &mut Vec<QuoteEvent>,
    ) {
        let after = self.top_of(side);
        if before == after {
            return;
        }
        let qside = QuoteSide::from(side);
        match after {
            Some((px, qty)) => {
                if let Ok(q) = QuoteEvent::live(self.symbol, qside, px, qty, ts) {
                    out.push(q);
                }
            }
            None => {
                if let Some((px, _)) = before {
                    out.push(QuoteEvent::cleared(self.symbol, qside, px, ts));
                }
            }
        }
    }
}

#[inline]
fn crosses(side: Side, price: Px, opposite_best: Px) -> bool {
    match side {
        Side::Buy => price >= opposite_best,
        Side::Sell => price <= opposite_best,
    }
}
