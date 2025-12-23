# Plans Index

This directory contains the authoritative project memory: implementation plans, architectural decisions, and long-term roadmap. Each plan file tracks a feature, refactor, bugfix, or milestone.

**Entry Point:** Start here to understand what's planned, in progress, or complete.

---

## Plan Files

| File                                                                                                   | Purpose                                                                                                                                                   | Status   | Last Updated |
| ------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- | ------------ |
| [001_immutable_event_frames_and_error_expansion.md](001_immutable_event_frames_and_error_expansion.md) | Implement zero-allocation event frames (FillEvent, QuoteEvent, OrderEvent) and extend error enum with matching-related variants and classification system | complete | 2025         |
| [002_deterministic_matcher_loop.md](002_deterministic_matcher_loop.md)                                 | Implement deterministic matcher loop with price-time priority, single-instrument order book, and proper FillEvent/QuoteEvent emission                     | planned  | 2025         |

---

## Status Legend

- **planned** – Design complete, ready to implement
- **in_progress** – Actively being implemented
- **blocked** – Waiting on dependencies or decisions
- **complete** – Implementation finished, all exit criteria met

---

## Other Documents

- [architecture.md](architecture.md) – High-level architecture map and repository structure overview

---

## How to Use This Index

1. **Starting new work?** Check existing plans to avoid duplication. If work is already planned, update that plan's status.
2. **Completing work?** Update the plan file's status to `complete` and tick off all checklist items.
3. **Creating new plan?** Follow the naming convention: `NNN_<short_topic>.md` where NNN is a zero-padded sequence number.
4. **Updating status?** Update both the plan file's status header and this index table.

---

**Maintenance Note:** Keep this index in sync with plan file status changes. Update the "Last Updated" date when plan files are modified.
