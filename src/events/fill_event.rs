use crate::{
    NyquestroError, NyquestroResult,
    types::{OrderID, Px, Qty, Ts},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillEvent {
    pub buyer_order_id: OrderID,
    pub seller_order_id: OrderID,
    pub price: Px,
    pub quantity: Qty,
    pub timestamp: Ts,
}

impl FillEvent {
    pub fn new(
        buyer_order_id: OrderID,
        seller_order_id: OrderID,
        price: Px,
        quantity: Qty,
        timestamp: Ts,
    ) -> NyquestroResult<Self> {
        if quantity.value() == 0 {
            return Err(NyquestroError::InvalidQuantity);
        }

        // if buyer_order_id == seller_order_id {
        //    return Err(NyquestroError::InvalidOrderID);
        //}

        Ok(FillEvent {
            buyer_order_id,
            seller_order_id,
            price,
            quantity,
            timestamp,
        })
    }

    pub fn get_buyer_order_id(&self) -> OrderID {
        self.buyer_order_id
    }

    pub fn get_seller_order_id(&self) -> OrderID {
        self.seller_order_id
    }

    pub fn get_price(&self) -> Px {
        self.price
    }

    pub fn get_quantity(&self) -> Qty {
        self.quantity
    }

    pub fn get_timestamp(&self) -> Ts {
        self.timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_event_new() {
        let fill_event = FillEvent::new(
            OrderID::new(1).unwrap(),
            OrderID::new(2).unwrap(),
            Px::new_from_dollars(10.0).unwrap(),
            Qty::new(10),
            Ts::now(),
        )
        .unwrap();
        assert_eq!(fill_event.get_buyer_order_id(), OrderID::new(1).unwrap());
        assert_eq!(fill_event.get_seller_order_id(), OrderID::new(2).unwrap());
        assert_eq!(fill_event.get_price(), Px::new_from_dollars(10.0).unwrap());
        assert_eq!(fill_event.get_quantity(), Qty::new(10));
    }
}
