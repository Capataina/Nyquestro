use crate::types::{OrderID, Px, Qty, Side, Status, Ts};

#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    order_id: OrderID,
    side: Side,
    price: Px,
    quantity: Qty,
    remaining_quantity: Qty,
    timestamp: Ts,
    status: Status,
}

impl Order {
    pub fn new(
        order_id: OrderID,
        side: Side,
        price: Px,
        quantity: Qty,
    ) -> Result<Order, &'static str> {
        if quantity.value() == 0 {
            return Err("Orders with zero or less quantity are invalid.");
        }

        Ok(Order {
            order_id,
            side,
            price,
            quantity,
            remaining_quantity: quantity,
            timestamp: Ts::now(),
            status: Status::Open,
        })
    }

    pub fn update_status(&mut self) {
        if self.quantity.value() == self.remaining_quantity.value() {
            self.status = Status::Open
        } else if self.quantity.value() > self.remaining_quantity.value()
            && self.remaining_quantity.value() != Qty::new(0).value()
        {
            self.status = Status::PartiallyFilled
        } else {
            self.status = Status::FullyFilled
        }
    }

    pub fn fill(&mut self, fill_amount: Qty) -> String {
        self.remaining_quantity = self.remaining_quantity.saturating_sub(fill_amount);
        self.update_status();

        format!(
            "Filled {} number of orders for the order id {:?}. Current status: {:?}.",
            fill_amount.value(),
            self.order_id,
            self.status
        )
    }

    pub fn get_order_id(&self) -> OrderID {
        self.order_id
    }

    pub fn get_side(&self) -> Side {
        self.side
    }

    pub fn get_price(&self) -> Px {
        self.price
    }

    pub fn get_quantity(&self) -> Qty {
        self.quantity
    }

    pub fn get_remaining_quantity(&self) -> Qty {
        self.remaining_quantity
    }

    pub fn get_timestamp(&self) -> Ts {
        self.timestamp
    }

    pub fn get_status(self) -> Status {
        self.status
    }
}
