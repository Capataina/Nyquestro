# Events and Error Model

**Last Updated:** 2026-02-02

## Scope / Purpose

- Describe the current event frame types and error taxonomy used across the crate.
- Capture how errors are classified and what validation exists at event boundaries.

## Current Implemented System

- `NyquestroError` is implemented in `src/errors.rs` using `thiserror::Error`.
- `NyquestroResult<T>` is a type alias for `Result<T, NyquestroError>`.
- `ErrorSeverity` is an enum with `Recoverable` and `Fatal` variants.
- Error classification is implemented as `severity(error: &NyquestroError) -> ErrorSeverity`.
- `severity()` currently marks validation-like errors as `Recoverable` and several operational errors as `Fatal`.
- `src/events/mod.rs` exposes three event modules: `fill_event`, `quote_event`, and `order_event`.
- `FillEvent` is an immutable `Copy` struct with buyer/seller IDs, price, quantity, and timestamp.
- `FillEvent::new(...) -> NyquestroResult<FillEvent>` rejects zero quantity via `NyquestroError::InvalidQuantity`.
- `FillEvent` does not currently reject self-match IDs because the buyer/seller equality check is commented out.
- `QuoteEvent` is an immutable `Copy` struct with price, quantity, side, and timestamp.
- `QuoteEvent::new(...) -> NyquestroResult<QuoteEvent>` rejects zero quantity via `NyquestroError::InvalidQuantity`.
- `OrderEvent` is a `Copy` enum with `New`, `Cancelled`, and `Rejected` variants and a `new()` constructor for the `New` variant.
- `OrderEvent::new(...) -> NyquestroResult<OrderEvent>` does not perform input validation and always returns `Ok`.
- Unit tests exist alongside event types in `src/events/*` and for `severity()` in `src/errors.rs`.
- Integration tests in `tests/event_tests.rs` cover event construction, equality/copy semantics, and basic error classification.

## Implemented Outputs / Artifacts (if applicable)

- None.

## In Progress / Partially Implemented

- Event constructors validate only a subset of invariants, and `OrderEvent::new()` currently validates none.
- `NyquestroError` contains both specific error cases and generic variants (`RecoverableError`, `FatalError`, and `ErrorSeverity { .. }`) that overlap conceptually with `ErrorSeverity`.
- There is no end-to-end event emission pipeline from order placement or matching, and events are currently constructed only in tests and ad-hoc code.

## Planned / Missing / To Be Changed

- Decide which event invariants are enforced in constructors versus at higher-level engine boundaries.
- Decide whether `ErrorSeverity` should be computed via a method on `NyquestroError` rather than a free function.
- Decide whether `NyquestroError` should retain generic “recoverable/fatal” variants or rely on classification for that axis.
- Add integration coverage for engine-level behaviour once an order book/matcher exists to emit events deterministically.

## Notes / Design Considerations (optional)

- Keeping event frames `Copy` and allocation-free is compatible with later fan-out and replay testing, but correctness depends on where validation is enforced.

## Discarded / Obsolete / No Longer Relevant

- None.

