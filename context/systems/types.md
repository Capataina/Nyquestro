# Types

*Maturity: comprehensive · Stability: stable*

## Scope / Purpose

`src/types.rs` defines the seven domain primitives every other module is built on: `OrderID`, `Symbol`, `Side`, `Px`, `Qty`, `Ts`, and the `Status` state machine. Wrappers exist so the compiler treats them as mutually incompatible — passing a price where a quantity is expected, or routing an `AAPL` order to the `MSFT` book, is a type error rather than a runtime bug.

## Boundaries / Ownership

- **Owns:** the six primitive types, their constructors, accessor methods, `Display` impls, and the `Status::can_transition_to` transition rule.
- **Does not own:** the `Order` entity (lives in `order.rs`), event frames (lives in `events/`), error variants (lives in `errors.rs` — but `NyquestroError::InvalidPrice { cents }` is referenced from `Px::from_cents`).
- **Imported by:** every other module in the crate. There is no module that does not depend on `types`.

## Current Implemented Reality

- **`OrderID(u64)`** — non-zero. `OrderID::new(0)` returns `Err(NyquestroError::InvalidOrderId)`.
- **`Symbol(u64)`** — 8-byte ASCII-packed instrument identifier. Big-endian packing means lexicographic `Ord` on the underlying `u64` matches lexicographic order on the original string. Two constructors:
  - `Symbol::from_const(s)` — `const fn`; truncates to 8 bytes silently. Use with literal strings: `Symbol::from_const("AAPL")`.
  - `<&str>::parse::<Symbol>()` via the standard `FromStr` trait — rejects empty input and strings longer than 8 bytes.
- **`Side`** — two-variant enum (`Buy`, `Sell`) with `opposite()`, `is_buy()`, `is_sell()` and a `Display` impl that writes `BUY` / `SELL` literally.
- **`Px(u64)`** — price in integer cents. Float arithmetic never used in price comparison. Two constructors:
  - `from_cents(u64)` — rejects 0.
  - `from_dollars(f64)` — rejects NaN, infinity, ≤ 0, and any value that rounds to zero cents. Uses `round()` (banker-style nearest) rather than truncation, fixing a historical sub-cent bug where `$10.999` produced 1099 cents.
- **`Qty(u32)`** — quantity in whole units. Zero is representable (it's the value of `remaining` after a full fill); rejection of zero is enforced at the construction boundaries that need it (`Order::new`, every event constructor). `checked_sub`/`checked_add` are exposed; `saturating_*` is *not*, because saturating arithmetic was the original mechanism behind the silent over-fill bug in the prior codebase.
- **`Ts(u64)`** — nanoseconds since UNIX epoch. `Ts::now()` falls back to `Ts(0)` if the clock is before 1970 rather than panicking. Convenience converters `nanos`/`micros`/`millis` and a `to_utc_datetime` for human-readable display.
- **`Status`** — one-way state machine: `Open → PartiallyFilled → FullyFilled` and `Open|PartiallyFilled → Cancelled`. `can_transition_to` is a `const fn` returning `bool`; the matrix is exhaustive in the source.

## Key Interfaces / Data Flow

```rust
OrderID::new(u64) -> NyquestroResult<OrderID>
Px::from_cents(u64) -> NyquestroResult<Px>
Px::from_dollars(f64) -> NyquestroResult<Px>
Qty::new(u32) -> Qty                            // infallible — zero is allowed
Qty::checked_sub(self, Qty) -> Option<Qty>      // None on underflow
Qty::checked_add(self, Qty) -> Option<Qty>      // None on overflow
Ts::now() -> Ts                                 // never panics
Ts::from_nanos(u64) -> Ts
Status::can_transition_to(self, Status) -> bool // const, exhaustive
Status::is_active(self) -> bool                 // Open | PartiallyFilled
Status::is_terminal(self) -> bool               // FullyFilled | Cancelled
```

State machine, depicted explicitly because the constructor fan-out hides it:

```
                 fill (partial)            fill (final)
   Open ─────────────────────────► PartiallyFilled ───────► FullyFilled
     │                                     │
     │ cancel                              │ cancel
     ▼                                     ▼
   Cancelled  ◄──────────── (terminal — no outgoing edges)
```

## Implemented Outputs / Artifacts

- `pub` types and methods listed above, all `Copy`-friendly (no `String`, no heap).
- `Display` impl for every type that meaningfully renders to a human (`OrderID` → `#42`, `Px` → `$10.05`, `Side` → `BUY`/`SELL`, `Status` → `OPEN`/`PARTIAL`/`FILLED`/`CANCELLED`).
- 12 inline unit tests (`mod tests`) plus 6 integration tests in `tests/types_test.rs` covering every constructor branch and the transition matrix.

## Known Issues / Active Risks

- `Px::from_dollars` accepts inputs up to `u64::MAX as f64 / 100` cents (~ $1.84 × 10¹⁷). Practically out of range for equities, but no upper soft-bound is enforced; a typo of a price as `1e18` would currently be accepted. Low priority because the engine never originates prices from untrusted floats; see `simulator` which uses cents directly.
- `Ts::now()` returns `Ts(0)` on clock-before-epoch rather than propagating an error. This is a deliberate choice to keep the type's `now()` infallible (nothing in the engine handles a clock-error today). Worth revisiting if/when an external time-source contract is added.
- `Qty(u32)` caps at ~4.29 × 10⁹. Sufficient for equities/crypto/futures notional sizes; would need to widen for currency (where retail quantities can exceed `u32`). Tracked here, no current consumer requires it.

## Partial / In Progress

None. The type set has been deliberately sized to match the matching engine's current scope (limit orders only). Order types beyond `Limit` (Market, IOC, FOK) are not yet represented because the engine does not implement them either.

## Planned / Missing / Likely Changes

- **`OrderType` enum** when Phase B+ work introduces market / IOC / FOK semantics.
- **`Symbol`/`Instrument`** primitive when the engine becomes multi-instrument; today the book is single-instrument and `Symbol` is implicit.
- **`AccountID`** alongside `OrderID` if/when self-match prevention extends to account-level (currently uses `OrderID` for self-match detection).

## Durable Notes / Discarded Approaches

- **Saturating arithmetic in `Qty` was deliberately removed.** The prior codebase used `Qty::saturating_sub` inside `Order::fill`, which is what produced the silent over-fill bug (a fill of 100 against remaining 3 succeeded as a "FullyFilled" of 3). The replacement is `checked_sub` returning `Option<Qty>`, with the caller forced to handle `None` explicitly. The `over_fill_returns_error_and_preserves_state` test pins this.
- **`Px` is `u64` cents, not `i64` or float.** Considered `i64` for headroom on signed arithmetic (spreads can be negative in pathological cases) but rejected because price comparison needs to be total-order without `Ord` corner cases on negative values; the spread calculation that genuinely needs sign handling lives in the UI layer where `i64` is constructed from two `Px::cents()` values explicitly.
- **`Ts` is `u64` nanoseconds, not a `chrono::DateTime`.** `chrono` is only used for human-readable display at the boundary (`Ts::to_utc_datetime`). The internal representation must be `Copy` and 8 bytes; `chrono::DateTime` is neither.
- **Idiomatic accessor names (`.id()`, `.price()`, `.quantity()`)** instead of `get_*` were chosen during the rewrite — see `notes/conventions.md`.

## Obsolete / No Longer Relevant

- `Qty::can_subtract` (returned `bool`) — removed; consumers use `Option::is_some` on `checked_sub` instead.
- `Qty::saturating_sub` — removed (see Durable Notes above).
- `Ts::is_before` / `Ts::is_after` taking raw `u64` — removed; the `PartialOrd`/`Ord` impl on `Ts` covers comparison directly.
- `Px::new_from_dollars` / `Px::new_from_cents` (the `new_` prefix) — renamed to `from_dollars` / `from_cents` to match Rust convention (`From`-style constructors).
