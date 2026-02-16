# Matching Engine and Order Book

**Last Updated:** 2026-02-02

## Scope / Purpose

- Track what exists today regarding order book and matching logic.
- Define concrete missing pieces required to reach a deterministic matcher loop.

## Current Implemented System

- There is no compiled order book or matcher module exposed by `src/lib.rs`.
- `src/matching_engine/order_book.rs` exists as a placeholder file and is currently empty.
- The demo binary does not perform matching and does not maintain a book.

## Implemented Outputs / Artifacts (if applicable)

- None.

## In Progress / Partially Implemented

- `Order::fill()` returns `NyquestroResult<()>`, which is compatible with a design where matching logic creates `FillEvent` rather than the order itself.
- Event frame types exist and are tested, which provides a target for future matcher outputs.
- `PriceLevel` exists as a container for same-price orders but does not support removals or matching.

## Planned / Missing / To Be Changed

- Implement an `OrderBook` type that owns bid/ask collections and enforces price-time priority.
- Implement deterministic matching for incoming limit orders, including partial fills across multiple counterparties.
- Define how matched price is chosen (e.g., resting order price) and encode that consistently in `FillEvent`.
- Emit `QuoteEvent` updates when best bid or best ask changes.
- Remove fully filled orders from the book and keep per-level total quantities correct.
- Wire the matching engine module into `src/lib.rs` once there is a minimal viable API.

## Notes / Design Considerations (optional)

- The immediate next step is correctness and determinism, and lock-free structures can be introduced later if the API boundary stays stable.

## Discarded / Obsolete / No Longer Relevant

- None.

