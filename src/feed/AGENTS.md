# Scope

`Nyquestro/src/feed`

## Purpose

External market-feed adapters and the bridge into internal events.

## Owns

- Direct files: `bridge.rs`, `coinbase.rs`, `mod.rs`.

## Inherits

Inherits the nearest parent `AGENTS.md`; this file adds only the local delta for this directory.

## Local constraints

- Translate external feed details at this boundary; internal book/event modules must not depend on exchange-specific wire shapes.
- Coinbase level-2 updates are bridged into virtual internal orders; sequence-gap handling belongs at this boundary rather than in the book.

## Update this file when

- Ownership, interfaces, generated/runtime boundaries, or directory-specific verification requirements change.
