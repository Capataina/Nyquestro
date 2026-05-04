//! Integration tests for `PriceLevel`.

use nyquestro::book::PriceLevel;
use nyquestro::errors::NyquestroError;
use nyquestro::order::Order;
use nyquestro::types::{OrderID, Px, Qty, Side, Symbol, Ts};

const SYM: Symbol = Symbol::from_const("TEST");

fn order(id: u64, price_cents: u64, qty: u32, ts_nanos: u64) -> Order {
    Order::new(
        OrderID::new(id).unwrap(),
        SYM,
        Side::Buy,
        Px::from_cents(price_cents).unwrap(),
        Qty::new(qty),
        Ts::from_nanos(ts_nanos),
    )
    .unwrap()
}

#[test]
fn empty_state() {
    let lvl = PriceLevel::new(Px::from_cents(100).unwrap());
    assert!(lvl.is_empty());
    assert_eq!(lvl.len(), 0);
    assert_eq!(lvl.total_quantity(), Qty::ZERO);
    assert!(lvl.front().is_none());
}

#[test]
fn fifo_ordering() {
    let mut lvl = PriceLevel::new(Px::from_cents(100).unwrap());
    lvl.push_back(order(1, 100, 5, 1)).unwrap();
    lvl.push_back(order(2, 100, 3, 2)).unwrap();
    lvl.push_back(order(3, 100, 2, 3)).unwrap();

    assert_eq!(lvl.front().unwrap().id().value(), 1);
    let popped = lvl.pop_front().unwrap();
    assert_eq!(popped.id().value(), 1);
    assert_eq!(lvl.front().unwrap().id().value(), 2);
}

#[test]
fn total_quantity_invariant() {
    let mut lvl = PriceLevel::new(Px::from_cents(100).unwrap());
    lvl.push_back(order(1, 100, 5, 1)).unwrap();
    lvl.push_back(order(2, 100, 3, 2)).unwrap();
    assert_eq!(lvl.total_quantity(), Qty::new(8));

    lvl.record_execution(Qty::new(2)).unwrap();
    assert_eq!(lvl.total_quantity(), Qty::new(6));

    lvl.pop_front().unwrap(); // removes id=1, but its remaining is still 5 (we only recorded execution; in real flow the order would have been mutated too)
    // In a realistic flow `front_mut().fill()` and `record_execution` go
    // hand-in-hand; this test isolates the PriceLevel invariants only.
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
fn remove_by_id_keeps_fifo() {
    let mut lvl = PriceLevel::new(Px::from_cents(100).unwrap());
    lvl.push_back(order(1, 100, 5, 1)).unwrap();
    lvl.push_back(order(2, 100, 3, 2)).unwrap();
    lvl.push_back(order(3, 100, 7, 3)).unwrap();

    lvl.remove_by_id(OrderID::new(2).unwrap()).unwrap();
    let ids: Vec<_> = lvl.iter().map(|o| o.id().value()).collect();
    assert_eq!(ids, vec![1, 3]);
    assert_eq!(lvl.total_quantity(), Qty::new(12));
}

#[test]
fn front_mut_allows_in_place_fill() {
    let mut lvl = PriceLevel::new(Px::from_cents(100).unwrap());
    lvl.push_back(order(1, 100, 5, 1)).unwrap();
    let front = lvl.front_mut().unwrap();
    front.fill(Qty::new(3)).unwrap();
    assert_eq!(lvl.front().unwrap().remaining(), Qty::new(2));
    // Caller is responsible for record_execution; PriceLevel does not snoop.
    lvl.record_execution(Qty::new(3)).unwrap();
    assert_eq!(lvl.total_quantity(), Qty::new(2));
}
