//! Integration tests for event frames.

use nyquestro::errors::NyquestroError;
use nyquestro::events::{FillEvent, OrderEvent, OrderRejectionReason, QuoteEvent, QuoteSide};
use nyquestro::types::{OrderID, Px, Qty, Side, Symbol, Ts};

const SYM: Symbol = Symbol::from_const("TEST");

fn id(n: u64) -> OrderID {
    OrderID::new(n).unwrap()
}
fn px(c: u64) -> Px {
    Px::from_cents(c).unwrap()
}
fn ts(n: u64) -> Ts {
    Ts::from_nanos(n)
}

#[test]
fn fill_self_match_rejected() {
    let same = id(1);
    let err = FillEvent::new(SYM, same, same, px(100), Qty::new(5), ts(1));
    assert!(matches!(err, Err(NyquestroError::SelfMatch(1))));
}

#[test]
fn fill_zero_quantity_rejected() {
    let err = FillEvent::new(SYM, id(1), id(2), px(100), Qty::ZERO, ts(1));
    assert!(matches!(err, Err(NyquestroError::InvalidQuantity)));
}

#[test]
fn fill_valid_constructs() {
    let f = FillEvent::new(SYM, id(1), id(2), px(100), Qty::new(3), ts(1)).unwrap();
    assert_eq!(f.buyer_order_id, id(1));
    assert_eq!(f.seller_order_id, id(2));
    assert_eq!(f.price, px(100));
    assert_eq!(f.quantity, Qty::new(3));
    assert_eq!(f.symbol, SYM);
}

#[test]
fn quote_live_rejects_zero_quantity() {
    let err = QuoteEvent::live(SYM, QuoteSide::Bid, px(100), Qty::ZERO, ts(1));
    assert!(matches!(err, Err(NyquestroError::InvalidQuantity)));
}

#[test]
fn quote_cleared_carries_zero_quantity() {
    let q = QuoteEvent::cleared(SYM, QuoteSide::Ask, px(100), ts(1));
    assert!(q.quantity.is_zero());
    assert_eq!(q.side, QuoteSide::Ask);
}

#[test]
fn order_event_placed_validates_quantity() {
    let err = OrderEvent::placed(id(1), SYM, Side::Buy, px(100), Qty::ZERO, ts(1));
    assert!(matches!(err, Err(NyquestroError::InvalidQuantity)));
}

#[test]
fn order_event_filled_validates_executed() {
    let err = OrderEvent::filled(id(1), SYM, Qty::ZERO, Qty::new(5), ts(1));
    assert!(matches!(err, Err(NyquestroError::InvalidQuantity)));
}

#[test]
fn order_event_lifecycle_round_trip() {
    let placed = OrderEvent::placed(id(1), SYM, Side::Buy, px(100), Qty::new(10), ts(1)).unwrap();
    assert_eq!(placed.order_id(), id(1));

    let filled = OrderEvent::filled(id(1), SYM, Qty::new(3), Qty::new(7), ts(2)).unwrap();
    assert_eq!(filled.order_id(), id(1));

    let cancelled = OrderEvent::cancelled(id(1), SYM, Qty::new(7), ts(3));
    assert_eq!(cancelled.order_id(), id(1));

    let rejected = OrderEvent::rejected(id(2), SYM, OrderRejectionReason::SelfMatch, ts(4));
    assert_eq!(rejected.order_id(), id(2));
}

#[test]
fn events_are_copy() {
    fn assert_copy<T: Copy>(_: T) {}
    assert_copy(FillEvent::new(SYM, id(1), id(2), px(100), Qty::new(1), ts(1)).unwrap());
    assert_copy(QuoteEvent::cleared(SYM, QuoteSide::Bid, px(100), ts(1)));
    assert_copy(OrderEvent::cancelled(id(1), SYM, Qty::new(1), ts(1)));
}
