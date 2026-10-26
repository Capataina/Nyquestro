# Plan: Binary Wire Protocol & Multi-Process Gateway

## Header

- **Status:** Planned (not started; large — multi-week)
- **Scope:** Move Nyquestro from "a program" to "a system." Split the engine and its clients into separate processes that talk over a compact binary protocol: a fixed-width little-endian order-entry frame (the README's "binary UDP gateway") and a market-data feed (depth snapshots + incremental updates) that external clients — including the strategy agent — subscribe to. This is the README's **📡 Order Gateway and Protocols** section.
- **Why this matters:** Right now everything runs in one process and talks via Rust function calls. A real exchange is a *network service*: clients connect from other processes/machines over a wire protocol. Building that demonstrates protocol design, binary serialisation, framing, and network-boundary thinking — a different and complementary skill to the matching core, and one that turns the project into something an engineer recognises as exchange-shaped.
- **Exit rule:** complete when (a) an order-entry frame format is defined, versioned, length-prefixed, and checksummed, (b) a gateway process accepts frames, validates them, and forwards to the engine, (c) a market-data publisher emits snapshots + incremental updates, (d) an external client process (the strategy agent) connects, reconstructs the book from the feed, and submits orders over the wire, (e) a fuzz harness proves the parser never panics on malformed input.

## Why binary, not JSON or FIX (the decision to be able to defend)

- **JSON:** text parsing on every message — scan for quotes and numeric boundaries, allocate, branch-mispredict. Hundreds of nanoseconds per message on the hot path. Fine for a REST API, wrong for order entry.
- **FIX (tag=value text):** the industry's lingua franca and worth supporting *eventually* for realism, but still text — verbose and parse-heavy.
- **Fixed-width binary:** an order message is a `#[repr(C)]` struct that maps to validated memory with a single bounds-checked copy. No parsing, no allocation, no scanning. Versioned + length-prefixed for forward compatibility; checksummed for corruption detection on a lossy UDP path. This is what OUCH (Nasdaq order entry) and SBE (Simple Binary Encoding, the FIX Trading Community's binary format) do.

Being able to articulate this trade-off — and to say "I used a binary frame because text parsing is hundreds of ns I can't spend per order" — is the hiring signal, as much as the code.

## Implementation Structure

### Modules / files affected

- `src/protocol/` (new):
  - `frame.rs` — `#[repr(C)]` frame structs: header (magic, version, length, msg-type, checksum) + bodies (`NewOrder`, `Cancel`, `Modify`, `Ack`, `Fill`, `Reject`).
  - `codec.rs` — encode/decode with explicit little-endian, length-prefix framing, checksum (CRC32 or a simple Fletcher), and a strict validator that rejects malformed frames *before* they touch engine state.
  - `md.rs` — market-data messages: `Snapshot` (top-N depth per side) + `Delta` (incremental level update) + sequence numbers for gap detection.
- `src/bin/gateway.rs` (new) — the gateway process: bind UDP (order entry) + a feed transport (UDP multicast or TCP fan-out for MD), validate, forward to the engine, ack.
- `src/bin/engine.rs` (new) — the engine process (or the gateway hosts the engine in-proc to start, then split).
- `src/agent/` — the strategy agent (from `medium-wins.md`) becomes a *real external client*: it connects over the protocol instead of calling functions, closing the loop the README describes.
- `Cargo.toml` — multiple `[[bin]]` targets; a CRC crate or hand-rolled checksum.
- `tests/fuzz_protocol.rs` + a `cargo-fuzz` target — malformed-frame fuzzing.

### Frame format (sketch)

```
header (fixed):
  magic:    u16   // 0x4E51 "NQ" — reject anything else immediately
  version:  u8    // protocol version; bump on breaking change
  msg_type: u8    // NewOrder / Cancel / Modify / Ack / Fill / Reject / ...
  length:   u16   // total frame length, validated against bytes received
  seq:      u32   // per-session sequence number (gap detection)
  checksum: u32   // over the body; mismatch → reject, never parse further

NewOrder body (fixed-width, little-endian):
  client_order_id: u64
  symbol:          u64   // reuse the ASCII-packed Symbol(u64) from types.rs — already wire-friendly!
  side:            u8
  order_type:      u8
  price_cents:     u64
  quantity:        u32
  ... padded to alignment
```

**A free win already in the codebase:** `Symbol(u64)` is an 8-byte big-endian ASCII pack (`systems/types.md`) — it is *already* a fixed-width wire value. `Px(u64)` is integer cents, `Qty(u32)` is a plain integer. The domain primitives were designed allocation-free and Copy, so they serialise to fixed-width fields almost for free. Note this in the design — it's evidence the type layer was built with this in mind.

## Algorithm / System Sections

### A) Framing & validation (the security-relevant part)

- **Length-prefix first**, validate `length` against bytes actually received before reading any body field — the classic parser-safety rule.
- **Magic + version + checksum** checked before the body is interpreted; any failure → a `Reject` frame, never a panic, never partial engine mutation.
- **Bounds-checked decode:** every field read is checked; in safe Rust this is the default, which is exactly why safe Rust is a *feature* for a network-facing parser (the README's "parser correctness under adversarial input is a security property" claim).

### B) Market data (snapshot + incremental)

- New subscriber gets a **snapshot** (top-N levels per side) + a sequence number, then **deltas** (one level changed) from there.
- **Gap detection:** if a subscriber sees a sequence jump, it requests a fresh snapshot. This is exactly how real depth feeds (ITCH included) work, and it's what the strategy agent's "order-book reconstruction" consumes.

### C) Transport

- Order entry over **UDP** (low latency, the README's choice) with the checksum + seq handling the unreliability. Start with localhost UDP; the protocol doesn't care.
- Market data over **UDP multicast** (one publish, many subscribers — how real feeds fan out) or TCP fan-out to start for simplicity. Note the trade-off.
- A `--in-proc` fallback that keeps everything in one process via channels, so the engine stays runnable without the network during development and tests.

## Integration Points

- **`types.rs` primitives** serialise to wire fields nearly 1:1 (see the free-win note) — the protocol layer is mostly framing, not field encoding.
- **The strategy agent** (`medium-wins.md` item 2) is the first real client: it goes from in-proc function calls to a network participant connecting "through the same binary protocol any other client would use" — which is the exact framing the README uses, finally made true.
- **The concurrent engine** (`lock-free-engine.md`) is the natural backend: gateway threads feed the ingress ring; the MD publisher is a ring consumer. These two large plans compose into the full picture (gateway → ring → shard → MD ring → subscribers).
- **Fuzzing** ties to `extensive-testing-framework.md` — the parser fuzz target lives in both.

## Debugging / Verification

- **Round-trip:** encode a `NewOrder`, decode it, assert field-equality.
- **Malformed input never panics:** `cargo-fuzz` the decoder; thousands of random byte strings → always `Ok(frame)` or `Err(reject)`, never a panic or UB.
- **Checksum catches corruption:** flip a body bit → decode rejects.
- **Length lies are caught:** a frame claiming `length` longer/shorter than bytes received → rejected before body interpretation.
- **Two-process smoke:** gateway process + client process on localhost; client submits, receives an Ack and subsequent Fills; MD subscriber reconstructs a book matching the engine's.
- **Gap recovery:** drop a delta on the MD path → subscriber detects the seq gap and re-snapshots.

## Completion Criteria

- [ ] `src/protocol/` with `frame.rs`, `codec.rs`, `md.rs`.
- [ ] Frame format: magic + version + length-prefix + seq + checksum, all validated before body interpretation.
- [ ] `NewOrder` / `Cancel` / `Modify` / `Ack` / `Fill` / `Reject` encode + decode round-trip.
- [ ] Gateway process accepts UDP order entry, validates, forwards to engine, acks.
- [ ] Market-data publisher emits snapshot + incremental deltas with sequence numbers.
- [ ] The strategy agent connects as an external process over the protocol and trades.
- [ ] `cargo-fuzz` target proves the decoder never panics on malformed input.
- [ ] `--in-proc` fallback keeps the engine runnable without the network.
- [ ] `systems/protocol.md` documents the frame format and the binary-vs-text rationale.
- [ ] This file is archived once all the above are checked.
