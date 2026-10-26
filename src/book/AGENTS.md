# Scope

`Nyquestro/src/book`

## Purpose

Order-book state, price levels, and market matching boundary.

## Owns

- Direct files: `market.rs`, `mod.rs`, `order_book.rs`, `price_level.rs`.

## Inherits

Inherits the nearest parent `AGENTS.md`; this file adds only the local delta for this directory.

## Local constraints

- Preserve price-time priority, exact quantity conservation, and cancellation/fill lifecycle invariants across market and price-level changes.
- Matching must remain deterministic for identical event sequences; FIFO order within a price level and best-price traversal are load-bearing contracts.

## Update this file when

- Ownership, interfaces, generated/runtime boundaries, or directory-specific verification requirements change.
