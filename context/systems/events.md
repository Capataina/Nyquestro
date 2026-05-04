# Events

*Maturity: comprehensive · Stability: stable*

## Scope / Purpose

`src/events/` defines three immutable, `Copy`, allocation-free event frames the matching engine emits: `FillEvent`, `QuoteEvent`, and `OrderEvent`. Every constructor validates its inputs at construction so downstream consumers (replay, fan-out, observability) can rely on the invariants without re-checking.

## Boundaries / Ownership

- **Owns:** the three event types, their constructors, the `OrderRejectionReason` enum, and the `QuoteSide` enum (with `From<Side>` for ergonomic conversion).
- **Does not own:** *when* events are emitted (that's `book::order_book`'s job) or *how* they are consumed (the dashboard reads them; nothing else does today).
- **Imported by:** `book::order_book` (constructs every event the engine emits), `ui::app` (reads `OrderEvent::Rejected` to count rejects, walks `FillEvent` for the tape), and the integration tests under `tests/events_test.rs` + `tests/matching_test.rs`.

## Current Implemented Reality

### `FillEvent`

```rust
pub struct FillEvent {
    pub buyer_order_id: OrderID,
    pub seller_order_id: OrderID,
    pub price: Px,
    pub quantity: Qty,
    pub timestamp: Ts,
}
```

Constructor `FillEvent::new(buyer, seller, price, qty, ts)` rejects:
- zero quantity → `InvalidQuantity`,
- self-match (`buyer == seller`) → `SelfMatch(id)`.

Public fields: yes — the type is `Copy` and validated at construction; downstream consumers reading `event.price` is the canonical access pattern.

### `QuoteEvent`

```rust
pub struct QuoteEvent {
    pub side: QuoteSide,    // Bid | Ask
    pub price: Px,
    pub quantity: Qty,      // 0 means "level cleared"
    pub timestamp: Ts,
}
```

Two constructors with different semantics:
- `QuoteEvent::live(side, price, qty, ts)` — non-zero qty required; rejects zero with `InvalidQuantity`.
- `QuoteEvent::cleared(side, price, ts)` — infallible; produces a zero-quantity quote indicating the level was fully consumed.

This split is deliberate: zero quantity is *meaningful* in a "cleared" quote and *invalid* in a "live" one. A single constructor that accepted zero would conflate the two.

`QuoteSide::from(Side)` maps `Buy → Bid`, `Sell → Ask`.

### `OrderEvent`

A four-variant enum tracking the lifecycle of an aggressor or resting order:

```rust
pub enum OrderEvent {
    Placed    { order_id, side, price, quantity, timestamp },
    Filled    { order_id, executed, remaining, timestamp },
    Cancelled { order_id, remaining, timestamp },
    Rejected  { order_id, reason: OrderRejectionReason, timestamp },
}
```

Four constructors, with the same validation discipline:
- `placed(...)` — rejects zero quantity.
- `filled(...)` — rejects zero `executed` (a fill of zero is meaningless).
- `cancelled(...)` — infallible; cancellation is a structural state, not a validation.
- `rejected(...)` — infallible; the variant exists *because* something failed validation upstream.

Two helper accessors handle variant-uniform reads:
- `event.order_id() -> OrderID`
- `event.timestamp() -> Ts`

`OrderRejectionReason` is a five-variant enum (`InvalidQuantity`, `InvalidPrice`, `InvalidOrderId`, `SelfMatch`, `DuplicateOrderId`) that captures every reason the engine currently rejects an order.

## Key Interfaces / Data Flow

The engine produces events into `SubmitResult { fills, quotes, lifecycle }`:

```
OrderBook::submit_limit
  ├─ for each cross of resting against aggressor:
  │     fills.push(FillEvent::new(...))
  │     lifecycle.push(OrderEvent::filled(aggressor, ...))
  │     [if resting fully filled]
  │       lifecycle.push(OrderEvent::filled(resting, ...))
  ├─ on self-match detection:
  │     lifecycle.push(OrderEvent::rejected(aggressor, SelfMatch, ...))
  ├─ on rest of remainder:
  │     lifecycle.push(OrderEvent::placed(...))
  └─ for each side whose top-of-book changed:
        quotes.push(QuoteEvent::live(...))    or QuoteEvent::cleared(...)
```

## Implemented Outputs / Artifacts

- Three event types, all `Debug + Clone + Copy + PartialEq + Eq + Hash`.
- 11 inline unit tests across the three modules + 9 integration tests in `tests/events_test.rs`.
- A worked-example `events_are_copy` test that statically asserts via `fn assert_copy<T: Copy>(_: T)`.

## Known Issues / Active Risks

- **`OrderEvent::Filled` does not record the counter-party id.** A consumer that needs to reconstruct the trade pair must walk the `fills` vector alongside the `lifecycle` vector. Acceptable today because the trade tape uses `FillEvent` directly and the only `OrderEvent` consumer is the rejection counter, but if a future replay log needs full trade attribution from `OrderEvent` alone, this becomes a limitation.
- **`QuoteEvent::cleared` does not include the previous quantity.** Consumers can't tell from a single cleared quote whether the level held 1 unit or 1 million before clearing. The current dashboard doesn't need this; an audit log might.

### Downstream impact

The engine's correctness rests partly on these events being *trustworthy*: a `FillEvent` with `buyer == seller` would mislead any replay consumer. The validation is therefore *defence in depth* — the book also rejects self-match before constructing the event, so a `FillEvent::new` failure in production indicates the book's own check was bypassed. Worth a panic-on-fatal one day.

## Partial / In Progress

None.

## Planned / Missing / Likely Changes

- **`OrderEvent::Modified`** when atomic order-modification arrives (a Tier-1 README feature beyond MVP).
- **`FillEvent::aggressor: Side`** field — currently the dashboard tape stores `aggressor` separately because `FillEvent` does not carry it. Folding it in would let the tape walk only `fills` instead of joining two lists.
- **Event versioning** if/when the binary wire protocol arrives — the README pitches a versioned + length-prefixed UDP frame, which would require either a wrapper or a `version: u16` field on each event.

## Durable Notes / Discarded Approaches

- **Events are `Copy`, not `Clone`-only.** The prior codebase had `OrderEvent` storing `OrderRejectionReason` *plus* a `String` reason field. The `String` was the obstacle to `Copy`. Replacing it with the enum keeps the type 8-byte-aligned and `Copy`, which matters for the planned event fan-out and replay loop.
- **The `live` / `cleared` split on `QuoteEvent` was deliberate.** A single `QuoteEvent::new(side, price, qty, ts)` that accepted zero would not distinguish "this level was just cleared" from "this is a malformed event with zero size". Two constructors, two intents.
- **`OrderEvent::placed` validates zero quantity, `OrderEvent::cancelled` does not.** Asymmetry intentional: a placed order with zero size is invalid (the constructor refuses it); a cancelled order with zero remaining is the natural state of "cancelling an order that just filled fully" and is meaningful.
- **Rejected variant reasons live in an enum, not a `String`.** Same `Copy`-preservation reason as the parent `OrderEvent`. The five reason variants are exhaustive over the engine's current rejection paths; adding a sixth requires touching one place.

## Obsolete / No Longer Relevant

- `OrderEvent::New` (the variant name) — renamed to `Placed` to disambiguate from the `New` rejection-reason name and to match the lifecycle vocabulary used in the trace docs.
- The blanket `get_*` accessors (`get_order_id`, `get_price`, `get_quantity`, `get_side`) — removed in favour of (a) public fields on the struct types (`Copy`-friendly direct read) and (b) two `match` helpers `order_id()` and `timestamp()` on the enum where uniform access matters.
- The commented-out self-match check in `FillEvent::new` (`// if buyer_order_id == seller_order_id { return Err(...) }`) — replaced with an active check returning `NyquestroError::SelfMatch`.
- Generic `OrderEvent::new(...)` constructor — removed; the four lifecycle states have four named constructors with state-specific validation.
