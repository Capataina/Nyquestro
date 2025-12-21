# 001: Immutable Event Frames and Error Enum Expansion

**Status:** planned  
**Last Updated:** 2024

---

## Goal and Scope

### What Will Exist When Complete

1. **Immutable Event Frames**

   - Zero-allocation, `Copy` structs for all matching engine outputs
   - `FillEvent` – records when orders match (buyer, seller, price, quantity, timestamp)
   - `QuoteEvent` – best bid/ask changes (price, quantity, side, timestamp)
   - `OrderEvent` – order lifecycle events (new, cancelled, rejected with reason)
   - All events use existing type primitives (`OrderID`, `Px`, `Qty`, `Ts`, `Side`)

2. **Error Enum Expansion**
   - Extended `NyquestroError` with matching-related error variants
   - Classification system: `Recoverable` vs `Fatal` error categories
   - Recovery strategy metadata (whether engine can continue or must shutdown)
   - Migration of `Order::new()` from `Result<Order, &'static str>` to `NyquestroResult<Order>`

### Explicitly Out of Scope

- Matching engine implementation (this is just the event/error infrastructure)
- Event serialization formats (JSON, binary, etc.) – that comes later
- Event publishing/consumption mechanisms (ring bus, multicast, etc.)
- Error recovery handlers (just classification)
- Performance optimisations beyond zero-allocation (SIMD, etc.)

### Deliverables Checklist

- [ ] `src/events.rs` module with `FillEvent`, `QuoteEvent`, `OrderEvent` structs
- [ ] All event types are `Copy`, use only existing primitives, zero-allocation
- [ ] Extended `NyquestroError` enum with matching-related variants
- [ ] Error classification system (`Recoverable`/`Fatal` categories)
- [ ] `Order::new()` migrated to use `NyquestroResult<Order>`
- [ ] `Order::fill()` signature updated to support event emission (or return `FillEvent`)
- [ ] Unit tests for all event types (construction, field access)
- [ ] Unit tests for error classification logic
- [ ] Integration test showing event emission from order operations

---

## Context and Justification

### Why This Work Matters Architecturally

**Event Frames:**

- The matching engine must produce structured outputs for market data, audit logs, and testing
- Current `Order::fill()` returns `String` – this allocates and isn't machine-readable
- Zero-allocation events are critical for ultra-low-latency design (no heap allocations in hot path)
- Immutable events enable deterministic replay testing (golden output hashing)

**Error Classification:**

- Matching engines need to distinguish recoverable errors (bad input, continue) from fatal errors (corruption, shutdown)
- Current error enum only covers validation, not runtime matching errors
- Classification enables proper error handling strategies (retry, circuit breaker, graceful shutdown)

### What This Enables Next

- Matching engine can emit structured events instead of strings
- Market data publishers can consume `FillEvent` and `QuoteEvent` frames
- Audit trail can record all events deterministically
- Error handling can make informed decisions (continue vs shutdown)
- Testing can verify event sequences match expected outputs

### Alternatives Considered and Why Rejected

1. **Allocated event structs (not `Copy`)**
   - Rejected: violates zero-allocation requirement for hot path
2. **String-based events (current approach)**
   - Rejected: not machine-readable, allocates, can't be efficiently serialized
3. **Separate error types per module**
   - Rejected: unified `NyquestroError` provides consistent error handling interface
4. **No error classification**
   - Rejected: matching engines need to distinguish recoverable vs fatal for operational safety

### Assumptions and Constraints to Validate

- [ ] All event fields can be represented by existing primitives (`OrderID`, `Px`, `Qty`, `Ts`, `Side`)
- [ ] Zero-allocation requirement means no `String`, `Vec`, or heap-allocated types in events
- [ ] Event structs are small enough to be `Copy` without performance penalty
- [ ] Error classification is sufficient for initial matching engine needs (can extend later)

---

## Interfaces and Contracts

### Public APIs to Add/Change

**New Module: `src/events.rs`**

- `FillEvent` struct with fields: buyer_order_id, seller_order_id, price, quantity, timestamp
- `QuoteEvent` struct with fields: side, price, quantity, timestamp
- `OrderEvent` enum with variants: `New`, `Cancelled`, `Rejected`
- `OrderRejectionReason` enum for rejection causes
- All types must be `Copy`, use only existing primitives

**Extended: `src/errors.rs`**

- Add matching-related error variants (e.g., `OrderAlreadyExists`, `OrderCannotBeCancelled`, `MatchingEngineError`)
- Add `ErrorSeverity` enum (`Recoverable`, `Fatal`)
- Add `severity()` method to `NyquestroError` for classification
- Keep existing variants unchanged for backwards compatibility

**Modified: `src/order.rs`**

- `Order::new()` return type: `Result<Order, &'static str>` → `NyquestroResult<Order>`
- `Order::fill()` return type: `String` → `FillEvent` (or callback-based emission)
- Map existing string error to appropriate `NyquestroError` variant

### Invariants That Must Hold

- All event structs are `Copy` and contain only `Copy` types
- All event structs are zero-allocation (no `String`, `Vec`, etc.)
- `FillEvent` has non-zero quantity and distinct buyer/seller IDs
- `QuoteEvent` has non-zero quantity
- Error classification is consistent (same error type always has same severity)
- `Order::new()` validation errors map to appropriate `NyquestroError` variants

### Must-Not-Break Contracts

- Existing `NyquestroError` variants remain unchanged (backwards compatibility)
- `NyquestroResult<T>` type alias continues to work
- Existing code using `PriceLevel::add_order()` continues to work (already uses `NyquestroResult`)

---

## Impacted Areas

### Files/Modules Likely to Change

- [x] `src/events.rs` – **NEW FILE** – all event type definitions
- [x] `src/errors.rs` – extend enum, add classification logic
- [x] `src/order.rs` – migrate `new()` to `NyquestroResult`, update `fill()` signature
- [x] `src/lib.rs` – export `events` module
- [ ] `tests/events_test.rs` – **NEW FILE** – event construction and validation tests
- [ ] `tests/errors_test.rs` – **NEW FILE** or extend existing – error classification tests

### Data Model Changes

- No database/serialization changes yet (events are in-memory structs)
- Event structs will be serialized later (out of scope)

### API/CLI Changes

- `Order::new()` return type changes (breaking change, but internal API)
- `Order::fill()` return type changes (breaking change, but internal API)
- Public API (`lib.rs` exports) adds `events` module

---

## Incremental Implementation Plan

### Step 1: Create Events Module Skeleton

**Intent:** Establish the module structure and basic event types  
**Expected Behaviour:** New `src/events.rs` file exists with `FillEvent`, `QuoteEvent`, `OrderEvent` struct definitions. Module exports from `lib.rs`. All types derive `Copy` and use existing primitives.  
**Verification:** `cargo check` passes. Can construct a `FillEvent` in a test.

**Checklist:**

- [x] `src/events.rs` created with basic struct definitions
- [x] All event types derive required traits (`Copy`, `Debug`, `Clone`, `PartialEq`, `Eq`)
- [x] Module exported in `src/lib.rs`

### Step 2: Implement FillEvent with Validation

**Intent:** Complete `FillEvent` with constructor and validation logic  
**Expected Behaviour:** `FillEvent` has constructor that validates buyer_id != seller_id and quantity > 0. Has getters for all fields.  
**Verification:** Tests pass for valid construction. Tests reject invalid inputs (zero quantity, same buyer/seller).

**Checklist:**

- [x] `FillEvent::new()` constructor with validation
- [x] Getters for all fields
- [x] Unit tests in `tests/events_test.rs`

### Step 3: Implement QuoteEvent

**Intent:** Complete `QuoteEvent` for best bid/ask updates  
**Expected Behaviour:** `QuoteEvent` has constructor that validates quantity > 0. Has getters for all fields.  
**Verification:** Tests pass for valid construction. Tests reject zero quantity.

**Checklist:**

- [x] `QuoteEvent::new()` constructor with validation
- [x] Getters for all fields
- [x] Unit tests

### Step 4: Implement OrderEvent and OrderRejectionReason

**Intent:** Complete order lifecycle event types  
**Expected Behaviour:** `OrderRejectionReason` enum maps to error types. `OrderEvent` enum has variants `New`, `Cancelled`, `Rejected` with appropriate fields. Constructors validate inputs (e.g., `New` requires quantity > 0).  
**Verification:** Tests pass for all `OrderEvent` variants. Tests validate rejection reasons map correctly.

**Checklist:**

- [ ] `OrderRejectionReason` enum defined
- [ ] `OrderEvent` enum with all variants
- [ ] Constructors and validation
- [ ] Unit tests

### Step 5: Extend Error Enum with Matching Errors

**Intent:** Add matching-related error variants  
**Expected Behaviour:** `NyquestroError` enum has new variants: `OrderAlreadyExists`, `OrderCannotBeCancelled`, `MatchingEngineError`. Error messages are descriptive. Existing variants unchanged.  
**Verification:** `cargo check` passes. Can construct all new error variants. Existing error handling still works.

**Checklist:**

- [ ] New error variants added to `NyquestroError`
- [ ] Error messages are descriptive
- [ ] Existing variants unchanged

### Step 6: Implement Error Classification System

**Intent:** Add `ErrorSeverity` and classification logic  
**Expected Behaviour:** `ErrorSeverity` enum exists (`Recoverable`, `Fatal`). `NyquestroError::severity()` method classifies all variants. Classification rules: validation errors → `Recoverable`, `MatchingEngineError` → `Fatal`, etc.  
**Verification:** Tests verify each error variant has correct severity. Classification logic is consistent.

**Checklist:**

- [ ] `ErrorSeverity` enum defined
- [ ] `severity()` method implemented for all variants
- [ ] Classification rules documented
- [ ] Unit tests for classification

### Step 7: Migrate Order::new() to NyquestroResult

**Intent:** Replace `Result<Order, &'static str>` with `NyquestroResult<Order>`  
**Expected Behaviour:** `Order::new()` return type is `NyquestroResult<Order>`. String error "Orders with zero or less quantity are invalid." maps to `NyquestroError::InvalidQuantity`. All call sites updated.  
**Verification:** `cargo check` passes. All existing tests still pass. Error messages are preserved.

**Checklist:**

- [ ] `Order::new()` return type changed
- [ ] Error mapping implemented
- [ ] Call sites updated (`src/main.rs`, tests)
- [ ] Tests still pass

### Step 8: Update Order::fill() to Return FillEvent

**Intent:** Replace `String` return with structured `FillEvent`  
**Expected Behaviour:** `Order::fill()` signature returns `FillEvent` instead of `String`. `FillEvent` constructed with order details (buyer/seller IDs, price, quantity, timestamp). No `format!()` calls. `src/main.rs` updated to handle `FillEvent`.  
**Note:** For now, use `self.order_id` for both buyer/seller (self-match scenario). Proper matching engine will provide partner order ID later.  
**Verification:** `cargo check` passes. `src/main.rs` compiles and runs. `FillEvent` is properly constructed.

**Checklist:**

- [ ] `Order::fill()` signature changed
- [ ] `FillEvent` construction implemented
- [ ] `src/main.rs` updated
- [ ] Tests updated if needed

---

## Testing and Validation

### Unit Tests

- [ ] `FillEvent` construction with valid inputs
- [ ] `FillEvent` rejects zero quantity
- [ ] `FillEvent` rejects same buyer/seller ID
- [ ] `QuoteEvent` construction with valid inputs
- [ ] `QuoteEvent` rejects zero quantity
- [ ] `OrderEvent::New` construction and validation
- [ ] `OrderEvent::Cancelled` construction
- [ ] `OrderEvent::Rejected` with all rejection reasons
- [ ] Error classification for all `NyquestroError` variants
- [ ] `Order::new()` error mapping to `NyquestroError`

### Integration Tests

- [ ] Order creation and fill produces `FillEvent`
- [ ] Event emission from order operations works end-to-end

### Validation Commands

- `cargo test` – all tests pass
- `cargo check` – no compilation errors
- `cargo clippy` – no linter warnings (if clippy is set up)

---

## Risks, Edge Cases, and Failure Modes

### Failure Modes

- [ ] **Event construction with invalid data** – validation should reject (zero quantity, invalid IDs)
- [ ] **Error classification inconsistency** – same error type should always have same severity
- [ ] **Breaking changes in `Order::new()`** – all call sites must be updated
- [ ] **Self-match in `FillEvent`** – currently allowed (placeholder), will be handled by matching engine later

### Detection Signals

- Unit tests catch invalid event construction
- Unit tests verify error classification consistency
- Compilation errors catch breaking changes in `Order::new()`

### Mitigations

- Comprehensive validation in event constructors
- Centralised classification logic in `severity()` method
- Update all call sites in same commit as `Order::new()` change

### Edge Cases

- Zero quantity in events (should be rejected)
- Same buyer/seller ID in `FillEvent` (currently allowed, will be handled later)
- Error classification for new error variants (must update `severity()` method)

---

## Exit Criteria

### Correctness

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] `cargo check` passes with no errors
- [ ] Event structs are `Copy` and zero-allocation (verified by inspection)
- [ ] Error classification is consistent (all variants have `severity()` implementation)

### Performance

- [ ] Event structs are `Copy` (no heap allocation)
- [ ] No `String` or `Vec` in event structs (verified by inspection)

### Operability

- [ ] Error messages are descriptive and actionable
- [ ] Error classification enables decision-making (recoverable vs fatal)

### Documentation

- [ ] Plan file updated with completion status
- [ ] `plans/README.md` updated with plan status

---

## Future Considerations

### Follow-on Work

- [ ] Event serialization (JSON, binary formats)
- [ ] Event publishing mechanisms (ring bus, multicast)
- [ ] More event types as matching engine grows (e.g., `TradeEvent`, `DepthEvent`)
- [ ] Error recovery handlers based on classification
- [ ] Self-match prevention in `FillEvent` (when matching engine provides partner IDs)

### Known Limitations

- `FillEvent` currently allows self-match (buyer_id == seller_id) – will be fixed when matching engine provides partner order IDs
- Error classification is basic – may need refinement as matching engine grows
- No event versioning yet – may need if event structure changes

### Deliberate Technical Debt

- Self-match handling deferred until matching engine implementation
- Event serialization deferred until publishing mechanisms are needed
