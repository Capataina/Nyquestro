# Plan: Concurrent Engine (the lock-free / sharded endgame)

## Header

- **Status:** Planned (not started; large — multi-week)
- **Scope:** Take the engine from single-threaded to a concurrent architecture that handles multiple producers (order sources), a matching core, and consumers (market-data publisher, risk, telemetry) without the latency cliffs that locks introduce. This is the README's entire **⚡ Concurrency and Performance** section: SPMC/SPSC ring buffers with cache-line-padded cursors, lock-free or single-writer book mutation, thread-to-core affinity, and the lock-free node pool.
- **Why this matters:** This is the most HFT-specific axis of the whole project and the one that most separates "good systems engineer" from "low-latency specialist." It's also where the most interesting trade-off in the field lives — and being able to argue that trade-off is itself the hiring signal.
- **Exit rule:** complete when (a) order ingress, matching, and at least one consumer run on separate threads, (b) hand-off is via a lock-free ring buffer (no `Mutex` on the hot path), (c) the benchmark harness shows throughput/latency under concurrent ingress, (d) determinism is preserved per-shard, (e) a written note explains the architecture choice and its trade-offs.

> [!important] This is NOT the same as the medium single-threaded optimisation
> [`medium-wins.md`](medium-wins.md) item 1 makes the *single thread* fast (tick-array, intrusive list, slab pool). This plan adds *concurrency*. Do the single-threaded optimisation first: a fast single-threaded core is the right foundation, and — as the design note below argues — is often the right *core* even in the final concurrent design.

## The key design decision (worth understanding before building)

There are two philosophies, and the interesting answer is counterintuitive:

1. **Lock-free shared book.** Multiple threads mutate one order book concurrently using atomics / compare-and-swap. Maximum theoretical parallelism on a single instrument; brutally hard to get correct (the ABA problem, memory-ordering subtleties, reclamation), and in *safe* Rust specifically, genuinely shared-mutable lock-free structures are very constrained.
2. **Single-writer per shard + lock-free hand-off.** Each order book is owned by exactly one thread (one instrument, or a shard of instruments). No thread ever shares mutable book state, so no locks and no data races are even *possible*. Threads communicate by passing messages through lock-free ring buffers. You scale by running many independent single-threaded shards, pinned to separate cores.

**The industry largely converged on #2**, and the canonical reference is the **LMAX Disruptor** (Thompson, Farley, Barker, Gee, Stewart, ~2011): a single-threaded business-logic core fed by a lock-free ring buffer, sustaining millions of orders/sec on one thread precisely *because* it never pays for locks, cache-line bouncing, or context switches. "Mechanical sympathy" — designing with the cache hierarchy and the CPU pipeline rather than against them — is the phrase that came out of that work.

**Why this matters for Nyquestro specifically:** the current single-threaded engine is not a naïve mistake to be undone — it is *exactly the right core* for the #2 design. The concurrency work is therefore not "rewrite the matcher to be lock-free"; it's "wrap the existing single-threaded matcher in a lock-free message-passing harness and shard it." That's a much safer, much more defensible build, and it pairs perfectly with safe Rust's guarantees. Being able to *say this* — "I chose single-writer sharding over a lock-free shared book because correctness is cheaper and the throughput is competitive, here's the Disruptor precedent" — is a stronger interview answer than any amount of CAS wizardry.

## Implementation Structure

### Modules / files affected

- `src/concurrent/` (new):
  - `ring.rs` — a bounded SPSC (and/or SPMC) ring buffer with cache-line-padded head/tail cursors to avoid false sharing. In safe Rust this is built on `crossbeam` primitives (`crossbeam::queue::ArrayQueue`, or `crossbeam-channel`) rather than hand-rolled `unsafe` — keeping the no-`unsafe` invariant.
  - `shard.rs` — a `BookShard` owning a `Market` (or one `OrderBook`), draining an ingress ring, emitting events to outbound rings.
  - `runtime.rs` — spawns shard threads, pins them to cores (`core_affinity` crate), routes orders to the correct shard by symbol.
- `Cargo.toml` — `crossbeam`, `core_affinity`.
- `src/main.rs` — `--concurrent [--shards N]` mode.
- `context/references/concurrency-design.md` (new) — the written architecture + trade-off note (the artefact an interviewer would read).

### Architecture

```
order sources ──▶ [ingress ring] ──▶ BookShard(thread, pinned core)
                                          │  owns Market, single-writer, no locks
                                          ├──▶ [md ring]   ──▶ market-data publisher thread
                                          ├──▶ [fill ring] ──▶ risk / telemetry thread
                                          └──▶ (determinism preserved within the shard)
   route by symbol-hash to one of N shards; each shard is independent.
```

- **No shared mutable book.** Each shard owns its books outright. Cross-shard there is nothing to synchronise.
- **Hand-off is the only concurrency primitive**, and it's a lock-free ring, not a mutex.
- **Determinism is per-shard:** within a shard, the same ingress order produces the same output (the existing determinism contract holds, because the matcher is unchanged). Across shards, global ordering is by arrival into each ring — document this honestly.

## Algorithm / System Sections

### A) The ring buffer

- Bounded, power-of-two capacity, single-producer/single-consumer to start (simplest, fastest). Cache-line-pad the producer and consumer cursors so they don't share a cache line (false sharing is the classic ring-buffer performance bug — two cores invalidating each other's cache line on every increment).
- Back-pressure policy: on full, the producer either spins (lowest latency, burns a core) or drops with a counter (like telemetry). Make it explicit and configurable.
- Build on `crossbeam` to stay within safe Rust; note in the design doc that a production C++ engine would hand-roll this with `unsafe` + explicit memory fences, and that the safe-Rust version trades a little headroom for provable absence of data races.

### B) Sharding

- Route by `symbol_hash % N`. Each shard is a thread pinned to a core via `core_affinity`. This is "thread-to-core affinity with hardware topology detection" from the README, in its simplest honest form.
- A single instrument always lands on one shard → that instrument's book is single-writer → no locks. Load-balancing across instruments is the scaling story.

### C) Lock-free node pool

- The slab pool from `medium-wins.md` item 1 becomes per-shard (so it's still single-threaded access, still no locks). If a global pool is ever needed, that's where a genuine lock-free free-list (`crossbeam` `SegQueue`) comes in — but per-shard pools avoid the need.

## Integration Points

- **The existing `Market` / `OrderBook` are reused unchanged inside a shard.** This plan is a harness around them, which is the whole point of the modular design. If swapping the engine into a shard requires changing the engine's API, prefer fixing the harness over changing the matcher.
- **Benchmark harness** gains a concurrent driver: multiple producer threads feeding the ingress ring, measuring end-to-end latency (ingress → fill emitted) under contention. This is the README's "concurrent session load" workload profile.
- **Telemetry** becomes a natural ring consumer rather than an inline call.

## Debugging / Verification

- **No `Mutex`/`RwLock` on the hot path** — grep the concurrent module; the only synchronisation is the ring.
- **No data races** — guaranteed by construction (no shared mutable state) and by safe Rust; `cargo test` under `--release` and ideally under ThreadSanitizer (`RUSTFLAGS=-Zsanitizer=thread` on nightly) for belt-and-braces.
- **Per-shard determinism** — same ingress sequence into one shard → identical output (reuses the existing determinism test, run through a shard).
- **Throughput scales with shards** — N shards on N cores should show near-linear throughput on independent instruments; document where it stops scaling (memory bandwidth, ring contention) honestly.
- **No `unsafe` introduced** — the no-`unsafe` invariant (`notes/safe-rust-philosophy.md`) holds; all concurrency via `crossbeam` / `core_affinity`.

## Completion Criteria

- [ ] `src/concurrent/` with `ring.rs`, `shard.rs`, `runtime.rs`.
- [ ] Lock-free ring buffer with cache-line-padded cursors (via `crossbeam`).
- [ ] `BookShard` owns its `Market`, single-writer, drains ingress, emits to outbound rings.
- [ ] `--concurrent --shards N` routes by symbol and pins shards to cores.
- [ ] At least one consumer (market-data or telemetry) runs as a separate ring-fed thread.
- [ ] Benchmark harness measures concurrent-ingress latency + throughput; near-linear scaling shown (and the ceiling explained).
- [ ] Per-shard determinism preserved and tested.
- [ ] No `unsafe`, no hot-path locks (both grep-verified).
- [ ] `context/references/concurrency-design.md` explains single-writer-sharding vs lock-free-shared, cites the Disruptor, and argues the choice.
- [ ] This file is archived once all the above are checked.
