use nyquestro::types::{OrderID, Px, Qty, Side, Ts};

#[test]
fn test_small_order_id_creation() {
    let order_id_small = OrderID::new(17).unwrap();
    assert_eq!(order_id_small.value(), 17);
}

#[test]
fn test_large_order_id_creation() {
    let order_id_large = OrderID::new(7657253765).unwrap();
    assert_eq!(order_id_large.value(), 7657253765);
}

#[test]
fn test_invalid_order_id_creation() {
    let order_id_zero = OrderID::new(0).unwrap_err();
    assert_eq!(order_id_zero, "OrderID cannot be zero.");
}

#[test]
fn test_price_init_dollars_and_conversion() {
    let price = Px::new_from_dollars(717.18).unwrap();
    assert_eq!(price.to_dollars(), 717.18);
    assert_eq!(price.to_cents(), 71718);
}

#[test]
fn test_price_init_cents_and_conversion() {
    let price = Px::new_from_cents(7636275).unwrap();
    assert_eq!(price.to_dollars(), 76362.75);
    assert_eq!(price.to_cents(), 7636275);
}

#[test]
fn test_price_invalid_values() {
    let price_zero_dollars = Px::new_from_dollars(0.0);
    let price_zero_cents = Px::new_from_cents(0);
    assert!(price_zero_dollars.is_err());
    assert!(price_zero_cents.is_err());
}

#[test]
fn test_price_negative_values() {
    let price_negative_dollars = Px::new_from_dollars(-7.4);
    assert!(price_negative_dollars.is_err());
}

#[test]
fn test_quantity_init() {
    let quantity = Qty::new(10);
    assert_eq!(quantity.value(), 10);
}

#[test]
fn test_quantity_can_subtract() {
    let quantity_large = Qty::new(8);
    let quantity_small = Qty::new(4);
    assert!(quantity_large.can_subtract(quantity_small));
    assert!(!quantity_small.can_subtract(quantity_large));
}

#[test]
fn test_quantity_subtraction_logic() {
    let quantity1 = Qty::new(8);
    let quantity2 = Qty::new(4);
    assert_eq!(quantity1.saturating_sub(quantity2).value(), 4);
}

#[test]
fn test_quantity_underflow_protection() {
    let quantity_large = Qty::new(7);
    let quantity_small = Qty::new(4);
    assert_eq!(quantity_small.saturating_sub(quantity_large).value(), 0);
    assert!(!quantity_small.can_subtract(quantity_large));
}

#[test]
fn test_side_opposite() {
    let side_buy = Side::Buy;
    let side_sell = Side::Sell;
    assert_eq!(side_buy.opposite(), side_sell);
    assert_eq!(side_sell.opposite(), side_buy);
}

#[test]
fn test_timestamp_init() {
    let time_now = Ts::now();
    assert!(time_now.nanos() > 0);
}

#[test]
fn test_timestamp_time_moves() {
    let first_time = Ts::now();
    let second_time = Ts::now();
    assert!(first_time < second_time);
}

#[test]
fn test_timestamp_from_nanos() {
    let time_from_nanos = Ts::from_nanos(653756757678648278);
    assert_eq!(time_from_nanos.nanos(), 653756757678648278);
}

#[test]
fn test_timestamp_conversions() {
    let time_in_nanos = Ts::from_nanos(1000000000);
    assert_eq!(time_in_nanos.nanos(), 1000000000);
    assert_eq!(time_in_nanos.micros(), 1000000);
    assert_eq!(time_in_nanos.millis(), 1000);
}

#[test]
fn test_timestamp_time_comparisons() {
    let time_early = Ts::from_nanos(25000);
    let time_late = Ts::from_nanos(50000);
    assert!(time_early.is_before(40000));
    assert!(time_early.is_after(20000));
    assert!(time_early.is_before(time_late.nanos()));
    assert!(time_late.is_after(time_early.nanos()));
    assert_eq!(time_late.duration_since(time_early.nanos()), 25000);
}
