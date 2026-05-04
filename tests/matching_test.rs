//! Integration tests for the order matching engine.
//!
//! Every test pins a fixed input sequence and asserts the exact `FillEvent`
//! / `QuoteEvent` / `OrderEvent` outputs. Determinism is the contract.

use nyquestro::book::OrderBook;
use nyquestro::events::{OrderEvent, OrderRejectionReason, QuoteSide};
use nyquestro::order::Order;
use nyquestro::types::{OrderID, Px, Qty, Side, Status, Symbol, Ts};

const SYM: Symbol = Symbol::from_const("TEST");

fn buy(id: u64, price: u64, qty: u32, ts: u64) -> Order {
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
fn sell(id: u64, price: u64, qty: u32, ts: u64) -> Order {
    Order::new(
        OrderID::new(id).unwrap(),
        SYM,
        Side::Sell,
        Px::from_cents(price).unwrap(),
        Qty::new(qty),
        Ts::from_nanos(ts),
    )
    .unwrap()
}

// ─── Resting & inspection ──────────────────────────────────────────────────

#[test]
fn resting_orders_define_top_of_book() {
    let mut book = OrderBook::new(SYM);
    let r1 = book.submit_limit(buy(1, 9990, 5, 1)).unwrap();
    assert!(r1.fills.is_empty());
    assert_eq!(r1.lifecycle.len(), 1);
    assert!(matches!(r1.lifecycle[0], OrderEvent::Placed { .. }));
    assert_eq!(r1.quotes.len(), 1);
    assert_eq!(r1.quotes[0].side, QuoteSide::Bid);

    book.submit_limit(buy(2, 9985, 3, 2)).unwrap();
    book.submit_limit(sell(3, 10005, 4, 3)).unwrap();

    assert_eq!(book.best_bid(), Some((Px::from_cents(9990).unwrap(), Qty::new(5))));
    assert_eq!(book.best_ask(), Some((Px::from_cents(10005).unwrap(), Qty::new(4))));
    assert_eq!(book.len(), 3);
}

// ─── Simple cross ──────────────────────────────────────────────────────────

#[test]
fn simple_cross_one_resting_one_aggressor() {
    let mut book = OrderBook::new(SYM);
    book.submit_limit(sell(1, 10000, 5, 1)).unwrap(); // resting ask

    let res = book.submit_limit(buy(2, 10000, 5, 2)).unwrap();
    assert_eq!(res.fills.len(), 1);
    let f = res.fills[0];
    assert_eq!(f.buyer_order_id.value(), 2);
    assert_eq!(f.seller_order_id.value(), 1);
    assert_eq!(f.price.cents(), 10000);
    assert_eq!(f.quantity, Qty::new(5));
    // Book is now empty.
    assert!(book.is_empty());
    // Quote: ask cleared.
    assert!(res.quotes.iter().any(|q| q.side == QuoteSide::Ask && q.quantity.is_zero()));
}

// ─── Multi-level sweep ─────────────────────────────────────────────────────

#[test]
fn aggressor_sweeps_three_levels() {
    let mut book = OrderBook::new(SYM);
    book.submit_limit(sell(1, 10000, 3, 1)).unwrap();
    book.submit_limit(sell(2, 10010, 4, 2)).unwrap();
    book.submit_limit(sell(3, 10020, 2, 3)).unwrap();

    // Buy 8 @ 10025 — sweeps three levels (3 + 4 + 1 = 8), partial of level 3.
    let res = book.submit_limit(buy(99, 10025, 8, 4)).unwrap();
    assert_eq!(res.fills.len(), 3);
    assert_eq!(res.fills[0].quantity, Qty::new(3));
    assert_eq!(res.fills[0].price.cents(), 10000);
    assert_eq!(res.fills[1].quantity, Qty::new(4));
    assert_eq!(res.fills[1].price.cents(), 10010);
    assert_eq!(res.fills[2].quantity, Qty::new(1));
    assert_eq!(res.fills[2].price.cents(), 10020);

    // Level 3 still has 1 unit remaining.
    assert_eq!(
        book.best_ask(),
        Some((Px::from_cents(10020).unwrap(), Qty::new(1)))
    );
    assert!(book.best_bid().is_none()); // aggressor was fully filled
}

// ─── Partial then rest ─────────────────────────────────────────────────────

#[test]
fn partial_match_then_rest_remainder() {
    let mut book = OrderBook::new(SYM);
    book.submit_limit(sell(1, 10000, 3, 1)).unwrap();

    // Buy 8 @ 10000 — matches 3, rests 5 @ 10000 (own price = match price here).
    let res = book.submit_limit(buy(2, 10000, 8, 2)).unwrap();
    assert_eq!(res.fills.len(), 1);
    assert_eq!(res.fills[0].quantity, Qty::new(3));

    // Bid side now has 5 resting at 10000.
    assert_eq!(
        book.best_bid(),
        Some((Px::from_cents(10000).unwrap(), Qty::new(5)))
    );
    assert!(book.best_ask().is_none());

    // Lifecycle: at minimum a Filled and a Placed.
    let kinds: Vec<_> = res
        .lifecycle
        .iter()
        .map(|e| match e {
            OrderEvent::Placed { .. } => "P",
            OrderEvent::Filled { .. } => "F",
            OrderEvent::Cancelled { .. } => "C",
            OrderEvent::Rejected { .. } => "R",
        })
        .collect();
    assert!(kinds.contains(&"P"));
    assert!(kinds.contains(&"F"));
}

// ─── FIFO within a level ──────────────────────────────────────────────────

#[test]
fn fifo_within_a_price_level() {
    let mut book = OrderBook::new(SYM);
    book.submit_limit(sell(10, 10000, 2, 1)).unwrap();
    book.submit_limit(sell(11, 10000, 2, 2)).unwrap();
    book.submit_limit(sell(12, 10000, 2, 3)).unwrap();

    let res = book.submit_limit(buy(99, 10000, 5, 4)).unwrap();
    assert_eq!(res.fills.len(), 3);
    assert_eq!(res.fills[0].seller_order_id.value(), 10);
    assert_eq!(res.fills[1].seller_order_id.value(), 11);
    assert_eq!(res.fills[2].seller_order_id.value(), 12);
    assert_eq!(res.fills[0].quantity, Qty::new(2));
    assert_eq!(res.fills[1].quantity, Qty::new(2));
    assert_eq!(res.fills[2].quantity, Qty::new(1));
    // Order 12 still has 1 remaining at 10000.
    assert_eq!(
        book.best_ask(),
        Some((Px::from_cents(10000).unwrap(), Qty::new(1)))
    );
}

// ─── Self-match rejection ──────────────────────────────────────────────────

#[test]
fn self_match_rejects_aggressor_leaves_resting() {
    let mut book = OrderBook::new(SYM);
    // Place a sell from id=42, then have id=42 try to buy across — match-time
    // rejection: the aggressor is rejected, the resting order is untouched.
    book.submit_limit(sell(42, 10000, 5, 1)).unwrap();
    let res = book.submit_limit(buy(42, 10000, 5, 2)).unwrap();
    assert!(res.fills.is_empty());
    assert!(matches!(
        res.lifecycle.last().unwrap(),
        OrderEvent::Rejected {
            reason: OrderRejectionReason::SelfMatch,
            ..
        }
    ));
    // Resting sell still in the book.
    assert_eq!(
        book.best_ask(),
        Some((Px::from_cents(10000).unwrap(), Qty::new(5)))
    );
}

// ─── Cancellation ─────────────────────────────────────────────────────────

#[test]
fn cancel_removes_order_from_book() {
    let mut book = OrderBook::new(SYM);
    book.submit_limit(buy(1, 9990, 5, 1)).unwrap();
    book.submit_limit(buy(2, 9990, 3, 2)).unwrap();

    let evt = book.cancel(OrderID::new(1).unwrap(), Ts::from_nanos(3)).unwrap();
    assert!(matches!(evt, OrderEvent::Cancelled { .. }));
    // FIFO remainder: only id=2 (3 units) is left at 9990.
    assert_eq!(
        book.best_bid(),
        Some((Px::from_cents(9990).unwrap(), Qty::new(3)))
    );
}

#[test]
fn cancel_unknown_id_errors() {
    let mut book = OrderBook::new(SYM);
    book.submit_limit(buy(1, 9990, 5, 1)).unwrap();
    let err = book.cancel(OrderID::new(99).unwrap(), Ts::from_nanos(2));
    assert!(err.is_err());
}

// ─── Determinism ──────────────────────────────────────────────────────────

#[test]
fn run_twice_identical_sequence_identical_output() {
    let sequence: Vec<Order> = vec![
        sell(1, 10010, 5, 1),
        sell(2, 10000, 3, 2),
        buy(3, 9995, 4, 3),
        buy(4, 10005, 7, 4),
        sell(5, 10005, 2, 5),
        buy(6, 10010, 10, 6),
    ];

    let mut a = OrderBook::new(SYM);
    let mut b = OrderBook::new(SYM);
    let ra: Vec<_> = sequence
        .iter()
        .copied()
        .map(|o| a.submit_limit(o).unwrap())
        .collect();
    let rb: Vec<_> = sequence
        .iter()
        .copied()
        .map(|o| b.submit_limit(o).unwrap())
        .collect();
    assert_eq!(ra, rb);
    assert_eq!(a.best_bid(), b.best_bid());
    assert_eq!(a.best_ask(), b.best_ask());
}

// ─── Top-of-book quote semantics ──────────────────────────────────────────

#[test]
fn quote_emitted_only_on_top_of_book_change() {
    let mut book = OrderBook::new(SYM);
    // First order on bid side establishes top — quote.
    let r1 = book.submit_limit(buy(1, 9990, 5, 1)).unwrap();
    assert_eq!(r1.quotes.len(), 1);

    // Second order at SAME price increases displayed size — quote (qty changed).
    let r2 = book.submit_limit(buy(2, 9990, 3, 2)).unwrap();
    assert_eq!(r2.quotes.len(), 1);
    assert_eq!(r2.quotes[0].quantity, Qty::new(8));

    // Third order at WORSE price — top unchanged, no quote.
    let r3 = book.submit_limit(buy(3, 9985, 1, 3)).unwrap();
    assert!(r3.quotes.is_empty());
}

#[test]
fn aggressor_full_fill_does_not_rest() {
    let mut book = OrderBook::new(SYM);
    book.submit_limit(sell(1, 10000, 10, 1)).unwrap();
    let res = book.submit_limit(buy(2, 10000, 10, 2)).unwrap();
    // Aggressor fully consumed — no Placed event.
    assert!(!res
        .lifecycle
        .iter()
        .any(|e| matches!(e, OrderEvent::Placed { order_id, .. } if order_id.value() == 2)));
    // Aggressor lifecycle starts at Filled.
    assert!(matches!(res.lifecycle[0], OrderEvent::Filled { .. }));
}

#[test]
fn aggressor_state_reflects_terminal_after_full_match() {
    // Internal sanity check: fill semantics on the aggressor align with the
    // events emitted.
    let mut book = OrderBook::new(SYM);
    book.submit_limit(sell(1, 10000, 5, 1)).unwrap();
    let res = book.submit_limit(buy(2, 10000, 5, 2)).unwrap();
    assert_eq!(res.fills.len(), 1);
    // Both filled events expected (one for aggressor, one for resting).
    let filled = res
        .lifecycle
        .iter()
        .filter(|e| matches!(e, OrderEvent::Filled { .. }))
        .count();
    assert_eq!(filled, 2);
    // No status leakage — terminal state is internal to Order.
    let _ = Status::FullyFilled;
}
