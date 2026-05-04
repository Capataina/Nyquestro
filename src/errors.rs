//! Error taxonomy.
//!
//! Every fallible operation in the crate returns [`NyquestroResult<T>`]. The
//! variants are scoped to the failure modes that actually occur — generic
//! `RecoverableError` / `FatalError` catch-alls have been removed.
//!
//! Severity is derived from the variant via [`NyquestroError::severity`]; it
//! is *not* a separate field on the error, and there is one obvious entry
//! point.

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorSeverity {
    /// Caller can retry, reformulate, or surface to the user without
    /// tearing down state.
    Recoverable,
    /// Indicates corrupted state; the engine should refuse further work
    /// from the affected context.
    Fatal,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum NyquestroError {
    // ── primitive validation (recoverable) ─────────────────────────────────
    #[error("OrderID cannot be zero")]
    InvalidOrderId,

    #[error("Symbol must be 1..=8 bytes")]
    InvalidSymbol,

    #[error("Symbol mismatch: order on {actual} routed to book for {expected}")]
    SymbolMismatch { expected: u64, actual: u64 },

    #[error("Price in cents must be non-zero, got {cents}")]
    InvalidPrice { cents: u64 },

    #[error("Price from float must be finite and positive, got {value}")]
    InvalidPriceFloat { value: f64 },

    #[error("Quantity must be non-zero")]
    InvalidQuantity,

    #[error("Quantity arithmetic would overflow")]
    QuantityOverflow,

    // ── order lifecycle (recoverable) ──────────────────────────────────────
    #[error("Fill {fill} exceeds remaining {remaining} on order {order_id}")]
    OverFill {
        order_id: u64,
        fill: u32,
        remaining: u32,
    },

    #[error("Cannot transition order {order_id} from {from} to {to}")]
    InvalidStatusTransition {
        order_id: u64,
        from: &'static str,
        to: &'static str,
    },

    #[error("Order {0} is already terminal and cannot be modified")]
    OrderTerminal(u64),

    // ── matching engine (recoverable) ──────────────────────────────────────
    #[error("Self-match rejected: order {0} would match against itself")]
    SelfMatch(u64),

    #[error("Order {0} not found in book")]
    OrderNotFound(u64),

    #[error("Order {0} already exists in book")]
    OrderAlreadyExists(u64),

    #[error("Price level {price_cents} not present in book")]
    PriceLevelMissing { price_cents: u64 },

    #[error("PriceLevel mismatch: expected {expected_cents}, got {actual_cents}")]
    PriceLevelMismatch {
        expected_cents: u64,
        actual_cents: u64,
    },

    // ── invariant breakage (fatal) ─────────────────────────────────────────
    #[error("Internal invariant violated: {0}")]
    InvariantViolation(&'static str),
}

impl NyquestroError {
    /// Single source of truth for severity classification. Add new variants
    /// here when extending [`NyquestroError`].
    pub fn severity(&self) -> ErrorSeverity {
        use NyquestroError::*;
        match self {
            // Caller-facing input validation + lifecycle + matching errors.
            InvalidOrderId
            | InvalidSymbol
            | SymbolMismatch { .. }
            | InvalidPrice { .. }
            | InvalidPriceFloat { .. }
            | InvalidQuantity
            | QuantityOverflow
            | OverFill { .. }
            | InvalidStatusTransition { .. }
            | OrderTerminal(_)
            | SelfMatch(_)
            | OrderNotFound(_)
            | OrderAlreadyExists(_)
            | PriceLevelMissing { .. }
            | PriceLevelMismatch { .. } => ErrorSeverity::Recoverable,

            // Bug in the engine itself.
            InvariantViolation(_) => ErrorSeverity::Fatal,
        }
    }

    #[inline]
    pub fn is_recoverable(&self) -> bool {
        self.severity() == ErrorSeverity::Recoverable
    }

    #[inline]
    pub fn is_fatal(&self) -> bool {
        self.severity() == ErrorSeverity::Fatal
    }
}

pub type NyquestroResult<T> = Result<T, NyquestroError>;

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recoverable_variants_classify_recoverable() {
        let cases = [
            NyquestroError::InvalidOrderId,
            NyquestroError::InvalidSymbol,
            NyquestroError::SymbolMismatch {
                expected: 1,
                actual: 2,
            },
            NyquestroError::InvalidPrice { cents: 0 },
            NyquestroError::InvalidPriceFloat { value: -1.0 },
            NyquestroError::InvalidQuantity,
            NyquestroError::QuantityOverflow,
            NyquestroError::OverFill {
                order_id: 1,
                fill: 5,
                remaining: 2,
            },
            NyquestroError::InvalidStatusTransition {
                order_id: 1,
                from: "FILLED",
                to: "OPEN",
            },
            NyquestroError::OrderTerminal(1),
            NyquestroError::SelfMatch(1),
            NyquestroError::OrderNotFound(1),
            NyquestroError::OrderAlreadyExists(1),
            NyquestroError::PriceLevelMissing { price_cents: 100 },
            NyquestroError::PriceLevelMismatch {
                expected_cents: 100,
                actual_cents: 101,
            },
        ];
        for case in cases {
            assert!(case.is_recoverable(), "{case:?} should be recoverable");
            assert!(!case.is_fatal());
        }
    }

    #[test]
    fn invariant_violation_is_fatal() {
        let e = NyquestroError::InvariantViolation("test");
        assert!(e.is_fatal());
        assert_eq!(e.severity(), ErrorSeverity::Fatal);
    }

    #[test]
    fn errors_format_human_readably() {
        let e = NyquestroError::OverFill {
            order_id: 7,
            fill: 100,
            remaining: 3,
        };
        assert_eq!(e.to_string(), "Fill 100 exceeds remaining 3 on order 7");
    }
}
