# UI

*Maturity: working · Stability: unstable — UI is the project's headline visual; expected iteration as it evolves toward GitHub-screenshot-grade*

## Scope / Purpose

`src/ui/` is the real-time observability dashboard, built with Ratatui + crossterm. It is the project's headline visual: the first thing an engineer sees when running `cargo run`. The dashboard renders a depth-of-book ladder, trade tape, latency percentiles, throughput, mid-price chart, and engine summary, all updating live as the simulator drives the matching engine.

Theme respect is structural: every color is `Color::Reset` or one of the ANSI 16. Hardcoded RGB anywhere would break user-curated terminal themes (Solarized-light, Catppuccin, accessibility palettes); the rule is enforced by convention and is the load-bearing single instance described in `notes/conventions.md`.

## Boundaries / Ownership

- **Owns:** `App` (the in-process state container), `Action` (parsed key intents), `EngineState` enum, the render dispatch (`panes::render`), every per-pane render function, the theme palette (`theme.rs`), terminal setup/teardown, the main event loop.
- **Does not own:** the matching engine (`book::OrderBook` is *owned by* App, not implemented by it), the simulator, the metrics registry. UI calls into them; it does not implement them.
- **Imported by:** `main.rs` (calls `nyquestro::ui::run(seed)`) and nothing else inside the crate.

## Current Implemented Reality

### Three modules

| Module | What it owns |
|--------|-------------|
| `ui/theme.rs` | Color palette (`BID = Green`, `ASK = Red`, `ACCENT = Yellow`, `CHROME = DarkGray`, `GOOD/WARN/ALERT = LightGreen/Yellow/Red`), `Style` helpers (`neutral`, `dim`, `bold`, `fg`, `fg_bold`, `fg_dim`), and the `block_bar(ratio, width)` sub-cell-precision bar renderer using `▏▎▍▌▋▊▉█` |
| `ui/app.rs` | `App` state, `Action` enum, `EngineState`, `TapePrint` ring entry, key-mapping, terminal setup/restore, the run loop (33ms render tick + 50ms sim tick + 10ms input poll) |
| `ui/panes.rs` | `render` (top-level), `render_top_status`, `render_keybinds`, `render_body`, `render_depth_of_book`, `render_trade_tape`, `render_latency`, `render_mid_chart`, `render_throughput`, `render_engine_summary`, plus formatters (`format_price_cents`, `format_latency_ns`, `format_clock_ns`, `format_duration`) |

### App state

```rust
pub struct App {
    pub book:           OrderBook,
    pub sim:            MarketSimulator,
    pub metrics:        MetricsRegistry,
    pub state:          EngineState,        // Running | Paused
    pub speed:          f64,                // simulator dt multiplier, [0.1, 50.0]
    pub tape:           VecDeque<TapePrint>,// bounded ring, ≤ 200, newest at front
    pub mid_history:    VecDeque<u64>,      // bounded ring, ≤ 600, oldest at front
    pub total_orders:   u64,                // lifetime counters (separate from windowed)
    pub total_fills:    u64,
    pub total_cancels:  u64,
    pub total_rejects:  u64,
    pub resting_ids:    Vec<OrderID>,       // refreshed every 250ms for cancel hints
    last_resting_refresh: Instant,
    started_at:         Instant,
}
```

### Layout

The dashboard renders into a vertical layout: 1-row top status, body (60% / 40% vertical split), 1-row keybind footer.

```
┌──────────── status row (1 line) ────────────────────────┐
│ NYQUESTRO · matching engine RUNNING │ mid $100.05 │ ×1.0 │ uptime 02:13 │
├─────────────────────────────────────────────────────────┤
│ Depth of Book (38%) │ Trade Tape (34%) │ Latency (28%)  │
├─────────────────────────────────────────────────────────┤
│ Mid Price (50%) │ Throughput (25%) │ Engine (25%)       │
├─────────────────────────────────────────────────────────┤
│ q quit  p pause  r reset  +/- speed · safe rust · ratatui │
└─────────────────────────────────────────────────────────┘
```

### Depth-of-book pane

The visual anchor — top-left, tallest pane. Asks rendered worst-first (top-down toward the spread row), bids rendered best-first below the spread row. Per row: indented price, displayed quantity, and a horizontal bar whose width is proportional to the level's quantity divided by the maximum across all visible levels. Bars use the `theme::block_bar` 1/8-cell precision renderer; bid bars are `Green`, ask bars are `Red`. The spread row sits in the middle and shows `── spread $X.XX / NN.N bp ──` in `DarkGray`.

### Trade tape pane

Newest-first, capped to whatever fits the pane height. Per row: `HH:MM:SS.MMM`, price, aggressor glyph (`▲` Buy `Green`, `▼` Sell `Red`), quantity. The tape ring drops oldest entries when full (200 max).

### Latency pane

Fixed 7-row card: header (`submit · N samples`), then p50 / p95 / p99 / p999 / max / mean rows. P99 and max are highlighted in `Yellow`. A sparkline of mid-price deltas fills the bottom row when there are at least two mid samples.

### Mid-price chart

Ratatui `Chart` with Braille markers, `GraphType::Line`, `ACCENT`-colored. Auto-bounded to `[lo - pad, hi + pad]` where `pad = 0.1 * (hi - lo)` clamped to ≥ 0.05.

### Throughput pane

Four rows (orders / fills / cancels / rejects), each showing `last_1s/s · last_10s/10s` from the `WindowSnapshot`.

### Engine summary pane

Two-section card: "book" (best bid `Green`, best ask `Red`, resting count) and "lifetime" (submitted / filled / cancelled / rejected totals tracked on `App` directly).

### Run loop

```
loop {
    if last_render.elapsed() >= 33ms:  draw → last_render = now
    if last_sim.elapsed() >= 50ms:     app.step(elapsed.as_secs_f64()) → last_sim = now
    if event::poll(10ms) && let Event::Key(k) = event::read() && app.handle_action(map_key(k)) { return }
}
```

The 33/50/10 cadence means render and sim ticks happen in the order: poll → maybe sim → maybe render → poll → … and the run loop is responsive to keypresses within ~10ms regardless of frame state.

### Keybinds

| Key | Action |
|-----|--------|
| `q`/`Q`/`Esc` | quit |
| `p`/`P`/space | toggle pause |
| `r`/`R` | reset (new market, reseed every per-symbol simulator, clear tapes + histories + metrics) |
| `+`/`=` | speed × 1.5 (cap 50.0) |
| `-`/`_` | speed / 1.5 (cap 0.1) |
| `Tab`/`s`/`S` | cycle the focused symbol (AAPL → MSFT → NVDA → AAPL …) |

## Key Interfaces / Data Flow

```rust
pub fn run(seed: u64) -> Result<(), Box<dyn std::error::Error>>;

pub struct App { ... }
impl App {
    pub fn new(seed: u64) -> Self;
    pub fn step(&mut self, dt_secs: f64);
    pub fn handle_action(&mut self, Action) -> bool;  // true = quit
    pub fn uptime(&self) -> Duration;
}

pub enum Action { Quit, TogglePause, Reset, SpeedUp, SpeedDown, None }
pub enum EngineState { Running, Paused }
```

`panes::render(frame, app)` is the only entry point into rendering — `App` is the read-only borrow target.

## Implemented Outputs / Artifacts

- The four module files.
- The headless mode (`--no-tui`) bypasses everything in `ui/` and runs a 10-second silent simulation that prints a 6-line summary.
- No automated tests — the UI surface is rendered, not unit-testable in the conventional sense. The `cargo build --release` pass and the headless smoke run are the validation gates.

## Known Issues / Active Risks

- **No automated UI tests.** Visual correctness is verified manually. The Ratatui ecosystem has a `TestBackend` we could hook in for snapshot-style assertions; not yet wired.
- **Panic during render does not restore the terminal.** `setup_terminal` calls `enable_raw_mode + EnterAlternateScreen`; `restore_terminal` reverses them. If the run loop panics, raw mode persists. A `std::panic::set_hook` that calls `restore_terminal` would close this gap.
- **Resting-id cache is stale by up to 250ms.** A cancel hint may target a recently-filled id and fail; this is silently swallowed (cancel returns Err, we don't increment cancels). For the dashboard's purposes, fine; for a deterministic test of cancel rate, not.
- **`render_latency`'s sparkline may render past the pane's bottom border** if the latency pane is shorter than 8 rows. We compute `used` as `lines.len() as u16` but don't clamp against `inner.height`. Acceptable in practice because the latency pane gets ≥ 60% of the top row's height, which on most terminals exceeds 8 rows; pathological tiny terminals could clip.
- **The dashboard runs on the same thread as the engine.** A long simulator step (e.g. someone cranks `speed` to 50× and the sim emits hundreds of actions) will stall the render path until it returns. The 50ms sim cadence makes this rare but not impossible.

### Downstream impact

The dashboard is the project's headline. Any visual bug — mis-rendered border, incorrect color, drifting bar widths — degrades the project's perceived quality more than a backend bug a reader will never see. Treating UI correctness as load-bearing is correct.

## Partial / In Progress

None.

## Planned / Missing / Likely Changes

- **`std::panic::set_hook` to restore terminal on panic.** Small fix, big resilience win.
- **`TestBackend` snapshot tests** for the render functions, even if just per-pane renders against fixed `App` state.
- **Configurable layout** so the user can hide panes that don't fit on narrow terminals. Currently the percentage-split fights small terminals.
- **Latency-pane visual upgrade** — replace the percentile-list-plus-sparkline with a Bookmap-style heatmap if the design iteration calls for it.
- **History tape paging** — `j`/`k` to scroll the tape ring back through history beyond the visible cap.
- **Order-flow imbalance indicator** in the depth-of-book pane (sum of bid quantity vs sum of ask quantity, displayed as a single signed gauge).
- **Color toggle** for users with red/green colour-blindness — `b` could swap to a bid=blue / ask=orange palette.

## Durable Notes / Discarded Approaches

- **Single-threaded run loop, not async.** Considered Tokio + `tokio::select!` between sim and render ticks (the design brief in `notes/dashboard-design.md` mentions this as a Ratatui ecosystem pattern). Rejected for the MVP: the engine and sim are fast enough that one thread is sufficient, and adding async machinery for a terminal app that runs at 30fps is overkill.
- **`Color::Reset` for backgrounds, ANSI-16 for foregrounds.** Hard rule. Tested across Catppuccin, Solarized-light, Tokyo Night themes — every one of them maps the ANSI 16 correctly because terminal themes are ANSI palettes by definition. RGB would have looked great on the dev laptop and broken on every other.
- **Block-element bars instead of `BarChart`.** Ratatui's `BarChart` widget can't do mirrored bid/ask layout cleanly. Pre-rendering each row as a `Paragraph` with block-element strings gives full control: bid bars grow rightward from the price column, ask bars grow rightward likewise (not mirrored — keeping the visual scan consistent), and color is the only side indicator.
- **Reset key reseeds with the constant `0xC0FFEE`, not a random seed.** Determinism: pressing `r` twice produces the same playback. If a user wanted true random reset, we'd need a config flag.
- **`render_top_status` is a `Paragraph` with `Span`s, not multiple widgets.** Tried multi-widget for cleaner section borders; it left an unwanted blank cell at every join. One paragraph with carefully-typed spans renders cleanly.

## Obsolete / No Longer Relevant

None — this module was authored fresh in the current rewrite.
