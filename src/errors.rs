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

    #[error("Order already exists")]
    OrderAlreadyExists,

    #[error("Order cannot be cancelled")]
    OrderCannotBeCancelled,

    #[error("Matching engine error")]
    MatchingEngineError,

    #[error("Recoverable error")]
    RecoverableError,

    #[error("Fatal error")]
    FatalError,

    #[error("Error severity cannot be determined")]
    ErrorSeverityCannotBeDetermined,

    #[error("Error severity is {severity}")]
    ErrorSeverity { severity: &'static str },
}

pub type NyquestroResult<T> = Result<T, NyquestroError>;
