use crate::{
    NyquestroError, NyquestroResult,
    order::Order,
    types::{Px, Qty},
};

pub struct PriceLevel {
    price: Px,
    orders: Vec<Order>,
    total_quantity: Qty,
}

impl PriceLevel {
    pub fn new(price: Px) -> NyquestroResult<Self> {
        Ok(PriceLevel {
            price,
            orders: Vec::new(),
            total_quantity: Qty::new(0),
        })
    }

    pub fn add_order(&mut self, order: Order) -> NyquestroResult<()> {
        if order.get_price() != self.price {
            return Err(NyquestroError::InvalidPrice {
                value: order.get_price().to_dollars(),
            });
        }

        self.orders.push(order.clone());

        self.total_quantity =
            Qty::new(self.total_quantity.value() + order.get_remaining_quantity().value());

        Ok(())
    }

    pub fn get_price(&self) -> NyquestroResult<Px> {
        Ok(self.price)
    }

    pub fn get_orders(&self) -> NyquestroResult<Vec<Order>> {
        Ok(self.orders.clone())
    }

    pub fn get_total_quantity(&self) -> NyquestroResult<Qty> {
        Ok(self.total_quantity)
    }
}
