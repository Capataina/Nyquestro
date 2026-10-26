# Safe Rust Philosophy

## 1. Current Understanding

The crate uses **safe Rust only** — no `unsafe` blocks, no `unsafe impl`, no `unsafe fn`. Verified by grep: zero hits across `src/` and `tests/`.

This is a hard rule, not a default. The README's pitch leans on it (`written in safe Rust, ... implementing the core primitive of every exchange and trading system`), and the rule shapes what optimisations the matching engine can reach for and which ones are explicitly deferred.

## 2. Rationale

A concurrent matching engine's most expensive bugs are data races. Safe Rust eliminates them at compile time. In a system where correctness costs are high — a fill that should not have happened, or a cancellation that race-conditions with a match — having the compiler verify the absence of unsynchronised shared mutable state is an engineering advantage, not a constraint.

The README spells this out:

> Safe Rust guarantees the absence of data races at compile time. ... Where unsafe Rust would normally be required for performance, such as manual memory management, cache-line-aligned allocations, and intrusive data structures, Nyquestro uses carefully designed safe abstractions that compile to equivalent machine code. The constraint forces cleaner architecture.

In practice, "compile to equivalent machine code" is aspirational for some optimisations — a slab allocator with intrusive free-list pointers genuinely needs `unsafe`. The pragmatic stance is:

- **Today:** the matching engine is single-threaded, uses `BTreeMap` + `VecDeque` for its internals, and is not optimising for cache layout. Safe Rust is sufficient.
- **Tomorrow:** when the engine becomes multi-threaded, the substitute is sharded structures (per-side, per-instrument) accessed via narrow synchronisation primitives, *not* unsynchronised raw pointers. If a substitute compiles to materially worse code, the question becomes whether the speedup justifies a single contained `unsafe` block — the rule is "safe by default, justify exceptions," not "safe forever regardless of cost."

## 3. What Was Tried

Nothing has been tried that required `unsafe` and was kept. The prior codebase was also safe Rust throughout.

## 4. Guiding Principles

### Correctness before performance

The README's D2 design decision: **build a correct, deterministic matching engine first**. Lock-free structures, allocation optimisation, and concurrency optimisations come later. This is the load-bearing principle behind the choice of `BTreeMap` and `VecDeque` in `src/book/` and the absence of any allocator-tuning code today.

The matching engine ships as a *known-correct* artefact (88 tests, zero failures, deterministic) before anyone is allowed to optimise it. Performance work that cannot prove it preserves the existing test suite is rejected.

### Determinism is part of correctness

A non-deterministic matching engine is impossible to test rigorously and impossible to debug under load. The rules:

- **No wall-clock reads in the matching loop.** `OrderBook::submit_limit` never calls `Ts::now()`. Resting-order timestamps are reused for fills, the aggressor's timestamp is reused for the placed/rejected lifecycle.
- **Fixed-seed RNG for the simulator.** `MarketSimulator::new(_, seed)` produces a byte-identical action stream across runs. The `r` reset key reseeds with the constant `0xC0FFEE` so reset playback is also reproducible.
- **One canonical match price per fill: the resting price.** Aggressor-price matching would still be deterministic, but it's not what conformant venues do.

### `unsafe` would require a justification block

If a future change introduces `unsafe`, it must be:
- contained in one named module or function,
- accompanied by a `# Safety` doc comment that names the invariant being assumed,
- accompanied by a test that exercises the boundary condition,
- documented in this note with the reason, the alternatives considered, and the speedup measured.

The barrier is not "we never use `unsafe`"; it is "we need a paper trail when we do."

### What the rule explicitly *does not* mean

The rule does not mean every standard-library type is a poor choice. `BTreeMap` is a fine ladder data structure for an MVP — its O(log n) is well within the engine's current latency budget. The rule means we cannot use `unsafe` *to make it faster* without going through the justification block above.

The rule also does not mean we cannot pull in dependencies that internally use `unsafe` (every standard-library container does). The contract is about *our code*, not the entire transitive dependency tree.

## 5. Trade-offs and Constraints

- **Lock-free structures via `unsafe` are off the table for now.** When the engine becomes multi-threaded, the substitutes are `parking_lot::RwLock`, sharded structures, or per-thread state with merge — not raw atomics on intrusive lists.
- **Slab allocation is off the table for now.** When allocator pressure becomes the bottleneck, the substitutes are `Vec` arenas, `bumpalo`, or upstream-into-`Box`-for-stable-addresses patterns.
- **Cache-line padding via `repr(align(64))` is fine** — that's safe.

## 6. Open Questions

- Is there a measurable performance cost today from staying in safe Rust? Not yet measured; the dashboard runs at 30fps on the dev machine with ~750 orders/sec sustained, which is two orders of magnitude below the engine's plausible ceiling.
- When (if ever) is the right time to introduce a single contained `unsafe` block for a slab allocator on the order-node fast path? Not before the engine has multi-instrument support, end-to-end benchmarks, and a measured allocator bottleneck — i.e. genuinely needs it.

## 7. Related Systems and Notes

- `systems/book.md` — the matching engine itself; "Planned" section discusses lock-free internals as deferred work.
- `notes/conventions.md` — the broader convention discipline that this rule is part of.
