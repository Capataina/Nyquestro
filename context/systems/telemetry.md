# Telemetry

*Maturity: working · Stability: stable*

## Scope / Purpose

`src/telemetry/` is a local-only flight recorder for the dashboard. Every keystroke (with the resulting `Action` and the post-action state), every engine event (submit, fill, cancel, reject, quote), every render-tick profile (step_us, render_us, actions, budget_left), every periodic state snapshot (latency percentiles, throughput counters, book state per symbol), every live-feed status, and every shutdown reason is appended to a single JSONL file at the platform-canonical application-data path:

- macOS: `~/Library/Application Support/Nyquestro/last-run.jsonl`
- Linux: `~/.local/share/nyquestro/last-run.jsonl`
- Windows: `%LOCALAPPDATA%\Nyquestro\last-run.jsonl`

Truncated on every dashboard launch — there is exactly one run on disk at any moment. Never uploaded, never aggregated, never analytics. The user's own audit trail.

## Boundaries / Ownership

- **Owns:** `TelemetryEvent` enum (~17 variants), `TelemetryHandle` (clone-cheap), `spawn_writer` (initialises the writer thread + returns the handle + resolved path), the writer's BufWriter loop, the bounded sync_channel, the dropped-event counter, the JSONL serialisation format with schema-version wrapper.
- **Does not own:** *what* gets recorded (call sites in `App`, `feed`, and `main` decide), *when* events flow (recorder is purely reactive), or analysis/visualisation (the file is consumed by external tools or future Claude sessions).
- **Imported by:** `ui::app` (every state-changing call site emits events), `main` (spawns the writer for both synthetic and live modes), `feed::coinbase` (emits feed-status events directly so disconnections and parse errors are captured even before they reach the bridge).

## Current Implemented Reality

### Three submodules

| Module | What it owns |
|--------|-------------|
| `telemetry/events.rs` | `TelemetryEvent` enum: `Startup`, `Key`, `Submit`, `Fill`, `Cancel`, `Reject`, `Quote`, `Frame`, `FrameSlow`, `PaneRender`, `Latency`, `Throughput`, `BookState`, `Snapshot`, `FeedStatus`, `FeedError`, `DroppedEvents`, `Resize`, `Shutdown`. Tagged with `kind` discriminant; per-variant fields flatten into the JSON line. |
| `telemetry/writer.rs` | `spawn_writer` — resolves the platform path, truncates the file, opens a `BufWriter<File>`, spawns a dedicated OS thread that drains a bounded `sync_channel` (capacity 8192), flushes every 200ms, reports `dropped_events` once per second when the channel was saturated. The handle's `record(event)` is non-blocking via `try_send`; on full it increments an `AtomicU64` drop counter. |
| `telemetry/mod.rs` | re-exports + module documentation. |

### Schema

Every line is one JSON object wrapped with `v` (schema version) and `t` (ISO-8601 UTC timestamp with microsecond precision):

```json
{"v":1,"t":"2026-05-04T22:14:23.456789Z","kind":"key","raw":"Tab","action":"CycleSymbol","selected_after":1,"mode_after":"live"}
```

Variants include their own per-event fields after `kind`. `v: 1` is current; future breaking changes bump to `v: 2`. Adding new `kind` values or new fields is additive (parsers should skip unknown kinds and ignore unknown fields).

### Threading model

```text
main thread                  feed thread                  telemetry thread
─────────────                ────────────                 ─────────────
App::dispatch                run_coinbase                 BufWriter<File>
  ├─ try_send(event) ─┐        ├─ try_send(status) ─┐       │ flush 200ms
  │                   │        │                    │       │ dropped 1Hz
App::handle_action    │        │                    │       │
  ├─ try_send(event) ─┤        │                    │       │
  ▼                   ▼        ▼                    ▼       ▼
            sync_channel<TelemetryEvent>(8192) ─────────────┘
            (drop on full + atomic counter)
```

The main loop and feed thread both hold cheap clones of the handle. Both call `try_send`. The telemetry thread is the only writer. If the channel saturates, events are dropped (counter incremented) rather than blocking — the structural guarantee that telemetry can never freeze the dashboard.

### What gets recorded

Per-frame:
- `Frame { step_us, render_us, actions, budget_left }` — every render tick.
- `FrameSlow { ..., reason }` — when `step_us + render_us > 33_000`. Reason heuristic: `"per_frame_budget_exhausted"` (action drain hit the 500-cap), `"render_blocked"` (render dominated), or `"step_dominated"`.

Per-second:
- `Latency { op, count, p50_ns, p99_ns, p999_ns, p9999_ns, max_ns }` × 3 ops.
- `Throughput { orders_1s, fills_1s, cancels_1s, rejects_1s, quotes_1s }`.
- `BookState { sym, levels_bid, levels_ask, depth_bid, depth_ask, ofi, microprice_c, spread_c }` × N symbols.

Per-action:
- `Submit`, `Fill`, `Cancel`, `Reject`, `Quote` (sampled 1-in-10).

Per-input:
- `Key { raw, action, selected_after, mode_after }` — `raw` is the original keystroke string (`"Tab"`, `"Char('s')"`, `"Esc"`, etc.) so different keys mapping to the same `Action` are still distinguishable.

Per-feed-event (live mode):
- `Snapshot { sym, raw_bids, raw_asks, capped }` on every Coinbase L2 snapshot.
- `FeedStatus { msg }` on connect / subscribe / reconnect.
- `FeedError { msg }` on parse failure or connection error.

Lifecycle:
- `Startup { mode, symbols, term, seed }` on `App::new` / `App::new_live`.
- `Shutdown { reason, uptime_ms }` on clean exit.

Backpressure:
- `DroppedEvents { count }` once per second when the channel was saturated since the last report.

## Key Interfaces / Data Flow

```rust
pub fn spawn_writer() -> std::io::Result<(TelemetryHandle, PathBuf)>;

#[derive(Clone)]
pub struct TelemetryHandle { ... }
impl TelemetryHandle {
    pub fn record(&self, event: TelemetryEvent);  // non-blocking, drop-on-full
    pub fn noop() -> Self;                          // for tests / opt-out
}
```

The `App` holds one handle (cloned once for the feed thread when in live mode). Every state-changing or event-producing function calls `record(...)` exactly once per event.

## Implemented Outputs / Artifacts

- The three module files (`telemetry/{mod,events,writer}.rs`).
- 3 inline unit tests in `writer.rs`: `noop_handle_accepts_records_without_panicking`, `handle_clones_cheaply`, `writer_truncates_and_writes`.
- `examples/telemetry_smoke.rs` — TUI-free verification that spawns the writer, emits one of every event class, sleeps for the flush cadence, then prints the file path + line counts. Verified working end-to-end on 2026-05-04.

## Known Issues / Active Risks

- **`PaneRender` event isn't currently emitted.** The variant exists but the panes don't yet record their own timing. Next iteration: wrap each render function in a small RAII timer + sample at 1Hz.
- **No on-disk rotation across runs.** By design — truncate-on-startup means we always have exactly one run on disk. If you want history, copy the file before relaunching.
- **No compression.** A 60-second run produces ~5–50 MB depending on event volume. Acceptable for "last run" semantics; could be `gzip`-on-close in a future iteration.
- **Quote events are sampled 1-in-10.** Busy live mode produces 1k+ quotes/sec; recording all of them would dominate the file and provide little debug value. Visible as occasional `quote` events; the periodic `book_state` snapshot covers the structural information.
- **No guard against telemetry-spawning when the file path is unavailable.** Falls back to `TelemetryHandle::noop()` cleanly with an stderr message; the dashboard continues without telemetry. Tested on macOS; Linux/Windows paths trusted via `dirs` crate.

### Downstream impact

A bug in telemetry would manifest as either:
- The file is missing — handles fall back to `noop()` and the dashboard still works.
- The file is malformed — parsers fail; the dashboard is still fine; debugging from telemetry is harder until the bug is found.
- The channel saturates — `dropped_events` counts grow; the dashboard never freezes (structural guarantee).

The structural guarantee is load-bearing: telemetry is debugging infrastructure. It must not be the reason the dashboard fails. The drop-on-full + atomic counter pattern delivers that guarantee.

## Partial / In Progress

`PaneRender` event class — variant defined but not yet emitted. Wrapping per-pane timing into the render path.

## Planned / Missing / Likely Changes

- **Per-pane timing** (the missing `PaneRender` emission).
- **`Resize` event emission** when the terminal is resized — currently the variant exists but the run loop doesn't watch for resize events. Easy add.
- **Compression on close** (gzip) — for portfolios where someone wants to share a long run.
- **A `--telemetry-off` CLI flag** for users who want to opt out per-run without recompiling.
- **A separate read-side tool** (`nyquestro-replay last-run.jsonl --filter slow_frame`) for friendlier inspection. Today `cat | jq` covers most cases.

## Durable Notes / Discarded Approaches

- **`std::sync::mpsc::sync_channel` over `crossbeam_channel`.** Considered crossbeam for its richer API, but `sync_channel` provides the bounded `try_send` semantics we need without an extra dependency. Match the dependency cost to the problem.
- **JSONL over bincode.** Bincode would produce smaller files (~30–50% reduction), but JSONL is greppable, parseable with any tool, and human-readable enough that a quick `head` answers most questions. The file is "last run only" so size is bounded; legibility wins.
- **Schema version (`v: 1`) baked into every line.** Considered a single header line, but each-line versioning means any single line can be parsed in isolation — useful when truncation, partial reads, or `tail -f` consume the file mid-run.
- **Drop-on-full backpressure, not block-on-full.** The Coinbase-snapshot incident (2026-05-04) showed what happens when the main loop blocks on something it shouldn't: the dashboard freezes and the user can't quit. Telemetry is debugging infrastructure; it must never be the reason the dashboard freezes. The drop counter + periodic `dropped_events` summary is the structural guarantee that keeps that promise.
- **Truncate-on-startup over rolling logs.** Considered keeping the last 5 runs in `last-1.jsonl` … `last-5.jsonl`. Rejected: the truncate-on-start model is simpler, has bounded disk usage, and matches the use case ("what just happened, last run only"). If the user wants history, they can `cp` the file before relaunching.

## Obsolete / No Longer Relevant

None — module is brand new in this iteration.
