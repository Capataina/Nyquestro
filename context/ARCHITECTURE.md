# Architecture

## Scope / Purpose

Nyquestro is a from-scratch limit-order matching engine in safe Rust, fronted by a real-time observability TUI built on Ratatui. The crate ships a single binary (`nyquestro`) that runs the engine, a deterministic synthetic-flow simulator, and the dashboard on one thread.

This document is the top-down structural map. Subsystem-level reality lives under `systems/`; design rationale lives under `notes/`.

## Repository Overview

- **Crate name:** `nyquestro`, edition 2024, single workspace member.
- **Binary:** `cargo run` launches the dashboard; `cargo run -- --no-tui [--seed N]` runs the headless 10-second simulation summary.
- **Library surface:** every subsystem is `pub` so external callers (and the integration-test harness) can compose pieces independently.
- **Test posture:** 88 tests at last run — 47 inline unit tests in `src/`, 41 integration tests under `tests/`.
- **External dependencies (5):** `chrono` (human-readable time at the edges), `thiserror` (error derive), `hdrhistogram` (latency percentiles), `ratatui` + `crossterm` (TUI), `rand` + `rand_chacha` (deterministic synthetic flow).

The README (root) describes the long-term ambition — lock-free book, binary UDP gateway, risk guard, market-making agent. This document describes what is actually implemented today.

## Repository Structure

```text
.
├── Cargo.toml              # crate metadata + deps + lint wiring
├── README.md               # long-term project pitch
├── src/
│   ├── lib.rs              # module wiring + crate-level re-exports
│   ├── main.rs             # binary entry; --no-tui flag, --seed flag
│   ├── types.rs            # OrderID, Side, Px, Qty, Ts, Status (foundation)
│   ├── errors.rs           # NyquestroError, ErrorSeverity, NyquestroResult
│   ├── order.rs            # Order entity + state machine
│   ├── events/
│   │   ├── mod.rs          # re-exports
│   │   ├── fill.rs         # FillEvent (validates self-match + zero qty)
│   │   ├── quote.rs        # QuoteEvent + QuoteSide
│   │   └── lifecycle.rs    # OrderEvent::{Placed, Filled, Cancelled, Rejected}
│   ├── book/
│   │   ├── mod.rs          # re-exports OrderBook + PriceLevel + SubmitResult
│   │   ├── price_level.rs  # FIFO queue at one price (VecDeque-backed)
│   │   └── order_book.rs   # BTreeMap-backed bid/ask ladders + submit_limit
│   ├── metrics/
│   │   ├── mod.rs
│   │   ├── windows.rs      # WindowedCounter (rolling 1s/10s/1min/5min)
│   │   ├── counters.rs     # CounterSet (orders/fills/cancels/rejects)
│   │   └── registry.rs     # MetricsRegistry + RegistrySnapshot + Op enum
│   ├── simulator/
│   │   ├── mod.rs
│   │   └── market.rs       # MarketSimulator: Poisson arrivals + OU mid-walk
│   └── ui/
│       ├── mod.rs
│       ├── theme.rs        # ANSI 16 + Color::Reset palette
│       ├── app.rs          # App state, event loop, key handlers
│       └── panes.rs        # render() per-pane functions
└── tests/
    ├── types_test.rs       # 6 tests
    ├── order_test.rs       # 8 tests
    ├── events_test.rs      # 9 tests
    ├── price_level_test.rs # 6 tests
    └── matching_test.rs    # 12 tests (cross, sweep, FIFO, determinism, …)
```

## Subsystem Responsibilities

| Subsystem | Owns | Canonical home |
|-----------|------|----------------|
| `types` | Domain primitives (`OrderID`, `Symbol`, `Side`, `Px`, `Qty`, `Ts`, `Status`) and the order state machine | `systems/types.md` |
| `errors` | `NyquestroError` enum, severity classification, `NyquestroResult<T>` | `systems/errors.md` |
| `order` | `Order` entity: validated construction, fill mechanics, status transitions, cancellation; carries `Symbol` for routing | `systems/order.md` |
| `events` | Three immutable `Copy` event frames emitted by the engine; every event carries `Symbol` so multi-instrument streams disambiguate | `systems/events.md` |
| `book` | Single-symbol `OrderBook` (BTreeMap bid/ask) + `PriceLevel` (FIFO) + multi-instrument `Market` wrapper holding one book per symbol; price-time matching, self-match rejection, top-of-book quote semantics; microstructure inspection (`microprice`, `ofi`, `spread_cents`, `depth`, `level_counts`) | `systems/book.md` |
| `metrics` | HDR latency histograms (Submit/Match/Cancel) with p50/p95/p99/p999/p9999/max + windowed counters (orders/fills/cancels/rejects/quotes) over 1s/10s/1min/5min | `systems/metrics.md` |
| `simulator` | Synthetic order-flow generator per symbol (Poisson per side, OU mid-walk, log-normal sizes); each sim instance is symbol-scoped | `systems/simulator.md` |
| `feed` | Live Coinbase Advanced Trade WebSocket client (`level2` channel, no auth) + L2-to-virtual-order bridge translating per-level updates into the engine's per-order `SimAction` stream | `systems/feed.md` |
| `telemetry` | Local-only flight recorder writing JSONL events (keys, engine events, frame profiles, periodic state snapshots, feed status) to `~/Library/Application Support/Nyquestro/last-run.jsonl`; truncated on every launch; drop-on-full backpressure | `systems/telemetry.md` |
| `ui` | Ratatui dashboard (six panes), terminal-theme-respecting palette, key-driven control with `Tab` cycling between symbols; supports both Synthetic mode (default) and Live mode (`--live coinbase`); rich infographics: gauge stack on Engine pane, sparklines on Throughput, distribution bars on Latency, size bars on Trade Tape, pressure bar on DOB, health-dot system on top status | `systems/ui.md` |

## Dependency Direction

The crate is strictly layered. Each layer depends only on layers below it; there are no cycles.

```
                ┌──────────────────┐
                │       ui         │  (rendering, input, app loop)
                └─────┬────────────┘
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
   ┌─────────┐  ┌──────────┐  ┌──────────┐
   │ metrics │  │ simulator│  │   book   │
   └────┬────┘  └─────┬────┘  └─────┬────┘
        │             │             │
        └─────────────┼─────────────┴────────┐
                      │                      │
                      ▼                      ▼
                  ┌────────┐             ┌────────┐
                  │ events │ ──────────► │ order  │
                  └───┬────┘             └───┬────┘
                      │                      │
                      └──────────┬───────────┘
                                 ▼
                          ┌────────────┐
                          │  errors    │
                          └─────┬──────┘
                                ▼
                          ┌────────────┐
                          │   types    │   (no deps inside the crate)
                          └────────────┘
```

`types` and `errors` are at the bottom — every other module imports them. `order` and `events` sit in the middle. `book`, `simulator`, and `metrics` are independent peers above. `ui` is the only sink; nothing imports `ui`.

## Core Execution / Data Flow

The dashboard's main loop is single-threaded; simulator, engine, and renderer all run on the same thread:

```
[every 50ms]                                           [every 33ms]
MarketSimulator::step(dt)                              Terminal::draw(panes::render)
        │                                                       │
        ▼                                                       │
Vec<SimAction> { Submit(Order), CancelHint, … }                │
        │                                                       │
        ▼                                                       │
App::handle_submit / handle_cancel_hint                         │
        │                                                       │
        ▼                                                       │
OrderBook::submit_limit ─────► PriceLevel::push_back / pop_front
        │                                  │
        ▼                                  ▼
SubmitResult { fills, quotes, lifecycle }  (book mutated in place)
        │
        ├─► tape ring (newest fills, ≤ 200)
        ├─► MetricsRegistry::record_latency(Op::Submit, …)
        ├─► MetricsRegistry::record_orders/fills/rejects(n)
        └─► self-match rejections counted as `total_rejects`
                                                                │
                                                                ▼
                                                     panes::render reads
                                                     (book, metrics.snapshot,
                                                     tape, mid_history,
                                                     sim.mid_cents, sim.config)
```

The render path is read-only on the engine state; nothing in `ui::panes` mutates the book or metrics.

### Inter-System Relationships

| A | B | Mechanism | What flows | Failure mode |
|---|---|-----------|------------|--------------|
| `types` | every other system | Direct import (`crate::types::*`) | Foundation primitives | Type-check failure compile-wide if `types` breaks |
| `errors` | every other system | Direct import (`NyquestroResult<T>`) | Error variants, severity classification | Compile-wide failure; recoverable vs fatal classification used at UI boundary |
| `order` | `book::price_level` | `PriceLevel` borrows `&mut Order` via `front_mut()` to apply fills | Quantity mutation + status transitions | `Order::fill` rejects over-fills before `PriceLevel::record_execution` sees a stale total |
| `order` | `events::lifecycle` | `OrderBook` constructs `OrderEvent::placed/filled/cancelled/rejected` per state change | Event records | An invalid `OrderEvent` would fail `NyquestroError::InvalidQuantity`; `OrderBook` propagates |
| `events` | `book::order_book` | Engine emits `FillEvent`, `QuoteEvent`, `OrderEvent` from `submit_limit` | Three event vectors per call | Self-match check inside `FillEvent::new` doubles as a defensive layer behind the book's own check |
| `book` | `ui::app` | `App::handle_submit` calls `book.submit_limit`; `panes::render_*` walks `book.bid_levels()` / `book.ask_levels()` read-only | Match results + book inspection | Errors from `submit_limit` increment `total_rejects` instead of crashing the UI |
| `book` | `metrics` | `App::handle_submit` records `Op::Submit` and `Op::Match` durations; counters tick on success | `Duration` per call + `record_orders/fills/cancels/rejects(1)` | Histogram saturates rather than panics on outliers (`auto(true)` autoresize) |
| `simulator` | `book` (indirectly via `App`) | `MarketSimulator::step` returns `SimAction::Submit(Order)`; `App` forwards to the book | Pre-validated `Order` instances | Order validation errors are rare-but-possible (e.g. degenerate price); they are counted as rejects |
| `ui::app` | `simulator` | Owns `MarketSimulator`; calls `step(dt)` per sim tick and `reseed(0xC0FFEE)` on `r` keypress | Forward time, drive flow | Pause toggles `EngineState::Paused`; `step` becomes a no-op |
| `ui::app` | `metrics` | Owns `MetricsRegistry`; `panes::render_*` reads `metrics.snapshot()` per render frame | Latency percentiles + counter snapshots | Snapshot is a value type; UI cannot mutate the registry |
| `metrics::counters` | `metrics::windows` | `CounterSet` holds four `WindowedCounter`s; each call to `record_*` lazily prunes entries older than 5 min | `(Instant, count)` records | Memory stays bounded; aging is amortised across record calls |

These relationships are the connective tissue the system files reference at their interface points; full description of any one of them lives in the owning system file (per `cross-system-analysis.md` priority rule).

### Critical Paths and Blast Radius

End-to-end trace of the dominant operation — *aggressive limit order matches one or more resting opposites and the dashboard reflects it*:

```
1. MarketSimulator::step(0.05)             [simulator/market.rs]
     ├─ OU drift on mid-real
     ├─ Poisson sample per (side, channel)
     └─ emits SimAction::Submit(order: Order)

2. App::handle_submit(order)               [ui/app.rs]
     └─ start = Instant::now()

3. OrderBook::submit_limit(order)          [book/order_book.rs]
   ├─ snapshot pre-state of best bid + ask (for change-detection)
   ├─ loop:
   │   ├─ probe opposite top (read-only)
   │   ├─ if not crossing → break
   │   ├─ if resting front.id == aggressor.id → SelfMatch, break
   │   ├─ compute trade_qty = min(aggressor.remaining, resting.remaining)
   │   ├─ resting.fill(trade_qty)            [order.rs — checked_sub]
   │   ├─ level.record_execution(trade_qty)  [book/price_level.rs]
   │   ├─ aggressor.fill(trade_qty)
   │   ├─ FillEvent::new(buyer, seller, …)   [events/fill.rs — defensive self-match check]
   │   ├─ OrderEvent::filled(…) for aggressor
   │   ├─ if resting.is_terminal → pop_front, OrderEvent::filled for resting
   │   └─ if level empty → BTreeMap::remove
   ├─ if remaining > 0 → push to same-side ladder, OrderEvent::placed
   └─ emit_quote_if_changed(side, opposite_side)

4. App returns from submit_limit            [ui/app.rs]
   ├─ metrics.record_latency(Op::Submit, elapsed)
   ├─ metrics.record_latency(Op::Match, elapsed)  [if fills.len() > 0]
   ├─ metrics.record_orders(1)
   ├─ for each fill → metrics.record_fills(1) + tape.push_front
   └─ for each Rejected lifecycle → metrics.record_rejects(1)

5. Next render tick (<= 33ms later)        [ui/panes.rs]
   ├─ book.bid_levels() / ask_levels()      → DOB ladder
   ├─ metrics.snapshot()                    → latency card + throughput card
   ├─ tape iterator                          → trade-tape pane
   └─ sim.mid_cents() / mid_history         → mid-price chart
```

**Blast radius of an interface change in this chain:**

- Change `Order::fill` semantics → directly affects `OrderBook::submit_limit` and every `tests/order_test.rs` + `tests/matching_test.rs` test.
- Change `FillEvent::new` validation → engine's own self-match check is a redundant *defence in depth*; if `FillEvent` accepted self-match, the book's `SelfMatch` check is the load-bearing one.
- Change `MetricsRegistry::record_latency` to take a different unit → every `App::handle_*` call site touches it.
- Change `Px::from_cents` to allow zero → cascades through the entire constructor surface; every `Px::from_*` call would need re-validation.

## Structural Notes / Current Reality

- The matching engine is **deterministic given the input order sequence**: it never calls `Ts::now()` during matching (resting timestamps are reused for fills). The `tests/matching_test.rs::run_twice_identical_sequence_identical_output` test pins this contract.
- Each `MarketSimulator` is also deterministic given a fixed seed (`ChaCha8Rng`); two `MarketSimulator::new(_, 42)` instances produce byte-identical action streams.
- **Multi-instrument by design.** The dashboard runs three default symbols (AAPL @ $150, MSFT @ $300, NVDA @ $500) each with its own simulator instance and its own `OrderBook` inside a shared `Market`. `Tab` cycles which symbol the dashboard focuses on. Adding a fourth symbol is a one-line change in `App::new`.
- **The engine is single-threaded.** No mutexes, no atomics — the dashboard's chosen architecture is "engine + sim + renderer on one thread", which is sufficient for the observability use case and avoids every concurrency hazard at this stage. The README's lock-free / SPMC ring-buffer ambition is deferred work.
- **Self-match policy is enforced at match time** in `OrderBook::submit_limit` (the aggressor is rejected wholly, the resting order is untouched). `FillEvent::new` *also* rejects self-match as a defensive invariant, so a malformed engine-emitted fill would still fail — useful when the book is composed externally.
- **Color discipline:** `src/ui/theme.rs` exposes only `Color::Reset` and ANSI-16 named colors. Hardcoded RGB anywhere would break user terminal themes (Solarized, Catppuccin, accessibility palettes). This rule is structural; one violation degrades the headline visual on a meaningful fraction of users' terminals.
- **No `unsafe` anywhere in the crate.** Verified by grep over `src/`. The README's "safe Rust" claim is upheld today.

### Coverage

This pass inspected: every `.rs` file under `src/` (every file authored in the current session), every test file under `tests/`, `Cargo.toml`, `README.md`, the `git log` window, the running binary's two seed outputs.

Not inspected at code-line depth: `Cargo.lock` (treated as derivable), `target/` (build artefacts), `.gitignore` (config). Inferred-only claims: `crossterm`'s exact behaviour around alternate-screen restore on panic — the `setup_terminal` / `restore_terminal` pair calls the documented APIs but no panic-handler test exists in the repository today.

### Reading Guide

- New reader, "what does this thing do?" → `architecture.md` (this file) → `systems/book.md`.
- Engineer about to change matching semantics → `systems/book.md` then `systems/order.md`.
- Engineer about to change a primitive → `systems/types.md` then read the integration tests in `tests/types_test.rs` + `tests/order_test.rs` for behavioural pinning.
- Designer working on the dashboard → `systems/ui.md` and `notes/dashboard-design.md`.
- Anyone confused about why an idiom is in the code → `notes/conventions.md`, `notes/safe-rust-philosophy.md`.
