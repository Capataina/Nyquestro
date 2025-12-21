# Nyquestro Architecture Map

**Last Updated:** 2025

This document provides a top-down semantic overview of the Nyquestro repository structure. It serves as the primary navigation aid for understanding what exists and where.

---

## Repository Root

The project root contains standard Rust project files and configuration:

- **`Cargo.toml`** – Package manifest defining the crate name (`nyquestro`), edition (2024), and dependencies (`chrono`, `thiserror`)
- **`README.md`** – Project description, roadmap, and feature checklist
- **`.gitignore`** – Standard Rust ignore patterns
- **`Cargo.lock`** – Dependency lock file (generated)

---

## Source Code (`src/`)

The `src/` directory contains the core library implementation. The module structure is defined in `lib.rs`, which serves as the library entry point.

### Core Modules

- **`lib.rs`** – Library root module. Declares all public modules (`errors`, `order`, `price_level`, `types`) and re-exports error types for convenience.

- **`types.rs`** – **Core abstraction layer.** Defines the type-safe primitives that form the foundation of the order book:

  - `OrderID` – Non-zero u64 wrapper for order identification
  - `Side` – Enum for Buy/Sell with `opposite()` helper
  - `Px` – Price in cents (u64), with dollar/cents conversion
  - `Qty` – Quantity (u32) with saturating subtraction
  - `Ts` – Nanosecond-precision timestamp wrapper
  - `Status` – Order lifecycle state (Open, PartiallyFilled, FullyFilled, Cancelled)

  All types include validation logic and conversion methods. These are the building blocks used throughout the system.

- **`errors.rs`** – **Error boundary.** Defines `NyquestroError` enum using `thiserror` for structured error handling:

  - `InvalidOrderID`
  - `InvalidPrice { value: f64 }`
  - `InvalidQuantity`
  - `OrderNotFound { id: u64 }`

  Also defines `NyquestroResult<T>` as a convenience alias.

- **`order.rs`** – **Order entity.** Represents a single limit order with:

  - Immutable fields: `order_id`, `side`, `price`, `quantity`, `timestamp`
  - Mutable state: `remaining_quantity`, `status`
  - Methods: `new()`, `fill()`, `update_status()`, and getters

  Currently uses standard Rust types (not lock-free). This will need to evolve for the lock-free matching engine.

- **`price_level.rs`** – **Price level container.** Groups orders at the same price:

  - Maintains a `Vec<Order>` (FIFO semantics implied, not enforced)
  - Tracks `total_quantity` across all orders at the level
  - Validates that added orders match the level's price

  **Note:** Currently uses `Vec<Order>`, which is not lock-free. This is a placeholder that must be replaced with atomic price buckets and intrusive FIFO lists per the roadmap.

- **`main.rs`** – **Example/demo entry point.** Contains demonstration code showing:

  - Timestamp creation and duration calculations
  - Order creation and filling
  - Status transitions

  This is for development/testing purposes, not production code.

---

## Tests (`tests/`)

Integration tests live in the `tests/` directory, separate from unit tests (which would be in `src/` alongside the code).

- **`types_test.rs`** – Comprehensive test suite for all type primitives:

  - OrderID validation (zero rejection)
  - Price creation and conversion (dollars ↔ cents)
  - Quantity arithmetic and underflow protection
  - Side opposite logic
  - Timestamp creation, comparison, and unit conversions

  These tests validate the correctness of the foundational type system.

---

## Build Artifacts (`target/`)

Standard Rust build output directory. Contains compiled artifacts, dependencies, and incremental compilation state. Not part of the source architecture.

---

## Current Architecture Notes

### What Exists

- Type-safe primitives with validation
- Basic order and price level data structures
- Error handling infrastructure
- Test coverage for types

### What's Missing (Per Roadmap)

- Lock-free data structures (atomic price buckets, epoch GC)
- Matching engine logic
- Event frame system (immutable quotes/fills)
- Ingress gateways (FIX, UDP, WebSocket)
- Market data publishing
- Concurrency primitives (ring bus, NUMA awareness)
- Risk and compliance systems
- Observability (tracing, metrics)

### Architectural Boundaries

The current codebase is in a **foundational phase**. The modules are structured to support future expansion, but the implementation is not yet lock-free or production-ready. Key boundaries to establish:

1. **Core matching engine** – Will need to be lock-free and separate from ingress/egress
2. **Event system** – Immutable event frames for market data publishing
3. **Gateway layer** – Protocol adapters (FIX, UDP, WebSocket) that feed into the engine
4. **Risk layer** – Pre-trade checks and post-trade monitoring

---

## Navigation Guide

- **Starting a new feature?** Check `README.md` for roadmap context, then consult `plans/` for existing plans.
- **Understanding types?** Start with `src/types.rs` – it's the foundation.
- **Adding error handling?** See `src/errors.rs` for the error taxonomy.
- **Working on matching logic?** Review `src/order.rs` and `src/price_level.rs`, but note they need lock-free redesign.
- **Writing tests?** Follow the pattern in `tests/types_test.rs` for integration tests.

---

**Maintenance Note:** Update this document when new subsystems are added or when responsibility boundaries change. Do not update for minor internal refactors.
