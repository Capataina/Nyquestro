# Plan: Medium Wins (the depth that reads as "HFT engineer")

## Header

- **Status:** Planned (not started)
- **Scope:** Three multi-day pieces that move the project from "careful, correct engine" to "this person understands what the job actually is." Bundled so they aren't lost; each is independent and gets its own done-marker.
- **Why this matters:** The small wins make the repo *look* finished. These make it *read as expert*. For HFT specifically, **depth beats breadth**: one deeply-measured optimisation with before/after evidence impresses a Jane Street / HRT interviewer more than five additional order types, because measure-diagnose-fix-prove is the actual day job. Most candidate projects never do it.
- **Exit rule:** archived when all three sub-items are ticked.

Items:

1. Hot-path data-structure optimisation (single-threaded)
2. Strategy agent (market-making bot)
3. Latency post-mortem (a written artefact)

> [!important] How this differs from the large lock-free plan
> Item 1 here is about **algorithmic and memory speed on one thread** — making `submit_limit` do less work and touch fewer cache lines. It stays single-threaded. The separate [`lock-free-engine.md`](lock-free-engine.md) plan is about **concurrency** — multiple threads without locks. These are two genuinely different hard problems that both live under the word "performance", and it's worth being able to articulate the difference in an interview: *"first I made the single-threaded path fast; concurrency is a separate axis I'd add by sharding, not locking."*

---

## 1. Hot-path data-structure optimisation (single-threaded)

- **What:** Replace the `BTreeMap<Px, PriceLevel>` ladder + `VecDeque` FIFO with the canonical real-engine structure, and **measure the difference**:
  - **Price ladder as a flat array indexed by price-tick.** Prices move in discrete ticks within a bounded band around the touch. An array slot per tick gives O(1) best-price lookup (track a cursor to the best occupied slot) and O(1) insert-at-price, versus the BTreeMap's O(log n) plus pointer-chasing (cache misses). Handle the band edge: re-base or fall back to a map for far-out prices.
  - **Intrusive doubly-linked list per level + `HashMap<OrderID, node>`.** Today cancel-by-id is O(n) (`remove_by_id` scans the VecDeque). An intrusive list (the prev/next links live *inside* the order node) plus an id→node map makes cancel O(1): look up the node, unlink it. "Intrusive" means no per-node heap allocation for the list itself.
  - **Slab / pool allocator for order nodes.** Pre-allocate a big `Vec` of node slots, recycle via a free-list, so order insert/remove never calls the system allocator on the hot path (malloc contention + fragmentation disappear from the tail).
- **Why for hiring:** This *is* the matching-engine craft. "I profiled it, saw the BTreeMap lookups and the allocator dominating the tail, moved to a tick-array + intrusive list + slab pool, and cut p99 by X" is a sentence that gets you past the first technical round. The point is not just the structures — it's that you measured before and after.
- **Steps:**
  - [ ] **First, prove the problem.** Land `benchmark-harness.md` (or a criterion micro-bench from `extensive-testing-framework.md`), capture baseline p50/p99/p99.9 + peak RSS for the current BTreeMap design. Do not optimise on vibes.
  - [ ] Introduce the tick-array ladder behind the existing `OrderBook` public surface (keep the API; swap the internals — this is why the modular design matters).
  - [ ] Intrusive list + id→node map for O(1) cancel.
  - [ ] Slab pool for nodes.
  - [ ] Re-run the benchmark; record before/after in the post-mortem (item 3).
  - [ ] Keep all 101 existing tests green throughout — the public behaviour must not change, only the speed.
- **Done when:** internals are swapped, all tests pass, and there's a measured before/after table showing the win (or honestly showing where it didn't help, which is also a real result).

## 2. Strategy agent (market-making bot)

- **What:** The participant side the README pitches heavily and the code doesn't yet have. A simple bot that consumes the market-data view, quotes both sides of the spread, and manages inventory + PnL:
  - **Order-book reconstruction** from the feed (top N levels per side).
  - **Spread / mid / microprice** computation (the engine already exposes `microprice`, `spread_cents`, `ofi`, `depth` — reuse them).
  - **Two-sided quoting:** post a bid and an ask around mid to capture the spread.
  - **Inventory-aware skew:** as the bot accumulates a position, shift its quotes to encourage trades that flatten it (the core defence against directional drift).
  - **Order-flow-imbalance skew:** when one side of the book is much heavier, lean quotes in the likely direction (a first defence against adverse selection).
  - **PnL accounting:** realised + unrealised, with an end-of-run summary.
- **Why for hiring:** A matching engine alone shows infrastructure depth. An engine *plus* a participant that reads the book and manages risk shows you understand the **whole feedback loop** — both the venue and the trader. Almost no candidate project has both sides. This is also where the microstructure vocabulary (adverse selection, inventory risk, OFI) becomes something you've *built*, not just read about.
- **Steps:**
  - [ ] `src/agent/` (new) — a `MarketMaker` that takes a read-only market view + an order-submission channel.
  - [ ] Wire it as an optional participant in synthetic mode (`--with-mm`), quoting against the simulator's flow.
  - [ ] Implement mid/spread quoting → inventory skew → OFI skew, in that order, each a separate commit.
  - [ ] End-of-run PnL report (reuse the benchmark reporter's table style).
  - [ ] A dashboard pane (or benchmark line) showing the bot's inventory + PnL over the run.
- **Done when:** the bot runs alongside the engine, quotes both sides, visibly manages inventory, and reports PnL.
- **Honesty caveat to bake into its docs:** it is a *demonstration of the mechanics*, not a profitable strategy, and makes no claim of edge on real markets. Say so plainly — overclaiming here reads as naïve to exactly the audience you're targeting.

## 3. Latency post-mortem (a written artefact)

- **What:** A single document (`context/references/latency-postmortem.md` or a README-linked write-up) that takes **one** optimisation from item 1, and tells the story: baseline numbers → what the profiler/counters showed was slow → the change → after numbers → what it cost. With real evidence (benchmark output, and ideally CPU-counter or sampling-profiler data).
- **Why for hiring:** This one *document* is worth more than several features. It demonstrates the actual craft — honest measurement, root-causing a tail, a change justified by evidence, and a re-measurement — which is precisely what the job is. It's also the perfect thing to walk an interviewer through.
- **Steps:**
  - [ ] Pick one change from item 1 (the cancel O(n)→O(1) is a clean, legible story).
  - [ ] Capture before numbers (benchmark harness).
  - [ ] Show the diagnosis: why was the tail slow? (the `remove_by_id` scan, or allocator pressure). Use a profiler (`samply`, `cargo-flamegraph`) or `perf stat` on Linux for cache-miss / branch-mispredict counters if available.
  - [ ] Make the change; capture after numbers.
  - [ ] Write it up as a short narrative with the before/after table and the evidence.
- **Done when:** the write-up exists, links real numbers, and reads as "I measured, diagnosed, fixed, re-measured."

## Notes

- **Order matters:** item 1's baseline must be captured *before* the optimisation, so do `benchmark-harness.md` first. Item 3 is the write-up *of* item 1, so it lands last. Item 2 (the agent) is independent and can be done any time.
- These are the highest-leverage items in the whole backlog for the stated hiring goal. If only one thing gets done, do item 1 + its post-mortem (item 3).
