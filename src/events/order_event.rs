use crate::types::{OrderID, Px, Qty, Side, Ts};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderRejectionReason {
    InvalidQuantity,
    InvalidPrice,
    InvalidSide,
    InvalidTimestamp,
    InvalidOrderID,
    InvalidOrderStatus,
    InvalidOrderType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderEvent {
    New {
        order_id: OrderID,
        price: Px,
        quantity: Qty,
        side: Side,
        timestamp: Ts,
    },
    Cancelled {
        order_id: OrderID,
        timestamp: Ts,
    },
    Rejected {
        order_id: OrderID,
        reason: OrderRejectionReason,
        timestamp: Ts,
    },
}
