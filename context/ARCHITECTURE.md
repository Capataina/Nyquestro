# Nyquestro Architecture

**Last Updated:** 2026-02-02

This document provides a top-down overview of repository structure, subsystem responsibilities, dependency direction, and current execution flow. Update it only when subsystem boundaries or responsibility splits change.

## Repository Overview

- Nyquestro is a Rust library crate (`nyquestro`) with a demo binary (`src/main.rs`) and integration tests (`tests/`).
- The current implementation covers core primitives, basic order modelling, event frame types, and an error taxonomy.
- There is no implemented matching engine loop or order book subsystem wired into the crate.
- The root `README.md` defines intent; divergences between intent and implementation are tracked as explicit debt rather than silently redefining scope.

## Directory Tree (Major Components)

- `Cargo.toml` and `Cargo.lock` define the crate and its dependencies.
- `src/` contains the library modules and the demo binary.
- `src/events/` contains immutable event frame types.
- `src/matching_engine/` exists but is not currently part of the compiled crate and contains an empty placeholder file.
- `tests/` contains integration tests for primitives and events.
- `target/` contains build artefacts and is not part of the architecture.

## Subsystems and Responsibilities

- `src/types.rs` defines shared type primitives (`OrderID`, `Side`, `Px`, `Qty`, `Ts`, `Status`) used across the crate.
- `src/errors.rs` defines `NyquestroError`, `NyquestroResult<T>`, and `ErrorSeverity` classification.
- `src/events/*` defines `FillEvent`, `QuoteEvent`, and `OrderEvent` as in-memory frames.
- `src/order.rs` defines the `Order` entity and its internal state transitions.
- `src/price_level.rs` defines a `PriceLevel` container using `Vec<Order>` (placeholder, not lock-free).
- `src/main.rs` is a demo entry point that exercises timestamps, order creation, and filling.
- `tests/*` validates primitives, event constructors, and error severity classification.

## Dependency Direction and Data Flow

- `types` is a foundational dependency for `order`, `price_level`, and `events`.
- `errors` is a foundational dependency for `order`, `price_level`, and `events`.
- `events` depends on `types` and `errors`, and does not currently depend on `order` or `price_level`.
- `price_level` depends on `order`, plus `types` and `errors`.
- The demo binary and integration tests depend on the library modules.

## Core Pipelines and Execution Flow

- Demo path: construct primitives → create `Order` → call `Order::fill()` → print debug output.
- Test path: construct primitives and events → assert invariants and classification behaviour.
- Missing path: order ingress → book placement → matching → event emission.

## Current Implementation Status (Architectural)

- Event frame types exist and are exercised by integration tests.
- The error type and severity classification exist and are exercised by unit tests and integration tests.
- The order book / matcher subsystem is not implemented, and `src/matching_engine/` is not wired into the crate.
- Data structures are not lock-free and include cloning and allocation patterns that are placeholders for later design work.

## Intent vs Reality Divergences (Tracked Debt)

- The root `README.md` states that there are no external crates yet, but `Cargo.toml` currently depends on `chrono` and `thiserror`.
- This divergence should be resolved by removing the dependencies or revisiting intent outside this documentation workflow (the `README.md` is immutable here).

