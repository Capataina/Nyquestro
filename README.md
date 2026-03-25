# Nyquestro

> A high-performance order matching engine written in safe Rust, implementing the core primitive of every exchange and trading system: a correct, lock-free order book that matches buyers against sellers at microsecond-range latency, with a real-time risk layer, a binary wire protocol, and a rigorous benchmarking harness that measures and explains every nanosecond.

---

## Why Nyquestro exists

Every trade that happens on any exchange, whether equity, crypto, futures, or FX, passes through a matching engine. It is the most latency-sensitive piece of software in finance. A matching engine that processes an order in 10 microseconds instead of 12 is worth millions in annual revenue to a firm operating at scale. The firms that win at this game, Jane Street, HRT, Citadel Securities, Jump, have spent decades optimising every layer of this pipeline: the data structures, the memory layout, the wire protocol, the risk controls, the hardware.

Nyquestro is a from-scratch implementation of that core system, built not to compete with production infrastructure but to understand it deeply: why lock-free structures matter here, what cache behaviour looks like under a realistic order flow, how a binary protocol differs from text-based alternatives, and what the latency distribution actually looks like when you measure it honestly.

The goal is not a toy. It is a system with correct matching semantics, measured performance characteristics, and documented design decisions, built to the standard that an engineer at a quantitative trading firm would recognise as serious work.

---

## What a matching engine actually does

At its core, a matching engine maintains two sorted lists of resting orders: one for buyers (the bid side) and one for sellers (the ask side). When a new order arrives, the engine checks whether it can be matched against an existing resting order at an acceptable price. If it can, a trade executes. If it cannot, the new order joins the book and waits.

This description sounds simple. The engineering is not.

**Price-time priority** is the rule that governs which resting order gets matched first: the best price wins, and among orders at the same price, the one that arrived earliest wins. Implementing this correctly under concurrent load, where orders can arrive, cancel, and execute simultaneously from multiple sessions, requires careful reasoning about atomicity and ordering guarantees.

**Partial fills** mean a single incoming order may match against multiple resting orders, consuming them one by one until either the incoming order is fully filled or no more matching resting orders exist. The remaining quantity must be tracked precisely across each step.

**Cancellations** must be atomic with respect to matching. An order being cancelled at the same moment it would be matched is not a race condition to be handled with a mutex. It is a correctness requirement to be solved with atomic operations.

**The hot path**, the code executed for every single order, must avoid heap allocation, lock contention, and cache misses. A single unexpected allocation or a cache line bounced between cores adds latency that compounds across millions of operations per day.

Nyquestro addresses all of these problems from first principles.

---

## Architecture

Nyquestro is structured as a set of composable, independently testable layers:

```
┌──────────────────────────────────────┐     ┌──────────────────────────────────────┐
│         Market Data Publisher        │────▶│        Strategy Agent                │
│  Depth feed, incremental updates     │     │  Order book reconstruction           │
│  Fill event stream                   │     │  Order flow imbalance signals        │
│  Immutable audit log                 │     │  Market-making and quoting logic     │
└─────────────────▲────────────────────┘     │  Inventory and adverse selection mgmt│
                  │ Fill events, book updates └─────────────────┬────────────────────┘
┌─────────────────┴────────────────────┐                       │ Orders via binary protocol
│         Matching Engine Core         │◀──────────────────────┘
│  Lock-free order book (bid/ask)      │
│  Price-time priority matcher         │
│  Partial fill handling               │
│  Atomic order lifecycle management   │
└─────────────────▲────────────────────┘
                  │ Risk-cleared orders
┌─────────────────┴────────────────────┐
│           Risk Guard                 │
│  Fat-finger limits, position bounds  │
│  Rolling VaR circuit breaker         │
│  Per-session throttle enforcement    │
└─────────────────▲────────────────────┘
                  │ Validated order events
┌─────────────────┴────────────────────┐
│           Order Gateway              │
│  Binary UDP protocol, FIX acceptor   │
│  Frame validation, session tracking  │
└──────────────────────────────────────┘
```

Each layer has a single, well-defined responsibility. The matching engine core has no knowledge of the wire protocol. The risk guard has no knowledge of the book structure. The market data publisher consumes fill events without touching the matching logic. The strategy agent is a fully external participant, connecting through the same binary protocol any other client would use, watching the market data feed, and submitting orders based on its own logic. This separation makes each layer testable in isolation and replaceable without cascading changes.

---

## Design Decisions

### Why lock-free and not mutex-protected?

A mutex under high throughput is a latency cliff. When one thread holds the lock, every other thread blocks, and OS scheduling can leave a blocked thread waiting for hundreds of microseconds before it is rescheduled. In a system where the target hot-path latency is measured in single-digit microseconds, a mutex acquisition that occasionally takes 200 microseconds is catastrophic.

Lock-free data structures using atomic compare-and-swap operations allow concurrent progress without blocking. A thread that loses a CAS race retries immediately without yielding to the OS scheduler. Under the contention profile of an order book, many readers and occasional writers at specific price levels, this produces dramatically better tail latency.

Nyquestro implements its order book using atomic price-level buckets: each price level is an intrusive FIFO list managed with atomic operations. Order insertion, cancellation, and matching each proceed without acquiring any lock.

### Why safe Rust specifically?

The "without unsafe" constraint is a deliberate engineering choice, not a limitation. Safe Rust guarantees the absence of data races at compile time. In a concurrent system where correctness bugs are expensive and subtle, having the compiler verify that no shared mutable state is accessed without proper synchronisation is a genuine engineering advantage, not a crutch.

Where unsafe Rust would normally be required for performance, such as manual memory management, cache-line-aligned allocations, and intrusive data structures, Nyquestro uses carefully designed safe abstractions that compile to equivalent machine code. The constraint forces cleaner architecture.

### Why a binary wire protocol over JSON or FIX?

JSON requires text parsing on every message. A parser scanning for quote characters and numeric boundaries on the critical path adds hundreds of nanoseconds per message and introduces branch mispredictions. FIX tag=value encoding is an improvement but still text-based and verbose.

Nyquestro's binary UDP protocol uses fixed-width little-endian frames. An order submission message is a compact struct that maps directly to a validated in-memory representation with a single memcpy. No parsing. No allocation. No scanning. The frame format is versioned and length-prefixed for forward compatibility, and each frame includes a checksum for corruption detection on the UDP path.

### Why a slab allocator for order nodes?

The default system allocator is not designed for the allocation pattern of a matching engine: millions of small, same-sized allocations with unpredictable lifetimes. Each call to the allocator under pressure involves lock contention, fragmentation bookkeeping, and occasional kernel transitions.

A slab allocator pre-allocates a fixed pool of same-sized order node slots and recycles them via a lock-free free-list. Order insertion pulls from the pool in O(1) with no system call. Order cancellation or fill returns the node to the pool. Allocator churn disappears from the latency profile entirely.

---

## Performance Model

Nyquestro is benchmarked with a rigorous harness that measures latency distributions, not averages.

Averages hide the tail. A p50 latency of 800ns looks excellent. If the p999 is 400 microseconds, the system is broken for one in every thousand orders. Nyquestro measures and documents p50, p99, p999, and maximum latency across several workload profiles:

- **Single-threaded throughput**: maximum order processing rate with no contention
- **Concurrent session load**: multiple concurrent order streams against a live book
- **Deep book sweep**: matching a large aggressive order against many resting orders across multiple price levels
- **Cancel storm**: high-frequency cancellation load against a fragmented book

Each benchmark is run against real hardware with `perf stat` cache profiling to identify whether latency spikes correspond to L3 cache misses, branch mispredictions, or lock contention. The results are documented alongside what drives them and what would need to change to push them lower.

---

## Risk Layer

A matching engine without risk controls is a liability. Nyquestro implements a real-time risk guard that sits between the order gateway and the matching core.

**Fat-finger protection** rejects orders where the price or size deviates beyond a configurable threshold from the current market. A sell order at 10% of the current mid-price is almost certainly a mistake, not a strategy.

**Position and PnL bounds** track each session's running inventory and unrealised PnL in real time, rejecting orders that would breach pre-set exposure limits. This is updated atomically with every fill event.

**Rolling VaR circuit breaker** maintains a rolling variance window over recent fill prices and halts trading for a session if the implied Value-at-Risk breaches a threshold. This is the same mechanism used in production risk systems to prevent runaway algorithms from accumulating dangerous positions during abnormal market conditions.

**Per-session throttles** enforce a maximum order submission rate per connected session, preventing a misbehaving client from saturating the engine's ingress capacity.

The risk layer is designed to fail safe: if the risk computation itself encounters an error, the default action is rejection, not pass-through.

---

## Strategy Agent

A matching engine in isolation demonstrates infrastructure depth. A strategy agent that connects to it demonstrates quantitative reasoning, the ability to think like a participant in a market, not just the operator of one.

Nyquestro includes a built-in market-making agent that runs as an independent process alongside the engine. It connects via the binary protocol, subscribes to the market data feed, reconstructs the live order book from incremental updates, and continuously posts limit orders on both sides of the spread to capture the bid-ask difference. This is the core business of every market maker in existence: buy at the bid, sell at the ask, repeat millions of times per day.

The mechanics are non-trivial. A naive market maker that blindly posts quotes will be destroyed by **adverse selection**, the phenomenon where informed traders disproportionately trade against you when your quote is on the wrong side of where the price is about to move. Managing this requires the agent to monitor **order flow imbalance**: when the bid side of the book is significantly heavier than the ask side, price is likely to move up, and the agent should skew its quotes or temporarily pull its sell-side exposure.

**Inventory risk** is the second constraint. A market maker that accumulates a large directional position during a trending market is exposed to mark-to-market losses. The agent tracks its running inventory and adjusts quote placement to rebalance toward flat, widening spreads, skewing prices, or leaning more aggressively on the side that reduces exposure.

The agent implements and documents these mechanisms explicitly:

**Order book reconstruction** consumes incremental depth updates from the market data feed and maintains an accurate local view of the top levels on each side. The quality of every decision downstream depends on the accuracy of this reconstruction.

**Spread and mid-price calculation** computes the best bid, best ask, mid-price, and effective spread at each book update. These are the reference points around which all quote placement logic is anchored.

**Order flow imbalance signal** measures the ratio of bid-side to ask-side depth at each update as a directional indicator. A sustained imbalance above a threshold triggers quote skew in the direction of the pressure.

**Inventory-aware quote placement** offsets quote prices relative to mid based on the current inventory position, so the agent naturally encourages trades that reduce exposure and discourages trades that increase it.

**Fill tracking and PnL accounting** records every fill received from the engine, computes realised and unrealised PnL continuously, and surfaces a clean performance summary at the end of a replay session.

The agent is intentionally simple. It is not a production trading system and makes no claim to profitability on real markets. Its purpose is to demonstrate a complete understanding of the feedback loop between market microstructure and participant behaviour: the infrastructure side in Nyquestro's engine, and the participant side in this agent.

---

## Testing and Validation

Correct matching semantics are non-negotiable. Nyquestro validates correctness through several complementary approaches.

**Determinism tests** feed a fixed sequence of orders and verify the fill output matches a golden reference byte-for-byte. The matching engine is fully deterministic given the same input sequence, and this property is enforced in CI.

**Property-based tests** use randomised order sequences to verify invariants that must hold regardless of input: price-time priority is never violated, no order is matched against itself, partial fills always sum to the correct total quantity, and cancelled orders never appear in fill events.

**Fuzz harness** feeds malformed binary frames into the gateway parser to verify it never panics, never produces undefined behaviour, and always returns a well-formed error. Parser correctness under adversarial input is a security property, not just a reliability one.

**Replay against real market data** uses historical Nasdaq ITCH feed data to drive the engine under realistic order flow distributions, measuring latency under authentic load rather than synthetic benchmarks designed to look good.

---

## Supported Order Semantics

The engine is designed around the full set of order behaviours found in real exchange systems, covering the spectrum from simple resting orders to aggressive immediate-execution types with strict fill-or-cancel conditions. Beyond basic order placement, the engine handles atomic modification and cancellation, partial fill tracking across multiple price levels, protection against self-matching, enforcement of minimum price increments, and support for hidden quantity orders where only a portion of the total size is displayed in the public book at any time.

---

## What This Project Is Not

- Not a retail trading tool or signal dashboard
- Not a portfolio of chart patterns or technical indicators
- Not a wrapper around an existing exchange API
- Not a backtesting framework for off-the-shelf strategies

It is an end-to-end implementation of both sides of a market: the exchange infrastructure that processes orders and enforces rules, and the participant logic that reads the book, manages risk, and decides what to trade. Two systems, one codebase, the complete picture.

---

## Features and Roadmap

---

### 🔧 Core Infrastructure

- [x] Strongly-typed domain primitives covering order identity, side, price, quantity, and timestamp, each as distinct types that the compiler treats as incompatible
- [x] Zero-allocation event structs for the core quote and fill events that flow through the hot path
- [x] Structured engine error handling that distinguishes recoverable operational errors from fatal state corruption
- [ ] Pre-allocated memory pool for order nodes with lock-free recycling, eliminating heap allocation from the critical path entirely
- [ ] Per-price-level data structures managed with atomic operations, forming the backbone of the lock-free order book
- [ ] Configuration system supporting file-based and environment variable overrides with live reload capability

---

### ⚡ Matching Engine

- [ ] Resting order support with correct price-time priority insertion and book maintenance
- [ ] Aggressive order support with immediate price-level sweep and leftover quantity handling
- [ ] Immediate-or-cancel semantics: execute what is available now and discard the remainder
- [ ] Fill-or-kill semantics: execute the full quantity immediately or reject the entire order
- [ ] Correct partial fill tracking across multi-level sweeps with precise quantity accounting at every step
- [ ] Atomic order cancellation that is consistent with the matching loop and produces no phantom fills
- [ ] Atomic order modification that adjusts price or quantity while preserving time priority where the rules allow
- [ ] Self-match prevention with configurable policy for which side of the conflicting pair gets cancelled
- [ ] Minimum price increment enforcement at the point of order acceptance
- [ ] Hidden quantity order support where displayed size differs from total resting size

---

### 🚀 Concurrency and Performance

- [ ] Single-producer multi-consumer ring buffer with cache-line-padded cursors to eliminate false sharing between threads
- [ ] Thread-to-core affinity with automatic hardware topology detection to keep hot threads on the same NUMA node as their data
- [ ] Lock-free node recycling pool so that order allocation and deallocation never touch the system allocator on the hot path
- [ ] Vectorised bulk cancellation sweep that processes multiple price levels with a single atomic operation per level
- [ ] Manual cache prefetch hints and branch prediction annotations in the inner matching loop
- [ ] Kernel-bypass packet ingress for zero-copy receive from the NIC directly into userspace _(stretch)_
- [ ] SIMD-accelerated price comparison in the sweep loop for parallel evaluation of multiple price levels per instruction _(stretch)_

---

### 📡 Order Gateway and Protocols

- [ ] Local command-line interface for submitting test orders by hand, useful for smoke testing and development
- [ ] Binary UDP gateway using compact fixed-width frames with versioning, length prefixing, and checksums for integrity
- [ ] FIX protocol TCP acceptor covering session management, heartbeating, and message resend handling
- [ ] Market data multicast publisher emitting depth snapshots and incremental book updates to subscribed clients
- [ ] Control plane interface for querying engine state, toggling risk parameters, and managing sessions

---

### 🔒 Risk and Compliance

- [ ] Order rejection based on configurable price and size deviation thresholds relative to the current market, catching obvious input errors before they reach the book
- [ ] Real-time per-session inventory and unrealised PnL tracking with hard limits that block orders breaching exposure thresholds
- [ ] Rolling Value-at-Risk monitor that triggers an automatic session kill-switch when estimated risk exceeds a configurable bound
- [ ] Per-session order rate limiting to prevent a single misbehaving client from monopolising engine capacity
- [ ] Immutable append-only event journal capturing every order lifecycle event for post-trade audit and replay
- [ ] Outbound compliance feed mirroring all fill events to a downstream compliance system

---

### 📊 Observability and Benchmarking

- [ ] HDR latency histogram capturing p50, p99, p99.9, and p99.99 across each distinct workload profile
- [ ] Hardware performance counter integration surfacing L3 cache miss rate, branch misprediction rate, and instructions per cycle alongside latency results
- [ ] Flamegraph generation scripts for one-command hot-path profiling against a running engine
- [ ] Operational metrics feed exposing book depth, order throughput, fill rate, and latency summaries to a time-series monitoring system
- [ ] Hardware NIC timestamp integration for nanosecond-precision end-to-end latency measurement on PTP-capable hardware _(stretch)_

---

### 🛠️ Testing and Validation

- [ ] Determinism test suite that replays a fixed input sequence and compares output byte-for-byte against a golden reference, enforced in CI
- [ ] Property-based randomised test suite verifying that price-time priority invariants hold across arbitrary order sequences
- [ ] Fuzz harness exercising the binary frame parser and FIX parser against malformed and adversarial input to verify absence of panics or undefined behaviour
- [ ] Historical market data replayer that drives the engine with real Nasdaq ITCH order flow for authentic load testing
- [ ] Continuous integration pipeline covering compilation, linting, formatting, and latency regression checks on every commit

---

### 🛡️ Resilience and Security

- [ ] Strict frame length and field validation at the gateway boundary so malformed input is rejected before it reaches engine state
- [ ] Crash-only recovery model where the engine can always restart from the event journal into a consistent state with no manual intervention
- [ ] Clean shutdown path that drains in-flight events and flushes the audit journal before process exit
- [ ] Undefined behaviour audit of the core data structures under the Rust memory model checker
- [ ] Process isolation configuration with a minimal privilege profile for production deployment

---

### 🤖 Strategy Agent

- [ ] Incremental order book reconstructor that consumes the depth update feed and maintains an accurate local view of the top levels on each side
- [ ] Spread and mid-price computation updated on every book event, forming the reference point for all quoting decisions
- [ ] Order flow imbalance signal measuring the relative weight of bid-side versus ask-side depth as a directional pressure indicator
- [ ] Two-sided market-making logic that continuously posts resting orders on both sides of the spread to capture the bid-ask difference
- [ ] Quote skew mechanism that adjusts quote placement directionally when sustained order flow imbalance exceeds a threshold
- [ ] Running inventory tracker updated atomically on every fill, with mark-to-market PnL computed continuously
- [ ] Inventory-aware quote pricing that offsets quotes relative to mid in proportion to current directional exposure, nudging the position back toward flat
- [ ] Adverse selection detector that monitors fill rate and direction to identify when the agent's quotes are being systematically picked off by informed flow
- [ ] End-of-session performance report covering realised PnL, unrealised PnL, total fills, spread captured, and maximum drawdown across the replay
- [ ] Cross-instrument correlation signal driving relative-value quote adjustment between two correlated instruments _(stretch)_

---

### 🌐 Stretch Goals

- [ ] On-chain settlement adapter bridging the engine to Ethereum and Solana, where order book mechanics meet smart contract settlement
- [ ] Full kernel-bypass networking stack for production-grade ingress latency on supported hardware
- [ ] eBPF-based near-zero-overhead telemetry collecting per-instruction timing data on a live running engine
- [ ] WebAssembly build target for the matching core enabling browser-based order flow replay and visualisation
- [ ] GPU-accelerated order book experiment exploring parallel price-level evaluation on CUDA hardware

---

## Long-Term Direction

Nyquestro is structured to grow in two directions beyond the core matching engine.

The first is deeper hardware engagement: kernel-bypass networking via AF_XDP for zero-copy packet ingress, SIMD-accelerated price comparison in the matching sweep inner loop, and hardware timestamp hooks using PTP-enabled NICs for nanosecond-precision latency measurement. These are the techniques that separate serious low-latency infrastructure from well-optimised software.

The second is a DEX adapter layer that bridges the matching engine to on-chain settlement, connecting traditional order book mechanics to Ethereum and Solana where the matching problem has the same structure but the settlement layer is a smart contract rather than a clearinghouse. This creates a natural connection to the DeFi analytics work in Aurix and makes Nyquestro a point where traditional and decentralised market microstructure converge.
