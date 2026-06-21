# Plan: Benchmark Harness (`--benchmark`)

## Header

- **Status:** Planned (not started)
- **Scope:** A headless benchmark mode — `cargo run --release -- --benchmark [--duration 20] [--seed N] [--orders N] [--json]` — that drives the existing engine + simulator at full speed with no TUI, runs for a fixed duration (default ~20s) or a fixed order count, then prints a rich terminal report: per-operation latency percentiles (p50 / p90 / p99 / p99.9 / p99.99 / max), throughput (orders·fills·cancels per second), and the process resource footprint (peak resident memory, CPU time, CPU utilisation). An optional `--json` flag emits the same numbers as machine-readable JSON for CI and for auto-generating the README's latency table.
- **Why this matters:** The README *talks* about measuring latency distributions but shows **no actual numbers**. The HDR histograms already exist inside `MetricsRegistry` — they're just invisible because they only ever surface inside the live dashboard. This mode turns them into a headline artefact: an employer or engineer clones the repo and runs **one command** to see real, reproducible numbers on their own hardware. "It's fast" becomes "p99 submit latency is X ns on an M-series Mac, here's how to reproduce it."
- **Exit rule:** complete when (a) `--benchmark` runs headless with no terminal control, (b) it prints the latency table + throughput + resource footprint, (c) `--json` produces parseable output, (d) the same `--seed` produces the same order *stream* (latency numbers vary run-to-run, but the workload is reproducible), (e) the README carries a generated numbers table with a one-line reproduce command.

## Honesty notes (read before building)

- **No GPU path exists.** Nyquestro is a pure-CPU matching engine plus a terminal renderer; there is nothing on a GPU to measure. The footprint section measures **CPU time, CPU utilisation, and peak RSS memory** instead. Do not add a GPU number — it would be a fabricated metric.
- **This is a closed-loop (service-time) benchmark, not an open-loop (response-time) one.** We drive orders in a tight loop and time each `submit_limit` call individually, so we measure how long the engine takes to *service* an order once it has it. This is the honest, useful headline number, but it under-reports the tail under a real arrival-rate model — the classic "coordinated omission" pitfall (Gil Tene). A future refinement (noted in `extensive-testing-framework.md`) drives at a fixed intended arrival rate and times from intended-arrival, which captures queueing delay. Document this limitation in the report footer so a sharp reader sees we know the difference.
- **End-to-end vs micro-benchmark.** This mode measures the engine *as wired into the app* (includes the simulator producing the flow). For isolated hot-path microbenchmarks (warm cache, no sim overhead, statistical rigour, regression gating) use `criterion`, tracked in `extensive-testing-framework.md`. The two are complementary: this one answers "how does the whole thing perform"; criterion answers "did this commit make `submit_limit` slower". Cross-link both.

## Implementation Structure

### Modules / files affected

- `src/main.rs` — parse `--benchmark`, `--duration`, `--orders`, `--seed`, `--json`; dispatch to `bench::run` instead of the TUI loop.
- `src/bench/` (new):
  - `mod.rs` — re-exports `BenchConfig`, `run`, `BenchReport`.
  - `driver.rs` — the headless drive loop: build `Market` + per-symbol `MarketSimulator`, step in a tight loop, feed every `SimAction` through the book, record each op's latency into the existing `MetricsRegistry`.
  - `resource.rs` — peak RSS + CPU time sampling via `getrusage`.
  - `report.rs` — the terminal report renderer (table + ASCII histogram + throughput sparkline) and the `--json` serialiser.
- `src/lib.rs` — `pub mod bench;`.
- `Cargo.toml` — add `libc = "0.2"` (for `getrusage`); `serde`/`serde_json` already present from telemetry.
- `README.md` — a "Measured performance" section with the generated table + reproduce command.

### Reuse, not reinvention

- **Latency math is already done.** `MetricsRegistry` holds per-`Op` HDR histograms with p50/p95/p99/p999/p9999/max snapshots (`systems/metrics.md`). The driver just records into it exactly as `App::handle_submit` does today; the reporter reads `registry.snapshot()`.
- **Visual helpers already exist.** `src/ui/theme.rs` exposes `sparkline`, `distribution_bar` (log-scale), and bar helpers built for the dashboard. The report reuses them so the headless output looks like the dashboard's latency pane, just printed once to stdout instead of rendered per frame.

## Algorithm / System Sections

### A) Drive loop

```
warmup:  run W seconds (default 2) WITHOUT recording — fills the book, warms caches,
         lets the OU mid-walk settle. Stats from warmup are discarded.
measure: loop until elapsed >= duration (or order count reached):
           for each symbol:
             actions = sim.step(dt_fixed)        // dt fixed, NOT wall-clock, for reproducible flow
             for action in actions:
               t0 = Instant::now()
               result = book.submit_limit(order) // or cancel
               dt = t0.elapsed()
               registry.record_latency(Op::Submit, dt)
               if !result.fills.is_empty() { registry.record_latency(Op::Match, dt) }
               registry.record_orders/fills/cancels/rejects(...)
             sample throughput into a per-second ring (for the sparkline)
```

Key choices:
- **Fixed `dt` per step**, not wall-clock — so a given `--seed` produces the same order stream regardless of how fast the machine runs it. The *flow* is reproducible; the *timing* is what we're measuring.
- **No sleeps, no frame budget, no input** — this is the difference from the dashboard loop. We want the engine saturated.
- **Warmup excluded** — cold-cache and empty-book numbers would flatter or distort the tail; a 2s warmup is standard practice.

### B) Resource sampling (`resource.rs`)

```rust
// Peak resident set size + CPU time over the whole process, sampled at end of run.
// getrusage(RUSAGE_SELF) → ru_maxrss, ru_utime, ru_stime.
pub struct ResourceFootprint {
    pub peak_rss_bytes: u64,   // ru_maxrss — NOTE unit differs by platform (see below)
    pub user_cpu: Duration,    // ru_utime
    pub system_cpu: Duration,  // ru_stime
    pub wall: Duration,        // measured separately
}
// CPU utilisation = (user_cpu + system_cpu) / wall  (can exceed 100% if multi-threaded;
// today the engine is single-threaded so expect ~100% of one core during the measure phase).
```

**Platform unit trap:** `ru_maxrss` is **bytes on macOS** but **kilobytes on Linux**. Normalise to bytes behind a `#[cfg(target_os)]` so the reported number is correct on both. This is exactly the kind of cross-platform footgun worth getting right once and documenting.

### C) The report (`report.rs`)

Printed once at the end. Reuses `theme::sparkline` / `theme::distribution_bar`. Target shape:

```
Nyquestro benchmark — synthetic flow, 3 symbols, seed 42, 20.0s (2.0s warmup excluded)

Throughput     1,284,000 orders/s   |   412,000 fills/s   |   88,000 cancels/s
Orders/s over run  ▂▃▅▆▇█▇▇▆▇█▇▆▅▆▇█▇▆▇   (each cell = 1s)

Latency (nanoseconds)
  op       p50     p90     p99    p99.9   p99.99     max
  submit   ...     ...     ...     ...      ...       ...
  match    ...     ...     ...     ...      ...       ...
  cancel   ...     ...     ...     ...      ...       ...

submit latency distribution (log buckets)
  64ns  ████████████████████  61%
 128ns  ████████              22%
 256ns  ███                    9%
   1µs  █                      ...
  ...

Footprint   peak RSS 41.2 MB   |   CPU 99.1% of 1 core   |   user 19.8s sys 0.3s
Note: closed-loop service-time measurement (see plans/benchmark-harness.md §Honesty).
```

The ASCII histogram comes straight from the HDR histogram's recorded-value buckets; `distribution_bar` already renders log-scale bars for the dashboard's latency pane, so this is a re-render of existing data to stdout.

### D) `--json` output

```json
{
  "config": { "mode": "synthetic", "symbols": ["AAPL","MSFT","NVDA"], "seed": 42, "duration_s": 20, "warmup_s": 2 },
  "throughput": { "orders_per_s": 1284000, "fills_per_s": 412000, "cancels_per_s": 88000 },
  "latency_ns": { "submit": { "p50": ..., "p99": ..., "p999": ..., "max": ... }, "match": {...}, "cancel": {...} },
  "footprint": { "peak_rss_bytes": 43200512, "cpu_util_pct": 99.1, "user_cpu_s": 19.8, "system_cpu_s": 0.3 },
  "meta": { "measurement": "closed-loop-service-time", "host_note": "fill in CPU model" }
}
```

This is what a CI job parses to (a) post the numbers and (b) regenerate the README table. It's also what the small-wins CI plan consumes.

## Integration Points

- **`MetricsRegistry`** — recorded into exactly as the live app does; `snapshot()` read by the reporter. No changes to metrics needed.
- **`MarketSimulator`** — driven headless with fixed `dt`; the existing `reseed` path gives reproducible flow.
- **`ui::theme`** — `sparkline` / `distribution_bar` reused by `report.rs`. If those helpers are currently `pub(crate)` scoped under `ui`, lift the two needed ones into a shared spot (or re-export) so `bench` can call them without depending on the whole UI. Small refactor, note it.
- **README** — gains the table; the reproduce command is literally the benchmark invocation.

## Debugging / Verification

- **Reproducible flow:** two runs with `--seed 42` produce identical order *counts* and identical fill *counts* (timing differs, workload doesn't).
- **`--orders N` mode** stops after exactly N submitted orders regardless of duration.
- **Warmup actually excludes:** assert the histogram is empty at the end of warmup (record only starts in the measure phase).
- **Resource numbers are sane:** peak RSS in the tens of MB, CPU util near 100% of one core (single-threaded today), wall ≈ duration.
- **`--json` is valid JSON:** pipe through `jq .` with exit 0.
- **No TTY required:** runs cleanly when stdout is a pipe (CI), no crossterm/alternate-screen calls on this path.

## Completion Criteria

- [ ] `src/bench/` exists with `driver.rs`, `resource.rs`, `report.rs`, `mod.rs`.
- [ ] `--benchmark` dispatches to `bench::run` with no TUI setup on that path.
- [ ] `--duration`, `--orders`, `--seed`, `--json` flags parse and work.
- [ ] Warmup phase runs and is excluded from recorded stats.
- [ ] Report prints latency table + throughput + sparkline + ASCII histogram + resource footprint.
- [ ] `ru_maxrss` normalised to bytes on macOS *and* Linux (cfg-gated).
- [ ] `--json` emits parseable output with the schema above.
- [ ] README has a "Measured performance" section with a generated table and the one-line reproduce command.
- [ ] Report footer documents the closed-loop / coordinated-omission caveat.
- [ ] `systems/` gains a short `bench.md` (or this is folded into `metrics.md`) describing the harness.
- [ ] This file is archived once all the above are checked.
