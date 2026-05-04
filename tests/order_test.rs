//! Integration tests for `Order` — the state machine, fill/cancel
//! semantics, and resistance to bad input.

use nyquestro::errors::NyquestroError;
use nyquestro::order::Order;
use nyquestro::types::{OrderID, Px, Qty, Side, Status, Symbol, Ts};

const SYM: Symbol = Symbol::from_const("TEST");

fn order(id: u64, qty: u32) -> Order {
    Order::new(
        OrderID::new(id).unwrap(),
        SYM,
        Side::Buy,
        Px::from_cents(100).unwrap(),
        Qty::new(qty),
        Ts::from_nanos(1),
    )
    .unwrap()
}

#[test]
fn lifecycle_open_to_filled() {
    let mut o = order(1, 10);
    assert_eq!(o.status(), Status::Open);
    o.fill(Qty::new(4)).unwrap();
    assert_eq!(o.status(), Status::PartiallyFilled);
    assert_eq!(o.filled(), Qty::new(4));
    assert_eq!(o.remaining(), Qty::new(6));

    o.fill(Qty::new(6)).unwrap();
    assert_eq!(o.status(), Status::FullyFilled);
    assert_eq!(o.filled(), Qty::new(10));
    assert_eq!(o.remaining(), Qty::ZERO);
}

#[test]
fn over_fill_returns_error_and_preserves_state() {
    let mut o = order(1, 10);
    let snapshot = o;
    let err = o.fill(Qty::new(11));
    assert!(matches!(err, Err(NyquestroError::OverFill { .. })));
    assert_eq!(o, snapshot);
}

#[test]
fn fill_after_terminal_rejected() {
    let mut o = order(1, 10);
    o.fill(Qty::new(10)).unwrap();
    let err = o.fill(Qty::new(1));
    assert!(matches!(err, Err(NyquestroError::OrderTerminal(1))));
}

#[test]
fn cancel_after_partial() {
    let mut o = order(1, 10);
    o.fill(Qty::new(3)).unwrap();
    o.cancel().unwrap();
    assert_eq!(o.status(), Status::Cancelled);
    assert_eq!(o.remaining(), Qty::new(7));
}

#[test]
fn cancel_after_filled_rejected() {
    let mut o = order(1, 10);
    o.fill(Qty::new(10)).unwrap();
    let err = o.cancel();
    assert!(matches!(err, Err(NyquestroError::OrderTerminal(1))));
}

#[test]
fn zero_quantity_fill_rejected() {
    let mut o = order(1, 10);
    let err = o.fill(Qty::ZERO);
    assert!(matches!(err, Err(NyquestroError::InvalidQuantity)));
    assert_eq!(o.remaining(), Qty::new(10));
}

#[test]
fn zero_quantity_construction_rejected() {
    let err = Order::new(
        OrderID::new(1).unwrap(),
        SYM,
        Side::Buy,
        Px::from_cents(100).unwrap(),
        Qty::ZERO,
        Ts::from_nanos(1),
    );
    assert!(matches!(err, Err(NyquestroError::InvalidQuantity)));
}

#[test]
fn observing_state_does_not_consume() {
    let o = order(1, 10);
    let _id = o.id();
    let _s = o.status();
    let _r = o.remaining();
    // Order is still readable.
    assert_eq!(o.id().value(), 1);
}
