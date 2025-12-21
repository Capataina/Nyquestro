use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Recoverable,
    Fatal,
}

pub fn severity(error: &NyquestroError) -> ErrorSeverity {
    match error {
        NyquestroError::InvalidOrderID => ErrorSeverity::Recoverable,
        NyquestroError::InvalidPrice { .. } => ErrorSeverity::Recoverable,
        NyquestroError::InvalidQuantity => ErrorSeverity::Recoverable,
        NyquestroError::OrderNotFound { .. } => ErrorSeverity::Fatal,
        NyquestroError::OrderAlreadyExists => ErrorSeverity::Fatal,
        NyquestroError::OrderCannotBeCancelled => ErrorSeverity::Fatal,
        NyquestroError::MatchingEngineError => ErrorSeverity::Fatal,
        NyquestroError::RecoverableError => ErrorSeverity::Recoverable,
        NyquestroError::FatalError => ErrorSeverity::Fatal,
        NyquestroError::ErrorSeverityCannotBeDetermined => ErrorSeverity::Fatal,
        NyquestroError::ErrorSeverity { .. } => ErrorSeverity::Recoverable,
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_severity() {
        assert_eq!(
            severity(&NyquestroError::InvalidOrderID),
            ErrorSeverity::Recoverable
        );
        assert_eq!(
            severity(&NyquestroError::InvalidPrice { value: 10.0 }),
            ErrorSeverity::Recoverable
        );
        assert_eq!(
            severity(&NyquestroError::InvalidQuantity),
            ErrorSeverity::Recoverable
        );
        assert_eq!(
            severity(&NyquestroError::OrderNotFound { id: 1 }),
            ErrorSeverity::Fatal
        );
    }
    #[test]
    fn test_severity_fatal() {
        assert_eq!(
            severity(&NyquestroError::OrderAlreadyExists),
            ErrorSeverity::Fatal
        );
        assert_eq!(
            severity(&NyquestroError::OrderCannotBeCancelled),
            ErrorSeverity::Fatal
        );
        assert_eq!(
            severity(&NyquestroError::MatchingEngineError),
            ErrorSeverity::Fatal
        );
        assert_eq!(
            severity(&NyquestroError::RecoverableError),
            ErrorSeverity::Recoverable
        );
        assert_eq!(severity(&NyquestroError::FatalError), ErrorSeverity::Fatal);
    }
    #[test]
    fn test_severity_recoverable() {
        assert_eq!(
            severity(&NyquestroError::ErrorSeverityCannotBeDetermined),
            ErrorSeverity::Fatal
        );
        assert_eq!(
            severity(&NyquestroError::ErrorSeverity {
                severity: "Recoverable"
            }),
            ErrorSeverity::Recoverable
        );
    }
    #[test]
    fn test_severity_error_severity_cannot_be_determined() {
        assert_eq!(
            severity(&NyquestroError::ErrorSeverityCannotBeDetermined),
            ErrorSeverity::Fatal
        );
    }
    #[test]
    fn test_severity_error_severity_recoverable() {
        assert_eq!(
            severity(&NyquestroError::ErrorSeverity {
                severity: "Recoverable"
            }),
            ErrorSeverity::Recoverable
        );
    }
}
