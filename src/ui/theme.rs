//! Color and style helpers for the dashboard.
//!
//! All colors fall into one of three categories:
//!
//! 1. **Reset** — defer to the user's terminal theme (default fg/bg).
//! 2. **Semantic ANSI 16** — `Green` for buy/bid, `Red` for sell/ask,
//!    `Yellow` for emphasis, `DarkGray` for chrome.
//! 3. **None** — typography only (`bold`, `dim`).
//!
//! Hardcoded RGB is forbidden; it breaks on Solarized-light, on macOS
//! Terminal.app's restricted palette, and on accessible high-contrast
//! themes.

use ratatui::style::{Color, Modifier, Style};

pub const BID: Color = Color::Green;
pub const ASK: Color = Color::Red;
pub const ACCENT: Color = Color::Yellow;
pub const CHROME: Color = Color::DarkGray;
pub const GOOD: Color = Color::LightGreen;
pub const WARN: Color = Color::LightYellow;
pub const ALERT: Color = Color::LightRed;

#[inline]
pub fn neutral() -> Style {
    Style::default()
}

#[inline]
pub fn dim() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

#[inline]
pub fn bold() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

#[inline]
pub fn fg(c: Color) -> Style {
    Style::default().fg(c)
}

#[inline]
pub fn fg_bold(c: Color) -> Style {
    Style::default().fg(c).add_modifier(Modifier::BOLD)
}

#[inline]
pub fn fg_dim(c: Color) -> Style {
    Style::default().fg(c).add_modifier(Modifier::DIM)
}

/// Render a horizontal block-element bar for a 0..=1 ratio. Uses 1/8th
/// sub-cell precision via `▏▎▍▌▋▊▉█`.
pub fn block_bar(ratio: f64, width: usize) -> String {
    let r = ratio.clamp(0.0, 1.0);
    let cells = r * width as f64;
    let full = cells.floor() as usize;
    let remainder = cells - full as f64;
    // Index 0 = empty, 8 = full block.
    const PARTS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
    let part_idx = (remainder * 8.0).round() as usize;
    let mut s = String::with_capacity(width);
    for _ in 0..full.min(width) {
        s.push('█');
    }
    if full < width {
        s.push(PARTS[part_idx.min(8)]);
        for _ in (full + 1)..width {
            s.push(' ');
        }
    }
    s
}

/// Render a sparkline using the standard `▁▂▃▅▆▇█` block-element scale.
/// Values are linearly mapped against the maximum sample in `data`. An
/// empty slice or all-zero slice produces a flat-bottomed bar.
pub fn sparkline(data: &[u64], width: usize) -> String {
    if data.is_empty() || width == 0 {
        return " ".repeat(width);
    }
    const LEVELS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    // Take the trailing `width` samples — the most recent ones — so the
    // sparkline behaves like a right-aligned scrolling chart.
    let start = data.len().saturating_sub(width);
    let recent = &data[start..];
    let max = recent.iter().copied().max().unwrap_or(0).max(1);
    let mut s = String::with_capacity(width);
    for v in recent {
        let idx = ((*v as f64 / max as f64) * 7.0).round().clamp(0.0, 7.0) as usize;
        s.push(LEVELS[idx]);
    }
    // Left-pad if the data is shorter than the requested width.
    if s.chars().count() < width {
        let pad = width - s.chars().count();
        let mut padded = String::with_capacity(width);
        for _ in 0..pad {
            padded.push(' ');
        }
        padded.push_str(&s);
        return padded;
    }
    s
}

/// Render a stacked horizontal bar showing the proportions of `left` vs
/// `right`. Returns the bar string. The left portion fills up to the
/// `left / (left + right)` fraction of `width`; the right fills the
/// remainder. Both portions use full blocks `█`; the renderer caller
/// applies left/right colours separately by splitting the string at
/// `left_width` cells.
pub fn pressure_bar(left: u64, right: u64, width: usize) -> (String, String, f64) {
    if width == 0 {
        return (String::new(), String::new(), 0.0);
    }
    let total = left + right;
    if total == 0 {
        return (
            " ".repeat(width / 2),
            " ".repeat(width - width / 2),
            0.0,
        );
    }
    let left_frac = left as f64 / total as f64;
    let left_cells = (left_frac * width as f64).round() as usize;
    let left_cells = left_cells.min(width);
    let right_cells = width - left_cells;
    let l = "█".repeat(left_cells);
    let r = "█".repeat(right_cells);
    (l, r, left_frac)
}

/// Render a horizontal axis with bid on the left, ask on the right, and a
/// `╋` marker placed at the position corresponding to the microprice.
/// Returns a string of length `width + 1` (the `╋` adds one cell).
pub fn microprice_axis(
    bid_cents: u64,
    ask_cents: u64,
    microprice_cents: u64,
    width: usize,
) -> String {
    if ask_cents <= bid_cents || width < 3 {
        return "─".repeat(width);
    }
    let span = (ask_cents - bid_cents) as f64;
    let offset = (microprice_cents.saturating_sub(bid_cents)) as f64;
    let frac = (offset / span).clamp(0.0, 1.0);
    let pos = (frac * (width - 1) as f64).round() as usize;
    let mut s = String::with_capacity(width + 1);
    for i in 0..width {
        if i == pos {
            s.push('╋');
        } else {
            s.push('━');
        }
    }
    s
}

/// Render a latency distribution shape — a horizontal bar with markers
/// at p50/p99/p999/p9999/max positions. `axis_max_ns` is the right edge
/// (e.g. 100_000 = 100µs); values beyond are clamped. Uses log-scale so
/// 1µs and 100µs both appear meaningfully.
pub fn distribution_bar(
    p50: u64,
    p99: u64,
    p999: u64,
    p9999: u64,
    max: u64,
    axis_max_ns: u64,
    width: usize,
) -> String {
    if width < 5 {
        return "─".repeat(width);
    }
    let log_max = (axis_max_ns.max(2) as f64).ln();
    // Position helper: log-scale 1ns..axis_max_ns mapped to 0..(width-1).
    let pos = |ns: u64| -> usize {
        if ns == 0 {
            return 0;
        }
        let ln_v = (ns as f64).ln().min(log_max);
        let frac = (ln_v / log_max).clamp(0.0, 1.0);
        ((frac * (width - 1) as f64).round() as usize).min(width - 1)
    };
    let positions = [
        (pos(p50), '╫'),
        (pos(p99), '╫'),
        (pos(p999), '╫'),
        (pos(p9999), '╫'),
        (pos(max), '╫'),
    ];
    let mut chars: Vec<char> = std::iter::once('┝')
        .chain(std::iter::repeat_n('─', width.saturating_sub(2)))
        .chain(std::iter::once('┥'))
        .collect();
    for (idx, ch) in positions {
        if idx > 0 && idx < width.saturating_sub(1) {
            chars[idx] = ch;
        }
    }
    chars.into_iter().collect()
}

/// Render a twin progress bar using `▰▱`. `value`/`max` clamped to
/// `width` cells.
pub fn twin_progress(value: u64, max: u64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let max = max.max(1);
    let filled = ((value as f64 / max as f64) * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    let mut s = String::with_capacity(width);
    for _ in 0..filled {
        s.push('▰');
    }
    for _ in 0..empty {
        s.push('▱');
    }
    s
}

/// Health-dot levels for the engine-health indicator on the top status
/// row. Green = nominal, Yellow = degraded (1+ slow frame in last 10s
/// or p99 over 10µs), Red = severe (sustained slow frames or feed
/// disconnected).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthLevel {
    Green,
    Yellow,
    Red,
}

impl HealthLevel {
    pub fn colour(self) -> Color {
        match self {
            HealthLevel::Green => GOOD,
            HealthLevel::Yellow => WARN,
            HealthLevel::Red => ALERT,
        }
    }
}
