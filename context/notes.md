# Notes

Index of project-knowledge note files. Each entry summarises the takeaway in one line; the detail lives in the file.

- [conventions](notes/conventions.md) — idiomatic accessors, `checked_*` over `saturating_*`, ANSI-16-only colors, `Copy` events, single-source severity classification
- [safe-rust-philosophy](notes/safe-rust-philosophy.md) — no `unsafe` in the crate, correctness before performance, deterministic matching as a first-class invariant
- [dashboard-design](notes/dashboard-design.md) — design brief that shaped the TUI: layout, color conventions, named-source patterns, synthetic-flow parameters, deliberate avoid-list
- [hft-firm-priorities](notes/hft-firm-priorities.md) — what Jane Street, Citadel, HRT, Jump, Optiver, DRW, Tower, Two Sigma actually look at; honest Rust-vs-C++ landscape; what makes the project most signal-rich for hiring
- [free-data-sources](notes/free-data-sources.md) — the zero-cost-forever constraint and the inventory of free real-market sources (crypto WebSocket as primary, ITCH samples / LOBSTER as secondary)
- [telemetry-policy](notes/telemetry-policy.md) — local-only, no network, truncate-on-startup; what we record, where, how to inspect, how to opt out

## Active plans

These are the concrete next-step plans tracked in `plans/`. **Recommended priority order is captured in [`notes/hft-firm-priorities.md` §8](notes/hft-firm-priorities.md).** TL;DR: live-crypto-feed and property-based-tests are tier 1.

- [extensive-testing-framework](plans/extensive-testing-framework.md) — multi-layer testing pyramid (proptest, proptest-state-machine, criterion, insta, stress, llvm-cov, cargo-mutants); 5-day buildout; supersedes property-based-tests as the canonical Day-1-and-beyond plan
- [property-based-tests](plans/property-based-tests.md) — `proptest` invariants (price-time priority, no self-match, fills sum, etc.); now Day 1 of the testing-framework plan
- [itch-replay-harness](plans/itch-replay-harness.md) — Phase 1 LOBSTER CSV replay + Phase 2 raw ITCH 5.0 binary parser; validates engine against real NASDAQ-derived flow
- [cpp-reference-impl](plans/cpp-reference-impl.md) — small (~200–400 LOC) C++ matching loop cross-validated against Rust; eliminates "no C++ on CV" objection
- [extended-order-types](plans/extended-order-types.md) — IOC, FOK, AON, iceberg, peg, market
- [risk-layer](plans/risk-layer.md) — fat-finger, position limits, throttle, rolling-VaR circuit breaker; pre-trade guard between submission and the engine
- [recovery-and-event-log](plans/recovery-and-event-log.md) — append-only event log + snapshot-and-delta book recovery

Three earlier plans (`live-crypto-feed`, `telemetry-and-profiling`, `dashboard-infographics`) shipped 2026-05-04 and were removed 2026-06-21 per plan hygiene — their durable content lives in `systems/feed.md`, `systems/telemetry.md`, and `systems/ui.md` respectively.

## Active work areas

- `ui/` is the project's headline visual (multi-instrument support landed; panic-during-render restore shipped in `7525618` but is untested — needs a pseudo-terminal harness, see `systems/ui.md`); `TestBackend` snapshot tests are the next concrete iteration.
- `book/` carries the matching engine + the new `Market` multi-instrument wrapper. Feature-complete to MVP.
- `metrics/` records per-op latency (Submit/Match/Cancel) with p50/p95/p99/p999/p9999/max, plus 1s/10s/1min/5min windowed counters for orders/fills/cancels/rejects/quotes.
- `feed/` is brand-new — Coinbase Advanced Trade `level2` WebSocket client + L2-to-virtual-order bridge. `cargo run -- --live coinbase` shows live BTC-USD/ETH-USD/SOL-USD depth. Sequence-gap detection is the obvious next iteration.
