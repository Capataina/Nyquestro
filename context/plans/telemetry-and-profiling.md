# Plan: Telemetry & Profiling

## Header

- **Status:** Planned (implementation in flight as of 2026-05-04 — see also Tier 1 in `notes/hft-firm-priorities.md` §8)
- **Scope:** Build a comprehensive black-box flight recorder. Record every keystroke, mode change, engine event, frame timing, periodic state snapshot, and live-feed event into a single JSONL file at `~/Library/Application Support/Nyquestro/last-run.jsonl` (or platform equivalent). Truncate-on-startup so we always have exactly one run on disk. Emit a per-frame profiling event so we can detect frame-budget exhaustion and diagnose the cause.
- **Why this matters:** Without telemetry, debugging is "Caner sends a screenshot, Claude guesses what happened." With telemetry, the file is the audit trail — Claude reads the JSONL and can reconstruct the run end-to-end. Also: per-frame profiling tells us *which subsystem* is eating frame budget when the dashboard feels laggy. Without it, we'd guess.
- **Exit rule:** complete when (a) every event class listed below records correctly, (b) `~/Library/Application Support/Nyquestro/last-run.jsonl` is the canonical location and truncates on every run, (c) a 30-second run produces a parseable JSONL file with no malformed lines, (d) per-frame timing is recorded and slow frames (>33ms) emit a `frame_slow` event with breakdown.

## Implementation Structure

### Modules / files affected

- `Cargo.toml` — add `dirs = "5"` (already have `chrono`, `serde`, `serde_json` from the live feed).
- `src/telemetry/` (new):
  - `mod.rs` — re-exports `TelemetryEvent`, `TelemetryHandle`, `spawn_writer`.
  - `events.rs` — `TelemetryEvent` enum (~17 variants), wrapping logic.
  - `writer.rs` — `spawn_writer`, the file-writer thread, schema versioning.
- `src/lib.rs` — `pub mod telemetry`.
- `src/ui/app.rs` — `App` holds `TelemetryHandle`; emits events at every state-changing call site.
- `src/main.rs` — spawns writer on startup, passes handle to `App::new`/`new_live`.
- `src/feed/coinbase.rs` — accepts an optional telemetry handle so the feed thread can record FeedStatus / FeedError events directly.
- `src/feed/bridge.rs` — emits Snapshot events with `raw_bids/raw_asks/capped` so we can see when Coinbase ships big books.
- `notes/telemetry-policy.md` (new) — local-only, no network, schema versioning, how to inspect.

### Schema

Every line is one JSON object with this wrapper:

```json
{"v": 1, "t": "2026-05-04T22:14:23.456789Z", "kind": "<discriminant>", ...event-specific fields...}
```

`v` = schema version (bump on breaking changes). `t` = ISO-8601 UTC. `kind` = the event discriminant. Remaining fields depend on `kind`.

### `TelemetryEvent` variants

| `kind` | Fields | Emitted when |
|--------|--------|--------------|
| `startup` | `mode` (`"synthetic"`/`"live"`), `symbols: [String]`, `term: [u16, u16]`, `seed: Option<u64>` | `App::new`/`App::new_live` |
| `key` | `raw: String`, `action: &str`, `selected_after: Option<usize>`, `mode_after: Option<&str>` | Every keypress that resolves to an `Action` (also `Action::None` for diagnostics) |
| `submit` | `sym`, `side`, `px_c`, `qty`, `id` | Every order submitted to the engine |
| `fill` | `sym`, `px_c`, `qty`, `buyer: u64`, `seller: u64` | Every `FillEvent` emitted |
| `cancel` | `sym`, `id`, `remaining` | Every successful cancel |
| `reject` | `sym`, `id`, `reason: &str` | Every `OrderEvent::Rejected` |
| `quote` | `sym`, `side`, `px_c`, `qty` | Every `QuoteEvent` (sampled — see throttling below) |
| `frame` | `step_us`, `render_us`, `actions`, `budget_left` | Every render tick (33ms) |
| `frame_slow` | `step_us`, `render_us`, `actions`, `reason: &str` | When `step_us + render_us > 33000` |
| `pane_render` | `pane: &str`, `us` | Per-pane render timing (sampled) |
| `latency` | `op`, `count`, `p50_ns`, `p99_ns`, `p999_ns`, `p9999_ns`, `max_ns` | Every 1s, per op |
| `throughput` | `orders_1s`, `fills_1s`, `cancels_1s`, `rejects_1s`, `quotes_1s` | Every 1s |
| `book_state` | `sym`, `levels_bid`, `levels_ask`, `depth_bid`, `depth_ask`, `ofi`, `microprice_c: Option<u64>`, `spread_c: Option<u64>` | Every 1s, per symbol |
| `snapshot` | `sym`, `raw_bids`, `raw_asks`, `capped` | Every Coinbase L2 snapshot received |
| `feed_status` | `msg: String` | Every `FeedEvent::Status` |
| `feed_error` | `msg: String` | Parse errors, connection errors |
| `dropped_events` | `count: u64` | Periodically when channel back-pressure caused drops |
| `resize` | `cols`, `rows` | On terminal resize |
| `shutdown` | `reason: &str`, `uptime_ms: u64` | On clean exit |

Quote events are sampled at 1-in-10 to keep file size reasonable (in busy live mode we may see 1000+/sec). Submit/Fill/Cancel are *not* sampled — the audit trail has to be complete.

### File location

Cross-platform via `dirs::data_local_dir()`:

- macOS: `~/Library/Application Support/Nyquestro/last-run.jsonl`
- Linux: `~/.local/share/nyquestro/last-run.jsonl`
- Windows: `%LOCALAPPDATA%\Nyquestro\last-run.jsonl`

Behaviour:
- Create parent dir if missing.
- Open file with `O_TRUNC | O_CREATE | O_WRONLY` — wipes previous run on every startup. No accumulating logs, ever.
- If file creation fails (permissions, disk full), log to stderr and continue without telemetry. Telemetry must never be the reason the app fails to start.

### Threading

Event-emitting code (main thread, feed thread) sends `TelemetryEvent` through a `std::sync::mpsc::SyncSender` with bounded capacity 8192. A dedicated OS thread drains the channel:

```rust
spawn_writer() -> TelemetryHandle {
    let path = resolve_path();
    let file = create_truncated(path);
    let (tx, rx) = sync_channel(8192);
    let dropped = Arc::new(AtomicU64::new(0));
    
    thread::spawn(move || {
        let mut writer = BufWriter::with_capacity(64 * 1024, file);
        let mut last_flush = Instant::now();
        let mut last_dropped_report = Instant::now();
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => write_event(&mut writer, event),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if last_flush.elapsed() > Duration::from_millis(200) {
                let _ = writer.flush();
                last_flush = Instant::now();
            }
            if last_dropped_report.elapsed() > Duration::from_secs(1) {
                let n = dropped.swap(0, Ordering::Relaxed);
                if n > 0 {
                    write_event(&mut writer, TelemetryEvent::DroppedEvents { count: n });
                }
                last_dropped_report = Instant::now();
            }
        }
        let _ = writer.flush();
    });
    
    TelemetryHandle { tx, dropped }
}
```

`TelemetryHandle::record(event)` calls `tx.try_send(event)`; on `Full`, increments the `dropped` counter atomically. The main thread *never* blocks on telemetry. Drop counts surface as their own event so a reviewer can tell when the channel was saturated.

### Profiling

- **Per-frame:** wrap `step()` and `terminal.draw(...)` in `Instant::now()` measurements; emit a `frame` event per render tick.
- **Slow-frame detection:** if `total_us > 33_000` (the render tick budget), emit a `frame_slow` with reason heuristics: `"per_frame_budget_exhausted"` when `actions == 500`, `"render_blocked"` when `render_us > step_us`, `"step_dominated"` otherwise.
- **Per-pane:** wrap each `render_*` function in a small RAII timer; emit `pane_render` events sampled at 1-in-30 (so 1Hz at 30fps) to keep noise low.

### Periodic snapshots

A 1-second tick in `App::step` (gated by `last_snapshot_tick.elapsed() > Duration::from_secs(1)`) emits:
- One `latency` event per op (3 events per tick).
- One `throughput` event with the 1-second windowed rates.
- One `book_state` event per symbol.

That's ~7 events/sec of "state-over-time" data — enough to reconstruct the run's evolution without dominating the file.

## Algorithm / System Sections

### A) Event encoding

Use serde with `#[serde(tag = "kind", rename_all = "snake_case")]`. The wrapper struct flattens the event:

```rust
#[derive(Serialize)]
struct LoggedEvent<'a> {
    v: u8,
    t: String,
    #[serde(flatten)]
    inner: &'a TelemetryEvent,
}
```

Each `write_event` call: `serde_json::to_writer(&mut writer, &logged)?; writer.write_all(b"\n")?;`. `to_writer` is allocation-free (writes directly into the BufWriter).

### B) Backpressure

The 8192-event buffer absorbs ~10 seconds of bursty activity (peak ~1000 events/sec in live mode at high speed). When it fills, `try_send` returns `Full` and we drop. This is correct: the alternative (blocking the main thread on disk I/O) is what we're explicitly avoiding.

### C) Schema versioning

`v: 1` is the current schema. Future iterations (e.g., adding new event kinds, renaming fields) bump to `v: 2`. Parsers should:
- Accept any `v` they recognise.
- Skip unknown `kind` values rather than failing.
- Fail loudly on unparseable lines, never silently.

### D) Inspection workflow

```bash
# Quick scan
$ cat ~/Library/Application\ Support/Nyquestro/last-run.jsonl | head

# Per-event-kind counts
$ jq -r '.kind' < last-run.jsonl | sort | uniq -c | sort -rn

# Slow frames
$ grep '"kind":"frame_slow"' last-run.jsonl

# Latency over time
$ jq -c 'select(.kind == "latency" and .op == "submit") | [.t, .p99_ns]' < last-run.jsonl

# What happened around a specific time
$ jq -c 'select(.t >= "2026-05-04T22:14:30" and .t <= "2026-05-04T22:14:31")' < last-run.jsonl
```

These are the patterns Claude will use when reviewing a run from a transcript.

## Integration Points

- **`App::new` / `App::new_live`** receive `TelemetryHandle` as a parameter. They emit `Startup` and store the handle for the run.
- **`App::dispatch`** emits one event per `SimAction` it handles.
- **`App::step`** is the periodic-snapshot anchor; emits `latency` / `throughput` / `book_state` once per second.
- **`run_loop`** wraps `step()` and `draw()` with timers; emits `frame`/`frame_slow`.
- **`panes::render_*`** receive a thin `&PaneRecorder` via `Frame` thread-locals (or a wrapping struct); each function records its own timing.
- **`feed::run_coinbase`** receives `Option<TelemetryHandle>` so the feed thread can emit `feed_status` / `feed_error` directly without going through the bridge.
- **`feed::Bridge`** emits `snapshot` events with `raw_bids` / `raw_asks` / `capped` counts.
- **`restore_terminal`** sequence emits `shutdown` with a reason string before the writer thread is dropped.

## Debugging / Verification

- **A 30-second synthetic run produces a parseable JSONL file** with at least: 1 `startup`, ~30 `frame` per second, periodic `latency`/`throughput`/`book_state`, one `shutdown`. No malformed lines.
- **A 30-second live run produces additional events**: `feed_status` (subscribe), `snapshot` (per symbol), continuous `submit`/`cancel` from the bridge, eventual `feed_error` if connection drops.
- **Slow-frame detection works**: artificially insert `std::thread::sleep(Duration::from_millis(50))` into `step()`, run for 5 seconds, confirm `frame_slow` events appear.
- **Drop-on-full works**: artificially throttle the writer thread (e.g. block in the writer for 30s), run, confirm `dropped_events` count grows and final file is still parseable.
- **Schema versioning works**: inspect any line, confirm `v: 1` is present.

## Completion Criteria

- [ ] `Cargo.toml` has `dirs = "5"`.
- [ ] `src/telemetry/` exists with the three submodules.
- [ ] `TelemetryEvent` covers all 17 listed variants.
- [ ] `spawn_writer` works on macOS and Linux (verified by truncating + appending events to the platform-correct path).
- [ ] `App` holds `TelemetryHandle`; instrumentation emits events at all listed call sites.
- [ ] `run_loop` records per-frame profiling.
- [ ] Slow-frame detection emits `frame_slow` with reason heuristics.
- [ ] Periodic snapshots emit at 1Hz: latency, throughput, book_state.
- [ ] Drop-on-full backpressure works without blocking the main thread.
- [ ] `notes/telemetry-policy.md` exists and documents the local-only stance.
- [ ] `systems/telemetry.md` exists and documents the implementation.
- [ ] This file is archived once all the above are checked.
