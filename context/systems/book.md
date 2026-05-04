# Book

*Maturity: comprehensive · Stability: unstable — MVP-quality matching loop, performance + concurrency work explicitly deferred*

## Scope / Purpose

`src/book/` houses the matching engine itself: `OrderBook` (the bid/ask ladder, single-symbol), `PriceLevel` (the FIFO queue at one price), and `Market` (the multi-instrument wrapper holding one `OrderBook` per `Symbol`). This is the system the rest of the project exists to observe.

It implements:
- **Strict price-time priority** — best price first, FIFO within price.
- **Deterministic matching** — given a fixed input sequence, the produced `FillEvent`/`QuoteEvent`/`OrderEvent` outputs are byte-identical across runs.
- **Self-match rejection at match time** — the aggressor is wholly rejected; the resting counterparty is untouched.
- **Top-of-book quote semantics** — quotes are emitted only when best price or displayed quantity changes on the affected side.
- **Microstructure inspection surface** — `microprice()`, `ofi(n)`, `spread_cents()`, `depth(n)`, `level_counts()`, `top_n_bids(n)`, `top_n_asks(n)` for direct read by the dashboard's engine pane.
- **Multi-instrument routing** — `Market::submit_limit(order)` reads `order.symbol()` and routes to the per-symbol book, auto-registering the symbol on first sight.

It does *not* implement (yet): market orders, IOC/FOK semantics, order modification, atomic cancellation under concurrency, lock-free structures, slab allocation. These are README-tier features that sit on top of the MVP described here.

## Boundaries / Ownership

- **Owns:** `OrderBook` (two `BTreeMap<Px, PriceLevel>` ladders), `PriceLevel` (one `VecDeque<Order>` + a running `total_quantity`), `SubmitResult` (the structured output of `submit_limit`), the matching algorithm, the cancel algorithm, and the top-of-book change detector.
- **Does not own:** order construction (lives in `order::Order`), event validation (lives in `events/*`), wire-protocol concerns (none yet), or persistence (none yet).
- **Imported by:** `ui::app` (the engine's only caller today), `simulator::market` indirectly via `App`, every test in `tests/matching_test.rs` + `tests/price_level_test.rs`, and the headless mode in `main.rs`.

## Current Implemented Reality

### Data structures

```
OrderBook
  bids: BTreeMap<Px, PriceLevel>   // best is iter().next_back()
  asks: BTreeMap<Px, PriceLevel>   // best is iter().next()

PriceLevel
  price:           Px
  orders:          VecDeque<Order>  // FIFO, front is oldest
  total_quantity:  Qty              // running sum, O(1) read
```

`BTreeMap` keyed by `Px` gives sorted price ladders for free; `VecDeque` gives O(1) push_back and O(1) pop_front for the FIFO at each price.

### Submission algorithm

`OrderBook::submit_limit(order) -> NyquestroResult<SubmitResult>` runs four phases:

1. **Snapshot** the pre-state of best bid + best ask on both sides (used in phase 4 for change detection).
2. **Aggressive matching loop:**
   - probe the opposite side's best level; if not crossing, break;
   - if the resting front's id == aggressor's id, set `self_match_detected` and break (phase 3);
   - otherwise compute `trade_qty = min(aggressor.remaining, resting.remaining)`;
   - mutate inside a tight scope: `resting.fill(trade_qty)`, `level.record_execution(trade_qty)`, then `aggressor.fill(trade_qty)` after the borrow drops;
   - emit `FillEvent` (price = resting price, ts = resting ts), and `OrderEvent::filled` for both parties as appropriate;
   - if the resting order is now terminal, `level.pop_front()` and `OrderEvent::filled` for the resting side too;
   - if the level is empty, `BTreeMap::remove` it.
3. **Self-match handling:** if detected, push `OrderEvent::rejected(aggressor, SelfMatch, ts)` and skip the resting phase.
4. **Resting:** if the aggressor still has `remaining > 0` and is active, push it to the same-side ladder, emit `OrderEvent::placed`.
5. **Quote emission:** for each side whose top-of-book *changed*, emit a `QuoteEvent::live` (or `cleared` if the side became empty).

### Self-match policy (match-time rejection)

When the aggressor's id matches the resting front's id, the aggressor is rejected wholesale — no fills are produced, the resting order is untouched. The resting side's `OrderEvent::rejected` carries `OrderRejectionReason::SelfMatch`. The `tests/matching_test.rs::self_match_rejects_aggressor_leaves_resting` test pins this: after the rejection, `book.best_ask()` is still `Some(...)` for the original resting sell.

### Quote emission semantics

A quote is emitted only when `top_of_side_before != top_of_side_after`. This catches three transitions:
- best price moved (e.g. best ask 10000 consumed, new best is 10010);
- best price unchanged but displayed quantity changed (a new resting order at the same best);
- side became empty (emitted as `QuoteEvent::cleared`).

The `tests/matching_test.rs::quote_emitted_only_on_top_of_book_change` test verifies that adding a resting order at a *worse* price than the current best does *not* emit a quote.

### Cancellation

`cancel(id, ts) -> NyquestroResult<OrderEvent>` walks both sides linearly through their levels until it finds the order with the given id. O(N_levels × N_orders_per_level) — acceptable for MVP since cancellations are rare relative to fills, and the dashboard's resting-id cache is only refreshed every 250ms.

## Key Interfaces / Data Flow

```rust
pub struct SubmitResult {
    pub fills:     Vec<FillEvent>,
    pub quotes:    Vec<QuoteEvent>,
    pub lifecycle: Vec<OrderEvent>,
}

impl OrderBook {
    pub fn new() -> Self;
    pub fn submit_limit(&mut self, Order) -> NyquestroResult<SubmitResult>;
    pub fn cancel(&mut self, OrderID, Ts) -> NyquestroResult<OrderEvent>;

    // Inspection — read-only, no clones.
    pub fn best_bid(&self) -> Option<(Px, Qty)>;
    pub fn best_ask(&self) -> Option<(Px, Qty)>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn bid_levels(&self) -> impl DoubleEndedIterator<Item = (&Px, &PriceLevel)>;
    pub fn ask_levels(&self) -> impl Iterator<Item = (&Px, &PriceLevel)>;
}
```

`PriceLevel`'s public surface:

```rust
PriceLevel::new(Px) -> Self
push_back(&mut self, Order) -> NyquestroResult<()>     // rejects price mismatch
front() -> Option<&Order>
front_mut() -> Option<&mut Order>
record_execution(&mut self, Qty) -> NyquestroResult<()>  // decrement total_quantity
pop_front() -> Option<Order>                              // updates total_quantity
remove_by_id(&mut self, OrderID) -> Option<Order>         // O(n), updates total
iter() -> impl Iterator<Item = &Order>                    // FIFO order
len() / is_empty() / price() / total_quantity()
```

The borrow split inside the matching loop deserves explicit notation:

```rust
let (resting_id, resting_ts, resting_done, trade_qty) = {
    let resting = level.front_mut().expect("non-empty");        // &mut borrow OPENED
    let trade = Qty::new(order.remaining().value()
        .min(resting.remaining().value()));
    resting.fill(trade)?;                                       // mutate
    (resting.id(), resting.timestamp(), resting.status().is_terminal(), trade)
};                                                              // &mut borrow CLOSED
level.record_execution(trade_qty)?;                              // can now use &mut level
order.fill(trade_qty)?;
```

This explicit scope is what lets the same `&mut PriceLevel` be used twice per fill (once via `front_mut`, once via `record_execution`) without the borrow checker rejecting the call.

## Implemented Outputs / Artifacts

- The matching loop, the cancel walk, the inspection API.
- 8 inline unit tests in `book/price_level.rs` covering FIFO, total-quantity invariant, push-back rejection, removal, in-place fill via `front_mut`.
- 12 integration tests in `tests/matching_test.rs`: simple cross, three-level sweep, partial-then-rest, FIFO within a level, self-match rejection, cancellation (success + unknown-id), determinism on a 6-order sequence, top-of-book quote semantics, aggressor-full-fill-does-not-rest, aggressor terminal state.
- 6 integration tests in `tests/price_level_test.rs`.

## Known Issues / Active Risks

- **`OrderBook::cancel` is O(N) per call** because it walks all levels of both sides until it finds the id. For an MVP this is acceptable; for a high-cancel-rate workload (per the README's HFT framing) it would need an `OrderID → (Side, Px, queue_pos)` index. Every additional resting order at the time of cancel costs work.
- **No instrument dimension.** The book is implicitly single-instrument. Adding multi-instrument support requires keying everything by `Symbol` first; this is a wholesale change to the data structure, not a tweak.
- **`SubmitResult.lifecycle` does not preserve a strict ordering invariant across event kinds.** Within each phase, ordering is deterministic (matching traversal order is FIFO+price-time); across phases, the order is "fills in matching order, then rejection if any, then placed if any, then quotes". A consumer that expects time-ordered interleaving would need to re-sort by timestamp.

### Downstream impact

The matching engine is the single most load-bearing system. A bug here:
- corrupts the trade tape (fills appear that should not have happened, or are missing);
- corrupts the metrics (latency histograms get bogus samples);
- corrupts the dashboard's depth-of-book ladder (stale `total_quantity` makes the bars wrong size).

Therefore: any change to `submit_limit` or `cancel` must rerun the full `tests/matching_test.rs` suite. The `run_twice_identical_sequence_identical_output` test is the canonical determinism guard.

## Partial / In Progress

None — the MVP is complete to spec. The engine ships as a working, tested artefact.

## Planned / Missing / Likely Changes

The README pitches a long Tier-1/2/3 feature set; from the matching engine's perspective the biggest planned increments are:

- **Market / IOC / FOK orders.** Requires an `OrderType` field on `Order`; the matching loop's "rest the remainder" phase becomes conditional.
- **Order modification.** Atomic price/qty changes preserving time priority where the rules allow.
- **Hidden / iceberg quantity.** Displayed vs total size at a level — `PriceLevel::total_quantity` becomes "displayed total" and a separate "true total" is tracked.
- **`OrderID → location` index** for O(log n) or O(1) cancellation.
- **Lock-free internals.** Per the D2 design decision in `notes/safe-rust-philosophy.md`, this is explicitly deferred behind a stable public API. The `BTreeMap`+`VecDeque` internals can be swapped for atomic structures without changing `submit_limit`'s signature.
- **Multi-instrument support.** A `BTreeMap<Symbol, OrderBook>` wrapper, or a refactor of `OrderBook` to be parameterised by symbol.

## Durable Notes / Discarded Approaches

- **Match price = resting price.** The aggressor receives price improvement when it crosses. Considered "match at incoming price" but rejected because resting-price matching is the standard for price-time-priority limit books and is what every conformant venue does. The `tests/matching_test.rs::aggressor_sweeps_three_levels` test asserts the per-fill prices match the resting levels.
- **Top-of-book change detection uses snapshot+compare, not delta-tracking.** Snapshot before mutation, compare after; emit a quote iff `(price, qty)` differs. Considered tracking deltas inside the matching loop but rejected because (a) the snapshot is `Option<(Px, Qty)>` — 16 bytes — and (b) the mutation logic is far simpler when the comparison happens once at the end.
- **Self-match check is in two places.** `FillEvent::new` rejects, *and* `OrderBook::submit_limit` rejects before constructing a `FillEvent`. This is defence-in-depth: an external constructor that calls `FillEvent::new` directly still gets the protection, and the engine's own check is the load-bearing one for the dashboard's reject counter.
- **Determinism is preserved by never calling `Ts::now()` in `submit_limit`.** Resting-order timestamps are reused for fills. The aggressor's timestamp is reused for the placed/rejected lifecycle. Two runs of the same input sequence produce byte-identical event vectors.
- **`pub` fields on event types instead of getters.** The events are `Copy` — there is no encapsulation to defend.

## Obsolete / No Longer Relevant

- `src/matching_engine/order_book.rs` (the 0-byte placeholder file) — deleted; the new home is `src/book/order_book.rs`.
- `src/price_level.rs` (the flat-layout file with `Vec<Order>` and clone-on-add) — replaced with `src/book/price_level.rs` using `VecDeque` and ownership-transfer semantics.
- The IMPLEMENT_NOW plan file — its Phase A (hardening) and Phase B (OrderBook MVP) are both implemented; the plan file itself was deleted as part of this rewrite (along with the rest of the prior context/).
