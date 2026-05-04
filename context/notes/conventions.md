# Conventions

## 1. Current Understanding

This file captures the unmarked conventions used across the crate that are *not* enforced by `clippy` or `rustfmt`. These are the patterns a newcomer would not guess from a single file but that, if violated, produce subtle bugs or visual regressions.

## 2. Rationale

Each convention below was chosen during the Phase A rewrite to fix a specific class of bug present in the prior codebase. Capturing them here means future sessions can preserve the discipline without re-deriving the reasoning.

### Idiomatic Rust accessors — no `get_` prefix

- `Order::id()`, `.side()`, `.price()`, `.quantity()`, `.remaining()`, `.timestamp()`, `.status()`.
- Event types use `pub` fields (`Copy`-friendly), not getter methods.
- The two-method `order_id()` / `timestamp()` accessors on `OrderEvent` exist because the enum has variant-uniform reads; they are the exception that proves the rule.

**Why:** The prior codebase had `get_status(self) -> Status` consuming the order; the demo binary cloned the order six times in a single println. Idiomatic Rust takes `&self` and never consumes.

### `checked_*` arithmetic over `saturating_*` for `Qty`

- `Qty::checked_sub`, `Qty::checked_add` — return `Option<Qty>`.
- `Qty::saturating_sub` is *not* exposed.

**Why:** The original silent over-fill bug (G1) was caused by `Order::fill` calling `Qty::saturating_sub`, which silently clamped a fill of 100 against remaining 3 to a "FullyFilled" of 3. The `checked_*` API forces the caller to handle `None` explicitly. The `tests/order_test.rs::over_fill_returns_error_and_preserves_state` test pins this.

### Validated constructors return `NyquestroResult<T>`

- Every fallible constructor: `OrderID::new`, `Px::from_cents`, `Px::from_dollars`, `Order::new`, `FillEvent::new`, `QuoteEvent::live`, `OrderEvent::placed`, `OrderEvent::filled`, `PriceLevel::push_back`.
- Infallible constructors: `Qty::new` (zero is allowed at the primitive level), `OrderEvent::cancelled` / `OrderEvent::rejected` (the variants exist *because* something failed upstream), `QuoteEvent::cleared`.

**Why:** The prior codebase mixed `Result<T, &'static str>` with `NyquestroResult<T>`. Standardising on one error type removes a class of "which error type does this constructor return?" friction and lets `?` propagate uniformly.

### Caller-supplied timestamps for determinism

- `Order::new(id, side, price, quantity, ts)` — `ts` is required.
- The matching loop in `OrderBook::submit_limit` reuses each resting order's existing timestamp for fills.
- `Order::new_now` exists as a convenience that calls `Ts::now()`, but is for tests and demos — not for matching.

**Why:** Two runs of the same input sequence must produce byte-identical event vectors. Calling `Ts::now()` inside the matching loop would break this. The `tests/matching_test.rs::run_twice_identical_sequence_identical_output` test pins the contract.

### Events are `Copy`, allocation-free

- `FillEvent`, `QuoteEvent`, `OrderEvent` — all `Copy + Debug + Clone + PartialEq + Eq + Hash`.
- `OrderRejectionReason` is an enum, not a `String` — preserving `Copy` on the parent.
- `events_are_copy` integration test statically asserts via `fn assert_copy<T: Copy>(_: T)`.

**Why:** The planned event fan-out and replay loops cannot be efficient if events allocate. `String` reasons would have been ergonomic but block `Copy`.

### Single-source severity classification

- `NyquestroError::severity(&self) -> ErrorSeverity` is a method on the enum, not a free function and not stored as a field.
- `is_recoverable` / `is_fatal` are shortcuts over `severity`.

**Why:** The prior taxonomy had generic `RecoverableError` / `FatalError` variants *and* a free `severity(error)` function — two classification mechanisms that could disagree. Method form is the canonical Rust idiom and is exhaustive over variants by construction.

### ANSI-16-only color in the UI

- `src/ui/theme.rs` exposes only `Color::Green`, `Color::Red`, `Color::Yellow`, `Color::DarkGray`, `Color::LightGreen`, `Color::LightYellow`, `Color::LightRed`. Backgrounds default to `Color::Reset`.
- Hardcoded RGB (`Color::Rgb(...)`) anywhere in the crate is forbidden.

**Why:** Terminal themes (Catppuccin, Solarized-light, Tokyo Night, accessibility palettes) are ANSI palettes by definition. They remap the ANSI 16 to coherent palettes; they cannot remap RGB. One RGB color in the theme breaks the UI on a meaningful fraction of users' terminals — and this dashboard is the project's headline visual, so theme respect is structural.

This is a single coherent rule whose violation breaks cross-system: violation in `theme.rs` propagates through every pane that consumes the helper. It is the canonical structural-importance convention in the crate (one rule × multiple consumers, not three independent occurrences).

## 3. What Was Tried

The "free function severity" form was tried in the prior codebase and failed in the way described above — it allowed two competing classification mechanisms to coexist.

The "saturating arithmetic on `Qty`" was the prior codebase's choice and produced the silent over-fill bug. The replacement was `checked_*`.

## 4. Guiding Principles

- **Force the caller to handle the failure mode.** `Option`/`Result` over silent clamping. The compiler and the test suite are then the enforcement mechanism.
- **One canonical entry point per cross-cutting concern.** Severity classification, error type, theme palette — each has one obvious home, and the rest of the crate consumes it.
- **Determinism is structural, not a property to "achieve later".** Time, randomness, and event order are all engineered to be reproducible from a fixed input — supporting both replay testing and the dashboard's "press `r` to start over" experience.
- **Visual chrome respects the user's terminal.** Color decisions must work on every theme the user might pick. RGB is forbidden everywhere it might leak.
