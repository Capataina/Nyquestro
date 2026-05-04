# Telemetry Policy

## 1. Current Understanding

Nyquestro records a comprehensive event log of every run to a single local file. The file is **local-only**: no network upload, no aggregation, no analytics, no third-party transmission. The file is yours to inspect, share, or delete.

## 2. Where it lives

The platform-canonical "application data" directory:

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/Nyquestro/last-run.jsonl` |
| Linux | `~/.local/share/nyquestro/last-run.jsonl` |
| Windows | `%LOCALAPPDATA%\Nyquestro\last-run.jsonl` |

Resolved at runtime via `dirs::data_local_dir()`. If the directory cannot be created (permissions, missing parent), telemetry is skipped silently — telemetry must never be the reason the app fails to start.

## 3. Lifecycle

- **Truncate-on-startup.** Every `cargo run` (or every binary launch) wipes the previous file and starts fresh. There is exactly one run on disk at any time, ever.
- **Append during run.** Events are written line-by-line as they occur, with a 200ms BufWriter flush cadence so the file is always near-current on disk.
- **Final flush on exit.** `restore_terminal` triggers a clean shutdown event and final flush before the writer thread exits.

This means: the file is always either (a) the last completed run, (b) the run currently in progress, or (c) absent (if telemetry failed to initialise).

## 4. What's recorded

See `plans/telemetry-and-profiling.md` and `systems/telemetry.md` for the canonical event-by-event description. In summary:

- **Every keystroke** (raw key + parsed `Action` + resulting state).
- **Every engine event**: submit, fill, cancel, reject, quote.
- **Per-frame profiling**: step duration, render duration, actions processed, dropped frames.
- **Per-pane render timing**: which pane took how long.
- **Periodic state snapshots** (every 1s): latency percentiles, throughput counters, book state per symbol.
- **Live-feed events**: connection status, subscriptions, snapshot received, parse errors.
- **App lifecycle**: startup config, terminal resize, shutdown reason.

What's not recorded:
- No personally-identifying information (no usernames, hostnames, IP addresses, account IDs).
- No third-party API credentials (the live feed is unauthenticated; even if we ever add a CDP key, it would not be logged).
- No source-code paths or working-directory information.

## 5. What you can do with it

The file is JSONL — one JSON object per line. Greppable, parseable with any tool:

```bash
# Inspect the last run
cat ~/Library/Application\ Support/Nyquestro/last-run.jsonl | head -20

# Count events by kind
jq -r '.kind' < last-run.jsonl | sort | uniq -c | sort -rn

# Find slow frames
grep '"kind":"frame_slow"' last-run.jsonl

# Look at latency evolution
jq -c 'select(.kind == "latency" and .op == "submit") | [.t, .p99_ns]' < last-run.jsonl
```

When debugging an issue with someone else (Claude, a colleague, an open-source contributor), you can:
- Cat the file and paste relevant excerpts.
- Send the entire file (it's local data; you decide what to share).
- Delete it whenever you want — it'll be regenerated on the next run.

## 6. Schema versioning

Every line includes a `v` field marking the schema version. Current is `v: 1`. Future iterations bump on breaking changes. Parsers should:

- Accept any `v` they recognise.
- Skip unknown `kind` values rather than failing.
- Fail loudly on unparseable lines, never silently.

This is documented in `plans/telemetry-and-profiling.md`.

## 7. Rationale

Why local-only:

- **Zero infrastructure cost.** No analytics service, no S3 bucket, no telemetry endpoint to maintain.
- **Zero privacy concerns.** Anything we record is data the user has direct access to and direct control over.
- **Zero latency overhead from upload.** The writer thread is local-disk-only; the main loop is never blocked on network.
- **Trustworthy by construction.** The user can inspect what's written, verify our claims, and disable the system entirely (delete the directory and the file won't be regenerated until the next run).

Why JSONL:

- **Greppable.** No binary parser required; `grep`, `jq`, `awk`, `head`, `tail` all work.
- **Append-friendly.** Each line stands alone; truncation at any point produces valid lines up to the truncation.
- **Tool-friendly.** Every editor, IDE, log analyser, and shell script knows JSONL.
- **Schema-evolvable.** Adding a field to an event doesn't break old parsers; bumping `v` signals a breaking change.

Why truncate-on-start:

- **Bounded disk usage.** A long-running session can produce a few hundred MB; an unbounded log over months would be a real problem. Truncation ensures the file size is always proportional to the current run, not to history.
- **No log rotation logic.** No cron, no `logrotate`, no multi-file accumulation. One file, one run.
- **Predictable for users.** "What's in the file" always means "what happened in the most recent run." No archaeology.

## 8. Disabling telemetry

Three options, in order of granularity:

1. **Per-run:** delete the file at any time. The current run's events that have already been flushed are gone; subsequent flushes will recreate the file. This won't disable telemetry for the current run, just remove past data.
2. **Per-installation:** delete the entire `Nyquestro/` directory. Same effect plus removes any other app-data we might add in the future.
3. **At build time:** if you want telemetry permanently off, build with `cargo build --release --no-default-features` (a feature flag will be added when this becomes a stated need; today it's always on because telemetry is a load-bearing debugging tool).

Note that disabling telemetry significantly reduces our ability to debug issues from a transcript. The trade-off is the user's call.

## 9. Related Systems and Notes

- `plans/telemetry-and-profiling.md` — full implementation plan, event schema, threading model.
- `systems/telemetry.md` — canonical reference for the implementation.
- `notes/conventions.md` — connects to the no-telemetry-on-hot-path discipline (drop-on-full backpressure).
