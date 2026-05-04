# Order

*Maturity: comprehensive · Stability: stable*

## Scope / Purpose

`src/order.rs` defines the `Order` entity — a single resting or aggressing limit order. It owns a small state machine: validated construction, checked fills, one-way status transitions, and cancellation. Every mutation either succeeds and updates state, or returns a classified error and leaves state untouched.

## Boundaries / Ownership

- **Owns:** the `Order` struct, its constructor (with caller-supplied timestamp + a `_now` convenience variant), `fill`, `cancel`, the private `transition_to` helper, and read-only accessors for every field.
- **Does not own:** market-wide matching logic (lives in `book::order_book`), event emission (lives in `book` + `events`), or order *placement* into a price level (`PriceLevel::push_back` does that).
- **Imported by:** `book::price_level` (stores `Order`s), `book::order_book` (mutates the front of a level), `simulator::market` (constructs orders to submit), `tests/order_test.rs`, and the dashboard's `App` indirectly via the simulator.

## Current Implemented Reality

```rust
pub struct Order {
    id: OrderID,
    side: Side,
    price: Px,
    quantity: Qty,        // immutable — original size
    remaining: Qty,       // mutates with fills
    timestamp: Ts,        // immutable — caller-supplied for determinism
    status: Status,
}
```

- All seven fields are `Copy` ⇒ `Order` is `Copy`. This matters for the matching loop, which clones the front of a level cheaply.
- `Order::new(id, side, price, quantity, ts)` is the canonical constructor — caller supplies the timestamp so the engine's matching path stays deterministic. `Order::new_now` is a convenience that calls `Ts::now()` for tests and demos.
- `fill(amount)` is checked at three layers: terminal-status guard → zero-amount guard → `checked_sub` for over-fill detection. Any failure returns `Err(...)` and leaves `Order` untouched.
- `cancel()` transitions to `Cancelled` if the order is still active; rejects otherwise.
- `transition_to(next)` (private) is the single place a status change happens. It calls `Status::can_transition_to` and returns `InvalidStatusTransition` if the move is illegal.

## Key Interfaces / Data Flow

```rust
Order::new(OrderID, Side, Px, Qty, Ts) -> NyquestroResult<Order>
Order::new_now(OrderID, Side, Px, Qty) -> NyquestroResult<Order>

// Read-only — `&self`, no clones, no consumption.
o.id() / o.side() / o.price() / o.quantity() / o.remaining()
o.filled()        // quantity - remaining
o.timestamp()
o.status()
o.is_active()     // status.is_active()

// Mutations.
o.fill(amount: Qty) -> NyquestroResult<()>
o.cancel()        -> NyquestroResult<()>
```

Fill semantics, depicted:

```
fill(amount)
  ├─ if status.is_terminal()       → Err(OrderTerminal(id))
  ├─ if amount.is_zero()            → Err(InvalidQuantity)
  ├─ remaining.checked_sub(amount)
  │    ├─ None → Err(OverFill { order_id, fill, remaining })
  │    └─ Some(new_remaining) →
  │         ├─ next_status = if new_remaining.is_zero() { FullyFilled } else { PartiallyFilled }
  │         ├─ transition_to(next_status)?
  │         └─ self.remaining = new_remaining
  └─ Ok(())
```

The transition step happens *before* the remaining-quantity write so a `Cancelled` order whose status was already terminal cannot be partially filled by accident: the transition guard refuses any move from `Cancelled`/`FullyFilled`.

## Implemented Outputs / Artifacts

- The `Order` type with `Debug + Clone + Copy + PartialEq + Eq + Display`.
- 9 inline unit tests + 8 integration tests in `tests/order_test.rs` covering: zero-quantity rejection, partial-then-full lifecycle, over-fill rejection with state preservation, terminal rejection, cancellation pre/post fill, `&self` accessors not consuming the order.
- `Display` impl that renders human-readable lines like `Order#1 BUY 50@$101.05 (50 remaining, OPEN)`.

## Known Issues / Active Risks

- The `filled()` accessor uses `quantity().value() - remaining().value()` directly, relying on the invariant that `quantity ≥ remaining` is maintained by the constructor and `fill`. The invariant is jointly maintained — there is no single explicit check after construction. Acceptable today because every mutation goes through `fill` (which uses `checked_sub`) and `cancel` (which doesn't touch `remaining`). Any future mutator that touches `remaining` must preserve this.
- `Order` is `Copy`, which means the matching loop in `OrderBook::submit_limit` reads the resting order's fields *after* mutating them through a `&mut` reference borrowed from `PriceLevel::front_mut`. This works because the inner scope releases the borrow before the outer scope reads. Worth noting for any future refactor that flattens that scope.

### Downstream impact

A misbehaving `Order::fill` directly corrupts:
- the parent `PriceLevel`'s `total_quantity` (because the engine calls `record_execution(executed)` based on `Order`'s claimed fill amount),
- the engine's `FillEvent` quantity (which trusts the `trade_qty` we passed into `Order::fill`),
- the dashboard's tape and fill counter.

The over-fill protection is therefore *the* load-bearing invariant of the matching engine. The `tests/order_test.rs::over_fill_returns_error_and_preserves_state` test is the canonical pinning test; do not weaken it.

## Partial / In Progress

None. The state machine is closed under the four-status model (Open → PartiallyFilled → FullyFilled / Cancelled).

## Planned / Missing / Likely Changes

- **`replace` / `modify`** for atomic price/quantity changes preserving time priority where rules allow. Not yet implemented; the README mentions it as a Tier-1 README feature beyond MVP.
- **`OrderType` field** when market / IOC / FOK arrive. The current `Order` represents a Limit; type-erased.
- **`AccountID` field** for account-level self-match prevention (currently the engine self-matches on `OrderID` only).

## Durable Notes / Discarded Approaches

- **`Order::new` takes a caller-supplied `Ts`.** The prior version called `Ts::now()` internally. That made the matching engine non-deterministic — running the same input sequence twice produced different `FillEvent::timestamp` values. The fix was to push timestamp ownership to the caller; the matching loop reuses each resting order's existing timestamp for fills, so the entire flow is deterministic given the input sequence. The `tests/matching_test.rs::run_twice_identical_sequence_identical_output` test pins this contract.
- **Accessors take `&self`, never `self`.** The prior version had `get_status(self) -> Status` which consumed the order — the demo binary literally cloned the order six times in a single println to read its fields. The replacement is idiomatic Rust: `id()`, `side()`, `price()`, etc., all `&self`. The `observing_state_does_not_consume` test pins this.
- **`fill` returns `Err` *without* mutating state.** The prior version used `saturating_sub` and silently clamped. The new flow validates first, then commits. The two-phase commit (transition first, then write `remaining`) is what makes this safe: if the status transition fails, `remaining` is never touched.

## Obsolete / No Longer Relevant

- `Order::update_status` (was a public method that recomputed status from the quantity comparison) — removed. Status changes happen exclusively inside `fill` and `cancel` via the `transition_to` helper, which goes through `Status::can_transition_to`.
- `get_*`-prefixed accessors (`get_order_id`, `get_side`, `get_price`, …) — removed in favour of idiomatic Rust accessors.
- `get_status(self)` consuming form — removed; accessor is `status(&self)`.
