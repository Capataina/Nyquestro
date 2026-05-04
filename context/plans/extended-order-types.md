# Plan: Extended Order Types

## Header

- **Status:** Planned (not started)
- **Scope:** Implement IOC, FOK, AON, iceberg/hidden, and peg order types on top of the existing Limit-only matching engine. Add corresponding test coverage.
- **Why this matters:** Real exchanges support all of these. An MVP that supports only `Limit` is incomplete to a quant's eye. Adding them shows microstructure depth and exercises the engine's matching-loop generality.
- **Exit rule:** complete when (a) all five types parse, validate, and route correctly through `submit_limit`, (b) each has dedicated example tests covering the canonical cases, (c) the dashboard's order-event tape can render rejection reasons specific to each type.

## Implementation Structure

### Modules / files affected

- `src/types.rs` — add `OrderType` and `TimeInForce` enums.
- `src/order.rs` — add `order_type: OrderType` field to `Order`; constructor variants per type.
- `src/book/order_book.rs` — branch matching behaviour on `OrderType` after the cross check.
- `src/events/lifecycle.rs` — extend `OrderRejectionReason` with FOK / AON / IOC failure variants.
- `tests/order_types_test.rs` (new).

### `OrderType` enum

```rust
pub enum OrderType {
    Limit,        // resting order, default behaviour today
    Market,       // priced at the touch + N ticks, no resting
    IOC,          // immediate-or-cancel: match what's available, cancel the rest
    FOK,          // fill-or-kill: full quantity must match immediately, else reject
    AON,          // all-or-none: only matches at full quantity, can rest
    Iceberg { displayed: Qty }, // shows only `displayed` size at a time, refills from hidden remainder
    Peg { reference: PegRef, offset_cents: i64 }, // tracks bid/ask/mid with an offset
}

pub enum PegRef { BestBid, BestAsk, Mid }
```

### Responsibility boundaries

- `OrderType` is a value enum on `Order`. The matching loop dispatches on it.
- Iceberg state (the hidden remainder) is part of the `Order`, not a separate side-table.
- Peg orders are *re-priced* on every quote-update tick by the engine, not by the simulator.

## Algorithm / System Sections

### A) IOC

After matching against opposite side, *do not rest the remainder*. Emit `OrderEvent::Cancelled { remaining, ... }` for any leftover.

**Playbook:**
- [ ] In `submit_limit`, after the matching loop: if `order_type == IOC && order.remaining > 0`, emit Cancelled and skip the resting phase.
- [ ] Test: IOC order against thin book partially fills, leftover is cancelled (not rested).
- [ ] Test: IOC order against deep book fully fills, no Cancelled event.

### B) FOK

Before matching, *probe* whether the full quantity can be matched immediately. If not, reject without any partial fills.

**Playbook:**
- [ ] Add a probe phase: walk the opposite side and sum quantity until ≥ aggressor's quantity. If insufficient, `OrderEvent::Rejected { reason: FokInsufficientLiquidity }`.
- [ ] Test: FOK that exactly matches deep book → fully filled.
- [ ] Test: FOK against a book with insufficient depth → rejected, book unchanged.

### C) AON

Similar to FOK but the order *can rest* if it doesn't match — and once resting, it only fills when a counter-order can match its full quantity.

**Playbook:**
- [ ] On submit: try to match in full; if can't, rest with `order_type: AON` flag.
- [ ] On future submits: when crossing a resting AON, the aggressor must have ≥ AON's quantity, else skip the AON and continue to the next price level.
- [ ] Test: small order can't take liquidity from a large resting AON.
- [ ] Test: AON's quantity grows by aggregating multiple aggressors? No — AON requires single-aggressor full-fill.

### D) Iceberg

Show only `displayed` quantity. When that displayed slice fully fills, refill from the hidden remainder, re-emitting `OrderEvent::Placed` for the next slice (with the same id).

**Playbook:**
- [ ] `Order` carries `displayed: Qty` and `hidden: Qty`. The book sees only `displayed`.
- [ ] When the displayed portion fully fills: if `hidden > 0`, transfer `min(displayed, hidden)` from hidden to displayed, emit a fresh `Placed`-equivalent (or a new `OrderEvent::IcebergRefilled`).
- [ ] Test: 100-unit iceberg with 10-unit display fills 100 units across multiple slices.

### E) Peg

Re-priced on every top-of-book change. Engine maintains a list of pegged orders and re-inserts them at the new price when the reference moves.

**Playbook:**
- [ ] After every `quote_update`, walk pegged orders, recompute their target price, and reinsert at the new level.
- [ ] Test: peg-best-bid + 1 cent always sits one cent above the bid.
- [ ] Test: peg adjusts when its reference disappears (book becomes one-sided).

### F) Market

Effectively a Limit at "infinitely aggressive" price. The simulator already simulates it by pricing 5 ticks through the touch. Promoting Market to a first-class type means the engine can express the intent without faking a price.

**Playbook:**
- [ ] `OrderType::Market` orders skip the price comparison and match against the best opposite regardless.
- [ ] Edge: Market against an empty opposite side → reject with `OrderRejectionReason::NoLiquidity`.

## Integration Points

- The `OrderType` field propagates through:
  - `Order::new(..., order_type)`,
  - `OrderEvent::Placed { order_type, ... }`,
  - The dashboard's tape (different glyph per type? Optional polish).
- The simulator gains a knob for *which fraction* of submitted orders are which type. Default ~80% Limit / 15% IOC / 5% FOK is realistic for crypto venues.

## Debugging / Verification

- Iceberg refill emits two events for what looks like one order — make sure tape consumers don't double-count.
- Peg re-pricing during high-frequency book updates can become a hot path — measure before optimising.
- FOK's "probe phase" should be O(levels), not O(orders); cap it at 50 levels.

## Completion Criteria

- [ ] `OrderType` enum lives in `types.rs`.
- [ ] All five non-Limit types pass dedicated example tests.
- [ ] `proptest` invariants (per `plans/property-based-tests.md`) are extended to assert that, e.g., FOK never partially fills.
- [ ] Dashboard's engine pane shows a per-type fill ratio (`Limit 73%, IOC 18%, ...`).
- [ ] `systems/book.md` is updated to describe the type-conditional matching path.
- [ ] This file is archived once all the above are checked.
