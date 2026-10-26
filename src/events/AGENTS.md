# Scope

`Nyquestro/src/events`

## Purpose

Quote, fill, and lifecycle event types exchanged between matching subsystems.

## Owns

- Direct files: `fill.rs`, `lifecycle.rs`, `mod.rs`, `quote.rs`.

## Inherits

Inherits the nearest parent `AGENTS.md`; this file adds only the local delta for this directory.

## Local constraints

- Event structs are cross-subsystem contracts; update producers, consumers, nearest scoped `AGENTS.md` files, and tests together.

## Update this file when

- Ownership, interfaces, generated/runtime boundaries, or directory-specific verification requirements change.
