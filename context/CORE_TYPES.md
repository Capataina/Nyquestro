# Core Types

**Last Updated:** 2026-02-02

## Scope / Purpose

- Define shared primitive types and invariants for the rest of the crate.
- Provide simple conversion helpers for price, quantity, and timestamp representations.

## Current Implemented System

- Type primitives are defined in `src/types.rs`.
- `OrderID` is a `u64` wrapper that rejects zero via `OrderID::new(id) -> Result<OrderID, &'static str>`.
- `Side` is an enum with `Buy` and `Sell` variants and an `opposite()` helper.
- `Px` is a `u64` wrapper representing cents with constructors from dollars or cents that reject non-positive values via `Result<Px, &'static str>`.
- `Qty` is a `u32` wrapper with `new(value) -> Qty` and no non-zero validation.
- `Qty::can_subtract(other)` is a comparison helper used to avoid underflow.
- `Qty::saturating_sub(other)` returns `Qty(0)` when the subtraction would underflow.
- `Ts` is a `u64` wrapper representing nanoseconds since the UNIX epoch.
- `Ts::now()` reads `SystemTime` and panics on system clock errors via `unwrap()`.
- `Ts` provides `is_before(u64)`, `is_after(u64)`, and `duration_since(u64)` helpers that take raw nanoseconds.
- `Ts` provides `nanos()`, `micros()`, and `millis()` accessors derived from the stored nanoseconds.
- `Ts::to_utc_datetime()` converts to `chrono::DateTime<Utc>`.
- `Status` is an enum with `Open`, `PartiallyFilled`, `FullyFilled`, and `Cancelled` variants.

## Implemented Outputs / Artifacts (if applicable)

- None.

## In Progress / Partially Implemented

- Primitive constructors are split between `NyquestroError`-based APIs and `&'static str` string errors.
- `Px::new_from_dollars()` converts floating-point dollars to cents via a cast, which truncates rather than rounds.
- `Qty::new(0)` is permitted even though other parts of the system treat zero quantity as invalid.
- `Ts::now()` panics on time retrieval failures rather than returning a classified error.

## Planned / Missing / To Be Changed

- Decide whether `OrderID` and `Px` constructors should return `NyquestroResult<T>` to align error handling.
- Decide whether `Px` construction should avoid floating-point inputs in core APIs to prevent rounding ambiguity.
- Decide whether `Qty::new()` should validate non-zero, or whether non-zero is enforced only at order and event boundaries.
- Decide whether `Ts::now()` should avoid panics by returning a recoverable error when the system clock is invalid.

## Notes / Design Considerations (optional)

- Keeping core primitives allocation-free and `Copy`-friendly supports later low-latency goals without forcing premature data-structure choices.

## Discarded / Obsolete / No Longer Relevant

- None.

