# Plan: Event Log + Book Recovery

## Header

- **Status:** Planned (not started)
- **Scope:** Append-only event log for every submit / cancel / fill / quote, with a snapshot+delta recovery path. Goal: `Market` state is always reconstructable from the log; a crash mid-session loses no committed events.
- **Why this matters:** Production matching engines treat their event log as the system of record. Recruiters specifically ask "how do you survive a crash?" The answer they want is "snapshot at known checkpoints, replay deltas, idempotent application".
- **Exit rule:** complete when (a) every event the engine emits is also written to an append-only log, (b) `Market::recover_from(log_path)` rebuilds the exact pre-crash state, (c) a test crashes mid-replay and demonstrates correct recovery.

## Implementation Structure

### Modules / files affected

- `src/log/` (new):
  - `mod.rs`
  - `writer.rs` — `EventLogWriter`: append-only file, line-delimited JSON or framed binary
  - `reader.rs` — `EventLogReader`: streaming parser, `Iterator<Item = NyquestroResult<LoggedEvent>>`
  - `snapshot.rs` — `Snapshot`: serialised `Market` state + log offset
- `src/book/order_book.rs` — emits to writer on every state change.
- `tests/recovery_test.rs` (new).

### Storage format

- **Per-event:** length-prefixed framed binary (`bincode` for compactness, `serde_json` for readability — pick one; bincode wins on size).
- **Per snapshot:** the full `Market` state at a known log offset.
- **File layout:** `nyq.log` (append-only events), `nyq.snapshot` (latest snapshot, overwritten atomically via rename).

### `LoggedEvent` shape

```rust
pub enum LoggedEvent {
    SubmitLimit { symbol: Symbol, order: Order },
    Cancel { symbol: Symbol, order_id: OrderID, ts: Ts },
    Fill(FillEvent),
    Quote(QuoteEvent),
    Lifecycle(OrderEvent),
}
```

Note that we log *both* the input action (Submit/Cancel) *and* the resulting events (Fill/Quote/Lifecycle). The engine is deterministic, so input alone would suffice for replay; but logging output gives us cheap divergence detection.

## Algorithm / System Sections

### A) Append-only writer

**Playbook:**
- [ ] `EventLogWriter::new(path)` opens for append + `O_DSYNC` (Linux) / `F_FULLFSYNC` (macOS).
- [ ] `write(LoggedEvent)` serialises with bincode, prefixes with `u32` length, writes, fsyncs.
- [ ] Periodic snapshot every `N` events (e.g. 100k); snapshot writes to a tempfile then renames atomically.

### B) Recovery

**Playbook:**
- [ ] `Market::recover_from(snapshot_path, log_path)`:
  - load latest snapshot,
  - read log from snapshot's recorded offset to EOF,
  - apply each `Submit`/`Cancel` deterministically.
- [ ] Verify recovery by comparing the engine's recovered output against the logged output for the post-snapshot tail.

### C) Crash safety

**Playbook:**
- [ ] Test: write 1000 events, kill the process, recover. Final state matches expected.
- [ ] Test: corrupt the last log frame (truncate). Recovery refuses to apply the partial frame; fails cleanly.

### D) Snapshot atomicity

**Playbook:**
- [ ] Snapshot writes to `nyq.snapshot.tmp`, fsyncs, then `rename`s to `nyq.snapshot`. The rename is atomic on POSIX.
- [ ] If a crash happens during snapshot write, recovery falls back to the previous snapshot + the full log since.

## Integration Points

- The `App` in dashboard mode does not write to disk by default — keep the demo lightweight.
- A `--persist <dir>` CLI flag enables logging for runs that want recovery semantics.
- ITCH replay (per `plans/itch-replay-harness.md`) can be re-cast as "feed messages through the engine which logs them, then verify recovery produces identical state".

## Debugging / Verification

- Determinism property: a recovered engine, fed the same post-recovery input, produces the same output as a fresh engine fed the entire input.
- Log-output divergence detection: if the recovered engine's emitted events differ from the logged events (post-snapshot), there's an engine non-determinism we missed.

## Completion Criteria

- [ ] `src/log/` exists with writer + reader + snapshot.
- [ ] `Market::recover_from(snapshot, log)` returns a `Market` byte-equivalent to pre-crash.
- [ ] `tests/recovery_test.rs` covers: clean recovery, mid-stream crash, truncated final frame, snapshot rotation.
- [ ] `systems/log.md` (new system file) documents the layer.
- [ ] `--persist <dir>` CLI flag works on `cargo run`.
- [ ] This file is archived once all the above are checked.
