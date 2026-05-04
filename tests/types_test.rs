//! Integration tests covering the public surface of `types.rs` and
//! cross-module interactions.

use nyquestro::types::{OrderID, Px, Qty, Side, Status, Ts};

#[test]
fn order_id_round_trip() {
    let id = OrderID::new(7).unwrap();
    assert_eq!(id.value(), 7);
    assert_eq!(format!("{id}"), "#7");
}

#[test]
fn px_canonical_paths() {
    let from_cents = Px::from_cents(12_345).unwrap();
    assert_eq!(from_cents.cents(), 12_345);
    assert_eq!(from_cents.to_dollars(), 123.45);

    let from_dollars = Px::from_dollars(50.25).unwrap();
    assert_eq!(from_dollars.cents(), 5025);

    // Round-half-to-even / nearest cents — historical truncation bug fixed.
    assert_eq!(Px::from_dollars(10.999).unwrap().cents(), 1100);
}

#[test]
fn qty_supports_zero_for_remaining_qty() {
    let z = Qty::ZERO;
    assert!(z.is_zero());
    assert_eq!(z.value(), 0);
}

#[test]
fn ts_monotonic_within_a_call() {
    let a = Ts::now();
    let b = Ts::now();
    assert!(b.nanos() >= a.nanos());
}

#[test]
fn side_opposite_round_trip() {
    assert_eq!(Side::Buy.opposite(), Side::Sell);
    assert_eq!(Side::Sell.opposite(), Side::Buy);
}

#[test]
fn status_state_machine() {
    assert!(Status::Open.can_transition_to(Status::PartiallyFilled));
    assert!(!Status::FullyFilled.can_transition_to(Status::Open));
    assert!(Status::PartiallyFilled.is_active());
    assert!(Status::Cancelled.is_terminal());
}
