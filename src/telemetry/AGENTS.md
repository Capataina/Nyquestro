# Scope

`Nyquestro/src/telemetry`

## Purpose

Structured telemetry events and durable writer boundary.

## Owns

- Direct files: `events.rs`, `mod.rs`, `writer.rs`.

## Inherits

Inherits the nearest parent `AGENTS.md`; this file adds only the local delta for this directory.

## Local constraints

- Telemetry must stay off correctness-critical matching semantics and must not silently drop or reorder declared events.
- Telemetry is local-only: do not add network export, truncate the run log on startup, and keep opt-out behaviour explicit.

## Update this file when

- Ownership, interfaces, generated/runtime boundaries, or directory-specific verification requirements change.
