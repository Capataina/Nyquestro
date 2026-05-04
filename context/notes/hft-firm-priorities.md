# HFT Firm Priorities

What Jane Street, Citadel Securities, HRT, Jump, Optiver, DRW, Tower Research, and Two Sigma actually look at in candidates and projects. Captured so future iterations of Nyquestro can be scored against the right rubric, not against generic "system design" intuition.

## 1. Current Understanding

These firms are not interchangeable. Their stacks, cultures, hiring rubrics, and the impressions a project lands on them differ in ways that matter for what to build next.

### Per-firm character

| Firm | Primary languages | Cultural anchor | Asset focus |
|------|-------------------|-----------------|-------------|
| **Jane Street** | OCaml (trading), C++ (perf-critical), Python | Math + functional programming + correctness-through-types | ETFs, options, equities |
| **Citadel Securities** | C++, some Python | Latency obsession, kernel bypass, FPGA, co-location | US equities (~40% of retail flow), options |
| **HRT** | C++, Python | Pure quant + academic, very smart, very competitive | Multi-asset |
| **Jump Trading** | C++, Python, some Rust | Distributed systems + crypto | Crypto + traditional |
| **Two Sigma** | Java, C++, Python | ML-heavy, research-flavoured | Multi-asset |
| **DRW** | Java, C++, some Rust | Multi-asset, more open to non-C++ backgrounds | Equities, futures, crypto |
| **Optiver** | C++, Java, **active Rust adoption** | Options market-making, education-friendly | Options |
| **Tower Research** | C++, some Rust | HFT pure-play | Multi-asset |

### What they all watch (the universal shortlist)

- **Tail latency, not mean.** p99.9 / p99.99 / p99.999 / max. A candidate showing only p50 gets dismissed politely. The phrase "tick-to-trade" specifically means network-in to network-out, not internal compute alone.
- **Determinism.** Run the same input twice, get byte-identical output. Replay against a recorded feed is the proof. A non-deterministic engine is uninterviewable; you cannot test it rigorously and you cannot debug it under load.
- **Microstructure literacy.** Order Flow Imbalance (OFI), microprice, effective spread, realised vs quoted spread, queue position, adverse selection cost, fill probability per level. A candidate who can talk about *why* the microprice differs from the mid in unbalanced books has signalled domain depth more than ten thousand benchmark numbers.
- **Property-based correctness.** Price-time priority always holds. No order matches itself. Partial fills sum to the correct total. Cancelled orders never appear in fills. These are invariants a `proptest` or `quickcheck` suite can pin in 200 lines.
- **Risk controls before perf.** Fat-finger price/qty bounds, position limits per session, rolling VaR circuit breaker, per-session throttle, fail-safe-on-error policy. A matching engine without risk is a liability — they'll ask about this in the first 15 minutes.
- **Capacity.** Millions of orders/sec, billions of messages/day. Sustained, not peak. Synthetic load is fine if the curve flattens cleanly.
- **Recovery.** Book state reconstructable from event log. Can the engine come back from a crash? Can it replay from a snapshot + delta?
- **Order types beyond Limit.** IOC (immediate-or-cancel), FOK (fill-or-kill), AON (all-or-none), iceberg / hidden, peg, stop, stop-limit. Real exchanges support all of these; an MVP that supports only Limit is incomplete to a quant's eye.

### What they each *also* watch

- **Jane Street:** clarity of thought; "explain a tricky bug you fixed"; OCaml-idiom appreciation. They love a candidate who can articulate why a strong type system catches bugs early. A Rust matching engine is a *positive* signal here, not a negative one — they recognise the same engineering values.
- **Citadel Securities:** raw C++ depth, lockless idioms, cache discipline, memory barriers, kernel-bypass familiarity. The bar is "show me you know what an L1 cache miss costs and where one would happen in this code".
- **HRT:** algorithmic depth, math fluency, pure-CS rigor. They will hand you a problem and grade reasoning, not vibes.
- **Optiver:** options pricing intuition, structured education path, active Rust hiring. They are the most Rust-friendly of the big firms and they explicitly value pedagogical clarity.

## 2. Rationale

Captured so we don't optimise the project for the wrong audience. A from-scratch Rust matching engine with a beautiful TUI is *not* automatically a good HFT portfolio piece. It becomes one when:

- the engine engages with the universal shortlist above (microstructure metrics, determinism, property-based tests, recovery, risk),
- the project demonstrates understanding of multi-instrument and multiple order types,
- the dashboard surfaces metrics a working trader would recognise (OFI, microprice, quote update rate, fill probability) — not just generic latency percentiles.

## 3. Rust vs C++ — Honest Take

The honest landscape is that C++ dominates HFT. Most production matching engines, gateways, and risk systems are C++. The question is whether a Rust project moves the needle for someone whose CV has no C++.

**Entry-level (new grad, junior):** A deep Rust project with the universal-shortlist content above is a *strong* signal. It demonstrates the underlying engineering values — correctness through types, low-level systems mastery, no `unsafe`, deterministic semantics. They will still ask C++ questions in the interview; the project gets you the interview.

**Senior:** They will expect C++ on top. Without it, the door is mostly closed at HRT, Citadel-Sec proper, Tower; partly open at Jump, DRW, Two Sigma, Optiver; relatively open at Jane Street if your math is strong.

**For Jane Street specifically:** OCaml, not C++, is the cultural match. They respect a deep Rust project the same way they respect a deep OCaml one — both languages encode correctness-through-types. Rust is *not* a worse signal than C++ for them; arguably better.

**For Optiver specifically:** They are actively hiring Rust. A Rust matching engine is on-genre.

### The "no C++ on CV" fix

A small (~200 LOC) C++ reference matching loop, cross-validated against the Rust engine via golden-output tests, is one weekend's work and eliminates the objection cold. Captured as `plans/cpp-reference-impl.md`.

## 4. What Was Tried (in this project's history)

- The README originally pitched the full HFT stack (lock-free book, UDP gateway, risk guard, market-making agent, ITCH replay) before any of it existed. The 2026-05-04 rewrite landed Phase A/B/C of that pitch (hardened core + matching MVP + observability TUI). The Tier-1+ items remain — they are tracked as plans in `plans/` and prioritised in this note.
- Synthetic-only data was the right call for the MVP (zero external dependency, deterministic, free). Beyond MVP, real-data validation is the differentiator. Path tracked in `plans/itch-replay-harness.md` and `plans/live-crypto-feed.md`.

## 5. Guiding Principles

- **Treat tail latency as the headline number, not p50.** The dashboard already shows p50/p95/p99/p999/max — adding p9999 is the right move. Captioning the latency pane "submit" is misleading; we should show all three operations (Submit / Match / Cancel) in parallel.
- **Microstructure metrics earn pane real-estate.** OFI, microprice, spread (cents + bps), quote update rate, order/fill ratio. These are what a working quant looks at. Adding them to the dashboard is the highest-yield-per-hour change available right now.
- **Property-based tests over example-based.** The `proptest` invariants are 200 LOC of code that cover 10⁶ scenarios; the example-based suite covers a few dozen. Capacity scales differently.
- **Multi-instrument is non-negotiable.** A single-symbol engine is a toy. Three synthetic symbols with a `Tab` cycle keybind is a half-day of work and elevates the project from "matching loop demo" to "tiny exchange".
- **Free real data over synthetic.** Crypto WebSocket feeds (Coinbase, Binance, Kraken) are free, real-time, full L2/L3, 24/7. Hooking one of them up is the single most impressive next step. Tracked in `notes/free-data-sources.md` and `plans/live-crypto-feed.md`.
- **Risk before performance.** A perf demo without a risk layer is "look how fast it goes wrong". A 200-line risk-guard module gates that conversation.
- **The C++ reference is for the CV, not the project.** A weekend's work to eliminate the "no C++" objection in interviews. Doesn't need to be production-grade.

## 6. Trade-offs and Constraints

- **No paid data.** Hard constraint. Crypto WebSockets and academic ITCH samples (LOBSTER) are sufficient.
- **No co-location, no FPGA.** We will not be measuring tick-to-trade in nanoseconds against a real wire. The dashboard is enough to demonstrate the *engineering values*; the absolute numbers are not the point.
- **No kernel-bypass.** `io_uring` exists in Rust (`tokio-uring`) but is not on the priority list for this project. Standard sockets via `tokio` would be sufficient if/when networking is added.

## 7. Open Questions

- **Which firms to target first?** Optiver and Jane Street are the highest Rust-receptiveness; Optiver actively hires Rust, Jane Street values the same correctness-through-types philosophy.
- **What to ship before applying?** Minimal credible portfolio: multi-instrument + ITCH replay (synthetic or LOBSTER) + property-based tests + risk-layer stub. All tracked in `plans/`.
- **C++ ref impl: native build or via `cxx` interop?** Probably native build with a shared test harness comparing event vectors byte-for-byte.

## 8. Recommended Next-Step Priority Order

Captured 2026-05-04 after multi-instrument + dashboard polish landed. The plans in `plans/` are independent enough that any of them is a sensible next pickup; the order below is what *most increases the project's signal* per hour of work.

### Tier 1 — pick one of these next

| # | Plan | Effort | Visual impact | Hiring signal | Why this rank |
|---|------|--------|---------------|----------------|---------------|
| 1 | **`plans/extensive-testing-framework.md`** | 5 days, day-by-day shippable | None until benchmarks land | **Highest** | The full multi-layer pyramid (property + stateful-property + criterion benches + insta snapshots + stress + coverage + mutation testing). HFT firms specifically ask "how do you test a matching engine?" — this plan is the answer. Day 1 alone (`plans/property-based-tests.md`) is the minimum viable Tier 1 deliverable; the full plan is the differentiating one. |
| ~~2~~ | ~~`plans/live-crypto-feed.md`~~ | — | — | — | **DONE 2026-05-04.** Live Coinbase Advanced Trade `level2` feed shipped; `cargo run -- --live coinbase` shows real BTC-USD/ETH-USD/SOL-USD depth. See `systems/feed.md`. |

The testing framework is now the unambiguous next pick. The live feed shipped, dropping it from Tier 1. The framework compounds with every later plan: stress tests need stress harness, ITCH replay needs property-test infrastructure to validate, the C++ ref impl needs cross-validation tests, the risk layer needs rejection-mode tests. All of it builds on the testing pyramid.

### Tier 2 — ship after Tier 1

| # | Plan | Effort | Notes |
|---|------|--------|-------|
| 3 | **`plans/risk-layer.md`** | 1–2 days | Production-thinking signal. Common interview question. Adds visible rejection counters and a "kill switch" indicator to the dashboard. |
| 4 | **`plans/itch-replay-harness.md`** Phase 1 | 2 days | LOBSTER CSV replay. The "engine validates against real NASDAQ data" prestige bullet. Phase 1 is friendlier than raw ITCH; Phase 2 (raw binary) can come later. |

### Tier 3 — when the basics are settled

| # | Plan | Effort | Notes |
|---|------|--------|-------|
| 5 | **`plans/extended-order-types.md`** | 2–3 days | IOC, FOK, AON, iceberg. Microstructure depth, but no wow moment. |
| 6 | **`plans/cpp-reference-impl.md`** | 2–3 days (separate weekend) | Career bullet only, no project-runtime value. Eliminates "no C++ on CV" objection. Do this *outside* the main project's iteration loop. |
| 7 | **`plans/recovery-and-event-log.md`** | 2–3 days | Important eventually but not time-sensitive; less interview-relevant than the above. |
| 8 | **`plans/itch-replay-harness.md`** Phase 2 | 2 days | Raw ITCH 5.0 binary parser. Prestige bullet for "I can write a binary protocol parser from spec." |

### Why this ordering

- **Live data demo before rigour proof:** A live crypto feed converts a "synthetic-only" project into a "real-data" project for one day's work; any test suite is invisible to a recruiter skimming. But (2) is still tier 1 because it's faster than (1) and the rigour signal is the strongest single thing for serious technical screens.
- **Risk before ITCH:** A dashboard with a risk layer is closer to "production matching engine" than a dashboard that replays NASDAQ data. Both are good; risk layer is closer to the interview-question shortlist.
- **C++ ref impl is parallel:** It belongs on a different track because it doesn't change the Rust project — it's a CV-grade artefact you do over a weekend separate from main iteration.

### What this ordering deliberately deprioritises

- **Multi-venue arbitrage** (running Coinbase + Binance simultaneously and showing price differences). Cool demo but builds on (1); do after.
- **Lock-free internals.** README ambition. Premature without measured contention. Earns no hiring signal until the project is multi-threaded, and we deliberately deferred multi-threading.
- **Web UI.** A web dashboard is *less* impressive than a TUI for HFT firms (terminals are the cultural anchor). Don't build one.
- **Yet another order type.** AON / iceberg / peg adds depth but doesn't open new conversations. Tier 3 for a reason.

## 10. Related Systems and Notes

- `notes/free-data-sources.md` — the free-data constraint and resources.
- `notes/dashboard-design.md` — the dashboard's design rationale; this note is the *what to add to the dashboard* sequel.
- `plans/itch-replay-harness.md`, `plans/property-based-tests.md`, `plans/cpp-reference-impl.md`, `plans/extended-order-types.md`, `plans/risk-layer.md`, `plans/recovery-and-event-log.md`, `plans/live-crypto-feed.md` — the concrete next steps.
