# Plan: Dashboard Infographics

## Header

- **Status:** Planned (implementation in flight as of 2026-05-04)
- **Scope:** Replace numeric tables with visual gauges across every dashboard pane that's currently number-heavy. Engine pane → gauge stack. Throughput pane → per-row sparklines. Trade tape → size bars. Latency pane → distribution-shape bar. DOB → bid/ask pressure bar. Top status → health dot system. Numbers stay (precision matters) but the visual shape draws the eye to "is this normal or not."
- **Why this matters:** The dashboard is the project's headline visual. After the live-feed screenshot, the most common feedback was "it's a lot of text and numbers." Eight rows of `key: value` in the Engine pane communicates poorly compared to gauges. The HFT-cultural anchor (Bloomberg Terminal, Bookmap, Quantower, btop) is dense visual gauges with numbers as confirmation, not numbers as the primary medium.
- **Exit rule:** complete when (a) every pane below has its visual upgrade landed, (b) the dashboard renders cleanly at 200×55, 140×40, and 120×30 canvas sizes, (c) no pane has degraded compared to the current iteration.

## Implementation Structure

### Modules / files affected

- `src/ui/theme.rs` — new gauge-rendering helpers: `marker_axis`, `stacked_bar`, `health_dot`, `distribution_bar`. Keep the existing `block_bar` and ANSI-16-only colour discipline.
- `src/ui/panes.rs` — per-pane render functions get visual upgrades.
- `src/ui/app.rs` — `App` gets two new fields: `recent_rates: PerMetric<VecDeque<u64>>` (for throughput sparklines) + `frame_health: FrameHealth` (for the health dot system, tracking last_slow_frame, last_reject_burst, etc.).
- New helper module `src/ui/widgets.rs` (optional split if `panes.rs` gets too large).

### Visual conventions (additions to existing palette)

All new visualisations stick to ANSI 16 + `Color::Reset` per `notes/conventions.md`. New glyphs added to the vocabulary:

| Glyph | Use |
|-------|-----|
| `▰▱` | Twin progress bars (filled / empty cells). |
| `▌▐` | Half-cell quantity markers for stacked bars. |
| `┃` | Vertical separator on horizontal axes. |
| `╋` | Marker on a horizontal axis (microprice position). |
| `┝┥` | Range markers on a distribution bar. |
| `●` | Health dot. (Already in use; standardise.) |

### Pane-by-pane plan

#### 1. Engine pane — gauge stack (highest-impact)

Current: 9 lines of `key: value` text.

Target shape:

```
microstructure
  best bid     $80121.76 × 294486
  best ask     $80121.77 × 23316
  
  spread       $0.01 / 0.0bp   ▏░░░░░░░░░░░░░  tight
  
  bid                                     ask
  $80121.76 ━━━━━━━━━━━╋━━━━━━━━━━━━ $80121.77
                        ↑ microprice $80121.77
  
  OFI top-10   +0.622   [████████│······]   bid-leaning
  
  depth top-10
    bid  ████████████████████  796k
    ask  █████                 185k
    ratio 4.3:1 bid heavy
  
  levels
    bid  ▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰░░░░░  189
    ask  ▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰░░░░  207

lifetime
  submitted  6552
  filled       38
  cancelled  6118
  rejected      0
```

Each metric has a numeric value (precision) plus a visual gauge (shape). The microprice axis is the most novel — visualises "which side does the engine think the trade-able mid leans toward" instantly.

Implementation:
- `theme::spread_gauge(spread_cents, reference_cents) -> String` → bar where 1 cell = 0.5bp, max 20 cells = 10bp. Colours: green if ≤ 1bp, yellow if ≤ 5bp, red if > 5bp.
- `theme::microprice_axis(bid_cents, ask_cents, microprice_cents, width) -> Vec<Span>` → horizontal axis with bid on left, ask on right, microprice marker positioned at the appropriate cell.
- `theme::stacked_ratio_bar(left, right, width) -> (String, String)` → pair of horizontal bars sized to absolute values, capped at width, with a "ratio" annotation.
- `theme::twin_progress_bar(value, max, width) -> String` using `▰▱` blocks.

#### 2. Throughput pane — sparklines + ratio gauge

Current: 5 metric rows × 4 window columns of integers.

Target: same metric rows, but each row gets a sparkline of recent 1-second rates plus a directional indicator:

```
metric    1s    10s   1min   5min   trend                      Δ vs 10s
orders    118   2610  13038  13038  ▁▂▃▅▇▆▅▆▇█▇▅▃▂▁▂▃▄▅▆▇█    ↑ +12%
fills       0     8     62     62  ▁▁▁▁▁▂▁▁▁▂▁▁▁▁▁▁▁▁▁▁▁▁    →
cancels   107  2550  11916  11916  ▂▃▄▅▆▇█▇▆▅▄▃▂▁▂▃▄▅▆▇█▇    ↑ +8%
rejects     0     0      0      0  ─────────────────────       
quotes     15   225   1008   1008  ▁▁▂▃▄▅▆▇█▇▆▅▄▃▂▁▁▂▃▄▅▆    ↑ +3%

order/fill 10s   326.25 ▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▱▱  16:1 cancel-heavy
windows: 1s · 10s · 1min · 5min
```

Implementation:
- `App::recent_rates: HashMap<&str, VecDeque<u64>>` with one ring per metric, capacity 60 (= 60 seconds of 1Hz samples).
- Push current 1s value once per second (gated by an `Instant`).
- Render sparkline using existing `Sparkline` widget or hand-rolled `▁▂▃▅▆▇█`.
- Trend arrow computed from `current_1s vs avg(last_30s)` ratio.

#### 3. Trade tape — size bars

Current: per row, time + price + glyph + quantity (text).

Target: same plus a 1/8-cell-precision size bar at the right of each row, sized as `qty / max_qty_in_visible_window`:

```
00:00:25.547  $99.95  ▼   12  ▎
00:00:25.480  $99.93  ▼  137  █████
00:00:25.450  $99.92  ▲  240  ████████
00:00:25.420  $99.92  ▼   18  ▎
00:00:25.390  $99.93  ▲    4  ▏
```

Implementation: in `render_trade_tape`, find `max_qty` from the visible slice, then `theme::block_bar(qty / max_qty, bar_width)` per row using existing helper.

#### 4. Latency pane — distribution shape bar per op

Current: 3 ops × 5 percentile columns of microsecond values.

Target: per op, a horizontal distribution bar with marks at p50/p99/p999/p9999/max positions, plus the budget colour dot:

```
op       samples     distribution (0..100µs log)                        p50   p99   p999  p9999  max     budget
submit    13038      ┝──────╫─╫──────╫─────────╫──────╫───┥             1.6µs 9.1µs 18µs  39µs   42µs    ●
match        30      ┝──────╫─────────╫──╫──╫──╫──┥                     6.1µs 41µs  41µs  41µs   41µs    ●
cancel    11916      ┝────────────╫──────────╫────╫──╫──╫──┥            67µs  158µs 188µs 226µs  253µs   ●

mid Δ over recent samples  ▂▃▅▇▆▅▄▃▂▃▄▅▆█▇▅▃▂▁
```

Each `╫` is a percentile marker. The visual shape immediately communicates tail-heaviness:
- Tightly clustered marks = no tail (great).
- Marks spreading rightward = long tail (look at why).

Implementation:
- `theme::distribution_bar(p50, p99, p999, p9999, max, axis_max_ns, width) -> Vec<Span>` produces the bar with markers.
- Log-scale axis so 1µs and 100µs both appear meaningfully (linear would compress everything to the left).

#### 5. DOB pane — bid/ask pressure bar at bottom

Current: depth ladder ends at the spread row; below the bid section the pane is empty.

Target: append a single-line stacked horizontal bar showing total bid quantity vs total ask quantity:

```
... [existing depth ladder] ...
 $99.80      80   ▏

bid pressure  ████████████░░░░░░  81% / 19% ask  (4.3:1 bid heavy)
```

Implementation: `theme::stacked_ratio_bar` reused from the engine pane work.

#### 6. Top status — combined health dot

Current: `NYQUESTRO · matching engine RUNNING | [BTC-USD ETH-USD SOL-USD] | mid $80121.99 | × 1.00 | uptime 00:43`

Target: prepend a health dot:

```
● NYQUESTRO · matching engine RUNNING | [BTC-USD ETH-USD SOL-USD] | mid $80121.99 | uptime 00:43 | live · subscribed
```

The dot's colour reflects combined health:
- Green: no slow frames in last 10s, latency p99 within budget, feed connected (live mode).
- Yellow: 1+ slow frames in last 10s OR p99 over 10µs OR feed reconnecting.
- Red: sustained issues — multiple slow frames, p99 over 50µs, or feed disconnected.

Speed multiplier is hidden in live mode (it doesn't apply).

Implementation:
- New field `App::health: AppHealth` updated each frame.
- `AppHealth::dot_colour(&self) -> Color` returns the right ANSI 16.
- `theme::health_dot(colour) -> Span` for consistent rendering.

#### 7. Symbol selector — per-symbol health dots

Current: `[BTC-USD ETH-USD SOL-USD]` with the active symbol bold-yellow.

Target: per-symbol dot reflecting that symbol's recent slow-frame or rejection-burst state:

```
● ●BTC-USD● ●ETH-USD ●SOL-USD
  ↑
  active symbol still bold-yellow
```

Each dot is one of:
- Green: this symbol's book and recent activity look healthy.
- Yellow: this symbol had a rejection burst or partial issues.
- Red: this symbol had a serious issue (snapshot mismatch, sustained slow frames during this symbol's flow).

Implementation: per-symbol `SymbolHealth` field on `SymbolState`, updated on every dispatch.

## Integration Points

- All new theme helpers live in `src/ui/theme.rs` (or `src/ui/widgets.rs` if it grows). Maintain the ANSI-16-only discipline.
- Health system depends on `AppHealth` + `SymbolHealth` data structures. Plumb them through the existing render path; no new threading.
- Telemetry plan integrates here: each visual upgrade also gets a corresponding telemetry event (e.g. `frame_slow` triggers a `health_yellow` transition).
- Build on top of the polish fixes (bridge timestamps, mid sampling) — those should land first because they're prerequisites for the mid-price chart and tape timestamps to be useful.

## Debugging / Verification

- **Visual smoke test:** run `cargo run --release -- --live coinbase` and confirm each pane renders with its new visual elements. Manually inspected.
- **Health dot transitions:** artificially induce a slow frame (insert `thread::sleep`) and confirm the top-status dot transitions from green to yellow.
- **Width adaptation:** resize the terminal to 100 cols and confirm the panes still render coherently (no overflow, no border drift).
- **Sparkline correctness:** synthetic mode at 50× should produce a noticeably busier sparkline than 1× — verify both visually.
- **Distribution bar accuracy:** run for 60s, manually verify that the distribution-bar markers correspond to the printed p50/p99/p9999/max numbers.

## Completion Criteria

- [ ] All 7 pane upgrades land and render correctly at 200×55.
- [ ] Health dot system in top status responds to slow-frame and feed events.
- [ ] Symbol selector shows per-symbol health dots.
- [ ] Speed multiplier is hidden in live mode.
- [ ] No clippy regressions; build is clean.
- [ ] `notes/conventions.md` updated to document the new glyphs (`▰▱`, `┃`, `╋`, `┝┥`, etc.).
- [ ] `systems/ui.md` updated to describe each pane's new visual language.
- [ ] Headline screenshot retaken for the README.
- [ ] This file is archived once all of the above are checked.
