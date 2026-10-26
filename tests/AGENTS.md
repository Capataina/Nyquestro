# Scope

`Nyquestro/tests`

## Purpose

Black-box behavioural tests for public matching and domain semantics.

## Owns

- Direct files: `events_test.rs`, `matching_test.rs`, `order_test.rs`, `price_level_test.rs`, `types_test.rs`.

## Inherits

Inherits the nearest parent `AGENTS.md`; this file adds only the local delta for this directory.

## Local constraints

- Assert public behaviour and invariants; keep fixtures deterministic and add regression coverage for every corrected edge case.
- Keep public module exports intentional, propagate errors with context, and run formatting plus relevant cargo tests after changes.

## Update this file when

- Ownership, interfaces, generated/runtime boundaries, or directory-specific verification requirements change.
