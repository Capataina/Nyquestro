use crate::{
    NyquestroError, NyquestroResult,
    types::{Px, Qty, Side, Ts},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuoteEvent {
    pub price: Px,
    pub quantity: Qty,
    pub side: Side,
    pub timestamp: Ts,
}

impl QuoteEvent {
    pub fn new(price: Px, quantity: Qty, side: Side, timestamp: Ts) -> NyquestroResult<Self> {
        if quantity.value() == 0 {
            return Err(NyquestroError::InvalidQuantity);
        }

        Ok(QuoteEvent {
            price,
            quantity,
            side,
            timestamp,
        })
    }

    pub fn get_price(&self) -> Px {
        self.price
    }

    pub fn get_quantity(&self) -> Qty {
        self.quantity
    }

    pub fn get_side(&self) -> Side {
        self.side
    }

    pub fn get_timestamp(&self) -> Ts {
        self.timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_quote_event_new() {
        let quote_event = QuoteEvent::new(
            Px::new_from_dollars(10.0).unwrap(),
            Qty::new(10),
            Side::Buy,
            Ts::now(),
        )
        .unwrap();
        assert_eq!(quote_event.get_price(), Px::new_from_dollars(10.0).unwrap());
        assert_eq!(quote_event.get_quantity(), Qty::new(10));
        assert_eq!(quote_event.get_side(), Side::Buy);
    }
}
