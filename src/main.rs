use std::{thread, time::Duration};

use nyquestro::{
    order::Order,
    types::{OrderID, Px, Qty, Side, Ts},
};

fn main() {
    let time_now = Ts::now();
    thread::sleep(Duration::from_millis(500));
    let time_after_sleep = Ts::now();

    println!("{}", time_after_sleep.duration_since(time_now.nanos()));

    let debug_order_id = OrderID::new(1).unwrap();
    let debug_order_side = Side::Buy;
    let debug_order_price = Px::new_from_dollars(15.0).unwrap();
    let debug_order_quantity = Qty::new(10);
    let mut debug_order = Order::new(
        debug_order_id,
        debug_order_side,
        debug_order_price,
        debug_order_quantity,
    )
    .unwrap();

    println!(
        "Order ID: {:?}, Order Side: {:?}, Order Price: {:?}, Order Quantity: {:?}, Order Remaining Quantity: {:?}, Order Status: {:?}.",
        debug_order.clone().get_order_id().value(),
        debug_order.clone().get_side(),
        debug_order.clone().get_price().to_dollars(),
        debug_order.clone().get_quantity().value(),
        debug_order.clone().get_remaining_quantity().value(),
        debug_order.clone().get_status()
    );

    debug_order.fill(Qty::new(4)).unwrap();
    println!(
        "Fill Event: Buyer Order ID: {:?}, Seller Order ID: {:?}, Price: {:?}, Quantity: {:?}, Timestamp: {:?}.",
        debug_order.get_order_id().value(),
        debug_order.get_order_id().value(),
        debug_order.get_price().to_dollars(),
        debug_order.get_quantity().value(),
        debug_order.get_timestamp().duration_since(time_now.nanos())
    );

    println!(
        "Order ID: {:?}, Order Side: {:?}, Order Price: {:?}, Order Quantity: {:?}, Order Remaining Quantity: {:?}, Order Status: {:?}.",
        debug_order.clone().get_order_id().value(),
        debug_order.clone().get_side(),
        debug_order.clone().get_price().to_dollars(),
        debug_order.clone().get_quantity().value(),
        debug_order.clone().get_remaining_quantity().value(),
        debug_order.clone().get_status()
    );

    debug_order.fill(Qty::new(6)).unwrap();
    println!(
        "Fill Event: Buyer Order ID: {:?}, Seller Order ID: {:?}, Price: {:?}, Quantity: {:?}, Timestamp: {:?}.",
        debug_order.get_order_id().value(),
        debug_order.get_order_id().value(),
        debug_order.get_price().to_dollars(),
        debug_order.get_quantity().value(),
        debug_order.get_timestamp().duration_since(time_now.nanos())
    );

    println!(
        "Order ID: {:?}, Order Side: {:?}, Order Price: {:?}, Order Quantity: {:?}, Order Remaining Quantity: {:?}, Order Status: {:?}.",
        debug_order.clone().get_order_id().value(),
        debug_order.clone().get_side(),
        debug_order.clone().get_price().to_dollars(),
        debug_order.clone().get_quantity().value(),
        debug_order.clone().get_remaining_quantity().value(),
        debug_order.clone().get_status()
    );
}
