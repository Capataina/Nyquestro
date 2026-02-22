# IMPLEMENT_NOW: Core Hardening + Minimal OrderBook MVP

## 1. Header

- **Status:** Draft (prompt-aligned; not executed)
- **Scope:** Harden core invariants (`types`/`order`/`events`/`errors`) and implement a minimal deterministic `OrderBook` that accepts limit orders, matches by price-time priority, and emits `FillEvent`/`QuoteEvent`.
- **Exit rule:** This file is complete when (a) hardening changes are implemented with tests, (b) an `OrderBook` API compiles and is exercised by integration tests with deterministic event sequences, and (c) context documents reflect reality. Then archive or delete this file.

## 2. Implementation Structure (must remove ambiguity about "where things go")

### Modules / files affected (expected)

- `src/types.rs` (core primitives and constructors)
- `src/errors.rs` (error taxonomy and severity classification)
- `src/order.rs` (order invariants and state transitions)
- `src/events/fill_event.rs` (fill frame invariants)
- `src/events/quote_event.rs` (quote frame invariants)
- `src/events/order_event.rs` (order event frame invariants)
- `src/events/mod.rs` (exports)
- `src/matching_engine/*` (new compiled modules; currently described as placeholder)
- `src/lib.rs` (public surface wiring)
- `tests/*` (integration tests for invariants + deterministic matching)
- `context/*` (update docs when behaviour changes; never change `README.md`)

### Responsibility boundaries (what must not leak)

- `types` remains allocation-free primitives; it must not depend on `order`, `events`, or `matching_engine`.
- `events` stays as immutable frames; it must not embed `Order` or `OrderBook` ownership.
- `order` owns per-order state transitions; it must not contain market-wide matching logic.
- `matching_engine` owns matching + book mutation + event sequencing; it must not expose internal container types.
- Error severity classification must have one obvious entrypoint; do not duplicate "fatal vs recoverable" logic across subsystems.

### Function inventory (required for each major task)

Keep helpers pure; keep orchestrators responsible for workflow sequencing. This list is the expected working surface; adjust only when you discover reality differs.

- **Hardening (helpers + invariants)**

  - `OrderID::new(...)` (`src/types.rs`)
    - Inputs/outputs: raw numeric ID -> validated ID wrapper or classified error.
    - Helper vs orchestrator: helper (pure).
    - Called by: `Order::new(...)`, event constructors, tests.
  - `Px::*` constructors (`src/types.rs`)
    - Inputs/outputs: raw numeric (and any existing float-based path) -> validated price wrapper or classified error.
    - Helper vs orchestrator: helper (pure).
    - Called by: `Order::new(...)`, `QuoteEvent::new(...)`, tests.
  - `Qty::new(...)` + any validation choice (`src/types.rs`)
    - Inputs/outputs: raw quantity -> quantity wrapper (and optionally a classified error if zero is disallowed at this layer).
    - Helper vs orchestrator: helper (pure).
    - Called by: `Order::new(...)`, event constructors, tests.
  - `Ts::now()` + time helpers (`src/types.rs`)
    - Inputs/outputs: system clock -> timestamp wrapper (avoid panics; return classified error if you decide to change signature).
    - Helper vs orchestrator: helper (impure due to clock read).
    - Called by: `Order::new(...)`, demo, tests.
  - `Order::fill(fill_amount)` (`src/order.rs`)
    - Inputs/outputs: fill quantity -> mutates `remaining_quantity` and `status`, or returns validation error without mutation.
    - Helper vs orchestrator: orchestrator (state mutation + invariants).
    - Called by: matching engine and tests.
  - `Order::status(&self) -> Status` (new) (`src/order.rs`)
    - Inputs/outputs: borrow order -> copy status.
    - Helper vs orchestrator: helper (pure).
    - Called by: tests, matching engine, any inspection surfaces.
  - `FillEvent::new(...)` / `QuoteEvent::new(...)` / `OrderEvent::new(...)` (`src/events/*`)
    - Inputs/outputs: fields -> immutable frame or classified error.
    - Helper vs orchestrator: helper (pure).
    - Called by: matching engine and tests.
  - `severity(...)` (or method equivalent) (`src/errors.rs`)
    - Inputs/outputs: `NyquestroError` -> `ErrorSeverity`.
    - Helper vs orchestrator: helper (pure).
    - Called by: tests and any caller needing classification.

- **OrderBook MVP (workflow + sequencing)**
  - `OrderBook` type (`src/matching_engine/order_book.rs`)
    - Inputs/outputs: owns and mutates book state; returns events as values.
    - Helper vs orchestrator: orchestrator.
    - Called by: integration tests (primary), optionally demo later.
  - `OrderBook::new(...)`
    - Inputs/outputs: optional config -> empty book.
    - Helper vs orchestrator: helper (constructs state).
    - Called by: tests and demo.
  - `OrderBook::submit_limit(order)` (name flexible; keep one submission entrypoint)
    - Inputs/outputs: incoming limit order -> deterministic `SubmitResult` and internal mutation.
    - Helper vs orchestrator: orchestrator (evaluate -> select -> mutate -> emit).
    - Called by: tests and demo.
  - `SubmitResult` (or similar)
    - Inputs/outputs: contains `Vec<FillEvent>` and `Vec<QuoteEvent>` in a documented deterministic order.
    - Helper vs orchestrator: data carrier.
    - Called by: tests.
  - Minimal inspection surface (only what tests need)
    - Inputs/outputs: pure view into top-of-book (e.g. `best_bid()`, `best_ask()`, or a "top snapshot" struct).
    - Helper vs orchestrator: helper (pure).
    - Called by: tests.

### Wiring summary (call order; English only)

- `tests/*` (and optionally `src/main.rs`) constructs `Order` values -> calls `OrderBook::submit_limit(...)` -> asserts returned `FillEvent`/`QuoteEvent` sequences and verifies final top-of-book using the minimal inspection API.
- `src/lib.rs` exports `OrderBook` + the result/inspection types as the stable surface; matching internals remain private so internals can change later.

## 3. Algorithm / System Sections

### A) Core correctness hardening (types, orders, events, errors)

**Explanation (required)**

This work removes surprising behaviour (panics, silent clamping, inconsistent error types) before introducing matching. "Correct" means: invariants are explicit; violations return classified errors deterministically; tests lock behaviour so the matching engine can rely on it without compensating for undefined edges.

**Defaults (recommended + alternative)**

- Recommended default: standardise on `NyquestroResult<T>` + specific `NyquestroError` variants for all public fallible constructors.
- Alternative: keep string errors in `types` only, translating at `order`/`events` boundaries (less churn, more inconsistency).

#### Discovery (bounded; minimum read/inspect steps)

- [ ] Read `src/types.rs` and inventory all fallible constructors; record current return types and preconditions.
- [ ] Read `src/errors.rs` and inventory error variants and how severity classification is computed today.
- [ ] Read `src/order.rs` focusing only on `Order::new(...)`, `Order::fill(...)`, status transitions, and getters.
- [ ] Read `src/events/fill_event.rs`, `src/events/quote_event.rs`, `src/events/order_event.rs` to list validations and any commented-out checks.
- [ ] Read `tests/*` to identify which assertions will fail after hardening; do not refactor unrelated tests.

#### Implementation playbook (required; procedural, checkbox-only actions)

Constructor error model

- [ ] Decide and record the constructor error rule (recommended default above, or the alternative).
- [ ] For each fallible constructor, map each invalid-input precondition to a specific `NyquestroError` variant (avoid catch-alls).
- [ ] Remove or migrate any `&'static str` error returns that are part of the public surface to the chosen `NyquestroResult<T>` model.
- [ ] Ensure no constructor panics on invalid input; if `Ts::now()` changes signature, update all callers and tests.

Order fill semantics

- [ ] Define the invariant: `fill_amount` must be non-zero and must be `<= remaining_quantity`.
- [ ] Implement the behaviour: on invalid fill amount, return a recoverable validation error and leave the order unchanged.
- [ ] Confirm status transitions are one-way: `Open` -> `PartiallyFilled` -> `FullyFilled` only; no transitions out of `FullyFilled`.
- [ ] Add a non-moving status accessor (e.g. `status(&self) -> Status`) and update callsites/tests to use it.

Event boundary invariants

- [ ] Decide the minimal constructor-time invariant set (recommended minimum):
  - [ ] `FillEvent`: quantity non-zero; buyer/seller IDs non-zero; decide self-match behaviour (see next section).
  - [ ] `QuoteEvent`: quantity non-zero; price valid per `Px`; side is present (enum).
  - [ ] `OrderEvent::New`: validate fields consistent with `Order::new(...)` inputs (at minimum: non-zero quantity, valid IDs/prices as applicable).
- [ ] If any constructor accepts invalid data "temporarily", document it explicitly and add a test that pins that behaviour.

Self-match handling (hardening-level decision)

- [ ] Choose a self-match policy boundary:
  - [ ] Recommended: reject self-match at match-time in `matching_engine` (policy boundary), and keep `FillEvent` rejecting buyer == seller (defensive invariant).
  - [ ] Alternative: allow buyer == seller in `FillEvent` and enforce policy only at match-time (simpler frames, weaker safety).

Error model overlap

- [ ] Choose the single severity entrypoint:
  - [ ] Recommended: add `NyquestroError::severity(&self) -> ErrorSeverity` and migrate callers.
  - [ ] Alternative: keep `severity(error: &NyquestroError) -> ErrorSeverity` as a free function and remove any competing entrypoints.
- [ ] Identify and retire any generic error variants that only duplicate classification concepts, but only after callsites are migrated.

#### Stop & verify checkpoints (required)

- [ ] Run the smallest tests that cover `types`/`order`/`events`/`errors` first; fix only failures caused by hardening changes.
- [ ] Add/adjust tests to pin:
  - [ ] primitive constructor failure variants
  - [ ] over-fill rejection leaves state unchanged
  - [ ] event constructor invariants (including the chosen self-match behaviour)
- [ ] Run the full test suite to confirm no accidental regressions.

#### Invariants / sanity checks

- Constructors return classified errors on invalid input; they do not panic.
- `Order::fill(...)` never silently clamps.
- Observing status does not move or clone the order.
- Event frames remain immutable, copy-friendly, and enforce the agreed invariant set.
- Severity classification has one obvious entrypoint and is stable under test.

#### Minimal explicit test requirements

- [ ] At least one integration test asserting each primitive constructor's failure variant(s) are stable.
- [ ] At least one `Order::fill(...)` test for exact fill, partial fill, and over-fill error-without-mutation.
- [ ] Event constructor tests for `FillEvent` and `QuoteEvent` rejecting zero quantity; plus a test covering the chosen self-match behaviour.

---

### B) Minimal deterministic `OrderBook` (naive internals, stable API)

**Explanation (required)**

This work introduces a compiled matching engine module with correctness-first internals. "Correct" means strict price-time priority, deterministic traversal, deterministic event ordering, and stable outputs for a fixed input sequence. Lock-free internals and allocation reduction are explicitly deferred behind a stable API.

Keep responsibilities separated: evaluation (marketability) -> selection (next resting order) -> state mutation (quantity updates/removals) -> event emission (fills/quotes).

**Defaults (recommended + alternative)**

- Recommended default: use a deterministic standard-library container for price levels (e.g. a sorted map) and FIFO within each level; optimise later.
- Alternative: reuse `src/price_level.rs` as a helper for levels and build book-wide ordering above it; this may reduce new code but can force ownership/cloning changes earlier.

#### Discovery (bounded; minimum read/inspect steps)

- [ ] Read `context/MATCHING_ENGINE.md` and `context/ARCHITECTURE.md` to confirm current wiring expectations (matching engine is not yet compiled/exposed).
- [ ] Inspect `src/lib.rs` to see the current public surface and where the `OrderBook` export should live.
- [ ] Inspect `src/matching_engine/` to confirm what exists today and what must be added for compilation.
- [ ] Re-skim `src/order.rs` and `src/events/*` only for the fields required to construct events deterministically (avoid broad refactors).

#### Implementation playbook (required; procedural, checkbox-only actions)

Public API and result contract

- [ ] Define the submission entrypoint (keep one): `OrderBook::submit_limit(order)` (name flexible; behaviour not).
- [ ] Define the deterministic output contract (recommended):
  - [ ] Return one structured `SubmitResult` containing zero-or-more `FillEvent`s and zero-or-more `QuoteEvent`s.
  - [ ] Document ordering rules: fills in strict matching traversal order; quotes only when top-of-book changes (see below).
  - [ ] No printing/logging/IO in the core path; mutation is internal, outputs are returned.
- [ ] Define the minimal inspection API required for tests (do not expose internal containers).
- [ ] Add `src/matching_engine/*` as compiled modules and export `OrderBook` and result/inspection types from `src/lib.rs`.

Data model decisions

- [ ] Choose match price semantics:
  - [ ] Recommended: match at the resting order price.
  - [ ] Alternative: match at the incoming order price (only choose with a specific reason; test it explicitly).
- [ ] Choose quote emission semantics:
  - [ ] Recommended: emit a `QuoteEvent` only when best bid or best ask changes in either price or displayed quantity at the top level.
  - [ ] Alternative: emit on any depth change (simpler, but tends to spam and is harder to test meaningfully).

Matching behaviour (limit orders only)

- [ ] Implement "evaluate marketability" as a clear first step: determine whether incoming crosses best opposite.
- [ ] Implement "select next resting order" deterministically: best price first, FIFO within that price level.
- [ ] Implement "apply fill" using `Order::fill(...)` invariants; ensure partial fills update remaining quantities correctly.
- [ ] Implement "remove fully filled resting orders" deterministically (no tombstones affecting traversal).
- [ ] If incoming remains after matching, rest the remainder at its limit price preserving time priority.
- [ ] Guarantee timestamp determinism: do not call `Ts::now()` during matching; use the order's existing timestamp.
- [ ] Emit `FillEvent` per counterparty fill, in strict traversal order.
- [ ] Emit `QuoteEvent` only after state mutation when top-of-book changes per the chosen rule.

Self-match policy (book-level decision)

- [ ] Choose a self-match policy for MVP:
  - [ ] Recommended: reject self-match at match-time (do not generate fills; return a recoverable error or an `OrderEvent::Rejected` - pick one and test it).
  - [ ] Alternative: allow self-match and treat it as a normal fill (only if explicitly exploring that behaviour).

Determinism tests (integration-level)

- [ ] Add an integration test that submits a fixed order sequence and asserts:
  - [ ] exact `FillEvent` sequence (IDs/side/price/qty/timestamp expectations)
  - [ ] exact `QuoteEvent` sequence (top-of-book changes only, per your rule)
  - [ ] final top-of-book state via the inspection API
- [ ] Add a "run twice" determinism test: same inputs -> identical outputs.

#### Stop & verify checkpoints (required)

- [ ] Confirm `src/matching_engine/*` compiles and `OrderBook` is exported from `src/lib.rs` with a minimal public surface.
- [ ] Run the smallest new integration test first ("simple cross") and lock ordering; only then add "sweep" and "partial then rest".
- [ ] Run the full test suite after each new scenario is added to avoid accumulating ambiguous failures.

#### Invariants / sanity checks

- Strict price-time priority: best price first, FIFO within price.
- Matching traversal order fully determines fill event order.
- No timestamps are generated during matching.
- Fully filled orders are removed and never participate in further matching.
- Quote events are emitted only when top-of-book changes per the chosen rule.

#### Minimal explicit test requirements

- [ ] One "simple cross" test (single resting order).
- [ ] One "sweep" test (multiple price levels).
- [ ] One "partial then rest" test (incoming partially matches then rests remainder).
- [ ] One determinism test (run the same sequence twice; identical results).

## 4. Integration Points

- `src/lib.rs` is the stable export surface for the MVP matching engine (`OrderBook` + result/inspection types).
- `tests/*` are the primary correctness gate for matching semantics and event sequencing.
- `src/main.rs` remains a demo; if extended, it should submit a small scripted sequence without adding IO/CLI work.

## 5. Debugging / Verification

Use deterministic, assertion-heavy tests rather than logs. When a match test fails, narrow it to one dimension at a time:

- traversal order (price then FIFO)
- match price choice (resting vs incoming)
- quote emission rule (top-of-book changes only)
- state mutation order (apply fills/remove filled -> recompute top-of-book -> emit quote)

Common failure patterns:

- Silent over-fill clamping masking matcher bugs (should be impossible after hardening).
- Quote spam (emitting on every fill rather than only on top-of-book changes).
- Non-deterministic timestamps (`Ts::now()` called during matching rather than at order creation).

## 6. Completion Criteria

- Hardening tasks in section A are implemented with tests; existing tests still pass.
- `OrderBook` compiles, is exported via `src/lib.rs`, and supports deterministic limit matching with event outputs.
- Integration tests cover the minimal scenarios and assert event ordering precisely.
- `context/ARCHITECTURE.md`, `context/ORDER_MODEL.md`, `context/EVENTS_AND_ERRORS.md`, and `context/MATCHING_ENGINE.md` are updated to reflect the new reality (no aspirational wording).
- This file (`context/IMPLEMENT_NOW_CORE_HARDEN_BOOK_MVP.md`) is archived or deleted once the exit rule is met.
