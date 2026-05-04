# Dashboard Design

## 1. Current Understanding

The TUI dashboard is the project's headline visual — the first thing an engineer sees when they `cargo run`. Its design follows a brief assembled from a focused research pass (May 2026) over best-in-class terminal dashboards (btop, gitui, lazygit, k9s, helix, atuin) and HFT trading-UI conventions (Bookmap, Bloomberg-style time-and-sales, depth-of-market ladders).

The current implementation in `src/ui/` faithfully follows the brief: six panes in three rows, ANSI-16 + `Color::Reset` only, block-element horizontal bars for the depth-of-book ladder, percentile card + sparkline for latency, decoupled render-tick (33ms) from sim-tick (50ms) cadence.

## 2. Rationale

### Layout

Six panes in three rows, with the depth-of-book ladder as the visual anchor (top-left, tallest). The hierarchy is:

1. **Depth of book** — top-left, ~38% width × 60% height. The visual anchor; what most viewers' eyes land on first.
2. **Trade tape** — top-middle, ~34% width × 60% height. The second-loudest element, conveying tempo.
3. **Latency** — top-right, ~28% width × 60% height. Headline numerical credibility (p50/p95/p99/p999/max).
4. **Mid-price chart** — bottom-left, ~50% width × 40% height. Visual continuity over time.
5. **Throughput** — bottom-middle, ~25% width × 40% height. Rolling rate per channel.
6. **Engine summary** — bottom-right, ~25% width × 40% height. Numeric snapshot of book state + lifetime totals.

Plus a single-line top status row (project + state + mid + speed + uptime) and a single-line keybind footer (q quit / p pause / r reset / +/- speed).

### Color discipline

Every color is `Color::Reset` or one of the ANSI 16. RGB is forbidden anywhere in the crate (`notes/conventions.md` codifies this).

| Semantic | Color | Use |
|----------|-------|-----|
| Bid / buy aggressor | `Green` | Bookmap / Quantower / CQG industry standard |
| Ask / sell aggressor | `Red` | Same. |
| Mid / neutral / chrome between | `Reset` (terminal default fg) | Lets the user's theme show through |
| Top-of-book / accent / number-of-interest | `Yellow` | Sparingly — one accent at a time |
| Borders | `DarkGray` | Almost-invisible chrome (gitui/lazygit pattern) |
| Status "RUNNING" / good | `LightGreen` | One accent per state |
| Status "PAUSED" / warn | `LightYellow` | |
| Alert (latency over threshold etc.) | `LightRed` | Reserved for actual attention-grabbers |

`bold` and `dim` modifiers are used for hierarchy *before* reaching for color. This is what keeps dense screens calm — the gitui / lazygit / helix lesson.

### Patterns deliberately copied

- **btop's rounded-corner borders** (`BorderType::Rounded`) for a softer look.
- **gitui/lazygit's pane-title-bold + dim chrome** — the title is the only emphasised text on a pane border.
- **Bookmap-style horizontal bid/ask depth bars** — block-element strings rendered as `Paragraph` rows, not `BarChart`. `BarChart` can't do the proportional-width-per-row layout cleanly.
- **k9s persistent action bar** — keybinds always visible at the bottom; never hidden in a help screen.
- **Helix-style left/center/right top status** — engine state | mid + symbol | tick rate / clock.
- **Bookmap aggressor coloring on the trade tape** — each print colored by aggressor side.
- **Latency percentile card + sparkline pairing** — the canonical HFT diagnostic shape: p50/p95/p99/p999/max as labelled rows, plus a 60-second sparkline below for tempo.
- **bottom/gping live-chart-plus-numeric-card pattern** — every chart sits next to a numeric card. Eyes need both.
- **tickrs-style decoupled tick cadence** — render tick separate from data tick. Render at 30fps, simulate at 20Hz.

### Patterns deliberately avoided

- **No 24-bit RGB anywhere.** Themes that look gorgeous in dev become unreadable on someone's Solarized-light fork.
- **No emoji or nerd-font glyphs.** Tempting (📈 🟢 ▲) but they break in tmux, WSL, basic xterm, screenshare, and recordings. The Geometric Shapes triangle (`▲ ▼`) is fine — it's font-ubiquitous.
- **No full-screen redraw on every event.** Decoupled tick cadence + Ratatui's diff-and-flush are what keep the dashboard from tearing.
- **No tape-on-the-right-with-newest-at-the-bottom.** Newest at the *top* is the universal time-and-sales convention; old prints scroll off downward.

## 3. What Was Tried

- **Multi-widget top status row.** Tried splitting the top row into three widgets via `Layout::Direction::Horizontal`; produced an unwanted blank cell at every join. Replaced with one `Paragraph` carrying typed `Span`s.
- **`Color::Rgb`-based palette, briefly.** Looked great in iTerm2 with the dev's custom palette; was unreadable in stock macOS Terminal.app. Reverted to ANSI 16. Documented in `notes/conventions.md`.
- **Bid bars mirrored from the right edge, ask bars from the left.** Tried it for the depth-of-book ladder; the horizontal scan was harder than left-anchored bars in both colors. Kept left-anchored.
- **`tokio::select!` async run loop.** Considered for the run loop's poll/tick orchestration. Rejected as overkill for a 30fps terminal app — single-threaded blocking event loop is simpler and the engine isn't doing IO.

## 4. Guiding Principles

- **The dashboard is the headline visual.** A reader's first impression of the project lands here. Visual bugs degrade perceived quality more than backend bugs the reader will never see.
- **Terminal theme respect is structural.** Hardcoded RGB anywhere will break the UI on a meaningful fraction of users' terminals.
- **Density without noise.** Every glyph must serve. Dim and bold do hierarchy work *before* color is reached for.
- **Decoupled cadence.** Render and data ticks are independent so neither can starve the other under load.
- **Real-time enough.** 30fps is the cadence the eye reads as "live." 60fps gains nothing in a TUI; the LCD's pixels can't render the difference at terminal scale.

## 5. Trade-offs and Constraints

- **Single-threaded loop** — fights long simulator steps under high speed. Acceptable today; will need a thread split when the matching engine becomes multi-threaded.
- **No `TestBackend` snapshot tests** — visual regressions are caught manually for now. Worth adding.
- **No panic-handler that restores the terminal** — a panic during render leaves the user in raw mode. `std::panic::set_hook` would close this.

## 6. Related Systems and Notes

- `systems/ui.md` — the implementation reality.
- `notes/conventions.md` — the ANSI-16-only rule lives there as a project-wide convention.
- `notes/safe-rust-philosophy.md` — overarching design philosophy this dashboard sits inside.
