/* ============================================================
   data.js - generated skeleton from upkeep-context arch_seed.py
   schema: v1
   Edit agent-owned sections per references/arch-fill-checklist.md.
   Re-runs of upkeep-context preserve agent prose via arch_merge.py.
   ============================================================ */
window.ARCH = JSON.parse(`{
  "_meta": {
    "deleted_node_ids": [],
    "deleted_edge_keys": [],
    "frontier_locked": false,
    "repotree_locked": false
  },
  "schema": "v1",
  "project": {
    "name": "Nyquestro",
    "file": "context/architecture.html",
    "head": "7525618",
    "headRange": "7525618..7525618",
    "regenerated": "2026-06-21",
    "stack": "Rust 2024 - single binary - single-threaded engine + TUI",
    "milestone": "MVP shipped: matching engine + multi-instrument + dashboard + live feed + telemetry",
    "tests": "102 passing (61 unit + 41 integration), 0 failing",
    "frameBudget": "500 action-dispatch/frame - 33ms render tick - 50ms sim tick",
    "commits": 5,
    "lines": 8722,
    "tagline": "Nyquestro is a from-scratch limit-order matching engine in safe Rust, fronted by a real-time Ratatui observability dashboard. A hardened primitives layer (integer-cents prices, checked-arithmetic quantities, ASCII-packed symbols) underpins a deterministic, multi-symbol order book that matches by price-time priority with match-time self-match rejection. The engine is driven either by a synthetic Ornstein-Uhlenbeck order-flow simulator or by a live Coinbase L2 WebSocket feed; HDR-histogram metrics and a JSONL flight recorder observe it. Everything runs single-threaded on one binary - engine, flow source, and renderer on the same thread - which is the correct core for a future single-writer-sharded design, not a limitation. The README pitches a larger ambition (lock-free book, binary UDP gateway, risk guard, market-making agent); this map describes what is actually implemented today.",
    "purpose": "Orient a new engineer to Nyquestro's runtime layers, ownership boundaries, dependency direction, and the order-submission flow that is the system's dominant operation. This is the map, not the territory - subsystem deep-dives live in context/systems/*.md, design rationale in context/notes/, and forward work in context/plans/.",
    "techStack": [
      { "name": "Rust", "meta": "2024 edition - zero unsafe across the crate" },
      { "name": "ratatui", "meta": "0.29 - TUI, crossterm backend, default-features off" },
      { "name": "crossterm", "meta": "0.28 - terminal control, raw mode, alt screen" },
      { "name": "hdrhistogram", "meta": "7.5 - p50/p99/p999/p9999 latency percentiles" },
      { "name": "tokio", "meta": "1.x - async runtime for the Coinbase feed thread" },
      { "name": "tokio-tungstenite", "meta": "0.24 - WebSocket, native-tls (macOS Security.framework)" },
      { "name": "futures-util", "meta": "0.3 - stream combinators on the feed" },
      { "name": "serde / serde_json", "meta": "1.x - JSONL telemetry + feed message parsing" },
      { "name": "rand / rand_chacha", "meta": "0.8 / 0.3 - ChaCha8 deterministic synthetic flow" },
      { "name": "chrono", "meta": "0.4 - human-readable timestamps at the edges" },
      { "name": "thiserror", "meta": "1.0 - NyquestroError derive" },
      { "name": "dirs / url", "meta": "5 / 2 - platform data dir + WS endpoint parsing" }
    ]
  },
  "nodes": [
    {
      "id": "types",
      "label": "types",
      "kind": "foundation",
      "layer": 0,
      "root": "src/types.rs",
      "tagline": "Strongly-typed domain primitives the entire crate is built on.",
      "owns": "OrderID, Symbol (8-byte ASCII pack so u64 lex order matches string order), Px (integer cents, from_dollars rounds), Qty (u32, checked arithmetic only), Ts (infallible now(), Ts(0) fallback), Side, Status (one-way state machine). Does NOT own the Order entity (that is the order node) or any error variants (errors node).",
      "files": [
        "types.rs - OrderID/Symbol/Px/Qty/Ts/Side/Status primitives + the Status transition table"
      ],
      "state": ["OrderID", "Symbol", "Px", "Qty", "Ts", "Side", "Status"]
    },
    {
      "id": "errors",
      "label": "errors",
      "kind": "foundation",
      "layer": 0,
      "root": "src/errors.rs",
      "tagline": "Scoped error taxonomy with a single severity classifier.",
      "owns": "NyquestroError (scoped variants only - no Recoverable/Fatal catch-alls), ErrorSeverity, NyquestroResult<T>, and the severity() classifier. Does NOT own any business logic; every other system imports these for fallible operations.",
      "files": [
        "errors.rs - NyquestroError enum (16 variants), ErrorSeverity, severity() classifier"
      ],
      "state": ["NyquestroError", "ErrorSeverity", "NyquestroResult"]
    },
    {
      "id": "order",
      "label": "order",
      "kind": "foundation",
      "layer": 1,
      "root": "src/order.rs",
      "tagline": "The Order entity: validated construction, fill mechanics, status transitions, cancellation.",
      "owns": "Order struct with caller-supplied timestamps (for determinism), two-phase fill() (transition then write remaining) using checked_sub for over-fill detection, &self accessors. Does NOT own matching (book) or event emission (events); carries Symbol for routing but does not route.",
      "files": [
        "order.rs - Order entity, fill()/cancel()/status transitions, validated construction"
      ],
      "state": ["Order"]
    },
    {
      "id": "events",
      "label": "events",
      "kind": "foundation",
      "layer": 1,
      "root": "src/events/",
      "tagline": "Three immutable Copy event frames emitted by the engine, each carrying Symbol.",
      "owns": "FillEvent (validates self-match + zero-qty), QuoteEvent + QuoteSide (live()/cleared() constructors), OrderEvent::{Placed,Filled,Cancelled,Rejected} + OrderRejectionReason. Does NOT own emission policy (the book decides when to emit) - it owns the frame shapes and their validation invariants.",
      "files": [
        "fill.rs - FillEvent, self-match + zero-qty validation as defence in depth",
        "lifecycle.rs - OrderEvent variants + OrderRejectionReason (no String alloc)",
        "quote.rs - QuoteEvent + QuoteSide, live()/cleared() split",
        "mod.rs - re-exports"
      ],
      "state": ["FillEvent", "QuoteEvent", "OrderEvent", "OrderRejectionReason"]
    },
    {
      "id": "book",
      "label": "book",
      "kind": "env",
      "layer": 2,
      "root": "src/book/",
      "tagline": "The matching engine: per-symbol BTreeMap order books with price-time priority.",
      "owns": "PriceLevel (VecDeque FIFO, running total_quantity), OrderBook (BTreeMap<Px,PriceLevel> per side, submit_limit matching loop, microstructure inspection: microprice/ofi/spread_cents/depth/level_counts), Market (BTreeMap<Symbol,OrderBook>, auto-registration). Does NOT own order construction (order) or the flow that drives it (simulator/feed). Determinism contract: never calls Ts::now() during matching.",
      "files": [
        "price_level.rs - VecDeque FIFO at one price, O(1) push_back/pop_front, O(n) remove_by_id",
        "order_book.rs - BTreeMap bid/ask ladders, submit_limit four-phase matching, microstructure",
        "market.rs - Market wrapper: one OrderBook per Symbol, auto-register on first submit",
        "mod.rs - re-exports OrderBook + PriceLevel + SubmitResult"
      ],
      "state": ["Market", "OrderBook", "PriceLevel", "SubmitResult"]
    },
    {
      "id": "simulator",
      "label": "simulator",
      "kind": "env",
      "layer": 2,
      "root": "src/simulator/",
      "tagline": "Synthetic order-flow generator: Ornstein-Uhlenbeck mid-walk + Poisson arrivals.",
      "owns": "MarketSimulator (per-symbol mid_real OU walk, Poisson per-side arrivals, log-normal sizes, distance-decay limit prices), SimConfig, SimAction::{Submit,CancelHint}. The dt fed to step() is capped (min(1/theta, 0.25)) for OU stability and bounded order bursts. Does NOT own the book it feeds; deterministic given a fixed ChaCha8 seed.",
      "files": [
        "market.rs - MarketSimulator: OU mid-walk (dt-capped), Poisson arrivals, gen_limit/gen_market",
        "mod.rs - re-exports MarketSimulator/SimConfig/SimAction"
      ],
      "state": ["MarketSimulator", "SimConfig", "SimAction"]
    },
    {
      "id": "metrics",
      "label": "metrics",
      "kind": "observer",
      "layer": 2,
      "root": "src/metrics/",
      "tagline": "HDR latency histograms + rolling windowed counters - measure the tail, not the average.",
      "owns": "MetricsRegistry (per-Op HDR histograms -> p50/p95/p99/p999/p9999/max), CounterSet (orders/fills/cancels/rejects/quotes), WindowedCounter (1s/10s/1min/5min rolling via VecDeque), Op enum, RegistrySnapshot. Read-only observer: nothing it does mutates engine state. Does NOT own the timing call sites (ui records into it).",
      "files": [
        "registry.rs - MetricsRegistry + RegistrySnapshot + Op enum (Submit/Match/Cancel)",
        "counters.rs - CounterSet over orders/fills/cancels/rejects/quotes",
        "windows.rs - WindowedCounter rolling 1s/10s/1min/5min via VecDeque pruning",
        "mod.rs - re-exports"
      ],
      "state": ["MetricsRegistry", "CounterSet", "WindowedCounter", "RegistrySnapshot", "Op"]
    },
    {
      "id": "feed",
      "label": "feed",
      "kind": "boundary",
      "layer": 3,
      "root": "src/feed/",
      "tagline": "Live Coinbase Advanced Trade L2 WebSocket boundary, translated to virtual orders.",
      "owns": "Coinbase WS client (level2 channel, no auth, native-tls), Bridge that maintains HashMap<(Symbol,Side,Px),OrderID> to translate per-level L2 deltas into the engine's per-order SimAction stream, FeedAction/FeedEvent, SNAPSHOT_LEVEL_CAP (50 levels/side, closest-to-touch). Does NOT own the engine; it is an alternative flow source equivalent to simulator. Failure isolated to the feed thread.",
      "files": [
        "coinbase.rs - tokio-tungstenite WS client, level2 subscribe, snapshot/delta parse",
        "bridge.rs - L2-to-virtual-order translation, idempotent cancel via (sym,side,px)->id map",
        "mod.rs - re-exports run_coinbase/Bridge/CoinbaseConfig/FeedAction/FeedEvent"
      ],
      "state": ["Bridge", "CoinbaseConfig", "FeedAction", "FeedEvent"]
    },
    {
      "id": "telemetry",
      "label": "telemetry",
      "kind": "observer",
      "layer": 3,
      "root": "src/telemetry/",
      "tagline": "Local-only JSONL flight recorder; can never freeze the dashboard.",
      "owns": "TelemetryEvent (~17 variants), TelemetryHandle (try_send + AtomicU64 drop counter), the writer thread that appends one JSON object per line to the platform data dir (truncate-on-launch, one run on disk). Bounded sync_channel(8192) means back-pressure drops events rather than blocking. Does NOT own any engine state; pure read-and-record observer.",
      "files": [
        "events.rs - TelemetryEvent enum (~17 variants) + serde wrapper with schema v:1",
        "writer.rs - spawn_writer, bounded sync_channel, BufWriter flush cadence, drop-on-full",
        "mod.rs - re-exports TelemetryEvent/TelemetryHandle/spawn_writer"
      ],
      "state": ["TelemetryEvent", "TelemetryHandle"]
    },
    {
      "id": "ui",
      "label": "ui",
      "kind": "observer",
      "layer": 4,
      "root": "src/ui/",
      "tagline": "Ratatui dashboard: owns the run loop, drives the engine, renders six infographic panes.",
      "owns": "App (Market + per-symbol state + metrics + telemetry handle + rate rings), the run loop (33ms render / 50ms sim tick / 500 action budget), key handling (Tab cycles symbols, +/- speed, pause, reset), the ANSI-16 theme helpers (sparkline, distribution_bar, microprice_axis, pressure_bar), the panic hook that restores the terminal. Sink of the dependency graph - nothing imports ui. Render path is read-only on engine state.",
      "files": [
        "app.rs - App state, run loop, key handlers, scaled_dt, install_panic_hook/restore_terminal_raw",
        "panes.rs - per-pane render fns (DOB, engine gauges, throughput, tape, latency, mid chart)",
        "theme.rs - ANSI-16 + Color::Reset palette, block/sparkline/distribution helpers",
        "mod.rs - re-exports"
      ],
      "state": ["App", "EngineState", "Mode"]
    },
    {
      "id": "entry",
      "label": "entry",
      "kind": "entry",
      "layer": 5,
      "root": "src/main.rs",
      "tagline": "Binary entry + crate wiring: flags, telemetry spawn, feed wiring, dashboard launch.",
      "owns": "main() flag parsing (--no-tui, --seed, --live coinbase), spawning the telemetry writer, wiring the Coinbase feed receiver into App::new_live, and launching the dashboard run loop. lib.rs is the module wiring + crate re-exports (every subsystem is pub for the integration-test harness). Does NOT own any subsystem logic; pure composition root.",
      "files": [
        "main.rs - binary entry, flag parsing, telemetry/feed wiring, dashboard or headless launch",
        "lib.rs - module declarations + crate-level re-exports (all subsystems pub)"
      ],
      "state": ["cli flags"]
    }
  ],
  "edges": [
    { "from": "order", "to": "types", "rel": "dep", "label": "OrderID/Px/Qty/Ts/Side/Status primitives" },
    { "from": "order", "to": "errors", "rel": "dep", "label": "NyquestroResult on fill / status transition" },
    { "from": "events", "to": "types", "rel": "dep", "label": "Symbol/Px/Qty/Side carried in every frame" },
    { "from": "events", "to": "errors", "rel": "dep", "label": "self-match / zero-qty validation returns NyquestroError" },
    { "from": "book", "to": "order", "rel": "strong", "label": "resting Order.fill() via PriceLevel::front_mut" },
    { "from": "book", "to": "events", "rel": "strong", "label": "emits Fill/Quote/Order frames per submit_limit" },
    { "from": "book", "to": "types", "rel": "dep", "label": "Px-keyed BTreeMap ladders, Symbol routing" },
    { "from": "book", "to": "errors", "rel": "dep", "label": "SelfMatch / InvalidQuantity propagation" },
    { "from": "simulator", "to": "order", "rel": "dep", "label": "gen_limit/gen_market build validated Order" },
    { "from": "simulator", "to": "types", "rel": "dep", "label": "Px/Qty/Side/Symbol/Ts construction" },
    { "from": "feed", "to": "simulator", "rel": "dep", "label": "L2 deltas translated into the same SimAction stream" },
    { "from": "feed", "to": "order", "rel": "dep", "label": "virtual per-order Orders from L2 price levels" },
    { "from": "feed", "to": "types", "rel": "dep", "label": "Symbol/Px/Qty/Side" },
    { "from": "metrics", "to": "types", "rel": "dep", "label": "Op enum keys the histograms" },
    { "from": "telemetry", "to": "events", "rel": "dep", "label": "serialises Fill/Quote/Order event data" },
    { "from": "telemetry", "to": "types", "rel": "dep", "label": "Symbol/Px/Qty in JSONL rows" },
    { "from": "ui", "to": "book", "rel": "strong", "label": "submit_limit + read-only DOB/microprice/ofi inspection" },
    { "from": "ui", "to": "simulator", "rel": "strong", "label": "drives MarketSimulator::step(scaled_dt) per sim tick" },
    { "from": "ui", "to": "metrics", "rel": "dep", "label": "records Op latencies; reads snapshot() per frame" },
    { "from": "ui", "to": "feed", "rel": "dep", "label": "consumes FeedAction in --live mode" },
    { "from": "ui", "to": "telemetry", "rel": "write", "label": "emits TelemetryEvent at every state-changing call site" },
    { "from": "ui", "to": "events", "rel": "dep", "label": "renders Fill/Order frames in the trade tape" },
    { "from": "entry", "to": "ui", "rel": "strong", "label": "main() launches the dashboard run loop" },
    { "from": "entry", "to": "feed", "rel": "dep", "label": "main wires the Coinbase receiver for --live" },
    { "from": "entry", "to": "telemetry", "rel": "dep", "label": "main spawns the JSONL writer thread" }
  ],
  "kindMeta": {
    "entry": { "label": "Entry point", "swatch": "neutral" },
    "foundation": { "label": "Foundation", "swatch": "slate" },
    "env": { "label": "Environment truth", "swatch": "cyan" },
    "boundary": { "label": "Control boundary", "swatch": "teal" },
    "learner": { "label": "Learner", "swatch": "violet" },
    "observer": { "label": "Observer (read-only)", "swatch": "amber" }
  },
  "layers": [
    { "name": "Primitives", "role": "types + errors - integer-cents prices, checked-arithmetic quantities, scoped error taxonomy. No intra-crate dependencies." },
    { "name": "Domain entities", "role": "order + events - the validated Order entity and the three immutable Copy event frames it produces." },
    { "name": "Engine + flow + measurement", "role": "book (matching), simulator (synthetic OU flow), metrics (HDR latency) - independent peers above the entities." },
    { "name": "Boundaries", "role": "feed (live Coinbase L2 in) + telemetry (JSONL flight recorder out) - the two edges of the system." },
    { "name": "Presentation", "role": "ui - the dashboard owns the run loop, drives the engine, renders read-only. The only sink." },
    { "name": "Entry", "role": "main.rs + lib.rs - composition root: flags, wiring, launch." }
  ],
  "layersNote": "Strictly layered, no cycles: each layer depends only on layers below it. ui reads from every layer below but is read-only on engine state and must never become a source of truth - panes::render never mutates the book or metrics. The whole engine is single-threaded by design (no mutexes, no atomics on the hot path except the telemetry drop counter); the README's lock-free / SPMC-ring ambition is deferred work, and the current single-threaded core is the right foundation for a future single-writer-sharded design.",
  "dataFlow": {
    "intro": "The dominant operation traced end-to-end: an aggressive limit order is generated, matched against one or more resting opposites, recorded, and reflected on the dashboard. In synthetic mode step 1 is the simulator; in --live mode the same SimAction stream arrives from the feed/bridge instead. Every step below is single-threaded on the dashboard's main thread.",
    "simsets": ["Generate", "Submit", "Match", "Record", "Render"],
    "steps": [
      { "n": 1, "sys": "simulator", "fn": "MarketSimulator::step(scaled_dt)", "set": "Generate", "reads": "cfg, ChaCha8 rng, mid_real (dt clamped to min(1/theta,0.25))", "writes": "Vec<SimAction::Submit(Order)>", "fail": true },
      { "n": 2, "sys": "ui", "fn": "App::step", "set": "Submit", "reads": "dt_secs, speed", "writes": "scaled_dt = dt_secs * speed; drains SimAction", "fail": false },
      { "n": 3, "sys": "ui", "fn": "App::handle_submit(order)", "set": "Submit", "reads": "order", "writes": "start = Instant::now()", "fail": false },
      { "n": 4, "sys": "book", "fn": "OrderBook::submit_limit", "set": "Match", "reads": "best bid + ask pre-state (change detection)", "writes": "begins matching loop", "fail": false },
      { "n": 5, "sys": "book", "fn": "matching loop probe", "set": "Match", "reads": "opposite top of book (read-only)", "writes": "break if not crossing", "fail": false },
      { "n": 6, "sys": "book", "fn": "self-match guard", "set": "Match", "reads": "resting.front.id vs aggressor.id", "writes": "SelfMatch -> reject aggressor, break", "fail": true },
      { "n": 7, "sys": "order", "fn": "resting.fill(trade_qty)", "set": "Match", "reads": "remaining (checked_sub)", "writes": "remaining -= trade_qty, status transition", "fail": true },
      { "n": 8, "sys": "book", "fn": "PriceLevel::record_execution", "set": "Match", "reads": "level total_quantity", "writes": "decrement level total", "fail": false },
      { "n": 9, "sys": "order", "fn": "aggressor.fill(trade_qty)", "set": "Match", "reads": "aggressor remaining", "writes": "remaining -= trade_qty", "fail": false },
      { "n": 10, "sys": "events", "fn": "FillEvent::new(buyer, seller, ...)", "set": "Match", "reads": "buyer/seller ids", "writes": "FillEvent (defensive self-match re-check)", "fail": true },
      { "n": 11, "sys": "events", "fn": "OrderEvent::filled", "set": "Match", "reads": "aggressor + resting state", "writes": "lifecycle frames", "fail": false },
      { "n": 12, "sys": "book", "fn": "pop_front / BTreeMap::remove", "set": "Match", "reads": "resting terminal? level empty?", "writes": "evict filled resting + empty level", "fail": false },
      { "n": 13, "sys": "book", "fn": "rest remainder + emit_quote_if_changed", "set": "Match", "reads": "remaining, top-of-book delta", "writes": "OrderEvent::placed + QuoteEvent; reuses resting Ts (never Ts::now())", "fail": true },
      { "n": 14, "sys": "metrics", "fn": "record_latency(Op::Submit/Match)", "set": "Record", "reads": "elapsed = start.elapsed()", "writes": "HDR histogram sample", "fail": false },
      { "n": 15, "sys": "metrics", "fn": "record_orders/fills/rejects", "set": "Record", "reads": "SubmitResult counts", "writes": "windowed counters + tape.push_front", "fail": false },
      { "n": 16, "sys": "telemetry", "fn": "TelemetryHandle::record", "set": "Record", "reads": "TelemetryEvent", "writes": "try_send (drop-on-full) -> JSONL writer thread", "fail": true },
      { "n": 17, "sys": "ui", "fn": "panes::render", "set": "Render", "reads": "book, metrics.snapshot(), tape, mid_history (read-only)", "writes": "frame buffer", "fail": false },
      { "n": 18, "sys": "ui", "fn": "Terminal::draw (33ms tick)", "set": "Render", "reads": "frame buffer", "writes": "terminal", "fail": false }
    ]
  },
  "failures": [
    { "step": "6,10", "link": "6 -> 10", "title": "Self-match rejection is match-time and double-defended", "body": "When a resting order and the aggressor share an OrderID, the book rejects the aggressor wholly and leaves the resting counterparty untouched (step 6). FillEvent::new re-checks self-match as defence in depth (step 10), so even a malformed externally-composed engine cannot emit a self-match fill. If either guard is removed, an order could trade against itself - a correctness violation that corrupts position and PnL accounting downstream." },
    { "step": "7", "link": "7 checked_sub", "title": "Over-fill is impossible: checked_sub only", "body": "Qty exposes only checked_sub / checked_add, never saturating_sub. The historical silent over-fill bug came from saturating_sub flooring an underflow at zero; refusing to expose it forces every caller to handle the underflow. Order::fill is two-phase (transition first, write remaining second) so a status-transition failure cannot leave an order half-filled. Break this and fills can exceed resting quantity, manufacturing phantom liquidity." },
    { "step": "13", "link": "1 -> 13", "title": "Determinism: matching never reads the clock", "body": "The matching loop never calls Ts::now(); fills inherit the resting order's timestamp (step 13), and the simulator is seeded ChaCha8 (step 1). The same input sequence therefore produces byte-identical events, pinned by tests/matching_test.rs::run_twice_identical_sequence_identical_output. This is what lets a trading day be replayed for debugging. Any Ts::now() call inside matching silently breaks replay determinism." },
    { "step": "16", "link": "16 try_send", "title": "Telemetry can never freeze the engine", "body": "The flight recorder uses a bounded sync_channel(8192) with try_send and an AtomicU64 drop counter (step 16). Under back-pressure it drops events and records the drop count rather than blocking the main thread on disk I/O. If this became a blocking send, a slow disk would stall the entire single-threaded engine + render loop." },
    { "step": "1", "link": "1 dt-cap", "title": "Simulator mid cannot diverge (OU stability)", "body": "The OU mid is integrated by explicit Euler, stable only while theta*dt < 2. step() clamps dt to min(1/theta, 0.25) and guards mid_real finite (step 1); without the cap a frame hitch at elevated speed diverged the mid to a near-u64::MAX value whose inter-sample delta overflowed ratatui's sparkline multiply and crashed the app (fixed in ceed2e9, regression-tested)." }
  ],
  "relationships": [
    { "a": "ui", "b": "book", "mech": "App::handle_submit calls submit_limit; panes walk bid_levels/ask_levels read-only", "data": "Order in; SubmitResult (fills/quotes/lifecycle) out; microstructure reads", "breaks": "Errors from submit_limit increment total_rejects instead of crashing the UI; a panic here is caught by the render-loop panic hook" },
    { "a": "ui", "b": "simulator", "mech": "App owns MarketSimulator; calls step(scaled_dt) per 50ms sim tick, reseed on 'r'", "data": "scaled_dt in; Vec<SimAction> out", "breaks": "Uncapped scaled_dt once diverged the OU mid - now capped inside step(); pause makes step a no-op" },
    { "a": "book", "b": "order", "mech": "PriceLevel borrows &mut Order via front_mut() to apply fills", "data": "trade_qty mutation + status transition", "breaks": "Order::fill rejects over-fills before record_execution sees a stale total" },
    { "a": "book", "b": "events", "mech": "submit_limit constructs Fill/Quote/Order frames per state change", "data": "three event vectors per call", "breaks": "FillEvent::new self-match check is a redundant defensive layer behind the book's own SelfMatch guard" },
    { "a": "ui", "b": "metrics", "mech": "App records Op::Submit/Match durations; panes read snapshot() per frame", "data": "Duration per call + counter ticks; RegistrySnapshot value out", "breaks": "Histogram auto-resizes rather than panicking on outliers; snapshot is a value type so UI cannot mutate the registry" },
    { "a": "ui", "b": "telemetry", "mech": "App holds TelemetryHandle; emits TelemetryEvent at every state-changing call site", "data": "~17 event kinds via try_send", "breaks": "Channel-full drops are counted, never block; telemetry failure degrades to no recording, never a stall" },
    { "a": "feed", "b": "book", "mech": "Bridge translates Coinbase L2 deltas into SimAction; App forwards to submit_limit", "data": "per-level L2 update -> virtual per-order Submit/Cancel", "breaks": "Snapshot capped at 50 levels/side protects the engine from Coinbase's 25k-level connect firehose; feed failure isolated to its thread" },
    { "a": "feed", "b": "simulator", "mech": "Feed emits the same SimAction enum the simulator produces (shared boundary contract)", "data": "SimAction::{Submit,CancelHint}", "breaks": "If SimAction's shape changes, both the synthetic and live paths must change together - it is the flow-source contract" },
    { "a": "simulator", "b": "book", "mech": "MarketSimulator::step returns SimAction::Submit(Order); App forwards to the book", "data": "pre-validated Order instances", "breaks": "Degenerate prices are rare but possible; they are counted as rejects, not crashes" },
    { "a": "order", "b": "types", "mech": "Order is built from OrderID/Px/Qty/Ts/Side/Status; fill() uses Qty::checked_sub", "data": "primitive construction + arithmetic", "breaks": "If Px allowed zero or Qty exposed saturating_sub, the over-fill / zero-price invariants would silently break" },
    { "a": "events", "b": "types", "mech": "Every event frame carries Symbol so multi-instrument streams disambiguate", "data": "Symbol/Px/Qty/Side", "breaks": "A missing Symbol on an event would make multi-symbol fills ambiguous downstream" },
    { "a": "metrics", "b": "ui", "mech": "MetricsRegistry is owned by App; the headless benchmark (planned) will record into the same registry", "data": "Op latencies + counter snapshots", "breaks": "The HDR percentiles exist but are only surfaced inside the dashboard today; benchmark-harness plan exposes them headless" },
    { "a": "telemetry", "b": "feed", "mech": "feed::run_coinbase accepts an optional TelemetryHandle to record FeedStatus/FeedError directly", "data": "feed_status / feed_error / snapshot(raw,capped) rows", "breaks": "Feed thread records its own status so a transcript reconstructs connection drops; optional, so the feed runs without telemetry" },
    { "a": "entry", "b": "ui", "mech": "main() parses flags and launches App::run / run_with_app", "data": "--no-tui / --seed / --live coinbase", "breaks": "main is the only composition root; mis-wiring telemetry or feed here would silently disable observability or live mode" }
  ],
  "stateOwnership": [
    { "owner": "types", "items": "OrderID/Symbol/Px/Qty/Ts/Side/Status - read by every other subsystem; none mutate these value types" },
    { "owner": "errors", "items": "NyquestroError/ErrorSeverity/NyquestroResult - imported crate-wide; severity() is the single classifier" },
    { "owner": "order", "items": "Order entity + its remaining/status lifecycle; mutated only via fill()/cancel(); read by book, simulator, feed, events" },
    { "owner": "events", "items": "FillEvent/QuoteEvent/OrderEvent frames - produced by book, consumed read-only by ui (tape) and telemetry (JSONL)" },
    { "owner": "book", "items": "Market/OrderBook/PriceLevel - the live order books; mutated only via submit_limit/cancel; read read-only by panes::render" },
    { "owner": "simulator", "items": "mid_real OU state + ChaCha8 rng + sim_clock_ns; owned per symbol; deterministic given seed" },
    { "owner": "metrics", "items": "MetricsRegistry (HDR histograms + windowed counters); written by ui call sites, read via value-type snapshot" },
    { "owner": "feed", "items": "Bridge (sym,side,px)->OrderID map for idempotent cancels; FeedAction/FeedEvent stream; lives on the feed thread" },
    { "owner": "telemetry", "items": "bounded sync_channel + AtomicU64 drop counter + the writer thread's BufWriter; one JSONL run on disk" },
    { "owner": "ui", "items": "App: per-symbol state, mid_history rings, tape ring, rate rings, EngineState/Mode, speed; owns the run loop" },
    { "owner": "entry", "items": "CLI flags + composition wiring only; owns no durable runtime state" }
  ],
  "coverage": {
    "cols": ["source", "tests", "rationale"],
    "rows": [
      { "label": "src/types.rs", "node": "types", "cells": { "source": 3, "tests": 3, "rationale": 3 }, "prev": {} },
      { "label": "src/errors.rs", "node": "errors", "cells": { "source": 3, "tests": 3, "rationale": 2 }, "prev": {} },
      { "label": "src/order.rs", "node": "order", "cells": { "source": 3, "tests": 3, "rationale": 3 }, "prev": {} },
      { "label": "src/events/", "node": "events", "cells": { "source": 3, "tests": 3, "rationale": 2 }, "prev": {} },
      { "label": "src/book/", "node": "book", "cells": { "source": 3, "tests": 3, "rationale": 3 }, "prev": {} },
      { "label": "src/simulator/", "node": "simulator", "cells": { "source": 3, "tests": 3, "rationale": 3 }, "prev": {} },
      { "label": "src/metrics/", "node": "metrics", "cells": { "source": 2, "tests": 2, "rationale": 2 }, "prev": {} },
      { "label": "src/feed/", "node": "feed", "cells": { "source": 2, "tests": 1, "rationale": 2 }, "prev": {} },
      { "label": "src/telemetry/", "node": "telemetry", "cells": { "source": 2, "tests": 1, "rationale": 2 }, "prev": {} },
      { "label": "src/ui/", "node": "ui", "cells": { "source": 3, "tests": 1, "rationale": 3 }, "prev": {} },
      { "label": "src/main.rs + lib.rs", "node": "entry", "cells": { "source": 2, "tests": 1, "rationale": 1 }, "prev": {} }
    ],
    "note": "Full source inspection this pass for types/order/book/simulator/ui (read and edited during the same session that fixed the OU dt-cap crash and added the panic hook). metrics/feed/telemetry read at interface depth and verified against their systems/*.md, not re-read line-by-line this pass. feed/telemetry have integration smoke tests (examples/) but no unit tests; ui has no automated tests (TUI render path). entry (main/lib) is composition wiring, inspected at the call-site level."
  },
  "milestones": [
    { "id": "mvp", "title": "MVP: hardened core + multi-instrument matching + dashboard + live feed + telemetry", "status": "done", "note": "Shipped 2026-05-05 in one step-change burst (13 commits)." },
    { "id": "crash-fix", "title": "OU dt-cap + render-loop panic hook", "status": "done", "note": "ceed2e9 + 7525618 - the sparkline-overflow crash and unreadable-panic robustness." },
    { "id": "benchmark", "title": "Headless --benchmark harness surfacing the HDR percentiles", "status": "next", "note": "Plan: context/plans/benchmark-harness.md - one command, reproducible latency numbers." },
    { "id": "perf", "title": "Hot-path data-structure optimisation (tick-array + intrusive list + slab pool)", "status": "planned", "note": "Plan: medium-wins.md #1 - the highest-leverage HFT-signal work; still single-threaded." },
    { "id": "agent", "title": "Market-making strategy agent (the participant side)", "status": "planned", "note": "Plan: medium-wins.md #2 - inventory + OFI-aware quoting, PnL accounting." },
    { "id": "concurrent", "title": "Single-writer-sharded concurrent engine + binary wire protocol", "status": "planned", "note": "Plans: lock-free-engine.md, wire-protocol-gateway.md - the exchange-shaped endgame." }
  ],
  "criticalPaths": [
    { "name": "Synthetic order-submission tick", "len": "18 steps - 7 subsystems", "steps": ["simulator", "ui", "book", "price_level", "order", "events", "metrics", "telemetry", "ui"], "blast": "The dominant operation. Changing Order::fill semantics ripples to submit_limit and every order/matching test; changing MetricsRegistry::record_latency's unit touches every App call site; changing FillEvent::new validation shifts which self-match guard is load-bearing. A regression anywhere in steps 4-13 corrupts matching correctness; a regression in 14-18 only degrades observability." },
    { "name": "Live Coinbase ingestion", "len": "feed thread -> bridge -> engine", "steps": ["feed (coinbase ws)", "feed (bridge)", "ui", "book"], "blast": "Alternative to step 1 of the submission tick. The bridge's (sym,side,px)->OrderID map is the load-bearing translation; if it desyncs, cancels become non-idempotent and the virtual book diverges from Coinbase. Snapshot cap (50/side) is what protects the engine from the 25k-level connect firehose. Feed failure is isolated to its tokio thread and surfaced as FeedAction::Status." }
  ],
  "notes": [
    { "tag": "design", "sev": "ok", "title": "Single-threaded by design, not by omission", "body": "Engine + simulator + renderer share one thread. No mutexes, no hot-path atomics (except the telemetry drop counter). This is sufficient for the observability use case, avoids every concurrency hazard, and is the correct single-writer core for a future sharded design - see plans/lock-free-engine.md." },
    { "tag": "design", "sev": "ok", "title": "Zero unsafe across the crate", "body": "grep over src/ confirms no unsafe. The README's safe-Rust claim holds today; concurrency work (crossbeam rings, core_affinity) is planned to preserve it. See notes/safe-rust-philosophy.md." },
    { "tag": "design", "sev": "ok", "title": "ANSI-16 colour discipline", "body": "theme.rs exposes only Color::Reset + ANSI-16 named colours. Hardcoded RGB would break user terminal themes (Solarized, Catppuccin, accessibility palettes). Structural rule: one violation degrades the headline visual on a meaningful fraction of terminals. See notes/dashboard-design.md." },
    { "tag": "live", "sev": "watch", "title": "OU dt-cap crash fixed but the render path has no automated tests", "body": "ceed2e9 fixed the sparkline-overflow crash (regression-tested in the simulator). 7525618 added a render-loop panic hook so a future panic leaves a readable terminal - but that restore-on-panic path is reasoned-correct, not unit-tested (needs a PTY harness). The ui render path generally has no automated coverage." },
    { "tag": "gap", "sev": "watch", "title": "README roadmap under-claims; no CI or surfaced benchmark numbers yet", "body": "The README's Matching Engine roadmap section is unticked despite the engine shipping; it under-sells the project. There is no CI pipeline and the HDR percentiles are not surfaced anywhere a reader can run. All three are tracked in plans/small-wins.md + plans/benchmark-harness.md." }
  ],
  "concept": {
    "root": "Limit-order matching engine + microstructure observability",
    "branches": [
      { "head": "Matching core", "kind": "env", "leaves": ["price-time priority", "partial fills", "self-match rejection", "FIFO per price level", "BTreeMap ladders"], "trunks": ["book", "order"] },
      { "head": "Market microstructure", "kind": "observer", "leaves": ["microprice", "order-flow imbalance (OFI)", "spread", "depth", "top-of-book quotes"], "trunks": ["book"] },
      { "head": "Flow sources", "kind": "env", "leaves": ["Ornstein-Uhlenbeck synthetic mid", "Poisson arrivals", "Coinbase L2 WebSocket", "virtual-order bridge"], "trunks": ["simulator", "feed"] },
      { "head": "Observability", "kind": "observer", "leaves": ["HDR latency percentiles", "windowed counters", "JSONL flight recorder", "Ratatui infographics"], "trunks": ["metrics", "telemetry", "ui"] }
    ],
    "note": "The matching rule itself is simple (min(qty), subtract, evict). The engineering value is everything around it: correctness under edge cases, determinism, honest tail measurement, and the microstructure surface that lets a participant reason about the book."
  },
  "glossary": [
    { "term": "LOB", "def": "Limit Order Book - the two sorted lists (bids, asks) of resting orders the engine maintains and matches against." },
    { "term": "Price-time priority", "def": "The matching rule: best price wins; among orders at the same price, the earliest-arrived (FIFO) wins." },
    { "term": "Aggressor / resting", "def": "The aggressor is the incoming order that crosses the spread; resting orders are those already sitting in the book waiting to be matched." },
    { "term": "Partial fill", "def": "An incoming order matching across multiple resting orders until filled or no more crossing liquidity exists; remaining quantity is tracked at each step." },
    { "term": "Self-match", "def": "An order trading against itself (same OrderID on both sides). Nyquestro rejects the aggressor wholly at match time and double-checks in FillEvent::new." },
    { "term": "Microprice", "def": "A volume-weighted mid: (bid_qty*ask_px + ask_qty*bid_px)/(bid_qty+ask_qty). A better fair-value estimate than the simple mid when the book is imbalanced." },
    { "term": "OFI", "def": "Order-Flow Imbalance - the relative weight of bid-side vs ask-side depth, used as a directional pressure signal." },
    { "term": "Spread / depth / touch", "def": "Spread = best ask - best bid; depth = resting quantity near the top; the touch is the best price on a side." },
    { "term": "OU process", "def": "Ornstein-Uhlenbeck - a mean-reverting stochastic walk dX = theta*(mu - X)*dt + sigma*sqrt(dt)*N(0,1), used for the synthetic fair-value mid." },
    { "term": "HDR histogram", "def": "High Dynamic Range histogram - records the full latency distribution so p50/p99/p99.9/p99.99 are exact, not estimated from an average." },
    { "term": "p99 / tail latency", "def": "The 99th-percentile latency. The tail (p99/p99.9), not the average, is what matters for a matching engine; one slow order in a thousand can be catastrophic." },
    { "term": "Determinism", "def": "Same input sequence -> byte-identical output. Achieved via integer prices, caller-supplied timestamps, and never calling the clock inside matching. Enables replay." },
    { "term": "L2 / level2", "def": "Market-data depth feed giving per-price-level aggregate quantity (vs L1 top-of-book or L3 per-order). Coinbase ships L2; the bridge fakes per-order from it." },
    { "term": "Tick", "def": "The minimum price increment (1 cent here). Also: a sim tick (50ms flow step) or render tick (33ms draw) in the dashboard loop." },
    { "term": "Slab / intrusive list", "def": "Future hot-path structures (plans/medium-wins.md): a slab pool recycles order nodes without malloc; an intrusive list makes cancel-by-id O(1)." },
    { "term": "Single-writer sharding", "def": "The planned concurrency model: one thread owns each book, threads communicate via lock-free rings (LMAX Disruptor style), scaling by independent shards not locks." }
  ],
  "decisions": [
    { "title": "Integer cents, never floats, for price", "why": "Px(u64) in cents makes matching exact and deterministic and kills float-comparison bugs; from_dollars rounds to nearest cent (fixing the historical $10.999 -> 1099 truncation bug).", "node": "types" },
    { "title": "checked_sub only - saturating_sub is not exposed", "why": "saturating_sub flooring an underflow at zero was the source of the silent over-fill bug; refusing to expose it forces every caller to handle underflow explicitly.", "node": "order" },
    { "title": "Caller-supplied timestamps; matching never calls Ts::now()", "why": "Fills reuse the resting order's timestamp so the same input sequence is byte-identical on replay - the determinism contract pinned by matching_test.rs.", "node": "book" },
    { "title": "Self-match rejected at match time, aggressor wholly", "why": "The aggressor is rejected and the resting counterparty untouched; FillEvent::new re-checks as defence in depth so an externally-composed book still cannot self-match.", "node": "book" },
    { "title": "Single-threaded engine + sim + renderer", "why": "Sufficient for observability, avoids every concurrency hazard, and is the correct single-writer core to later shard rather than lock. The README's lock-free ambition is deferred.", "node": "ui" },
    { "title": "Bounded telemetry channel, drop-on-full", "why": "try_send + AtomicU64 drop counter means a slow disk can never block the single-threaded engine; lost events are counted, not silently ignored.", "node": "telemetry" },
    { "title": "Coinbase snapshot capped at 50 levels/side", "why": "Coinbase ships 25k+ levels on connect; SNAPSHOT_LEVEL_CAP truncates to the closest-to-touch 50/side at the parse boundary to protect the engine from the firehose.", "node": "feed" },
    { "title": "OU step dt clamped to min(1/theta, 0.25)", "why": "Explicit Euler diverges once theta*dt >= 2; an uncapped wall-clock*speed dt at a frame hitch diverged the mid to near-u64::MAX and overflowed the sparkline (ceed2e9). The cap keeps theta*dt <= 1 and bounds the Poisson burst.", "node": "simulator" },
    { "title": "Render-loop panic hook restores the terminal", "why": "A panic inside terminal.draw used to leave the user on a raw-mode alternate screen with an unreadable backtrace; install_panic_hook restores cooked mode then chains to the default hook (7525618).", "node": "ui" }
  ],
  "risks": [
    { "sev": "med", "title": "Performance layer not built - BTreeMap is O(log n), not the HFT O(1)", "node": "book", "trigger": "Becomes load-bearing under high order volume; the tick-array + intrusive-list + slab-pool upgrade (medium-wins.md #1) is the highest-leverage hiring-signal work and is unstarted." },
    { "sev": "med", "title": "README roadmap under-claims shipped features", "node": "ui", "trigger": "The Matching Engine roadmap section is unticked despite the engine shipping; a sharp reader distrusts the gap. Tracked in small-wins.md (doc-drift fix)." },
    { "sev": "low", "title": "No CI; HDR percentiles not surfaced headlessly", "node": "metrics", "trigger": "No GitHub Actions; the latency histograms only appear inside the live TUI. small-wins.md (CI) + benchmark-harness.md close this." },
    { "sev": "low", "title": "Restore-on-panic path is reasoned-correct but untested", "node": "ui", "trigger": "Verifying it needs a pseudo-terminal subprocess harness; a panic in the render loop relies on the hook working without an automated test proving it." },
    { "sev": "low", "title": "examples/ fmt drift", "node": "entry", "trigger": "live_smoke.rs and telemetry_smoke.rs have pre-existing cargo fmt drift; harmless today, will fail the first CI fmt check (small-wins.md)." },
    { "sev": "low", "title": "Knuth small-lambda Poisson sampler accuracy bound", "node": "simulator", "trigger": "Accurate only for modest lambda; the dt-cap now bounds the worst case at lambda*dt <= 7.5 for the default config, but a much larger lambda config would want a transformed-rejection sampler." }
  ],
  "alerts": [
    { "sev": "ok", "text": "First session back after ~7-week dormancy (since the 2026-05-05 MVP burst); build verified green on a new machine (rustc 1.96, 0 warnings, 102 tests pass).", "meta": "2026-06-21" },
    { "sev": "watch", "text": "Five forward plans added this session (benchmark-harness, small/medium-wins, lock-free-engine, wire-protocol-gateway); none started.", "meta": "context/plans/" }
  ],
  "changeFrontier": [
    { "name": "src/book/", "node": "book", "bars": [100, 0, 0, 0, 0, 0, 0] },
    { "name": "src/events/", "node": "events", "bars": [100, 0, 0, 0, 0, 0, 0] },
    { "name": "src/feed/", "node": "feed", "bars": [100, 0, 0, 0, 0, 0, 0] },
    { "name": "src/metrics/", "node": "metrics", "bars": [100, 0, 0, 0, 0, 0, 0] },
    { "name": "src/simulator/", "node": "simulator", "bars": [100, 0, 0, 0, 0, 0, 21] },
    { "name": "src/telemetry/", "node": "telemetry", "bars": [100, 0, 0, 0, 0, 0, 0] },
    { "name": "src/ui/", "node": "ui", "bars": [100, 0, 0, 0, 0, 0, 2] }
  ],
  "kpis": [
    { "label": "Subsystems", "value": "11", "unit": "live", "delta": "7 dirs + types/errors/order/entry" },
    { "label": "Tests", "value": "102", "unit": "passing", "delta": "61 unit + 41 integ - 0 failing", "tone": "sage" },
    { "label": "Unsafe", "value": "0", "unit": "blocks", "delta": "safe Rust upheld" },
    { "label": "Last commit", "value": "7525618", "unit": "", "delta": "2026-06-21 - panic-hook fix" }
  ],
  "repoTree": {
    "name": "Nyquestro/",
    "anno": "From-scratch limit-order matching engine in safe Rust with a Ratatui observability dashboard",
    "children": [
      { "name": "CLAUDE.md", "anno": "Principal-engineer collaborator personality + startup behaviour for this repo", "file": true },
      { "name": "Cargo.lock", "anno": "Pinned dependency graph", "file": true },
      { "name": "Cargo.toml", "anno": "Crate metadata, deps, lints (unused_must_use deny), thin-LTO release profile - v0.1.1", "file": true },
      { "name": "README.md", "anno": "Directional pitch: the larger ambition (lock-free book, gateway, risk, strategy agent)", "file": true },
      { "name": "context/", "anno": "Repository memory layer - this folder",
        "children": [
          { "name": "ARCHITECTURE.md", "anno": "Legacy markdown architecture (superseded by architecture.html; pending removal)", "file": true },
          { "name": "_staleness-report.md", "anno": "Per-file staleness snapshot, overwritten each upkeep run", "file": true },
          { "name": "arch/", "anno": "Editable architecture explorer: data.js (project-specific) + 5 vendored shell files",
            "children": [
              { "name": "app.js", "anno": "Vendored shell - explorer app logic; do not hand-edit", "file": true },
              { "name": "features.js", "anno": "Vendored shell - persistence/state; do not hand-edit", "file": true },
              { "name": "graph.js", "anno": "Vendored shell - dependency graph renderer; do not hand-edit", "file": true },
              { "name": "index.html", "anno": "Vendored shell - DOM scaffold; do not hand-edit", "file": true },
              { "name": "styles.css", "anno": "Vendored shell - styling; do not hand-edit", "file": true }
            ] },
          { "name": "notes/", "anno": "Design rationale + durable lessons",
            "children": [
              { "name": "conventions.md", "anno": "Coding idioms and conventions used across src/", "file": true },
              { "name": "dashboard-design.md", "anno": "TUI layout + ANSI-16 palette rationale", "file": true },
              { "name": "free-data-sources.md", "anno": "Why crypto WebSockets (no-auth L2) for the live feed", "file": true },
              { "name": "hft-firm-priorities.md", "anno": "What Jane Street/Citadel/HRT/Optiver value - the hiring-signal frame", "file": true },
              { "name": "safe-rust-philosophy.md", "anno": "Why zero-unsafe is a feature, not a limitation", "file": true },
              { "name": "telemetry-policy.md", "anno": "Local-only, no-network, schema-versioned flight recorder policy", "file": true }
            ] },
          { "name": "notes.md", "anno": "Index of the notes/ files", "file": true },
          { "name": "plans/", "anno": "Forward execution plans (3 DONE, the rest the roadmap)",
            "children": [
              { "name": "benchmark-harness.md", "anno": "NEXT: headless --benchmark surfacing the HDR percentiles", "file": true },
              { "name": "cpp-reference-impl.md", "anno": "C++ comparison build of the matching core", "file": true },
              { "name": "dashboard-infographics.md", "anno": "DONE: the Ratatui infographics dashboard", "file": true },
              { "name": "extended-order-types.md", "anno": "IOC/FOK/AON/iceberg/peg/market order types", "file": true },
              { "name": "extensive-testing-framework.md", "anno": "proptest + criterion + insta + stress + mutation buildout", "file": true },
              { "name": "itch-replay-harness.md", "anno": "LOBSTER CSV then raw ITCH 5.0 binary replay", "file": true },
              { "name": "live-crypto-feed.md", "anno": "DONE: the Coinbase L2 WebSocket bridge", "file": true },
              { "name": "lock-free-engine.md", "anno": "LARGE: single-writer-sharded concurrent engine (LMAX-style)", "file": true },
              { "name": "medium-wins.md", "anno": "Perf optimisation + strategy agent + latency post-mortem (highest hiring leverage)", "file": true },
              { "name": "property-based-tests.md", "anno": "10 named matching invariants (Day 1 of the testing framework)", "file": true },
              { "name": "recovery-and-event-log.md", "anno": "Append-only journal + crash recovery", "file": true },
              { "name": "risk-layer.md", "anno": "Fat-finger / position limits / throttle / rolling-VaR guard", "file": true },
              { "name": "small-wins.md", "anno": "CI + doc-drift fix + dashboard demo GIF", "file": true },
              { "name": "telemetry-and-profiling.md", "anno": "DONE: the JSONL flight recorder", "file": true },
              { "name": "wire-protocol-gateway.md", "anno": "LARGE: binary protocol + multi-process gateway + MD publisher", "file": true }
            ] },
          { "name": "systems/", "anno": "Canonical per-subsystem implementation reality",
            "children": [
              { "name": "book.md", "anno": "Matching engine: OrderBook/PriceLevel/Market", "file": true },
              { "name": "errors.md", "anno": "NyquestroError taxonomy + severity", "file": true },
              { "name": "events.md", "anno": "Three immutable event frames", "file": true },
              { "name": "feed.md", "anno": "Coinbase L2 WebSocket + bridge", "file": true },
              { "name": "metrics.md", "anno": "HDR histograms + windowed counters", "file": true },
              { "name": "order.md", "anno": "Order entity + fill/cancel/state machine", "file": true },
              { "name": "simulator.md", "anno": "OU synthetic flow (dt-capped) + Poisson arrivals", "file": true },
              { "name": "telemetry.md", "anno": "JSONL flight recorder", "file": true },
              { "name": "types.md", "anno": "Domain primitives (Symbol/Px/Qty/Ts/...)", "file": true },
              { "name": "ui.md", "anno": "Ratatui dashboard + run loop + panic hook", "file": true }
            ] }
        ] },
      { "name": "examples/", "anno": "Headless integration smoke verifiers (CI-runnable, no TTY)",
        "children": [
          { "name": "live_smoke.rs", "anno": "Drains the Coinbase feed end-to-end, prints first 60 events", "file": true },
          { "name": "telemetry_smoke.rs", "anno": "Verifies the JSONL writer flushes 9 event classes without a TTY", "file": true }
        ] },
      { "name": "src/", "anno": "The crate",
        "children": [
          { "name": "book/", "anno": "Matching engine core", "node": "book",
            "children": [
              { "name": "market.rs", "anno": "Market: one OrderBook per Symbol, auto-register", "file": true },
              { "name": "mod.rs", "anno": "re-exports OrderBook/PriceLevel/SubmitResult", "file": true },
              { "name": "order_book.rs", "anno": "BTreeMap ladders + submit_limit four-phase matching + microstructure", "file": true },
              { "name": "price_level.rs", "anno": "VecDeque FIFO at one price, running total_quantity", "file": true }
            ] },
          { "name": "errors.rs", "anno": "NyquestroError (16 variants) + severity classifier", "file": true },
          { "name": "events/", "anno": "Three immutable Copy event frames", "node": "events",
            "children": [
              { "name": "fill.rs", "anno": "FillEvent + self-match/zero-qty validation", "file": true },
              { "name": "lifecycle.rs", "anno": "OrderEvent::{Placed,Filled,Cancelled,Rejected}", "file": true },
              { "name": "mod.rs", "anno": "re-exports", "file": true },
              { "name": "quote.rs", "anno": "QuoteEvent + QuoteSide (live/cleared)", "file": true }
            ] },
          { "name": "feed/", "anno": "Live Coinbase L2 boundary", "node": "feed",
            "children": [
              { "name": "bridge.rs", "anno": "L2-to-virtual-order translation, idempotent cancel map", "file": true },
              { "name": "coinbase.rs", "anno": "tokio-tungstenite WS client, level2 subscribe + parse", "file": true },
              { "name": "mod.rs", "anno": "re-exports run_coinbase/Bridge/FeedAction/FeedEvent", "file": true }
            ] },
          { "name": "lib.rs", "anno": "Module wiring + crate re-exports (all subsystems pub)", "file": true, "node": "entry" },
          { "name": "main.rs", "anno": "Binary entry: flags, telemetry/feed wiring, launch", "file": true, "node": "entry" },
          { "name": "metrics/", "anno": "HDR latency + windowed counters", "node": "metrics",
            "children": [
              { "name": "counters.rs", "anno": "CounterSet over orders/fills/cancels/rejects/quotes", "file": true },
              { "name": "mod.rs", "anno": "re-exports", "file": true },
              { "name": "registry.rs", "anno": "MetricsRegistry + RegistrySnapshot + Op enum", "file": true },
              { "name": "windows.rs", "anno": "WindowedCounter rolling 1s/10s/1min/5min", "file": true }
            ] },
          { "name": "order.rs", "anno": "Order entity: validated construction, two-phase fill, cancel", "file": true, "node": "order" },
          { "name": "simulator/", "anno": "Synthetic OU order flow", "node": "simulator",
            "children": [
              { "name": "market.rs", "anno": "MarketSimulator: dt-capped OU mid-walk + Poisson arrivals", "file": true },
              { "name": "mod.rs", "anno": "re-exports MarketSimulator/SimConfig/SimAction", "file": true }
            ] },
          { "name": "telemetry/", "anno": "JSONL flight recorder", "node": "telemetry",
            "children": [
              { "name": "events.rs", "anno": "TelemetryEvent (~17 variants) + serde wrapper", "file": true },
              { "name": "mod.rs", "anno": "re-exports", "file": true },
              { "name": "writer.rs", "anno": "spawn_writer, bounded channel, drop-on-full", "file": true }
            ] },
          { "name": "types.rs", "anno": "Domain primitives: OrderID/Symbol/Px/Qty/Ts/Side/Status", "file": true, "node": "types" },
          { "name": "ui/", "anno": "Ratatui dashboard + run loop", "node": "ui",
            "children": [
              { "name": "app.rs", "anno": "App state, run loop, key handlers, panic hook", "file": true },
              { "name": "mod.rs", "anno": "re-exports", "file": true },
              { "name": "panes.rs", "anno": "per-pane render fns (DOB/engine/throughput/tape/latency/mid)", "file": true },
              { "name": "theme.rs", "anno": "ANSI-16 palette + block/sparkline/distribution helpers", "file": true }
            ] }
        ] },
      { "name": "tests/", "anno": "41 integration tests across the five core subsystems",
        "children": [
          { "name": "events_test.rs", "anno": "9 tests - event frame validation", "file": true },
          { "name": "matching_test.rs", "anno": "12 tests - cross/sweep/FIFO/determinism/self-match", "file": true },
          { "name": "order_test.rs", "anno": "8 tests - fill/over-fill/state machine", "file": true },
          { "name": "price_level_test.rs", "anno": "6 tests - FIFO ordering, remove_by_id", "file": true },
          { "name": "types_test.rs", "anno": "6 tests - primitive construction + validation", "file": true }
        ] }
    ]
  },
  "bespoke": []
}`);
