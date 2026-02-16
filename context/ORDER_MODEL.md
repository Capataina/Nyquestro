# Order Model and Price Levels

**Last Updated:** 2026-02-02

## Scope / Purpose

- Describe the current in-memory representation of orders and price levels.
- Capture current invariants, state transitions, and known gaps that affect future matching work.

## Current Implemented System

- `Order` is implemented in `src/order.rs` as a struct with identifiers, side, price, quantity, timestamp, remaining quantity, and status.
- `Order::new(order_id, side, price, quantity) -> NyquestroResult<Order>` rejects zero quantity via `NyquestroError::InvalidQuantity`.
- `Order::new()` sets `timestamp` using `Ts::now()` and sets `status` to `Status::Open`.
- `Order::fill(fill_amount) -> NyquestroResult<()>` subtracts `fill_amount` from `remaining_quantity` using `Qty::saturating_sub()`.
- `Order::fill()` updates order status by calling `Order::update_status()`.
- `Order::update_status() -> NyquestroResult<()>` sets `Open`, `PartiallyFilled`, or `FullyFilled` based on `quantity` and `remaining_quantity`.
- `Status::Cancelled` exists as a type variant but there is no cancellation API on `Order`.
- `Order::get_status(self) -> Status` takes `self` by value, so callers must move or clone the order to read status.
- `PriceLevel` is implemented in `src/price_level.rs` as a struct containing a single `Px`, a `Vec<Order>`, and a `total_quantity`.
- `PriceLevel::add_order(order) -> NyquestroResult<()>` rejects orders whose price does not match the level price using `NyquestroError::InvalidPrice`.
- `PriceLevel::add_order()` clones the order and stores it in the internal `Vec<Order>`.
- `PriceLevel::total_quantity` is updated by adding each order’s remaining quantity when it is added to the level.
- `PriceLevel::get_orders() -> NyquestroResult<Vec<Order>>` clones and returns the full order list.

## Implemented Outputs / Artifacts (if applicable)

- None.

## In Progress / Partially Implemented

- `Order::fill()` does not validate that `fill_amount` is less than or equal to `remaining_quantity`, so over-fills silently clamp to zero.
- There is no API to remove orders from a `PriceLevel`, so `total_quantity` is not maintained through fills or removals.
- `PriceLevel` provides no explicit FIFO contract beyond the insertion order of `Vec<Order>`.
- The demo binary prints “Fill Event” text but does not emit or store a `FillEvent` value.

## Planned / Missing / To Be Changed

- Add fill-amount validation so `Order::fill()` rejects over-fills instead of clamping silently.
- Add order cancellation and cancellation state transitions, including how a cancelled order affects price level totals.
- Introduce an order book structure that owns and mutates price levels, including removal of filled orders.
- Replace cloning-heavy getters and storage patterns once a book/matcher API defines the required ownership model.

## Notes / Design Considerations (optional)

- The current `Vec<Order>` approach is a functional placeholder for correctness work but conflicts with the lock-free intent in the root `README.md`.

## Discarded / Obsolete / No Longer Relevant

- None.
