# Plan: Property-Based Test Suite

> [!note] Subsumed by the broader testing-framework plan
> This plan is now **Day 1 of [`plans/extensive-testing-framework.md`](extensive-testing-framework.md)**. The strategies (`tests/property/strategies.rs`) introduced here become the shared input-shape vocabulary for snapshot tests, stress tests, and benchmarks downstream. The 10 named invariants below remain the canonical property list.
>
> If you're picking the testing work up cold, start with the framework plan; this file is the deeper drill-down on Day 1's specific deliverables.

## Header

- **Status:** Planned (not started; first deliverable of the broader testing framework)
- **Scope:** Add a `proptest`-based test suite that pins the matching engine's invariants over generated input sequences. Each property covers ~10⁴ cases per run, rather than the few dozen covered by the existing example-based tests.
- **Why this matters:** HFT firms specifically ask "how do you test a matching engine?" Property-based testing is the canonical answer — example tests catch bugs you imagined, properties catch bugs you didn't.
- **Exit rule:** complete when (a) ≥ 6 named properties pass, (b) `cargo test` runtime stays under 30 seconds for the full property suite, (c) at least one property has caught a real bug during development (or the suite is documented as having found none, which is also a useful signal).

## Implementation Structure

### Modules / files affected

- `Cargo.toml` — add `proptest = "1.5"` to `[dev-dependencies]`.
- `tests/proptest_invariants.rs` (new) — the property suite.
- `tests/proptest_strategies.rs` (new) — `proptest::Strategy` impls for `Order`, `Symbol`, etc., shared across properties.

### Strategies (input generators)

```rust
fn arb_symbol() -> impl Strategy<Value = Symbol>;
fn arb_side() -> impl Strategy<Value = Side>;
fn arb_price(min_cents: u64, max_cents: u64) -> impl Strategy<Value = Px>;
fn arb_qty(max: u32) -> impl Strategy<Value = Qty>;
fn arb_order(symbol: Symbol, id_range: Range<u64>) -> impl Strategy<Value = Order>;
fn arb_order_sequence(len_range: Range<usize>) -> impl Strategy<Value = Vec<Order>>;
```

### Properties

Each property is a `proptest!` block with a one-line invariant and a body that runs the orders through `OrderBook` and asserts.

| # | Name | Invariant |
|---|------|-----------|
| P1 | `price_time_priority_holds` | For any order sequence, the resulting fills respect price-time priority: aggressors are matched against best price first, FIFO within a price level. |
| P2 | `no_self_match` | No `FillEvent` ever has `buyer_order_id == seller_order_id`. |
| P3 | `partial_fills_sum_correctly` | For any aggressor that gets partially filled, the sum of its `FillEvent.quantity` equals `aggressor.quantity − aggressor.remaining`. |
| P4 | `cancelled_orders_never_fill` | If an order is cancelled before any fills land on it, no `FillEvent` references it as buyer or seller. |
| P5 | `top_of_book_monotonic_under_worse_orders` | Adding a resting order at a price *worse than the current best* never moves the best. |
| P6 | `book_state_self_consistent` | After any sequence of submits and cancels, `book.best_bid()` price ≤ `book.best_ask()` price (or one is `None`). The book never crosses itself. |
| P7 | `total_quantity_matches_iter_sum` | For every price level, `level.total_quantity()` equals the sum of `order.remaining()` over `level.iter()`. |
| P8 | `determinism_under_replay` | Two fresh `OrderBook`s submitting the same input sequence produce byte-identical `SubmitResult` vectors. |
| P9 | `cancel_then_resubmit_distinct_id` | Cancelling an order and submitting a new one with the same id (which is illegal, gets rejected) does not corrupt book state. |
| P10 | `fill_event_quantity_never_zero` | No emitted `FillEvent` has `quantity == 0`. |

### Wiring

- Properties live in `tests/proptest_invariants.rs`.
- Strategies in `tests/proptest_strategies.rs` are imported via `mod proptest_strategies; use proptest_strategies::*;` (Rust 2024 integration tests can share modules within the same test binary if structured under `tests/common/`).
- `proptest!` defaults to 256 cases per property; we'll override to 100 for speed initially, raise to 1000 after the suite is stable.

## Algorithm / System Sections

### A) Strategy design

The trick with property tests on a matching engine is keeping inputs *interesting* enough to find bugs without being so degenerate that every test trivially passes. The strategy must:

- Generate orders within a small price band (so they have a real chance of crossing).
- Mix sides (~50/50 buy/sell).
- Mix sizes (log-normal, clipped).
- Occasionally generate same-id pairs (to exercise self-match rejection without breaking the duplicate-id invariant).
- Mix submit + cancel actions.

```rust
enum Action {
    Submit(Order),
    Cancel(OrderID),
}
fn arb_action_sequence(len: usize) -> impl Strategy<Value = Vec<Action>>;
```

### B) Bug-finding budget

Initial run target: 256 cases × 10 properties = 2,560 randomised matching-engine sessions in <30s. If any property fails, `proptest` will *shrink* the failing input down to a minimal reproducer, which gets committed as a regression test in the existing example suite.

### C) Deterministic seeding

`proptest` by default seeds from system entropy; failures get persisted to `tests/proptest-regressions/`. This is the right behaviour for a matching engine — we want CI to find new bugs, but we want the regression file checked in so a fixed version of the bug stays fixed.

## Integration Points

- The properties consume the public surface of `OrderBook` / `Market` only. No internals access. If a property requires reaching into internals, the test is wrong; refactor the public API instead.
- The existing example-based tests in `tests/matching_test.rs` stay. Property tests are additive.

## Debugging / Verification

- Run `cargo test --test proptest_invariants` to execute the suite.
- On failure, `proptest` prints both the original failing input and the shrunk minimal reproducer. The minimal repro goes into `tests/proptest-regressions/` automatically.
- Convert any minimal repro that reveals a real bug into a named example test in `tests/matching_test.rs` so it survives even if `proptest` is removed.

## Completion Criteria

- [ ] `Cargo.toml` has `proptest = "1.5"` under `[dev-dependencies]`.
- [ ] `tests/proptest_strategies.rs` has strategies for `Symbol`, `Order`, `Action`, sequences.
- [ ] `tests/proptest_invariants.rs` has the 10 properties listed above.
- [ ] `cargo test` includes them in its run; total runtime < 30s on default `cargo test`.
- [ ] At least one property has either caught a bug (which became a regression test) or is documented as having found nothing after ≥ 1000 cases.
- [ ] `systems/book.md` is updated to mention property-based test coverage.
- [ ] This file is archived once all the above are checked.
