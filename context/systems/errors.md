# Errors

*Maturity: comprehensive · Stability: stable*

## Scope / Purpose

`src/errors.rs` defines the crate's error taxonomy: a single `NyquestroError` enum, an `ErrorSeverity` classifier, and the `NyquestroResult<T>` alias used by every fallible operation. Severity is derived from the variant via a method, not stored in the error.

## Boundaries / Ownership

- **Owns:** `NyquestroError` (16 variants today), `ErrorSeverity` (`Recoverable` / `Fatal`), `severity(&self)` method, `is_recoverable`/`is_fatal` shortcuts, the `NyquestroResult<T>` alias.
- **Does not own:** error *handling* (callers decide whether to log, retry, increment a counter, etc.). The `ui::app` module currently maps errors to `metrics.record_rejects(1)`; that policy lives in the UI layer, not here.
- **Imported by:** every fallible constructor and every method that returns `NyquestroResult<T>` — i.e. essentially every non-accessor function in the crate.

## Current Implemented Reality

The variants partition cleanly into three groups:

| Group | Variants | Severity |
|-------|----------|----------|
| Primitive validation | `InvalidOrderId`, `InvalidSymbol`, `InvalidPrice { cents }`, `InvalidPriceFloat { value }`, `InvalidQuantity`, `QuantityOverflow` | Recoverable |
| Order lifecycle | `OverFill { order_id, fill, remaining }`, `InvalidStatusTransition { order_id, from, to }`, `OrderTerminal(u64)` | Recoverable |
| Matching engine | `SelfMatch(u64)`, `SymbolMismatch`, `OrderNotFound(u64)`, `OrderAlreadyExists(u64)`, `PriceLevelMissing { price_cents }`, `PriceLevelMismatch { expected_cents, actual_cents }` | Recoverable |
| Internal invariant | `InvariantViolation(&'static str)` | Fatal |

`thiserror::Error` provides `Display` and `Error::source` automatically. Every variant's `#[error("…")]` produces a single-line human-readable message that includes the salient fields.

## Key Interfaces / Data Flow

```rust
pub type NyquestroResult<T> = Result<T, NyquestroError>;
impl NyquestroError {
    pub fn severity(&self) -> ErrorSeverity;
    pub fn is_recoverable(&self) -> bool;
    pub fn is_fatal(&self) -> bool;
}
pub enum ErrorSeverity { Recoverable, Fatal }
```

`severity` is a `match` on the variant — adding a new variant requires updating it in one place (single source of truth). The match is exhaustive; the compiler enforces this.

Error flow is universally:

```
constructor / mutator
  └─ validates input
       └─ Err(NyquestroError::SomeVariant) ──► caller
            ├─ recoverable: counter++/log/UI feedback
            └─ fatal:        bug, should not occur in production
```

## Implemented Outputs / Artifacts

- `NyquestroError` (16 variants), all `Clone + Debug + PartialEq` — fields chosen to be `Copy` where possible so error matching is cheap.
- `severity` classifier with single-source-of-truth design.
- 3 inline unit tests covering: every recoverable variant classifies as recoverable, `InvariantViolation` is fatal, error formatting renders the field values.

## Known Issues / Active Risks

- `InvariantViolation(&'static str)` is the only Fatal variant today. If/when a second invariant breaks, the static-str carrier will become limiting (cannot embed runtime values without a format-step that allocates). Acceptable for now because the variant is exclusively used to mark genuine bugs.
- The error type is `Clone` but not `Copy` (because `InvalidPriceFloat { value: f64 }` and string-bearing variants prevent `Copy`). Most call sites pass errors by value to a `Result` so this is not a hot-path concern; document as a constraint if a future caller needs to fan out the same error to many recipients.

### Downstream impact

- A `SelfMatch` error from `OrderBook::submit_limit` increments `metrics.record_rejects(1)` and surfaces as an `OrderEvent::Rejected` with `OrderRejectionReason::SelfMatch` — the dashboard's reject counter ticks up, the engine continues running.
- An `InvariantViolation` from `PriceLevel::record_execution` (the only producer today) would propagate up through `OrderBook::submit_limit` and bubble to `App::handle_submit`, which currently swallows it as a generic reject. **This is the load-bearing path for "engine corrupted state" — we should consider a panic-on-fatal wrapper at the App boundary.** See *Planned* below.

## Partial / In Progress

None.

## Planned / Missing / Likely Changes

- **Panic-on-fatal at the App boundary.** `App::handle_submit` currently treats every error identically (counts a reject and continues). A fatal `InvariantViolation` should at minimum log loudly and ideally tear down cleanly with a panic-handler that restores the terminal. Not yet implemented.
- **Multi-instrument errors** (e.g. `InstrumentNotFound`) when the book becomes multi-instrument.
- **Wire-protocol errors** when the network gateway lands.

## Durable Notes / Discarded Approaches

- **Generic catch-all variants were removed.** The prior taxonomy had `RecoverableError`, `FatalError`, `ErrorSeverity { severity: &str }`, `ErrorSeverityCannotBeDetermined` — variants that duplicated the classification concept *inside the data model*. The replacement collapses them to one classifier method on the variant. This is not a stylistic preference; it removes a class of bugs where two different code paths would produce different `RecoverableError` variants for the same condition with no way to distinguish them.
- **`severity` is a method, not a free function.** The prior codebase had `pub fn severity(error: &NyquestroError) -> ErrorSeverity`. Method form is canonical Rust ("there is one obvious entry point") and doc-discoverable from the type.
- **`#[error("…")]` messages embed field values.** This was a deliberate choice — at the UI layer, error messages may reach the user; at the test layer, assertions become readable (`assert_eq!(err.to_string(), "Fill 100 exceeds remaining 3 on order 7")`).

## Obsolete / No Longer Relevant

- `NyquestroError::RecoverableError` (the variant) — removed; classification is now a method.
- `NyquestroError::FatalError` (the variant) — removed for the same reason.
- `NyquestroError::ErrorSeverity { severity: &'static str }` — removed; the existence of a *string-typed severity* inside the data model was a code smell that we deliberately deleted.
- `NyquestroError::ErrorSeverityCannotBeDetermined` — removed; the new severity method is exhaustive over variants and cannot fail.
- `NyquestroError::MatchingEngineError` (a generic catch-all) — removed; the matching engine's specific failure modes now have specific variants (`SelfMatch`, `OrderNotFound`, `PriceLevelMissing`, `PriceLevelMismatch`).
