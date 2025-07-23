use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum NyquestroError {
    #[error("Order ID cannot be zero")]
    InvalidOrderID,

    #[error("Price must be correct, got {value}")]
    InvalidPrice { value: f64 },

    #[error("Quantity cannot be zero")]
    InvalidQuantity,

    #[error("Order with ID {id} not found")]
    OrderNotFound { id: u64 },
}

pub type NyquestroResult<T> = Result<T, NyquestroError>;
