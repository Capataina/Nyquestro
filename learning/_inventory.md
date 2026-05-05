# Learning Archive Inventory

Phase 0 output. Every concept, system, decision, comparison, evolution thread, exercise candidate, materials file, interview-layer surface, and path enumerated with stable IDs. Every later phase's `_MANIFEST.md` cites IDs from this file to claim coverage; the Phase Z audit cross-references back against this inventory.

ID schema per `references/discovery-and-phase-planning.md`:

- `INV-F[NN]` foundations · `INV-C[NN]` core · `INV-DP[NN]` domain-patterns · `INV-A[NN]` advanced
- `INV-AR[NN]` architecture · `INV-S[NN]` systems · `INV-D[NN]` decisions · `INV-CMP[NN]` comparisons · `INV-E[NN]` evolution
- `INV-X[NN]` exercises · `INV-M[NN]` materials · `INV-I[NN]` interview-layer · `INV-G[NN]` glossary entries warranting an item · `INV-P[NN]` paths

Status fields appear in the inventory entries so Phase 0 verification can flag readiness:

- `present` — implemented in the current crate at HEAD.
- `partial` — partially implemented; gaps on the README path.
- `planned` — README-defined, not yet implemented.
- `theoretical` — domain context, never lives in code.

---

## Coverage Contract

Pulled directly from the README (`README.md` §Why Nyquestro exists / §What a matching engine actually does / §Architecture / §Performance Model / §Risk Layer / §Strategy Agent / §Testing and Validation / §Features and Roadmap) AND from the LifeOS-vault `notes/hft-firm-priorities.md`'s "universal shortlist" (which encodes what Jane Street, Citadel, HRT, Optiver, Jump, DRW, Tower, Two Sigma actually grade for in HFT projects).

Every bullet below is in `learning/`'s scope. Each becomes one or more INV items.

**Engineering-substance contract:**

1. From-scratch limit-order matching engine in safe Rust (no `unsafe`).
2. Price-time priority matching with multi-level sweep, partial fills, atomic cancellation/modification.
3. Self-match prevention with configurable policy.
4. Order types beyond Limit: IOC, FOK, AON, iceberg / hidden, peg, stop, stop-limit, market.
5. Hot-path discipline: zero allocation, no lock contention, no cache-line-bouncing, branch-prediction-aware.
6. Lock-free order book using atomic CAS at the per-price-level granularity (README ambition; current implementation is BTreeMap+VecDeque single-threaded).
7. Slab allocator for order nodes with lock-free recycling (README ambition; not implemented).
8. Single-producer multi-consumer ring buffer with cache-line-padded cursors (README ambition).
9. NUMA-aware thread-to-core affinity (README ambition).
10. Kernel-bypass packet ingress for zero-copy NIC receive (README stretch).
11. SIMD-accelerated price comparison in the sweep loop (README stretch).
12. Binary UDP gateway with fixed-width frames, versioning, length-prefix, checksum (README; not implemented).
13. FIX TCP acceptor with session management, heartbeats, resend (README; not implemented).
14. Market data multicast publisher: depth snapshots + incremental updates (README; not implemented).
15. Real-time risk guard: fat-finger, position/PnL bounds, rolling-VaR circuit breaker, per-session throttle, fail-safe-on-error (README; planned in `context/plans/risk-layer.md`).
16. Strategy agent: order-book reconstruction, spread/microprice, OFI signal, two-sided quoting, quote skew, inventory tracking, inventory-aware pricing, adverse-selection detector, end-of-session PnL report (README; not implemented).
17. HDR latency histograms with p50 / p99 / p999 / p9999 / max per Op (Submit / Match / Cancel) — implemented.
18. Hardware perf-counter integration (L3 cache miss, branch misprediction, IPC) — README; not implemented.
19. Determinism tests + property-based randomised invariants + fuzz harness + ITCH replay (README; partly implemented — determinism tested in `tests/matching_test.rs`; property-based + fuzz + ITCH planned in `context/plans/`).
20. Crash-only recovery: append-only event journal + snapshot+delta book reconstruction (README; planned in `context/plans/recovery-and-event-log.md`).
21. Multi-instrument routing — implemented (`Symbol(u64)` ASCII pack + `Market` wrapper).
22. Free-real-data validation: Coinbase Advanced Trade WebSocket — shipped (`src/feed/`); LOBSTER/ITCH planned.
23. Microstructure metrics surfaced in the dashboard: microprice, OFI, spread (cents + bps), quote update rate, depth ratio — implemented in `src/ui/panes.rs`.
24. JSONL local-only flight recorder with bounded sync_channel + drop-on-full backpressure — shipped.
25. Ratatui dashboard with infographics theme, ANSI-16-only colour palette, Synthetic + Live modes — shipped.

**Hiring-signal contract** (engineer-to-engineer defence vs generic clone):

26. Why this is not a generic LeetCode-grade matching loop — six concrete differentiators must be crisply articulable.
27. The senior-interview surface for HFT projects (microstructure literacy, tail latency framing, determinism, property-based correctness, risk-before-perf, recovery, multi-instrument, order-type breadth) — taught and rehearsed.
28. The Rust-vs-C++ honest landscape and the "no C++ on CV" interview defence — taught.
29. Per-firm calibration (Jane Street, Citadel, HRT, Optiver, Jump, DRW, Tower, Two Sigma) — taught.

---

## Foundations (INV-F)

Rust-systems-engineering prerequisites needed to read Nyquestro's source comfortably.

- **INV-F01: Safe Rust as engineering discipline** — what `unsafe` would buy in a matching engine, why Nyquestro chose to do without, and the "carefully designed safe abstractions that compile to equivalent machine code" claim. Status: `present`.
- **INV-F02: Newtype primitives & strong typing** — `OrderID`, `Px`, `Qty`, `Ts`, `Symbol(u64)` as distinct types the compiler treats as incompatible. Why integer-soup is dangerous in finance code. Status: `present`.
- **INV-F03: Edition 2024 features** — what the 2024 edition unlocks vs 2021; which features Nyquestro relies on. Status: `present`.
- **INV-F04: `Copy` semantics in event frames** — why `FillEvent`, `QuoteEvent`, `OrderEvent` are `Copy`; what changes when you make a struct `Copy`; cost on the hot path. Status: `present`.
- **INV-F05: `checked_*` vs `saturating_*` vs `wrapping_*` integer arithmetic** — what each does, when each is right, why Nyquestro uses `checked_sub` in `Order::fill` and rejects on overflow. Status: `present`.
- **INV-F06: `Result` types and severity classification** — `NyquestroResult<T>` shape, the `severity()` method on `NyquestroError`, recoverable vs fatal. Status: `present`.
- **INV-F07: When BTreeMap, VecDeque, HashMap each fit** — `BTreeMap<Px, PriceLevel>` for the ladder, `VecDeque<Order>` for FIFO at price, `HashMap<Symbol, OrderBook>` for symbol routing. Why these three choices, not e.g. `IndexMap` or `Vec`. Status: `present`.
- **INV-F08: Borrow checker as concurrency proof** — what data races the borrow checker mechanically prevents, what it doesn't. Status: `theoretical`.
- **INV-F09: Determinism as a first-class invariant** — why determinism is non-negotiable in matching engines (testability, debuggability, replayability, audit), how the matching loop avoids `Ts::now()` to enforce it. Status: `present`.
- **INV-F10: Tokio async-runtime essentials** — the bare-minimum needed to read `src/feed/`: spawn, mpsc, tokio-tungstenite. Status: `present` (used in `feed/`).

## Core (INV-C)

The HFT / market-microstructure / matching-engine concepts the README treats as central. Domain-introducing — the learner is taught these, not assumed to know them.

- **INV-C01: What a matching engine actually does** — the price-time-priority job; why it is the most latency-sensitive piece of software in finance. Status: `theoretical`.
- **INV-C02: Order book structure** — bid/ask ladders sorted by price; why two ladders, not one; how price levels nest within a side. Status: `theoretical`.
- **INV-C03: Price-time priority rule** — best price wins; ties broken by arrival order; the FIFO-at-price invariant. Status: `theoretical`.
- **INV-C04: Aggressive vs passive orders, taker vs maker** — what crosses the spread vs what rests; why passive flow is paid (rebates) and aggressive flow pays (taker fee) on most venues. Status: `theoretical`.
- **INV-C05: Cross detection** — when an incoming order would match against the opposite side; the price-comparison rule; why bids look at asks and vice versa. Status: `theoretical`.
- **INV-C06: Multi-level sweep** — how an aggressive order eats through resting liquidity at successive price levels; quantity accounting at each step. Status: `present`.
- **INV-C07: Partial fills** — fill semantics when incoming quantity exceeds resting at a level; cumulative fill tracking on the aggressor; lifecycle implications. Status: `present`.
- **INV-C08: Order lifecycle states** — placed / partially-filled / fully-filled / cancelled / rejected; one-way state transitions; the `Status::can_transition_to` rule. Status: `present`.
- **INV-C09: Self-match prevention** — what self-match is, why it matters (exchange compliance, anti-wash-trading, real-money risk), the configurable-policy spectrum (cancel newest / cancel oldest / cancel both / decrement-and-cancel). Status: `present` for cancel-newest; configurable policy is `planned`.
- **INV-C10: Top-of-book quote semantics** — when a `QuoteEvent` fires (only when best bid or ask actually changes); why exchange feeds publish quotes not full books; bid-vs-ask-change diff logic. Status: `present`.
- **INV-C11: FIFO at a price level** — the queue behind a price; why it must be FIFO, not LIFO or priority-by-size; queue-position implications for participants. Status: `present`.
- **INV-C12: Microprice** — formula `(bid_size · ask_price + ask_size · bid_price) / (bid_size + ask_size)`; why it is the size-weighted "true mid"; intuition for asymmetric books. Status: `present`.
- **INV-C13: Order Flow Imbalance (OFI)** — Cont/Kukanov definition; what it measures (directional pressure); how the engine and the dashboard compute and use it. Status: `present`.
- **INV-C14: Effective vs quoted spread** — quoted = ask − bid; effective = 2·|trade − mid|; why effective is the real cost participants pay. Status: `theoretical`.
- **INV-C15: Queue position** — what "being 5th in line at the bid" means; why it dominates a market-maker's expected fill rate. Status: `theoretical`.
- **INV-C16: Adverse selection** — informed flow disproportionately picks off makers on the wrong side of where price is about to move; why naive market-makers lose money even at zero variance. Status: `theoretical`.
- **INV-C17: Inventory risk** — the market-maker's mark-to-market exposure problem; why agents skew quotes to rebalance toward flat. Status: `theoretical`.
- **INV-C18: Tick size and minimum increment** — the smallest price unit a venue accepts; why tick size shapes book structure (small ticks → spread fragmentation). Status: `theoretical`.
- **INV-C19: Hidden / iceberg / displayed quantity** — total resting size vs displayed size; why some orders are partially hidden. Status: `planned` in the engine (README); `theoretical` for now.
- **INV-C20: Order types beyond Limit** — IOC, FOK, AON, peg, stop, stop-limit, market — definition, semantics, what each protects against. Status: `planned` (`context/plans/extended-order-types.md`).

## Domain Patterns (INV-DP)

Patterns the codebase uses repeatedly across systems. Distinct from foundations (Rust patterns) and from advanced (theory not yet realised).

- **INV-DP01: Event-sourced engine, immutable event frames** — the engine emits `FillEvent`, `QuoteEvent`, `OrderEvent`; each is `Copy`, validated at construction, never mutated. Why this shape supports replay, audit, and observability. Status: `present`.
- **INV-DP02: Validated constructors returning `NyquestroResult`** — every primitive (`Px`, `Qty`, `OrderID`) and every event (`FillEvent::new`, `OrderEvent::new`) validates inputs and returns `Result`. The "no constructor lies" discipline. Status: `present`.
- **INV-DP03: Single-source severity classification** — `NyquestroError::severity()` is the only place severity is decided; UI and telemetry both ask the error, never re-classify. Status: `present`.
- **INV-DP04: Snapshot + delta recovery** — book state reconstructable from periodic snapshot plus incremental deltas; the same shape used for market-data publication AND crash recovery. Status: `planned`.
- **INV-DP05: Bounded `sync_channel` as flight-recorder transport** — telemetry events queue into a 8192-slot bounded channel; producer never blocks; full-channel events are dropped and counted. Status: `present`.
- **INV-DP06: Drop-on-full backpressure** — telemetry MUST never freeze the dashboard; when the channel is full, increment a drop counter and continue. Status: `present`.
- **INV-DP07: Hot-path / observation-path separation** — the matching path is allocation-free, lock-free, deterministic; the observation path (telemetry, UI render, metrics histogram resize) absorbs all the messy non-deterministic work. Status: `present`.
- **INV-DP08: Multi-instrument routing via `Symbol(u64)` 8-byte ASCII pack** — symbols are 8-byte fixed-size keys, not heap strings; routing is `HashMap<Symbol, OrderBook>` lookup. Why this pattern, not `Vec<(String, OrderBook)>`. Status: `present`.
- **INV-DP09: ANSI-16 colour discipline** — TUI uses only `Color::Reset` and ANSI-16 named colours; never hardcoded RGB; respects the user's terminal theme. Status: `present`.
- **INV-DP10: In-process TUI as observability layer** — the dashboard lives in the same process as the engine; reads engine state read-only via `&self` borrows; no IPC overhead, no deserialisation cost. Status: `present`.
- **INV-DP11: Deterministic replay against fixed input** — same seed + same input sequence ⟹ byte-identical output; pinned by `tests/matching_test.rs::run_twice_identical_sequence_identical_output`. Status: `present`.
- **INV-DP12: Defence-in-depth invariant checking** — `FillEvent::new` rejects self-match independently of the book's own check; the redundancy is intentional (catches engine bugs that produce malformed fills). Status: `present`.

## Advanced (INV-A)

Theory the README invokes or that the project's roadmap will need. The learner needs this to defend the project at senior depth even though most of it is not yet realised in code.

- **INV-A01: Lock-free data structures** — atomic CAS, ABA problem, hazard pointers, RCU, epoch-based reclamation; why they matter under contention; what the README's "lock-free order book" claim actually demands. Status: `theoretical` (README ambition; current impl is single-threaded BTreeMap).
- **INV-A02: Intrusive linked lists** — pointer-walking without heap allocation; the standard structure for slab-allocated order nodes in production matching engines. Status: `theoretical`.
- **INV-A03: Slab allocators** — pre-allocated pools of same-sized slots; lock-free free-list recycling; why they replace the system allocator on the hot path. Status: `theoretical` (README ambition).
- **INV-A04: SPMC ring buffer with cache-line-padded cursors** — false-sharing elimination; the LMAX Disruptor pattern. Status: `theoretical` (README ambition).
- **INV-A05: NUMA architecture and core affinity** — why hot threads must stay on the same NUMA node as their data; topology detection. Status: `theoretical`.
- **INV-A06: Kernel bypass — DPDK, AF_XDP, RDMA** — what kernel bypass is, what it costs to deploy, what it gives you. Status: `theoretical` (README stretch).
- **INV-A07: SIMD for parallel price comparison** — vectorised `i64` comparisons across multiple price levels per instruction. Status: `theoretical`.
- **INV-A08: HDR histograms — Tene's algorithm** — how `hdrhistogram` crate represents distributions in O(log range) memory; bucket layout; resize semantics; what `auto(true)` does. Status: `present`.
- **INV-A09: Ornstein-Uhlenbeck mean-reverting processes** — SDE form `dX = θ(μ − X)dt + σ dW`; mean reversion intuition; why it is a reasonable mid-price model for synthetic flow. Status: `present` in `src/simulator/market.rs`.
- **INV-A10: Poisson order arrivals** — exponential inter-arrival times; rate parameter λ; why Poisson is the textbook order-flow model and where it breaks down. Status: `present`.
- **INV-A11: Log-normal size distributions** — multiplicative size noise; heavy-tailed but bounded variance; why log-normal beats uniform for synthetic order sizes. Status: `present`.
- **INV-A12: NASDAQ TotalView-ITCH 5.0 wire protocol** — message types (Add Order, Order Executed, Cancel, etc.); binary encoding; sequence numbers; the planned ITCH replay path. Status: `theoretical` (planned in `context/plans/itch-replay-harness.md`).
- **INV-A13: FIX 4.4 / 5.0 protocols** — tag=value text frames; session layer (heartbeats, resend); why FIX dominates institutional venues. Status: `theoretical` (README ambition).
- **INV-A14: Coinbase Advanced Trade L2 WebSocket protocol** — channel subscription, snapshot frame, update frame, sequence numbers, gap detection. Status: `present`.
- **INV-A15: Tail-latency theory — why p99/p999/p9999, not p50** — order-of-magnitude impact analysis; "the tail at scale" framing. Status: `theoretical`.
- **INV-A16: Hardware performance counters** — IPC, L3 cache miss rate, branch misprediction rate; what `perf stat` measures; how to correlate counters to source. Status: `theoretical`.
- **INV-A17: Crash-only software** — Candea/Fox 2003; the design philosophy; event journal + idempotent recovery. Status: `theoretical` (planned).
- **INV-A18: Sequence-gap detection and resync** — feed-handler invariant: sequence numbers must be monotone; detection, snapshot re-fetch, replay-to-current. Status: `partial` in `src/feed/coinbase.rs` (snapshot cap in place; resync-on-gap is "next iter").
- **INV-A19: Property-based testing — Hughes/Claessen QuickCheck** — generators, shrinkers, properties as universal-quantifier statements; why proptest beats example-based tests for invariants. Status: `theoretical` (planned in `context/plans/property-based-tests.md`).
- **INV-A20: Mutation testing** — `cargo-mutants` semantics; what survives mutations means; the coverage gap mutation testing reveals. Status: `theoretical` (planned).
- **INV-A21: Criterion benchmarking** — the framework's statistical model (bootstrap CI, regression detection, throughput modes); how to design a benchmark that produces stable numbers. Status: `theoretical` (planned).
- **INV-A22: LOBSTER dataset structure** — message-by-message + book-state CSV format; reconstruction semantics; how LOBSTER differs from raw ITCH. Status: `theoretical` (planned).
- **INV-A23: DEX adapter and on-chain settlement** — README §"Long-Term Direction" stretch: bridging matching engines to Ethereum / Solana smart-contract settlement; the structural similarity (matching loop unchanged) and the structural difference (settlement layer becomes a smart contract instead of a clearinghouse). Connection point to Aurix's DeFi analytics work. Status: `theoretical` (README stretch). *Added during Phase 0 verification gate.*

## Architecture (INV-AR)

Multi-zoom architectural surface. The `project/architecture/` floor demands ≥4 zoom levels.

- **INV-AR01: 10k-foot architecture** — the README's component diagram (Order Gateway → Risk Guard → Matching Engine → Market Data Publisher / Strategy Agent) plus how today's implementation maps onto it. Status: mixed.
- **INV-AR02: 1k-foot module dependency** — the strict-layered dependency graph from `context/architecture.md`: `types` ← `errors` ← `order` / `events` ← `book` / `simulator` / `metrics` ← `ui`; `feed` and `telemetry` as side-cuts. Status: `present`.
- **INV-AR03: 100-foot end-to-end data flow** — `MarketSimulator::step` → `App::handle_submit` → `OrderBook::submit_limit` → fills/quotes/lifecycle → metrics + tape + telemetry → render. Status: `present`.
- **INV-AR04: Code-level critical-path trace** — line-by-line of the four-phase `submit_limit` loop in `src/book/order_book.rs`. Status: `present`.
- **INV-AR05: Synthetic vs Live mode dichotomy** — how `--live coinbase` swaps the simulator for the feed bridge; the shared `SimAction` interface; what changes and what doesn't between modes. Status: `present`.
- **INV-AR06: README-target architecture vs current implementation** — the shipped foundational tier (matching MVP + multi-instrument + observability + Coinbase + telemetry + dashboard) vs the README's lock-free/UDP/risk/strategy-agent ambition; the labelled gap. Status: mixed.

## Systems (INV-S)

One file per `src/` module. Each `project/systems/*.md` must hit the engineering-depth floor (`hallmark-vs-generic`, `performance-and-concurrency`, `failure-mode-and-incident`, `observability-and-debuggability`, `edge-case-enumeration ≥8`, `anticipated-questions ≥8`).

- **INV-S01: types** — `OrderID`, `Side`, `Px`, `Qty`, `Ts`, `Status`, `Symbol(u64)`. Anchor: `src/types.rs`, `context/systems/types.md`. Status: `present`.
- **INV-S02: errors** — `NyquestroError`, severity classification, `NyquestroResult<T>`. Anchor: `src/errors.rs`, `context/systems/errors.md`. Status: `present`.
- **INV-S03: order** — Order entity, fill mechanics, state transitions, cancellation. Anchor: `src/order.rs`, `context/systems/order.md`. Status: `present`.
- **INV-S04: events** — `FillEvent` / `QuoteEvent` / `OrderEvent`, validation, `Copy` semantics. Anchor: `src/events/`, `context/systems/events.md`. Status: `present`.
- **INV-S05: book (matching engine)** — `OrderBook` (BTreeMap ladders + VecDeque FIFO), `PriceLevel`, `Market` multi-instrument wrapper, four-phase `submit_limit`, microstructure inspection (`microprice`, `ofi`, `spread_cents`, `depth`). Anchor: `src/book/`, `context/systems/book.md`. Status: `present`. **The headline file.**
- **INV-S06: metrics** — HDR histograms per Op (Submit / Match / Cancel) at p50/p95/p99/p999/p9999/max; counters; windowed counters at 1s/10s/1min/5min. Anchor: `src/metrics/`, `context/systems/metrics.md`. Status: `present`.
- **INV-S07: simulator** — `MarketSimulator` with OU mid-walk + Poisson arrivals + log-normal sizes; deterministic given a seed. Anchor: `src/simulator/`, `context/systems/simulator.md`. Status: `present`.
- **INV-S08: feed** — Coinbase Advanced Trade `level2` WebSocket client + L2-to-virtual-order bridge; sequence-gap detection in flight. Anchor: `src/feed/`, `context/systems/feed.md`. Status: `present` (gap-resync planned).
- **INV-S09: telemetry** — JSONL flight recorder at platform-canonical app-data path; bounded `sync_channel(8192)`; drop-on-full counter; truncate-on-startup; ~17 event variants. Anchor: `src/telemetry/`, `context/systems/telemetry.md`. Status: `present`.
- **INV-S10: ui (dashboard)** — Ratatui six-pane dashboard, ANSI-16-only theme, Synthetic + Live modes, `Tab` symbol cycling, health-dot system, gauge stack on Engine pane, sparklines on Throughput, distribution bars on Latency, size bars on Trade Tape, pressure bar on DOB. Anchor: `src/ui/`, `context/systems/ui.md`. Status: `present`.
- **INV-S11: testing-strategy.md** (mandatory cross-cutting file role) — current 88-test pyramid (47 inline unit + 41 integration); planned proptest/state-machine/criterion/insta/stress/llvm-cov/cargo-mutants buildout; hardest-to-test paths. Status: `partial` (88 tests present; full pyramid planned).
- **INV-S12: scaling-envelope.md** (mandatory cross-cutting file role) — current single-thread ceiling; predicted 2x/5x/10x bottlenecks; what the lock-free / SPMC / NUMA upgrades unlock. Status: `theoretical` for now.

## Decisions (INV-D)

Each `project/decisions/*.md` must hit the floor: `decision-depth`, Big-O / algorithmic-choice comparison when applicable, `cost-ledger`, `source-citation`, ≥5 challenge-questions.

- **INV-D01: Safe-Rust constraint (no `unsafe`)** — what `unsafe` would buy in a matching engine; what Nyquestro chose to do without. Cited: README §"Why safe Rust specifically", `notes/safe-rust-philosophy.md`. Status: `present`.
- **INV-D02: BTreeMap+VecDeque over heap-based / lock-free** — Big-O at each operation; why this trades performance for correctness simplicity at the MVP tier; what changes when we move to lock-free. Cited: `context/systems/book.md`, README §"Why lock-free and not mutex-protected" (the planned direction). Status: `present` for current; lock-free is `planned`.
- **INV-D03: `Symbol(u64)` 8-byte ASCII pack over String** — Copy-friendliness, allocation-free hot path, 8-byte cache line economy; what the cost is (limit of 8 ASCII chars, not Unicode). Status: `present`.
- **INV-D04: Deterministic match price = resting order's price** — why the resting price, not the aggressor's; the trade-price predictability invariant. Status: `present`.
- **INV-D05: Four-phase `submit_limit` loop (snapshot → match → handle-self-match → rest+emit-quotes)** — why this decomposition; what it makes testable that an interleaved version wouldn't. Status: `present`.
- **INV-D06: Single-source severity classification (`NyquestroError::severity()`)** — why severity lives on the error not on call sites; the maintainability argument. Status: `present`.
- **INV-D07: `Copy` semantics on event frames** — the trade-off (data is duplicated on every emit) vs the alternative (Arc<Event> with reference counting). Status: `present`.
- **INV-D08: JSONL telemetry over OTel / binary** — the local-only constraint; truncate-on-startup; what OTel would buy and what it would cost; why grep-ability beats the network export. Status: `present`. Cited: `notes/telemetry-policy.md`.
- **INV-D09: Drop-on-full backpressure over block** — the dashboard-must-never-freeze constraint; what blocking would do under feed firehose. Status: `present`.
- **INV-D10: ANSI-16 colour discipline over RGB** — terminal-theme respect (Solarized, Catppuccin, accessibility palettes); why hardcoded RGB is degraded UX on a meaningful fraction of users' terminals. Status: `present`. Cited: `notes/dashboard-design.md`.
- **INV-D11: In-process TUI vs separate UI process** — IPC cost, deserialisation cost, debuggability; the alternative (gRPC stream + separate UI process) and why it lost on this project. Status: `present`.
- **INV-D12: Coinbase Advanced Trade vs Binance / Kraken / OKX (initial venue choice)** — auth-free L2, high liquidity in BTC/ETH/SOL, public spec; what the alternatives look like. Status: `present`. Cited: `notes/free-data-sources.md`.
- **INV-D13: Multi-instrument as `Market` wrapper, not `Vec<OrderBook>`** — `HashMap<Symbol, OrderBook>` for O(1) routing; per-symbol simulator; why this scales to N instruments without per-call allocation. Status: `present`.
- **INV-D14: Synthetic flow before live (phase ordering)** — why MVP started with deterministic synthetic; what live data adds; the order matters for testability. Status: `present`.
- **INV-D15: HDR histograms over t-digest / GK-summaries / reservoir sampling** — what each gives you; why HDR's bounded memory at log range is the right fit; auto-resize semantics. Status: `present`.
- **INV-D16: Edition 2024 over Edition 2021** — what 2024 unlocks; what stays the same; why this project pinned to the most recent edition. Status: `present`.
- **INV-D17: Tokio over async-std / smol / mio-direct** — runtime choice for `feed/`; why tokio (ecosystem, mature WS support); what async-std would have changed. Status: `present`.
- **INV-D18: `tokio-tungstenite` over `fastwebsockets` / `ws-rs` / hand-rolled** — what each library trades; why tungstenite for native-tls + connect feature; the per-frame parse cost. Status: `present`.

## Comparisons (INV-CMP)

Side-by-side files where the comparison itself is the lesson. Distinct from decisions in that comparisons survey the design space; decisions explain the chosen point.

- **INV-CMP01: BTreeMap+VecDeque vs heap-based vs lock-free intrusive list** — algorithmic complexity, cache behaviour, contention model, code complexity for each. Status: `present` for first; second/third theoretical.
- **INV-CMP02: JSONL vs OTel vs Prometheus vs binary flight-recorders** — local-only vs network export, per-event cost, query model, what `jq` makes easy. Status: relevant to current.
- **INV-CMP03: In-process TUI vs separate UI process (gRPC / WS / shared-memory)** — latency, debuggability, deployment surface, memory footprint. Status: relevant to current.
- **INV-CMP04: Coinbase Advanced Trade WS vs Binance vs Kraken vs OKX vs Deribit** — message rate, schema cost, auth requirements, available depth, regional gotchas. Status: relevant to current.
- **INV-CMP05: ANSI-16 vs RGB vs 256-color-palette TUI themes** — accessibility, terminal compatibility, theme-respect. Status: relevant to current.
- **INV-CMP06: Safe Rust vs unsafe Rust vs C++ for matching engines** — what each unlocks, what each costs, where the productivity / performance / correctness frontier sits. Status: cross-cutting domain comparison.
- **INV-CMP07: HDR histograms vs t-digest vs GK-summaries vs reservoir sampling vs P²** — memory, accuracy, query model, mergeability. Status: cross-cutting domain comparison.
- **INV-CMP08: FIX vs binary UDP vs ITCH vs JSON for order/market-data wire protocols** — frame cost, parse cost, fault model, bandwidth. Status: cross-cutting domain comparison.
- **INV-CMP09: OU vs Hawkes vs constant-rate Poisson for synthetic order flow** — what each captures (mean reversion, self-excitation, memorylessness); when each is the wrong model. Status: relevant to current.
- **INV-CMP10: Synthetic flow vs ITCH/LOBSTER replay vs live WebSocket validation** — three rungs of validation rigour; what each lets you claim about correctness. Status: cross-cutting (synthetic + live present; replay planned).
- **INV-CMP11: Extensive testing framework — proptest + state-machine + criterion + insta + stress + llvm-cov + cargo-mutants** — what each test layer pins, what it doesn't, why all five matter together; the comparison drives the planned 5-day testing pyramid buildout per `context/plans/extensive-testing-framework.md`. Status: cross-cutting (current 88-test pyramid → planned multi-layer pyramid). *Added during Phase 0 verification gate.*

## Evolution (INV-E)

Timeline-shaped narratives; not bullet inventories.

- **INV-E01: Foundational period (Jun 2025 – Mar 2026)** — primitives, events, errors, the empty matching_engine.rs file; what was learned; why hardening preceded engine. Status: history.
- **INV-E02: 40-day quiet stretch (2026-03-25 to 2026-05-04)** — the gap before the step-change; what was being thought through. Status: history.
- **INV-E03: 2026-05-04 step-change session** — Phase A (hardening) + Phase B (matching MVP + multi-instrument) + Phase C (HDR + simulator + Ratatui) + Phase D (Coinbase + JSONL telemetry) shipped in 16 minutes of commits. Status: history.
- **INV-E04: Vault-vs-local-context divergence** — local `context/` regenerated 2026-05-04; LifeOS `Architecture.md` lagged; the documented gap. Status: pickup-relevant.
- **INV-E05: README aspiration vs implementation reality** — the persistent narrative thread; how it has shaped scope decisions; what remains roadmap. Status: cross-cutting.

## Exercises (INV-X)

Distributed across foundations / core / domain-patterns / project / coding-gate. Coding-gate is project-DSA-curated, not generic.

**Foundations:**
- **INV-X01:** Implement `Side` as a `Copy` enum with the conventions Nyquestro uses.
- **INV-X02:** Implement `Px` with `from_cents` / `from_dollars` / `checked_*` arithmetic; reject zero, reject overflow.
- **INV-X03:** Implement `Status` with the one-way transition rule; expose a `can_transition_to` predicate.
- **INV-X04:** Implement `OrderID` as a non-zero newtype.

**Core:**
- **INV-X05:** Implement `PriceLevel` (FIFO at price) over `VecDeque<Order>`; support push, pop_front, remove-by-id, total_quantity tracking.
- **INV-X06:** Implement single-side ladder `OrderBook` over `BTreeMap<Px, PriceLevel>`; no matching, just placement and inspection.
- **INV-X07:** Add cross detection and one-level fill on the single-side ladder; emit a single `FillEvent`.
- **INV-X08:** Add multi-level sweep with quantity accounting.
- **INV-X09:** Add self-match prevention (cancel-newest policy).
- **INV-X10:** Add `QuoteEvent` emission on top-of-book change.
- **INV-X11:** Compute microprice / OFI / spread / depth on a static book snapshot.

**Domain-patterns:**
- **INV-X12:** Build an event-sourced log + replay; verify byte-identical re-execution.
- **INV-X13:** Implement a bounded telemetry channel with drop counter; benchmark drop rate under firehose.
- **INV-X14:** Build snapshot + delta book recovery from a fragment of an ITCH-style feed.

**Project (extending Nyquestro):**
- **INV-X15:** Add an IOC order type to `OrderBook::submit_limit`.
- **INV-X16:** Add a new metric `Op` (e.g. `CancelByID`); thread it through `MetricsRegistry`.
- **INV-X17:** Port the feed bridge to a new venue (Binance L2 WS); reuse the SimAction interface.
- **INV-X18:** Add p9999 to the latency dashboard pane.
- **INV-X19:** Implement a fat-finger gate as a pre-engine guard; emit rejection events.
- **INV-X20:** Add sequence-gap detection + snapshot resync to `src/feed/coinbase.rs`.

**Coding-gate (project-DSA-curated):**
- **INV-X21:** BTreeMap range queries — implement "iterate all bid levels from best to worst" using `BTreeMap::range`.
- **INV-X22:** VecDeque rotation / `front_mut` / `pop_front` patterns — model the price-level FIFO.
- **INV-X23:** HashMap with 8-byte fixed-size keys — replicate `HashMap<Symbol, V>` performance characteristics.
- **INV-X24:** HDR histogram percentile algorithm — implement bucketed counts with O(log range) memory.
- **INV-X25:** Welford's online variance — power the rolling-VaR circuit-breaker calculation.
- **INV-X26:** Treiber stack / Michael-Scott queue (theory + simple SPSC) — what lock-free structures actually look like.
- **INV-X27:** Intrusive linked list (theory) — what slab-allocated order nodes hang off.

## Materials (INV-M)

`materials/comparable-systems.md` is mandatory with ≥5 production-grade systems.

- **INV-M01: comparable-systems.md** — at minimum: NASDAQ TotalView-ITCH, LSE Millennium Exchange, Eurex T7, CME Globex, Cboe BZX, Coinbase Exchange, Kraken Cryptofacilities, Deribit. ≥5 production systems with structural comparison.
- **INV-M02: HFT primary literature** — Easley/O'Hara market microstructure; Kyle 1985; Stoikov/Avellaneda market-making; Cont/Kukanov OFI papers; Hasbrouck VAR.
- **INV-M03: Rust performance literature** — Rust Performance Book, tokio internals (Carl Lerche talks), Klabnik/Nichols on safe abstractions, Heim's "Rust for low-latency" notes.
- **INV-M04: Lock-free data structures literature** — Herlihy & Shavit *The Art of Multiprocessor Programming*; Treiber 1986; Michael & Scott 1996; Maged Michael on hazard pointers; Fraser/Harris on epoch reclamation.
- **INV-M05: Wire-protocol references** — FIX 4.4 / 5.0 specs; NASDAQ TotalView-ITCH 5.0 spec; Coinbase Advanced Trade WS docs; Binance Streams docs.
- **INV-M06: Free-data-source playbook** — LOBSTER (academic samples), Coinbase / Binance / Kraken WebSocket, Deribit. Distilled from `notes/free-data-sources.md`.
- **INV-M07: HFT-firm public talks and blog posts** — Citadel CRA, Jane Street tech talks (OCaml + correctness), HRT recruiting talks, Optiver dev blog (Rust posts), Jump engineering blog.
- **INV-M08: Property-based / mutation testing references** — Hughes/Claessen QuickCheck paper; proptest book; Hypothesis paper; cargo-mutants rationale.

## Interview-Layer (INV-I)

`interview/` files defend the project against senior-interview probes. Floor: ≥8 anticipated questions per system in `qa-bank.md`; pitches at 60s/3min/10min; mock interviews across six archetypes.

- **INV-I01: 60-second pitch** — what Nyquestro is, why the engineering values are not generic, the headline numbers (88 tests, deterministic engine, multi-instrument live + synthetic, p9999 dashboard).
- **INV-I02: 3-minute pitch** — adds the per-system contour, the headline differentiators, the one place we already validated (LOC-of-test-vs-LOC-of-engine ratio, OFI in the dashboard).
- **INV-I03: 10-minute pitch** — full architectural walkthrough, decisions defended, frontier (what's next + why), the C++ ref-impl plan answer.
- **INV-I04: hallmarks** — the project-specific differentiators that distinguish Nyquestro from a generic textbook matching engine. Cited per-hallmark; engineer-to-engineer defensible.
- **INV-I05: qa-bank** — per-system Q&A, ≥8 each: types/errors/order/events/book/metrics/simulator/feed/telemetry/ui. ~80+ questions total. Cross-checked against systems file mechanism descriptions.
- **INV-I06: domain-prereqs** — assumed-knowledge map for HFT senior interviews: microstructure literacy floor, tail-latency framing, determinism/replay, property-based correctness, risk-before-perf, multi-instrument, order-type breadth.
- **INV-I07: evolution-narrative** — past → present → future, with the 2026-05-04 step-change session as the central narrative beat; what shaped the order; what's next.
- **INV-I08: frontier** — solid (matching MVP + multi-instrument + Coinbase + telemetry + dashboard) / shaky (sequence-gap recovery, full property-based suite) / unknown (lock-free contention behaviour, kernel-bypass deployment, market-maker PnL profile).
- **INV-I09: diagnostic** — pre-path quiz routing the learner to the right starting point.
- **INV-I10: behavioural** — STAR-shaped narratives: a design choice defended, a gotcha debugged, a refactor explained.
- **INV-I11: red-team** — adversarial follow-ups: "why didn't you use lock-free", "show me the proof of price-time priority", "what happens under split-brain", with cited answers.
- **INV-I12: onboarding-others** — how to ramp a new engineer up on Nyquestro: the order they read files, the first task, the discriminating "you've understood this" check.
- **INV-I13: mock-canonical** — standard interview Q&A flow (45-min screen + 60-min deep-dive).
- **INV-I14: mock-adversarial** — hostile interviewer; questions designed to corner.
- **INV-I15: mock-depth-probes** — one topic, 8 levels deep (e.g. "explain self-match" → "now defend the cancel-newest policy" → "now show me where it breaks under iceberg" → ...).
- **INV-I16: mock-breadth-probes** — sample many topics quickly; map breadth coverage.
- **INV-I17: mock-coding** — live coding-style: "implement a multi-level sweep on this stub", "explain the bug in this PriceLevel".
- **INV-I18: mock-ambiguous** — clarifying-question drills: an under-specified prompt; the disciplined questions to ask before coding.
- **INV-I19: recall** — flashcards on terminology, formulas, system invariants, key numbers.
- **INV-I20: articulation drills** — whiteboard scripts, non-expert explanations ("explain price-time priority to your aunt"), the interview-room reach-for-pen-and-paper move.
- **INV-I21: per-firm interview calibration** — per-firm character (Jane Street OCaml + correctness; Citadel C++ + kernel bypass + FPGA + co-location; HRT pure-quant + math; Optiver Rust-friendly + options; Jump distributed + crypto; DRW multi-asset + open-to-non-C++; Tower HFT pure-play; Two Sigma ML-heavy + research) and how the qa-bank emphasis + hallmarks emphasis differ per firm. Anchor: `notes/hft-firm-priorities.md`. *Added during Phase 0 verification gate.*

## Glossary (INV-G)

Entries warranting their own item (not every term — many will be defined inline). Substantial entries:

- **INV-G01..G40** — price-time priority · microprice · OFI · effective spread · queue position · adverse selection · inventory risk · market-making · IOC · FOK · AON · iceberg · peg · stop · stop-limit · self-match · ANSI-16 · ASCII pack · OU process · Poisson arrival · log-normal · HDR histogram · Tene · t-digest · lock-free · CAS · ABA · hazard pointer · RCU · intrusive list · slab allocator · NUMA · kernel bypass · DPDK · AF_XDP · RDMA · ITCH · FIX · MDFA · LOBSTER · Coinbase Advanced Trade · level2 · sequence number · snapshot+delta · property-based testing · proptest · mutation testing · Criterion · tail latency · p99/p999/p9999 · IPC · L3 cache miss · branch misprediction · checked_sub · saturating_sub · BTreeMap · VecDeque · `Symbol(u64)` · `NyquestroError` · `severity()` · `NyquestroResult<T>`.

## Paths (INV-P)

Multiple overlapping routes; one curriculum file is the wrong shape per `learning-architecture.md` §`paths/`.

- **INV-P01: foundations-path** — Rust + systems engineering prerequisites; suitable for an engineer comfortable in another language.
- **INV-P02: domain-theory-path** — HFT / market-microstructure foundations; suitable for an engineer with no exchange exposure.
- **INV-P03: project-architecture-path** — top-down through `architecture.md` + per-system files; suitable for an engineer ready to read code.
- **INV-P04: implementation-first-path** — bottom-up: types → events → order → book → metrics → simulator → feed → telemetry → ui; suitable for someone who learns by following dependency direction.
- **INV-P05: interview-readiness-path** — combines `interview/hallmarks` + `interview/qa-bank` + `decisions/` + `comparisons/` + `interview/evolution-narrative` + `interview/red-team` + the three pitches; the explicitly-CV-track route.
- **INV-P06: coding-gate-path** — DSA drills curated to Nyquestro's actual data structures (BTreeMap, VecDeque, HashMap, HDR histogram, Welford, Treiber, intrusive list).
- **INV-P07: refresh-path** — smallest set of files that re-warms the project after months away (Overview + architecture + ON-THE-TABLE summary + the last decisions file).
- **INV-P08: hiring-signal-articulation-path** — the explicitly-engineer-to-engineer-defence route; pitches + hallmarks + red-team + per-firm calibration.

---

## Inventory totals

| Category | Count |
|---|---|
| Foundations (INV-F) | 10 |
| Core (INV-C) | 20 |
| Domain-Patterns (INV-DP) | 12 |
| Advanced (INV-A) | 23 *(verification: +1)* |
| Architecture (INV-AR) | 6 |
| Systems (INV-S) | 12 (10 systems + testing-strategy + scaling-envelope) |
| Decisions (INV-D) | 18 |
| Comparisons (INV-CMP) | 11 *(verification: +1)* |
| Evolution (INV-E) | 5 |
| Exercises (INV-X) | 27 |
| Materials (INV-M) | 8 |
| Interview-layer (INV-I) | 21 *(verification: +1)* |
| Glossary (INV-G) | ~40 (range placeholder; specific entries finalised in Phase 11) |
| Paths (INV-P) | 8 |
| **Total INV items** | **~221** |

Inventory is exhaustive but allows additive growth. Items added later (during a content phase if a new surface emerges) get appended here and flagged in that phase's `_HANDOFF.md`.
