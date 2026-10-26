# Plan: Extensive Testing Framework

## Header

- **Status:** Planned (not started)
- **Scope:** Build a multi-layer testing architecture covering unit, integration, property-based, stateful-property-based, snapshot, stress, benchmark, mutation, and coverage testing. Wire each layer into CI. Defer fuzzing / loom / miri until their preconditions land. The goal is a testing story that is genuinely better than 80% of production Rust projects, not a check-every-box maximalist sprawl.
- **Why this matters:** A matching engine's value is its correctness. HFT firms specifically interview on "how do you test a matching engine?" — answering with a real testing pyramid (not "I have unit tests") is a differentiating signal. Beyond the hiring bullet, the layers compound: stress tests catch performance regressions, snapshot tests pin matching semantics, mutation tests catch holes in unit/property tests, property tests catch bugs none of the others would. Each layer covers a class of failure the others can't.
- **Exit rule:** complete when (a) all Tier 1 layers from the table below ship with at least the seed coverage listed, (b) every layer is wired into a CI workflow that runs on relevant triggers, (c) `notes/testing-architecture.md` documents the philosophy and trade-offs (which layers we built, which we deferred, why).

## Implementation Structure

### Layers in scope, ranked by build-now value

| Tier | Layer | Crate | What it catches | Effort | Notes |
|------|-------|-------|-----------------|--------|-------|
| 1 | Unit tests | std `#[cfg(test)]` | Per-function correctness | ✅ existing | No work — keep as-is |
| 1 | Integration tests | std `tests/` | Cross-module behaviour | ✅ existing | No work — keep as-is |
| 1 | Property tests | `proptest` | Bugs you didn't think of, via random + shrink | 1 day | Day 1 of buildout. See `plans/property-based-tests.md` for the 10 invariants. |
| 1 | Stateful property tests | `proptest-state-machine` | Bugs in long sequences modelled as state-machine transitions | 1 day | Models OrderBook as `StateMachineTest`; generates random valid transition sequences. |
| 1 | Benchmarks | `criterion` | Latency regressions; underwrites "p99 < 10µs" claim with reproducible numbers | 1 day | Day 2. Per-op micro-benches + a full-round-trip macro-bench. |
| 1 | Snapshot tests | `insta` | Regression in event-stream output | 0.5 day | Day 3. Pin `(input sequence) -> Vec<FillEvent>` mappings as golden files. |
| 1 | Stress tests | hand-rolled with `criterion` + `quanta` | Sustained-load p99 degradation, memory growth over millions of ops | 0.5 day | Day 4. `#[ignore]`-tagged so they don't run on every `cargo test`; opt-in via `--ignored`. |
| 1 | Coverage | `cargo-llvm-cov` | Untested lines and branches | 0.5 day | Day 5. CI step uploading lcov to Codecov (or equivalent). |
| 1 | Mutation testing | `cargo-mutants` | Holes in tests — *whether the tests actually catch bugs* | 1 day | Day 5. Wire as a weekly cron, not every-PR (slow). |
| 2 | Differential testing | hand-rolled | Rust ↔ C++ ref impl agreement | per `plans/cpp-reference-impl.md` | Will fall out of the C++ ref impl plan naturally. |
| 2 | Replay testing | hand-rolled | Engine matches NASDAQ behaviour on real data | per `plans/itch-replay-harness.md` | Phase 1 LOBSTER CSVs, Phase 2 raw ITCH. |
| 3 | Fuzzing | `cargo-fuzz` (libFuzzer) | Parser crashes on adversarial input | 1 day | Add when ITCH parser exists (Phase 2 of replay plan). Right tool for binary parsers; wrong tool for the matching engine itself. |
| 3 | Concurrency tests | `loom` | Data races under all possible thread interleavings | 1 day | Defer until engine becomes multi-threaded. Premature today. |
| 3 | Undefined behaviour | `miri` | UB in `unsafe` code | 0.5 day | Currently low-yield (no `unsafe`). Add a single CI green-tick "miri-clean" run as part of the no-unsafe claim. |
| 3 | Instruction-level benches | `iai` | Lower-variance micro-benchmarks (cache hit/miss comparisons) | 0.5 day | When microsecond comparisons matter; premature for the current engine. |

**Explicitly out of scope:**
- `tokio-test` — irrelevant in a single-threaded engine.
- `mockall` / `mockito` — the engine has no external dependencies to mock; introducing mocks here would actively damage the determinism story.
- `rstest` parametrised tests — overlap with property tests; duplicative.
- E2E tests via `pty-process` — `ratatui::backend::TestBackend` is friendlier and crate-native.

### Final directory layout (target shape)

```text
nyquestro/
├── src/                              # unit tests inline as today
├── tests/
│   ├── types_test.rs                  # integration (existing)
│   ├── order_test.rs
│   ├── events_test.rs
│   ├── price_level_test.rs
│   ├── matching_test.rs
│   ├── property/
│   │   ├── mod.rs
│   │   ├── strategies.rs               # arb_order, arb_action_sequence, arb_symbol
│   │   ├── invariants.rs               # 10 named properties
│   │   └── state_machine.rs            # OrderBook as StateMachineTest
│   ├── snapshot/
│   │   ├── mod.rs
│   │   └── event_streams.rs            # insta snapshots, fixed seeds
│   ├── stress/
│   │   ├── mod.rs
│   │   ├── sustained.rs                # p99 over 1.5M ops
│   │   └── memory.rs                   # RSS check after 10M ops
│   ├── replay/                         # from plans/itch-replay-harness.md
│   ├── differential/                   # from plans/cpp-reference-impl.md
│   ├── proptest-regressions/           # auto-generated minimal repros, committed
│   └── snapshots/                      # insta golden files, committed
├── benches/
│   ├── matching.rs                     # criterion: submit_limit
│   ├── cancel.rs                       # criterion: cancel
│   ├── round_trip.rs                   # criterion: submit + match + record_metrics
│   └── inspection.rs                   # criterion: best_bid / microprice / OFI
├── fuzz/                               # cargo-fuzz, only when parser exists
│   └── fuzz_targets/
│       └── itch_frame.rs
├── .cargo/
│   └── mutants.toml                    # cargo-mutants config (paths, exclusions)
└── .github/workflows/
    ├── test.yml                        # cargo test + clippy (every push)
    ├── coverage.yml                    # cargo-llvm-cov + Codecov upload
    ├── bench.yml                       # criterion regression detection (PRs)
    ├── mutants.yml                     # cargo-mutants (weekly cron)
    └── miri.yml                        # miri (weekly cron, after fuzz lands)
```

### Cargo manifest changes

```toml
[dev-dependencies]
proptest = "1.5"
proptest-state-machine = "0.3"
insta = { version = "1.40", features = ["yaml"] }
criterion = { version = "0.5", features = ["html_reports"] }
quanta = "0.12"

[[bench]]
name = "matching"
harness = false

[[bench]]
name = "cancel"
harness = false

[[bench]]
name = "round_trip"
harness = false

[[bench]]
name = "inspection"
harness = false

[profile.bench]
lto = "thin"
codegen-units = 1
debug = false
```

`cargo-mutants` and `cargo-llvm-cov` are tools, not dependencies — installed via `cargo install`.

## Algorithm / System Sections

### A) Property tests — `proptest` (Day 1)

Already specified in `plans/property-based-tests.md`. Use this plan's strategies (`tests/property/strategies.rs`) as the shared input-shape vocabulary for both the property suite and the snapshot suite.

**The strategies file is load-bearing.** Every other test layer that needs synthetic input pulls from here:

```rust
// tests/property/strategies.rs
use proptest::prelude::*;
use nyquestro::types::*;
use nyquestro::order::Order;

pub fn arb_symbol() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        Just(Symbol::from_const("AAPL")),
        Just(Symbol::from_const("MSFT")),
        Just(Symbol::from_const("NVDA")),
    ]
}

pub fn arb_side() -> impl Strategy<Value = Side> {
    prop_oneof![Just(Side::Buy), Just(Side::Sell)]
}

pub fn arb_price_near(mid_cents: u64, band_ticks: u64) -> impl Strategy<Value = Px> {
    let lo = mid_cents.saturating_sub(band_ticks).max(1);
    let hi = mid_cents.saturating_add(band_ticks);
    (lo..=hi).prop_map(|c| Px::from_cents(c).unwrap())
}

pub fn arb_qty(max: u32) -> impl Strategy<Value = Qty> {
    (1u32..=max).prop_map(Qty::new)
}

pub fn arb_order(symbol: Symbol, mid: u64, ts_base: u64) -> impl Strategy<Value = Order> {
    (
        1u64..1000,             // id range — small so collisions appear
        arb_side(),
        arb_price_near(mid, 50),
        arb_qty(100),
        ts_base..(ts_base + 1_000_000_000),
    ).prop_map(move |(id, side, price, qty, ts)| {
        Order::new(
            OrderID::new(id).unwrap(),
            symbol, side, price, qty,
            Ts::from_nanos(ts),
        ).unwrap()
    })
}

pub fn arb_action_sequence(len_range: std::ops::Range<usize>) -> impl Strategy<Value = Vec<Action>> {
    // Action = Submit(Order) | Cancel(OrderID), per the matching engine's surface.
    prop::collection::vec(arb_action(), len_range)
}
```

Strategy design notes:
- **Small id range (`1..1000`)** so two random orders sometimes collide → exercises self-match path. A uniform `u64` range never collides in 256 samples.
- **Tight price band (`mid ± 50 ticks`)** so orders actually cross. Uniform random over `u64::MAX` produces orders that can never match.
- **Quantity capped at 100** so over-fill paths are reachable — large random quantities never fully consume each other.

### B) Stateful property tests — `proptest-state-machine` (Day 1, second half)

`proptest-state-machine` models a system as a finite state machine where states are valid configurations and transitions are valid operations. The framework generates random sequences of valid transitions from random starting states and checks invariants after each.

For the matching engine, the model is:

```rust
// tests/property/state_machine.rs
use proptest::prelude::*;
use proptest_state_machine::{StateMachineTest, ReferenceStateMachine};

pub struct OrderBookSM;

#[derive(Debug, Clone)]
pub enum Transition {
    SubmitBuy { id: u64, price_cents: u64, qty: u32, ts: u64 },
    SubmitSell { id: u64, price_cents: u64, qty: u32, ts: u64 },
    Cancel { id: u64 },
}

#[derive(Debug, Clone, Default)]
pub struct ReferenceState {
    submitted_ids: std::collections::HashSet<u64>,
    cancelled_ids: std::collections::HashSet<u64>,
}

impl ReferenceStateMachine for OrderBookSM {
    type State = ReferenceState;
    type Transition = Transition;

    fn init_state() -> BoxedStrategy<Self::State> { /* fresh state */ }
    fn transitions(state: &Self::State) -> BoxedStrategy<Self::Transition> { /* valid moves */ }
    fn apply(state: Self::State, transition: &Self::Transition) -> Self::State { /* track id history */ }
    fn preconditions(state: &Self::State, transition: &Self::Transition) -> bool {
        match transition {
            Transition::Cancel { id } =>
                state.submitted_ids.contains(id) && !state.cancelled_ids.contains(id),
            _ => true,
        }
    }
}

impl StateMachineTest for OrderBookSM {
    type SystemUnderTest = OrderBook;
    type Reference = OrderBookSM;

    fn init_test(_state: &ReferenceState) -> Self::SystemUnderTest {
        OrderBook::new(Symbol::from_const("TEST"))
    }
    fn apply(book: Self::SystemUnderTest, state: &ReferenceState, t: Transition) -> Self::SystemUnderTest {
        // run the transition against the real book
        // assert invariants per-step:
        // - book never crosses (best_bid <= best_ask)
        // - PriceLevel total_quantity matches iter sum
        // - cancelled ids never appear in subsequent fills
        book
    }
}
```

This catches a class of bug the per-call property tests miss: the bug only manifests after a specific *sequence* of operations, not on any single operation in isolation.

Invariants checked per transition:
- Book never crosses itself (`best_bid <= best_ask`).
- PriceLevel `total_quantity()` equals `sum(order.remaining() for order in iter())`.
- An id that was cancelled never subsequently appears in a `FillEvent`.
- Order count is non-negative (no underflow).

### C) Benchmarks — `criterion` (Day 2)

Four benchmark files, each `harness = false` so criterion's runner takes over:

```rust
// benches/matching.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use nyquestro::book::OrderBook;
use nyquestro::types::Symbol;

fn bench_submit_no_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("submit_limit");
    for size in [10, 100, 1000, 10_000] {
        group.bench_with_input(BenchmarkId::new("resting_levels", size), &size, |b, &n| {
            b.iter_batched(
                || prebuild_book_with_n_orders(n),
                |mut book| {
                    let order = build_non_crossing_order(&book);
                    let _ = black_box(book.submit_limit(order));
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
}

fn bench_submit_with_full_match(c: &mut Criterion) { /* … */ }
fn bench_submit_with_partial_match(c: &mut Criterion) { /* … */ }
fn bench_submit_sweeping_n_levels(c: &mut Criterion) { /* … */ }

criterion_group!(benches, bench_submit_no_match, bench_submit_with_full_match,
                 bench_submit_with_partial_match, bench_submit_sweeping_n_levels);
criterion_main!(benches);
```

Bench scenarios per file:

- **`benches/matching.rs`** — `submit_limit` paths: rest only (no match), full single-level match, partial-then-rest, multi-level sweep at depths 2/5/10/20.
- **`benches/cancel.rs`** — `cancel` paths: target front of best level, target middle of best level, target deep level, target nonexistent id (error path).
- **`benches/round_trip.rs`** — submit + match + record_latency + record_orders. Closer to what the dashboard actually does per call.
- **`benches/inspection.rs`** — read-only paths: `best_bid`, `microprice`, `ofi(10)`, `top_n_asks(10)`, `level_counts`. The dashboard's render path calls these per frame.

CI integration: `bench.yml` runs `cargo bench -- --save-baseline ci-current` on the PR, then `cargo bench -- --baseline ci-main` against main's saved baseline. Regression threshold: 10% slowdown on any benchmark fails the PR.

### D) Snapshot tests — `insta` (Day 3)

Snapshot testing pattern: run a fixed input, capture the output as a serialised value, compare against a stored "golden" file. When the output changes, `insta` shows you the diff and asks you to accept or reject.

```rust
// tests/snapshot/event_streams.rs
use insta::assert_yaml_snapshot;
use nyquestro::book::OrderBook;
use nyquestro::types::Symbol;

#[test]
fn snapshot_simple_cross() {
    let mut book = OrderBook::new(Symbol::from_const("TEST"));
    book.submit_limit(make_sell(1, 10000, 5, 1)).unwrap();
    let result = book.submit_limit(make_buy(2, 10000, 5, 2)).unwrap();
    assert_yaml_snapshot!(result);
}

#[test]
fn snapshot_three_level_sweep() { /* … */ }

#[test]
fn snapshot_self_match_rejection() { /* … */ }

#[test]
fn snapshot_seed_42_first_100_actions() {
    let mut book = OrderBook::new(Symbol::from_const("TEST"));
    let mut sim = MarketSimulator::new(SimConfig::default(), 42);
    let mut all_results = Vec::new();
    for _ in 0..100 {
        for action in sim.step(0.05) {
            if let SimAction::Submit(o) = action {
                if let Ok(r) = book.submit_limit(o) {
                    all_results.push(r);
                }
            }
        }
    }
    assert_yaml_snapshot!(all_results);
}
```

Stored snapshots live in `tests/snapshots/`. Reviewing changes: `cargo insta review` opens an interactive diff. Accepting writes the new snapshot; rejecting fails the test until the regression is fixed.

The simulator-driven snapshot is the highest-value because it pins ~thousands of events worth of behaviour against a fixed seed. Any unintended change to matching semantics surfaces as a giant diff on the next test run. This is the canonical "integration test for everything at once" pattern.

### E) Stress tests — hand-rolled (Day 4)

Two `#[ignore]`-tagged tests so they don't run on every `cargo test` (they take ~30s each). Run via `cargo test -- --ignored`.

```rust
// tests/stress/sustained.rs
#[test]
#[ignore = "takes ~30s; run with --ignored"]
fn p99_under_10us_at_5k_ops_per_second() {
    use hdrhistogram::Histogram;
    let mut book = OrderBook::new(Symbol::from_const("TEST"));
    let mut sim = MarketSimulator::new(SimConfig::default(), 42);
    let mut hist: Histogram<u64> = Histogram::new(3).unwrap();

    let target = 1_500_000u64;
    let mut count = 0u64;
    let start = quanta::Instant::now();
    while count < target {
        for action in sim.step(0.001) {
            if let SimAction::Submit(o) = action {
                let t0 = quanta::Instant::now();
                let _ = book.submit_limit(o);
                let elapsed = t0.elapsed().as_nanos() as u64;
                hist.record(elapsed.max(1)).unwrap();
                count += 1;
                if count >= target { break; }
            }
        }
    }
    let wall = start.elapsed();
    let throughput = count as f64 / wall.as_secs_f64();

    println!("throughput: {throughput:.0} ops/sec");
    println!("p50:    {} ns", hist.value_at_quantile(0.50));
    println!("p99:    {} ns", hist.value_at_quantile(0.99));
    println!("p999:   {} ns", hist.value_at_quantile(0.999));
    println!("p9999:  {} ns", hist.value_at_quantile(0.9999));
    println!("max:    {} ns", hist.max());

    assert!(hist.value_at_quantile(0.99) < 10_000,
            "p99 = {} ns (target < 10µs)", hist.value_at_quantile(0.99));
    assert!(hist.value_at_quantile(0.9999) < 200_000,
            "p9999 = {} ns (target < 200µs)", hist.value_at_quantile(0.9999));
}
```

```rust
// tests/stress/memory.rs
#[test]
#[ignore = "takes ~60s and reads /proc; Linux/macOS only"]
fn rss_does_not_grow_unboundedly() {
    let mut book = OrderBook::new(Symbol::from_const("TEST"));
    let mut sim = MarketSimulator::new(SimConfig::default(), 42);

    let baseline_rss = read_rss();
    for batch in 0..10 {
        for _ in 0..1_000_000 {
            for action in sim.step(0.001) {
                match action {
                    SimAction::Submit(o) => { let _ = book.submit_limit(o); }
                    SimAction::CancelHint => {} // exercise via separate cancel path
                }
            }
        }
        let rss = read_rss();
        let delta_mb = (rss.saturating_sub(baseline_rss)) / 1_048_576;
        println!("batch {batch}: RSS = {} MiB (delta = {} MiB)", rss / 1_048_576, delta_mb);
        // Crude bound: 200MB growth across 10M ops indicates a leak.
        assert!(delta_mb < 200, "RSS grew by {delta_mb} MiB after {} M ops", (batch + 1));
    }
}

fn read_rss() -> u64 {
    // Linux: read /proc/self/status; macOS: task_info via libc
    // implementation detail — use the `peak_alloc` crate or platform-specific code
    todo!()
}
```

The sustained-load test directly underwrites the dashboard's headline claim. The memory-growth test catches `Vec`-grows-unbounded bugs, leaked level entries, and unbounded counter rings.

### F) Coverage — `cargo-llvm-cov` (Day 5, morning)

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html         # local: HTML report at target/llvm-cov/html/index.html
cargo llvm-cov --lcov --output-path lcov.info  # CI: upload to Codecov
```

CI workflow (`.github/workflows/coverage.yml`):

```yaml
name: coverage
on: [push, pull_request]
jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: llvm-tools-preview }
      - run: cargo install cargo-llvm-cov
      - run: cargo llvm-cov --lcov --output-path lcov.info
      - uses: codecov/codecov-action@v4
        with: { files: lcov.info }
```

No coverage threshold initially — measure first, set thresholds once the baseline is known. Don't game coverage by writing tautological tests.

### G) Mutation testing — `cargo-mutants` (Day 5, afternoon)

```bash
cargo install cargo-mutants
cargo mutants --in-place
```

`cargo-mutants` generates mutated versions of your source code (changes operators, swaps comparisons, drops statements) and runs the test suite against each. If a mutation survives — i.e. tests still pass with the bug — that's a hole in your tests. The output names the surviving mutation and points at the source file/line.

Example surviving mutations to watch for (all things our property tests should catch but might not):

- `==` mutated to `!=` in self-match check → property test "no self-match in fills" should kill this.
- `>` mutated to `>=` in `crosses()` → property test "book never crosses itself" should kill this.
- `+` mutated to `-` in `total_quantity` arithmetic → property test "PriceLevel total matches iter sum" should kill this.

Configuration (`.cargo/mutants.toml`):
```toml
exclude_globs = ["src/ui/**", "src/main.rs"]  # UI mutations are noise
test_tool = "cargo"
test_args = ["test", "--release"]
timeout_multiplier = 5.0  # release builds are fast; mutations may produce infinite loops
```

CI workflow (`.github/workflows/mutants.yml`): weekly cron, not per-PR. Generates a report, posts as a GitHub artifact. The first run will take 30-60 minutes; subsequent runs incrementally test only changed files (`--in-diff`).

A surviving mutation is not a CI failure — it's a finding that goes into a tracking issue. Many real bugs require multiple lines mutated together, which `cargo-mutants` won't try; we use the survivors as a prioritised list of "tests that should be sharper."

## Integration Points

- **Property strategies (`tests/property/strategies.rs`)** are the shared input-shape vocabulary. Snapshot tests, stress tests, and benchmarks all pull `arb_order` etc. from here. Strategy is a foundation, not just a per-property tool.
- **Stress test results inform benchmark thresholds.** If sustained load pushes p99 to 8µs, the criterion benchmark's regression threshold for that path should be < 10µs not < 1µs. Calibrate after Day 4.
- **Coverage informs property test priorities.** When `cargo-llvm-cov` reveals an uncovered branch, write the property that would exercise it. The two layers loop into each other.
- **Mutation testing closes the loop.** A surviving mutation that property tests should catch reveals a strategy gap (the inputs aren't reaching that code path); fix the strategy.
- **CI sequencing matters.** `test.yml` runs every push, fast (< 30s). `coverage.yml` runs every push, slower (< 2min). `bench.yml` runs PRs only. `mutants.yml` runs weekly. Don't slow the inner loop with the slow tests.

## Debugging / Verification

- **A property test failing should produce a minimal repro.** Verify shrinking is on by intentionally seeding a bug, running the suite, confirming the framework outputs a < 5-element failing case.
- **A criterion regression should fail the bench job.** Verify the threshold logic by intentionally inserting `std::thread::sleep(Duration::from_micros(1))` into a hot path, running the bench job, confirming PR fails.
- **An insta snapshot mismatch should fail the test.** Verify by manually editing a snapshot file, running `cargo test`, confirming clear diff output.
- **A surviving mutation should be visible.** Verify by intentionally weakening a test (assert removed, etc.), running `cargo mutants`, confirming the previously-killed mutation now survives.
- **CI green ≠ all layers passed.** Different jobs run on different triggers. Check the relevant workflow's status before merging.

## Completion Criteria

### Day-by-day buildout

- [ ] **Day 1 (proptest + state machine):**
  - [ ] `proptest = "1.5"` in `[dev-dependencies]`.
  - [ ] `tests/property/strategies.rs` with `arb_symbol`, `arb_side`, `arb_price_near`, `arb_qty`, `arb_order`, `arb_action_sequence`.
  - [ ] `tests/property/invariants.rs` with the 10 properties from `plans/property-based-tests.md`.
  - [ ] `tests/property/state_machine.rs` with `OrderBookSM` modelling submit/cancel transitions and per-step invariant checks.
  - [ ] `cargo test --test property` passes 256 cases per property in < 30 seconds.
  - [ ] At least one property has caught either a real bug (regression test added) or has been documented as having found nothing after ≥ 1000 cases.

- [ ] **Day 2 (criterion benchmarks):**
  - [ ] `benches/matching.rs`, `benches/cancel.rs`, `benches/round_trip.rs`, `benches/inspection.rs` all compile and produce HTML reports.
  - [ ] `cargo bench` runs in < 5 minutes total.
  - [ ] `.github/workflows/bench.yml` wired with regression detection at 10% threshold.
  - [ ] README cites a benchmark number from `cargo bench` output.

- [ ] **Day 3 (insta snapshots):**
  - [ ] `insta = "1.40"` in `[dev-dependencies]`.
  - [ ] `tests/snapshot/event_streams.rs` with at least 4 named scenarios (simple cross, multi-level sweep, self-match rejection, seed-42-100-actions).
  - [ ] Stored snapshots in `tests/snapshots/` committed to the repo.
  - [ ] `cargo insta review` runs against the suite without spurious diffs.

- [ ] **Day 4 (stress tests):**
  - [ ] `tests/stress/sustained.rs` passing with p99 < 10µs at 5k ops/sec sustained over 5 minutes.
  - [ ] `tests/stress/memory.rs` passing with RSS growth < 200 MiB across 10M ops.
  - [ ] Both tests `#[ignore]`-tagged; opt-in via `cargo test -- --ignored`.

- [ ] **Day 5 (coverage + mutation):**
  - [ ] `cargo llvm-cov` produces a baseline lcov report.
  - [ ] `.github/workflows/coverage.yml` wired and uploading to Codecov (or equivalent).
  - [ ] `cargo install cargo-mutants` succeeds locally.
  - [ ] First mutation test run completes; surviving mutations triaged into a tracking issue.
  - [ ] `.github/workflows/mutants.yml` wired as a weekly cron.

### Project-wide

- [ ] `notes/testing-architecture.md` exists, documenting the philosophy of the layered pyramid, what each layer catches, and which layers we explicitly deferred (loom, miri, fuzzing) with reasons.
- [ ] `systems/book.md` updated to reference the property/snapshot/stress test coverage as part of its correctness story.
- [ ] `notes/hft-firm-priorities.md` §8 updated to reflect this plan landing — testing architecture moves from "Tier 1 to do" to "done."
- [ ] README's "Testing and Validation" section rewrites to describe the actual implemented layers (no more aspirational language).
- [ ] This file is archived once all of the above are checked.

## Notes

- The build sequence (Days 1-5) is conservative: each day is meant to land cleanly without surprise. If a day's work overruns, ship what's done and continue tomorrow rather than racing through.
- A property test that finds a real bug becomes a celebration, not a setback. Capture the minimal repro into `tests/proptest-regressions/` (auto-generated) and ideally also as a named example test in the existing suite.
- Mutation testing's first run will surface ~tens of surviving mutations on the first try. This is normal. Triage them into "real test gaps" vs "equivalent mutations" (the mutation produced semantically-identical code that no test could distinguish) and address the real gaps over multiple iterations.
- The `cargo-mutants` survivors list is a deeper signal than coverage. 100% coverage with surviving mutations means your tests are tautological; 80% coverage with zero survivors means your tests are sharp. Optimise for the latter.
- For HFT-grade hiring conversations, the line you want to land is: **"94 tests across unit, integration, property, snapshot, stress, mutation, and benchmark layers, plus differential cross-validation against a C++ reference and replay validation against real NASDAQ data via LOBSTER."** Every layer above goes toward that sentence.
