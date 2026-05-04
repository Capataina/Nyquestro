# Metrics

*Maturity: working · Stability: stable*

## Scope / Purpose

`src/metrics/` provides the observability surface the dashboard renders from. It records:
- **Per-operation latency histograms** (HDR-backed, 3-significant-figure precision, 1ns–1hr range) for `Submit`, `Match`, `Cancel`. Snapshots expose p50, p95, p99, p999, **p9999**, max, mean.
- **Rolling event counters** for orders, fills, cancels, rejects, and quotes, with snapshots at 1s / 10s / 1min / 5min windows.

The whole registry is single-threaded (the dashboard runs the simulator and engine on one thread); no atomics, no mutexes.

## Boundaries / Ownership

- **Owns:** `MetricsRegistry`, `Op` enum, `RegistrySnapshot`, `LatencySnapshot`, `CounterSet`, `CounterSnapshot`, `WindowedCounter`, `WindowSnapshot`.
- **Does not own:** *what* gets recorded (every call site in `App::handle_*` decides) or *how* the snapshot is rendered (the `ui` layer reads it and decides).
- **Imported by:** `ui::app` (records every operation's timing + counter ticks), `ui::panes` (calls `metrics.snapshot()` per render frame).

## Current Implemented Reality

### Three modules

| Module | What it owns |
|--------|-------------|
| `metrics/windows.rs` | `WindowedCounter` — `VecDeque<(Instant, u64)>` with lazy aging on every record/snapshot; produces `WindowSnapshot { last_1s, last_10s, last_1min, last_5min }` |
| `metrics/counters.rs` | `CounterSet { orders, fills, cancels, rejects }` (4 `WindowedCounter`s) and `CounterSnapshot` |
| `metrics/registry.rs` | `MetricsRegistry` — wraps three HDR `Histogram<u64>`s + a `CounterSet` + a `started_at: Instant`; produces `RegistrySnapshot` containing per-op `LatencySnapshot { count, p50_ns, p95_ns, p99_ns, p999_ns, max_ns, mean_ns }` plus a `CounterSnapshot` and `uptime: Duration` |

### Histograms

Each `Op` (`Submit`, `Match`, `Cancel`) has its own `Histogram<u64>` with bounds `[1ns, 60×60×1e9 ns]` and 3-sig-fig precision. `auto(true)` is set so the histogram autoresizes if a sample exceeds the upper bound rather than panicking. Recorded values are clamped to `[1, u64::MAX]` so the histogram's lower bound is honoured.

### Windowed counters

`WindowedCounter::record(n)` appends `(Instant::now(), n)` and prunes entries older than 5 minutes. `sum_within(window)` walks from the back and stops when it sees an entry older than `now - window`. The 5-minute retention means at most ~5 minutes of records sit in memory; for the dashboard's expected event rate (~1k events/sec) that's at most a few hundred KB of `(Instant, u64)` pairs.

### Snapshot model

```rust
RegistrySnapshot {
    submit:   LatencySnapshot,
    match_op: LatencySnapshot,
    cancel:   LatencySnapshot,
    counters: CounterSnapshot {
        orders:  WindowSnapshot { last_1s, last_10s, last_1min, last_5min },
        fills:   WindowSnapshot,
        cancels: WindowSnapshot,
        rejects: WindowSnapshot,
    },
    uptime: Duration,
}
```

Snapshots are pure values (`Copy`), produced atomically per call. The UI reads one snapshot per render frame (33ms) and renders from it without touching the registry.

## Key Interfaces / Data Flow

```rust
pub enum Op { Submit, Match, Cancel }

impl MetricsRegistry {
    pub fn new() -> Self;
    pub fn record_latency(&mut self, op: Op, d: Duration);
    pub fn record_orders(&mut self, n: u64);
    pub fn record_fills(&mut self, n: u64);
    pub fn record_cancels(&mut self, n: u64);
    pub fn record_rejects(&mut self, n: u64);
    pub fn uptime(&self) -> Duration;
    pub fn snapshot(&self) -> RegistrySnapshot;
}
```

Recording flow (per submission):

```
App::handle_submit(order)
  ├─ start = Instant::now()
  ├─ book.submit_limit(order)
  ├─ elapsed = start.elapsed()
  ├─ metrics.record_latency(Op::Submit, elapsed)
  ├─ if !res.fills.is_empty():
  │     metrics.record_latency(Op::Match, elapsed)
  ├─ metrics.record_orders(1)
  ├─ for each fill:  metrics.record_fills(1)
  └─ for each Rejected: metrics.record_rejects(1)
```

Render-frame flow:

```
panes::render(frame, app)
  ├─ snap = app.metrics.snapshot()
  ├─ render_latency(...)     ← reads snap.submit
  ├─ render_throughput(...)  ← reads snap.counters
  └─ render_engine(...)      ← reads app.metrics.uptime() directly + lifetime totals from App
```

## Implemented Outputs / Artifacts

- The three module files + their public surface.
- 2 inline unit tests in `windows.rs` (record-then-snapshot, empty-snapshot-is-zero).
- The dashboard's Latency pane and Throughput pane consume directly from `RegistrySnapshot`.

## Known Issues / Active Risks

- **`record_latency` clamps to `[1ns, u64::MAX]` rather than rejecting.** A pathologically tiny duration (sub-nanosecond, possible on some platforms) is silently bumped to 1ns. Acceptable today because every recorded duration comes from `Instant::elapsed()` which is at minimum a few ns of overhead, but worth noting.
- **No per-operation latency persistence.** Histograms are in-memory only; a crash loses everything. The dashboard's narrative is real-time, so this is fine; for production-grade replay analysis we'd want HDR's serialise format.
- **Histograms autoresize but each record on an outlier is slower than the typical case.** `auto(true)` allocates new buckets when a sample exceeds the current bound. For HFT-grade latency targets this matters; for the MVP dashboard it's invisible.
- **The "Match" histogram records the same elapsed value as "Submit" when fills occurred** — there is currently no separate timing for the matching-loop alone vs the full submit including resting/quote-emission. The split-out is left for a future iteration.

### Downstream impact

A bad sample anywhere distorts the dashboard's headline percentiles. The latency card is one of the visual anchors; if the p99 row reads `41 µs` when the true p99 is `4 µs`, the project's claim about "honest measured performance" becomes false. The HDR + clamp + autoresize combination is engineered to make this hard to break.

## Partial / In Progress

None.

## Planned / Missing / Likely Changes

- **Match-only timing.** Split `Op::Match` from `Op::Submit` so the histogram captures the matching-loop subset of `submit_limit`'s wall time, not the whole call.
- **Per-thread HDR + atomic merge** for when the engine becomes multi-threaded.
- **Persistence / replay.** HDR has a serialise format (`hdrhistogram::Histogram::record_n` + `serialization::V2Serializer`); deferred until there's a need.
- **Atomic counters.** When the engine becomes multi-threaded, `WindowedCounter` would be replaced with a sharded atomic counter set; the public snapshot API would not change.

## Durable Notes / Discarded Approaches

- **Single-threaded by design.** Considered atomics from day one but the dashboard runs everything on one thread, so the contention argument doesn't apply yet. Adding atomics now would be premature optimisation.
- **HDR's autoresize is on.** Without it, a single 70-minute pause (e.g. someone leaves the dashboard running over lunch with a slow simulator step) would panic the recorder. With autoresize, the upper bound expands to fit.
- **Snapshots are pure values.** The UI never holds a `&mut MetricsRegistry`; it copies a snapshot once per frame. This makes it impossible for the renderer to corrupt the registry, and lets the snapshot be passed across thread boundaries without scaffolding when concurrency arrives.
- **5-minute retention on the windowed counters.** Long enough to render the longest displayed window (5 min); short enough that memory stays bounded under sustained load. Considered a 1-hour retention for retrospective analysis but rejected because the dashboard would never display anything beyond 5 minutes anyway and the memory cost would be ~12× higher.

## Obsolete / No Longer Relevant

None — this module was authored fresh in the current rewrite; there are no prior versions to deprecate.
