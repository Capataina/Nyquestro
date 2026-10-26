# Scope

`Nyquestro/src/ui`

## Purpose

Terminal dashboard state, panes, and visual theme.

## Owns

- Direct files: `app.rs`, `mod.rs`, `panes.rs`, `theme.rs`.

## Inherits

Inherits the nearest parent `AGENTS.md`; this file adds only the local delta for this directory.

## Local constraints

- Preserve ANSI-16 compatibility, bounded action dispatch per frame, and terminal restoration on every exit or panic path.

## Update this file when

- Ownership, interfaces, generated/runtime boundaries, or directory-specific verification requirements change.
