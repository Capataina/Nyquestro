use crate::{
    NyquestroResult,
    types::{OrderID, Px, Qty, Side, Ts},
};

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
        price: Px,
        quantity: Qty,
        side: Side,
        timestamp: Ts,
    },
    Rejected {
        order_id: OrderID,
        price: Px,
        quantity: Qty,
        side: Side,
        reason: OrderRejectionReason,
        timestamp: Ts,
    },
}

impl OrderEvent {
    pub fn new(
        order_id: OrderID,
        price: Px,
        quantity: Qty,
        side: Side,
        timestamp: Ts,
    ) -> NyquestroResult<Self> {
        Ok(OrderEvent::New {
            order_id,
            price,
            quantity,
            side,
            timestamp,
        })
    }
    pub fn get_order_id(&self) -> OrderID {
        match self {
            OrderEvent::New { order_id, .. } => *order_id,
            OrderEvent::Cancelled { order_id, .. } => *order_id,
            OrderEvent::Rejected { order_id, .. } => *order_id,
        }
    }
    pub fn get_price(&self) -> Px {
        match self {
            OrderEvent::New { price, .. } => *price,
            OrderEvent::Cancelled { price, .. } => *price,
            OrderEvent::Rejected { price, .. } => *price,
        }
    }
    pub fn get_quantity(&self) -> Qty {
        match self {
            OrderEvent::New { quantity, .. } => *quantity,
            OrderEvent::Cancelled { quantity, .. } => *quantity,
            OrderEvent::Rejected { quantity, .. } => *quantity,
        }
    }
    pub fn get_side(&self) -> Side {
        match self {
            OrderEvent::New { side, .. } => *side,
            OrderEvent::Cancelled { side, .. } => *side,
            OrderEvent::Rejected { side, .. } => *side,
        }
    }
    pub fn get_timestamp(&self) -> Ts {
        match self {
            OrderEvent::New { timestamp, .. } => *timestamp,
            OrderEvent::Cancelled { timestamp, .. } => *timestamp,
            OrderEvent::Rejected { timestamp, .. } => *timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_order_event_new() {
        let order_event = OrderEvent::new(
            OrderID::new(1).unwrap(),
            Px::new_from_dollars(10.0).unwrap(),
            Qty::new(10),
            Side::Buy,
            Ts::now(),
        )
        .unwrap();
        assert_eq!(order_event.get_order_id(), OrderID::new(1).unwrap());
        assert_eq!(order_event.get_price(), Px::new_from_dollars(10.0).unwrap());
        assert_eq!(order_event.get_quantity(), Qty::new(10));
        assert_eq!(order_event.get_side(), Side::Buy);
    }

    #[test]
    fn test_order_event_cancelled() {
        let order_event = OrderEvent::Cancelled {
            order_id: OrderID::new(1).unwrap(),
            price: Px::new_from_dollars(10.0).unwrap(),
            quantity: Qty::new(10),
            side: Side::Buy,
            timestamp: Ts::now(),
        };
        assert_eq!(order_event.get_order_id(), OrderID::new(1).unwrap());
        assert_eq!(order_event.get_price(), Px::new_from_dollars(10.0).unwrap());
        assert_eq!(order_event.get_quantity(), Qty::new(10));
        assert_eq!(order_event.get_side(), Side::Buy);
    }
    #[test]
    fn test_order_event_rejected() {
        let order_event = OrderEvent::Rejected {
            order_id: OrderID::new(1).unwrap(),
            price: Px::new_from_dollars(10.0).unwrap(),
            quantity: Qty::new(10),
            side: Side::Buy,
            reason: OrderRejectionReason::InvalidQuantity,
            timestamp: Ts::now(),
        };
        assert_eq!(order_event.get_order_id(), OrderID::new(1).unwrap());
        assert_eq!(order_event.get_price(), Px::new_from_dollars(10.0).unwrap());
        assert_eq!(order_event.get_quantity(), Qty::new(10));
        assert_eq!(order_event.get_side(), Side::Buy);
    }
    #[test]
    fn test_order_event_get_order_id() {
        let order_event = OrderEvent::New {
            order_id: OrderID::new(1).unwrap(),
            price: Px::new_from_dollars(10.0).unwrap(),
            quantity: Qty::new(10),
            side: Side::Buy,
            timestamp: Ts::now(),
        };
        assert_eq!(order_event.get_order_id(), OrderID::new(1).unwrap());
        assert_eq!(order_event.get_price(), Px::new_from_dollars(10.0).unwrap());
        assert_eq!(order_event.get_quantity(), Qty::new(10));
        assert_eq!(order_event.get_side(), Side::Buy);
    }
    #[test]
    fn test_order_event_get_price() {
        let order_event = OrderEvent::New {
            order_id: OrderID::new(1).unwrap(),
            price: Px::new_from_dollars(10.0).unwrap(),
            quantity: Qty::new(10),
            side: Side::Buy,
            timestamp: Ts::now(),
        };
        assert_eq!(order_event.get_price(), Px::new_from_dollars(10.0).unwrap());
        assert_eq!(order_event.get_quantity(), Qty::new(10));
        assert_eq!(order_event.get_side(), Side::Buy);
    }
    #[test]
    fn test_order_event_get_quantity() {
        let order_event = OrderEvent::New {
            order_id: OrderID::new(1).unwrap(),
            price: Px::new_from_dollars(10.0).unwrap(),
            quantity: Qty::new(10),
            side: Side::Buy,
            timestamp: Ts::now(),
        };
        assert_eq!(order_event.get_quantity(), Qty::new(10));
        assert_eq!(order_event.get_price(), Px::new_from_dollars(10.0).unwrap());
        assert_eq!(order_event.get_order_id(), OrderID::new(1).unwrap());
        assert_eq!(order_event.get_side(), Side::Buy);
    }
    #[test]
    fn test_order_event_get_side() {
        let order_event = OrderEvent::New {
            order_id: OrderID::new(1).unwrap(),
            price: Px::new_from_dollars(10.0).unwrap(),
            quantity: Qty::new(10),
            side: Side::Buy,
            timestamp: Ts::now(),
        };
        assert_eq!(order_event.get_side(), Side::Buy);
        assert_eq!(order_event.get_quantity(), Qty::new(10));
        assert_eq!(order_event.get_price(), Px::new_from_dollars(10.0).unwrap());
        assert_eq!(order_event.get_order_id(), OrderID::new(1).unwrap());
    }
}
