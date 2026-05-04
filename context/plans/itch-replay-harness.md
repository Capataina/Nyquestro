# Plan: ITCH / LOBSTER Replay Harness

## Header

- **Status:** Planned (not started)
- **Scope:** Drive the matching engine from recorded NASDAQ-equity flow and validate its output against the recorded book reconstruction. Phased:
  - **Phase 1 (LOBSTER CSV).** Parse LOBSTER's free `message` + `orderbook` CSVs, replay the messages through the engine, compare our reconstructed top-N book against LOBSTER's `orderbook` rows. Faster to implement; same data underneath; "engine matches NASDAQ behaviour" claim is unchanged.
  - **Phase 2 (raw ITCH 5.0 binary).** Parse NASDAQ TotalView-ITCH 5.0 binary frames directly. Source: Databento's free-clip access or whatever LOBSTER pre-derived sample they expose. Cross-validate against existing parsers in [shawfdong/itch5parser](https://github.com/shawfdong/itch5parser).
- **Why this matters:** ITCH is the lingua franca of US equity microstructure. A candidate who has touched real ITCH interviews stronger than one who has only seen JSON WebSockets. Phase 1 ships fastest; Phase 2 carries the prestige bullet ("hand-rolled binary ITCH parser, byte-cross-validated").
- **Exit rule:** Phase 1 complete when (a) LOBSTER message CSVs parse cleanly, (b) our reconstructed book matches LOBSTER's `orderbook` CSV row-for-row for at least one full sample day on at least one ticker, (c) hidden executions (LOBSTER event type 5) are documented and explicitly skipped. Phase 2 complete when (a) our raw ITCH 5.0 parser handles every message type listed below, (b) parser output cross-validates against `shawfdong/itch5parser` byte-for-byte on a shared input, (c) raw ITCH replay produces the same engine outputs as the equivalent LOBSTER CSV replay.

## Implementation Structure

### Modules / files affected (Phase 1 — LOBSTER CSV)

- `src/replay/` (new):
  - `mod.rs`
  - `lobster.rs` — LOBSTER CSV reader (`message` + `orderbook` formats)
  - `runner.rs` — drives the matching engine from a stream of `ReplayAction`s
  - `validator.rs` — per-event book-state comparator (our top-N vs LOBSTER's `orderbook` row)
- `tests/lobster_replay_test.rs` (new)
- `data/README.md` (new) — documents which sample day(s) we're using and the fetch script
- `scripts/fetch-lobster-sample.sh` (new) — fetches a free LOBSTER sample (no committed binary data in the repo; per-file license check first)

### Modules / files affected (Phase 2 — raw ITCH 5.0 binary)

- `src/itch/` (new):
  - `mod.rs`
  - `frame.rs` — the ITCH 5.0 message types listed below
  - `parser.rs` — streaming binary parser over `&[u8]` or `Read`
  - `replay.rs` — adapter that emits the same `ReplayAction`s as the LOBSTER reader so `replay::runner` is reused
- `tests/itch_test.rs` (new) — round-trip parse + serialize, cross-validate against [shawfdong/itch5parser](https://github.com/shawfdong/itch5parser) on a shared input file

### LOBSTER CSV format (Phase 1)

The free-tier samples (per [data.lobsterdata.com](https://data.lobsterdata.com/info/DataSamples.php)) ship two CSVs per ticker per day:

**`message_<level>.csv` columns:**

| Column | Type | Meaning |
|--------|------|---------|
| `time` | f64 | Seconds since midnight (with fractional seconds) |
| `event_type` | u8 | 1=submit, 2=partial-cancel, 3=full-cancel, 4=visible-execute, 5=hidden-execute, 7=halt |
| `order_id` | u64 | NASDAQ-assigned order reference |
| `size` | u32 | Order quantity |
| `price` | u64 | Price in $0.0001 units (10000 = $1.00) |
| `direction` | i8 | `1`=Buy, `-1`=Sell — invert into our `Side` enum at the boundary |

**`orderbook_<level>.csv` columns:** flat row of `ask_px_1, ask_qty_1, bid_px_1, bid_qty_1, ask_px_2, ask_qty_2, bid_px_2, bid_qty_2, …` for the level-N depth at the time of each message.

The validator's job: for each row in `message`, replay through our engine, then compare our `book.top_n_asks(N) + book.top_n_bids(N)` against the corresponding row in `orderbook`. Mismatch ⇒ either (a) parser bug, (b) engine bug, or (c) hidden-execution (event_type 5) — distinguish by checking event_type first.

### ITCH 5.0 message types in scope (Phase 2)

Out of the ~20 message types in the spec, the matching engine needs ~10:

| Type byte | Name | Why we need it |
|-----------|------|----------------|
| `S` | System Event | Market open/close markers; replay phase boundaries |
| `R` | Stock Directory | Maps stock locator → symbol string |
| `H` | Stock Trading Action | Halt / trading-state changes |
| `A` | Add Order — No MPID | Add a resting order to the book |
| `F` | Add Order with MPID | Same as A but with market-participant attribution |
| `E` | Order Executed | Resting order partially or fully filled |
| `C` | Order Executed with Price | Same as E with explicit print price (price improvement) |
| `X` | Order Cancel | Reduce remaining quantity |
| `D` | Order Delete | Remove order entirely |
| `U` | Order Replace | Atomic cancel + add (different order id) |
| `P` | Trade (Non-Cross) | Trades that did not pass through the visible book |
| `Q` | Cross Trade | Opening / closing cross |

Out of scope for the MVP: `B` (broken trade), `I` (NOII / opening cross imbalance), `N` (RPII), `J` (LULD), `V`/`W` (MWCB), `K` (regulatory).

### Responsibility boundaries

- `replay::runner` is the shared core: takes a stream of `ReplayAction { Submit(Order), Cancel(OrderID), Halt }` and drives the engine + collects validation diffs. Source-format-agnostic.
- `replay::lobster` decodes LOBSTER CSV rows into `ReplayAction`s.
- `replay::validator` compares the engine's reconstructed top-N book against LOBSTER's `orderbook` rows row-by-row.
- (Phase 2) `itch::frame` owns the binary layout. No engine awareness.
- (Phase 2) `itch::parser` owns the streaming parse over arbitrary `Read`. No replay logic.
- (Phase 2) `itch::replay` decodes ITCH frames into the same `ReplayAction`s `replay::runner` already consumes. The runner is reused unchanged.

### Function inventory

- `Frame::parse(&[u8]) -> NyquestroResult<(Frame, &[u8])>` — pure, returns frame + tail.
- `parse_stream<R: Read>(reader: R) -> impl Iterator<Item = NyquestroResult<Frame>>`.
- `Replay::new(market: &mut Market) -> Self`.
- `Replay::dispatch(&mut self, frame: Frame) -> NyquestroResult<()>` — applies the frame to the market state.
- `Replay::run<R: Read>(&mut self, reader: R) -> NyquestroResult<ReplayStats>`.

### Wiring

- The dashboard's `App` gets a `--replay <file>` CLI flag. When set, `App` swaps the synthetic `MarketSimulator` for a `Replay` reader. The render loop is unchanged.
- The integration test `tests/itch_test.rs::replay_recorded_day_produces_expected_executions` reads a checked-in *small* sample (≤ 100 messages), replays it, and asserts the emitted `FillEvent` set matches a recorded golden file.

## Algorithm / System Sections

### A) Binary parser

ITCH 5.0 frames are length-prefixed (2-byte big-endian length, then 1-byte message type, then type-specific payload). Common fields: 6-byte tracking number, 2-byte stock locator, 6-byte timestamp (nanoseconds since midnight, packed).

**Implementation playbook:**
- [ ] Define `enum Frame { System, Directory, AddOrder, ... }` with explicit fields per variant.
- [ ] Implement `Frame::parse(&[u8])` using `byteorder` or hand-rolled `u16::from_be_bytes`.
- [ ] Reject unknown message-type bytes with `NyquestroError::ItchUnknownMessage(byte)`.
- [ ] Reject short-buffer cases with `NyquestroError::ItchTruncated`.
- [ ] Round-trip test: parse a hand-crafted byte buffer, serialize back, byte-equal.

### B) Symbol resolution

ITCH uses an integer "stock locator"; the `Stock Directory` message maps locator → symbol string. We need a small `HashMap<u16, Symbol>` populated from `R` messages encountered during replay.

**Playbook:**
- [ ] On `R` frame: insert into the locator table.
- [ ] On any frame referencing a locator: resolve via lookup; emit `NyquestroError::ItchUnknownLocator` if missing.

### C) Order-id space

ITCH order references are 8-byte. Our `OrderID` is `u64`. Direct map.

### D) Replay determinism

ITCH timestamps are nanoseconds since midnight. We pass them through to the engine as `Ts`. The engine never reads the wall clock during matching, so the replay is byte-deterministic given the input file.

**Playbook:**
- [ ] On `S` Market Open: start a fresh `Market`.
- [ ] On `A`/`F`: construct `Order::new(...)` with the ITCH-supplied timestamp.
- [ ] On `E`/`C`: locate the resting order by id, apply a fill via the engine's existing public API, compare the engine's emitted `FillEvent` against the ITCH-supplied print.
- [ ] On `X`/`D`: call the engine's `cancel(id, ts)`.
- [ ] On `U`: cancel old, add new.
- [ ] On `P`/`Q`: out-of-book trade — log but don't apply to our book.

### E) Validation against a sample day

**Playbook:**
- [ ] Pick one symbol on one sample day.
- [ ] Replay all `A/E/C/X/D/U` messages for that symbol.
- [ ] Compare the engine's emitted fill stream against the ITCH-recorded `E`/`C` messages.
- [ ] Mismatches indicate either (a) an engine bug, (b) a parser bug, or (c) hidden-order activity that ITCH does not show in the public stream — distinguish by inspecting which message types preceded the discrepancy.

## Integration Points

- `Market::submit_limit` / `Market::cancel` — already-existing public surface. Replay calls them.
- `App::run` — gains a `--replay <file>` branch.
- The dashboard renders identically; the data source is the only thing that changes.

## Debugging / Verification

- Property: parsing a valid ITCH stream and serialising back produces byte-equal output.
- Property: replaying the same file twice into two fresh `Market` instances produces byte-identical event vectors.
- Discrepancy investigation: when our engine emits a fill that ITCH does not, the most likely cause is a self-match policy mismatch. NASDAQ's policy may differ from ours. Document any such divergence.

## Completion Criteria

### Phase 1 (LOBSTER CSV)

- [ ] `src/replay/{lobster,runner,validator}.rs` exist and compile.
- [ ] `cargo test` passes the LOBSTER round-trip test (parse a sample row, run through engine, byte-equal output).
- [ ] `cargo run -- --replay-lobster data/AAPL_2012-06-21_message.csv` runs without panic for a full sample day.
- [ ] Validator reports zero discrepancies (excluding hidden-execution event_type=5) on at least one full sample day for at least one ticker.
- [ ] `notes/free-data-sources.md` updated with the specific LOBSTER sample(s) used.
- [ ] `systems/book.md` updated to mention LOBSTER replay as a validation path.

### Phase 2 (raw ITCH 5.0 binary)

- [ ] `src/itch/{frame,parser,replay}.rs` exist and compile.
- [ ] Parser round-trip test passes (parse → serialize → byte-equal).
- [ ] `cargo run -- --replay-itch data/sample.itch5` runs to EOF without panic on a Databento-derived clip.
- [ ] Cross-validation: parser output byte-equals `shawfdong/itch5parser` output on the same input file (limit message-type coverage to what we both implement).
- [ ] `replay::runner` produces the same engine outputs whether driven from the LOBSTER CSV path or the equivalent raw ITCH path on the same trading session.
- [ ] This file is archived once both phases' criteria are checked.
