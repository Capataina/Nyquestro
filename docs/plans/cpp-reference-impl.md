# Plan: C++ Reference Matching Engine

## Header

- **Status:** Planned (not started)
- **Scope:** Build a minimal (~200–400 LOC) C++ reference implementation of the matching loop. Cross-validate against the Rust engine via golden-output testing on a shared input sequence.
- **Why this matters:** HFT recruiters skim CVs for C++. Without it, the door is partly closed even if the Rust project is excellent. A small, working C++ matching engine cross-validated against the Rust one is one weekend's work and eliminates the "no C++" objection cleanly. It is for the CV bullet, not for the project's runtime.
- **Exit rule:** complete when (a) the C++ engine compiles cleanly with `clang++ -std=c++20 -Wall -Wextra -Wpedantic`, (b) it implements `submit_limit` + `cancel` with the same semantics as the Rust engine, (c) a shared test harness compares event outputs byte-for-byte across both engines on at least 5 input sequences.

## Implementation Structure

### Modules / files affected

- `cpp/` (new top-level subdirectory):
  - `cpp/CMakeLists.txt`
  - `cpp/include/nyq/types.hpp` — `OrderID`, `Px`, `Qty`, `Side`, `Status`, `Symbol`
  - `cpp/include/nyq/order.hpp` — `Order` class + state machine
  - `cpp/include/nyq/order_book.hpp` — `OrderBook` class
  - `cpp/include/nyq/events.hpp` — `FillEvent`, `QuoteEvent`, `OrderEvent`
  - `cpp/src/order_book.cpp` — matching loop
  - `cpp/tests/test_basic.cpp` — Catch2 unit tests
  - `cpp/tests/test_cross_validate.cpp` — reads a JSON input sequence, runs the C++ engine, emits the event log; compared against a Rust-emitted log byte-for-byte
- `tools/cross_validate.sh` (new) — runs both engines on a fixed input, diffs the outputs

### Build system

- **CMake** + **Catch2** + **clang++ 16+** with `-std=c++20`.
- No external runtime deps (just standard library).
- A `cpp/.gitignore` for build outputs.

### C++ design choices for parity

| Rust design | C++ choice |
|-------------|-----------|
| `OrderID(u64)` | `enum class OrderID : std::uint64_t {}` for type safety |
| `Px(u64)` | `class Px { std::uint64_t cents_; };` with validating constructors |
| `Qty(u32)` | `class Qty { std::uint32_t value_; };` |
| `Side` | `enum class Side { Buy, Sell };` |
| `Status` | `enum class Status { Open, Partial, Filled, Cancelled };` |
| `BTreeMap<Px, PriceLevel>` | `std::map<Px, PriceLevel>` (keeps ordering, similar perf characteristics) |
| `VecDeque<Order>` | `std::deque<Order>` |
| `NyquestroResult<T>` | `std::expected<T, Error>` (C++23) or `tl::expected` for C++20 fallback |

The shape is a deliberate one-to-one Rust↔C++ mapping so cross-validation is a byte-equality check, not a semantic equivalence argument.

### Cross-validation harness

- A shared input format: JSON file with a list of actions (`{"submit": {...}}`, `{"cancel": {...}}`).
- Both engines emit a JSON event log.
- A `diff` between the two logs is the test pass criterion.

## Algorithm / System Sections

### A) Matching loop parity

The matching loop has to be byte-equivalent. The Rust loop is documented in `systems/book.md`; the C++ port follows the same four-phase structure:

1. Snapshot pre-state.
2. Aggressive matching loop.
3. Self-match rejection (if any).
4. Rest the remainder.
5. Quote emission on top-of-book change.

Any deviation produces a diff in the cross-validation harness — that's the unit test for "did I port it correctly".

### B) Self-match policy parity

Both engines reject the aggressor wholly on self-match. This must be byte-exact: same `OrderEvent::Rejected` shape, same reason, same timestamp passthrough.

### C) Determinism parity

The C++ engine must also be deterministic given a fixed input. No `std::chrono::system_clock` calls in the matching loop; every fill timestamp comes from the resting order's stored timestamp.

## Integration Points

- The C++ engine does *not* integrate into the Rust runtime. It lives in `cpp/` as a parallel artefact.
- Cross-validation lives in `tools/cross_validate.sh` and runs both engines on shared JSON input files.
- The CV bullet reads roughly: "Built a from-scratch order matching engine in safe Rust, with a parallel C++ reference implementation cross-validated to byte-identical output across 1000 randomised input sequences."

## Debugging / Verification

- A divergence in the cross-validation harness is the canonical bug signal. Walk the two logs side-by-side to find the first message that differs.
- The most likely sources of divergence are (a) integer overflow handling, (b) iteration order on `std::map` vs `BTreeMap` for equal keys (shouldn't happen — both are total-ordered by `Px`), (c) self-match policy differences.

## Completion Criteria

- [ ] `cpp/` compiles with `cmake --build` cleanly.
- [ ] `cpp/tests/` passes Catch2 tests.
- [ ] `tools/cross_validate.sh` produces byte-identical event logs across both engines on ≥ 5 fixed input sequences.
- [ ] Optional: a `proptest`-driven random-input cross-validation harness that runs 1000 sequences end-to-end without diverging.
- [ ] CV / README mentions the C++ reference implementation explicitly.
- [ ] This file is archived once all the above are checked.

## Notes

- The C++ engine does not need to match the Rust engine's *performance*. It just needs to match its *semantics*. That keeps scope tight.
- Consider using `std::expected` if C++23 is available; otherwise `tl::expected` (header-only, public domain) is fine.
- Avoid `boost`. The point is to demonstrate hand-rolled C++; pulling in Boost dilutes the signal.
