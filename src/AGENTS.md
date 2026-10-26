# Scope

`Nyquestro/src`

## Purpose

Primary application source and top-level module boundary.

## Owns

- Direct files: `errors.rs`, `lib.rs`, `main.rs`, `order.rs`, `types.rs`.
- Child boundaries: `book/`, `events/`, `feed/`, `metrics/`, `simulator/`, `telemetry/`, `ui/`.

## Inherits

Inherits the nearest parent `AGENTS.md`; this file adds only the local delta for this directory.

## Local constraints

- Keep public module exports intentional, propagate errors with context, and run formatting plus relevant cargo tests after changes.
- Preserve the strongly typed `OrderId`, `Price`, `Quantity`, `Timestamp`, side, order, and error boundaries; do not collapse these domain distinctions into primitive values.
- The crate remains safe Rust: do not introduce `unsafe`; prefer checked arithmetic where overflow would invalidate market state.

## Update this file when

- Ownership, interfaces, generated/runtime boundaries, or directory-specific verification requirements change.
