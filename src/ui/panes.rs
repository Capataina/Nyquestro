//! Pane rendering. Each function takes a `Frame` and an area, draws into
//! that area only, and assumes the parent layout has already split the
//! canvas.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Axis, Block, BorderType, Borders, Chart, Dataset, GraphType, Paragraph, Sparkline,
};
use ratatui::Frame;

use crate::book::OrderBook;
use crate::metrics::registry::LatencySnapshot;
use crate::types::{Px, Qty};
use crate::ui::app::{App, EngineState, Mode, SymbolState};
use crate::ui::theme;

const PANE_BORDER: BorderType = BorderType::Rounded;

pub fn render(frame: &mut Frame, app: &App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(20),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_top_status(frame, outer[0], app);
    render_body(frame, outer[1], app);
    render_keybinds(frame, outer[2]);
}

fn render_top_status(frame: &mut Frame, area: Rect, app: &App) {
    let health_dot = Span::styled("● ", theme::fg(app.health_level().colour()));
    let state_label = match app.state {
        EngineState::Running => Span::styled("RUNNING", theme::fg_bold(theme::GOOD)),
        EngineState::Paused => Span::styled("PAUSED ", theme::fg_bold(theme::WARN)),
    };
    let title = Span::styled("NYQUESTRO ", theme::bold());
    let separator = Span::styled(" │ ", theme::fg_dim(theme::CHROME));

    // Symbol selector — per-symbol health dot before each name; active
    // symbol gets the bold accent style.
    let mut symbol_spans: Vec<Span> = vec![Span::styled("[", theme::fg_dim(theme::CHROME))];
    for (i, state) in app.symbols.iter().enumerate() {
        let active = i == app.selected_idx;
        let style = if active {
            theme::fg_bold(theme::ACCENT)
        } else {
            theme::fg_dim(theme::CHROME)
        };
        if i > 0 {
            symbol_spans.push(Span::styled(" ", theme::neutral()));
        }
        let dot_colour = app.symbol_health(i).colour();
        symbol_spans.push(Span::styled("●", theme::fg(dot_colour)));
        symbol_spans.push(Span::styled(format!("{}", state.symbol), style));
    }
    symbol_spans.push(Span::styled("]", theme::fg_dim(theme::CHROME)));

    // Speed multiplier is meaningful only in synthetic mode (Live mode is
    // always real-time). Hide entirely in live mode rather than render a
    // misleading "× 1.00".
    let speed_span = match &app.mode {
        Mode::Synthetic => Some(Span::styled(
            format!("× {:.2}", app.speed),
            theme::fg_dim(theme::CHROME),
        )),
        Mode::Live { .. } => None,
    };
    let uptime = format_duration(app.uptime());
    let clock = Span::styled(
        format!("uptime {uptime}"),
        theme::fg_dim(theme::CHROME),
    );
    // Mid: in Synthetic mode, the simulator's OU mid; in Live mode, the
    // most recent observed mid_history sample.
    let mid_cents = match &app.mode {
        Mode::Synthetic => app.selected_state().sim.mid_cents(),
        Mode::Live { .. } => app
            .selected_state()
            .mid_history
            .back()
            .copied()
            .unwrap_or(0),
    };
    let mid = if mid_cents == 0 {
        "—".to_string()
    } else {
        format_price_cents(mid_cents)
    };
    let mid_span = Span::styled(format!("mid {mid}"), theme::fg(theme::ACCENT));
    let live_status = match &app.mode {
        Mode::Live { status, .. } => Some(status.clone()),
        Mode::Synthetic => None,
    };

    let mut spans = vec![
        health_dot,
        title,
        Span::raw("· matching engine "),
        state_label,
        separator.clone(),
    ];
    spans.extend(symbol_spans);
    spans.push(separator.clone());
    spans.push(mid_span);
    if let Some(s) = speed_span {
        spans.push(separator.clone());
        spans.push(s);
    }
    spans.push(separator.clone());
    spans.push(clock);
    if let Some(s) = live_status {
        spans.push(separator);
        spans.push(Span::styled(
            format!("live · {s}"),
            theme::fg(theme::GOOD),
        ));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Left),
        area,
    );
}

fn render_keybinds(frame: &mut Frame, area: Rect) {
    let bind = |k, label| {
        vec![
            Span::styled(format!(" {k} "), theme::fg_bold(theme::ACCENT)),
            Span::styled(format!("{label} "), theme::fg_dim(theme::CHROME)),
        ]
    };
    let mut spans = Vec::new();
    spans.extend(bind("q", "quit"));
    spans.extend(bind("p", "pause"));
    spans.extend(bind("r", "reset"));
    spans.extend(bind("+/-", "speed"));
    spans.extend(bind("tab", "symbol"));
    spans.push(Span::styled(
        " · safe rust · ratatui",
        theme::fg_dim(theme::CHROME),
    ));
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(36),
            Constraint::Percentage(32),
            Constraint::Percentage(32),
        ])
        .split(rows[0]);
    render_depth_of_book(frame, top_row[0], app);
    render_trade_tape(frame, top_row[1], app);
    render_latency(frame, top_row[2], app);

    let bot_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(28),
            Constraint::Percentage(32),
        ])
        .split(rows[1]);
    render_mid_chart(frame, bot_row[0], app);
    render_throughput(frame, bot_row[1], app);
    render_engine_summary(frame, bot_row[2], app);
}

// ─── Depth of book ────────────────────────────────────────────────────────

fn render_depth_of_book(frame: &mut Frame, area: Rect, app: &App) {
    let symbol = app.selected_symbol();
    let title = format!("Depth of Book — {} L2", symbol);
    let block = pane_block(&title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let book = match app.selected_book() {
        Some(b) => b,
        None => {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "  (book not initialised)",
                    theme::fg_dim(theme::CHROME),
                ))),
                inner,
            );
            return;
        }
    };

    let level_cap = inner.height.saturating_sub(2) as usize / 2;
    let asks: Vec<_> = book.top_n_asks(level_cap.max(1));
    let bids: Vec<_> = book.top_n_bids(level_cap.max(1));
    let max_qty = asks
        .iter()
        .chain(bids.iter())
        .map(|(_, q)| q.value())
        .max()
        .unwrap_or(1)
        .max(1) as f64;

    let bar_width = (inner.width as usize).saturating_sub(28);
    let mut lines: Vec<Line> = Vec::new();

    for (px, qty) in asks.iter().rev() {
        lines.push(level_line(*px, *qty, max_qty, bar_width, theme::ASK));
    }

    let spread_row = match (book.best_bid(), book.best_ask()) {
        (Some((b, _)), Some((a, _))) => {
            let spread_cents = a.cents().saturating_sub(b.cents());
            let bps = if a.cents() > 0 {
                spread_cents as f64 / a.cents() as f64 * 10_000.0
            } else {
                0.0
            };
            format!(
                "── spread {} / {:.1} bp ──",
                format_price_cents(spread_cents),
                bps
            )
        }
        _ => "── spread (book empty) ──".to_string(),
    };
    lines.push(Line::from(vec![Span::styled(
        spread_row,
        theme::fg_dim(theme::CHROME),
    )]));

    for (px, qty) in bids.iter() {
        lines.push(level_line(*px, *qty, max_qty, bar_width, theme::BID));
    }

    if asks.is_empty() && bids.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (empty book — simulator warming up...)",
            theme::fg_dim(theme::CHROME),
        )));
    } else {
        // Bid/ask total-pressure stacked bar at the bottom — visualises
        // imbalance at a glance. Uses depth-top-10 so a few extreme-tail
        // levels don't dominate the impression.
        let (depth_bid, depth_ask) = book.depth(10);
        let pressure_width = (inner.width as usize).saturating_sub(28);
        let (l_bar, r_bar, left_frac) =
            theme::pressure_bar(depth_bid.value() as u64, depth_ask.value() as u64, pressure_width);
        let pct_left = (left_frac * 100.0).round() as u32;
        let pct_right = 100 - pct_left;
        let ratio_str = if depth_ask.value() == 0 {
            "—".to_string()
        } else if depth_bid.value() >= depth_ask.value() {
            format!("{:.1}:1 bid", depth_bid.value() as f64 / depth_ask.value().max(1) as f64)
        } else {
            format!("1:{:.1} ask", depth_ask.value() as f64 / depth_bid.value().max(1) as f64)
        };
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  pressure  ", theme::fg_dim(theme::CHROME)),
            Span::styled(l_bar, theme::fg(theme::BID)),
            Span::styled(r_bar, theme::fg(theme::ASK)),
            Span::styled(
                format!("  {pct_left}% / {pct_right}%  "),
                theme::neutral(),
            ),
            Span::styled(ratio_str, theme::fg_dim(theme::CHROME)),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn level_line(price: Px, qty: Qty, max_qty: f64, bar_width: usize, color: Color) -> Line<'static> {
    let ratio = qty.value() as f64 / max_qty;
    let bar = theme::block_bar(ratio, bar_width);
    Line::from(vec![
        Span::styled(format!("  {:>8} ", format_price_cents(price.cents())), theme::neutral()),
        Span::styled(format!("{:>5}  ", qty.value()), theme::fg(color)),
        Span::styled(bar, theme::fg(color)),
    ])
}

// ─── Trade tape ───────────────────────────────────────────────────────────

fn render_trade_tape(frame: &mut Frame, area: Rect, app: &App) {
    let symbol = app.selected_symbol();
    let title = format!("Trade Tape — {} · newest first", symbol);
    let block = pane_block(&title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let tape = &app.selected_state().tape;
    let cap = inner.height as usize;

    // Visible-window max for size-bar normalisation. Re-computed per
    // render frame so the bars adapt to the prevailing trade size in
    // view (a single whale print doesn't compress everything else to
    // invisibility on the next normal print).
    let visible: Vec<_> = tape.iter().take(cap).collect();
    let max_qty = visible
        .iter()
        .map(|p| p.quantity.value())
        .max()
        .unwrap_or(1)
        .max(1) as f64;
    // Reserve space for the size bar after the existing columns.
    let bar_width = (inner.width as usize).saturating_sub(34);

    let mut lines: Vec<Line> = Vec::with_capacity(cap);
    for print in visible {
        let (color, glyph) = match print.aggressor {
            crate::types::Side::Buy => (theme::BID, "▲"),
            crate::types::Side::Sell => (theme::ASK, "▼"),
        };
        let ts = format_clock_ns(print.at.nanos());
        let bar = theme::block_bar(print.quantity.value() as f64 / max_qty, bar_width);
        lines.push(Line::from(vec![
            Span::styled(format!(" {ts}  "), theme::fg_dim(theme::CHROME)),
            Span::styled(
                format!("{:>9}", format_price_cents(print.price.cents())),
                theme::fg(color),
            ),
            Span::styled(format!("  {glyph} "), theme::fg(color)),
            Span::styled(format!("{:>5} ", print.quantity.value()), theme::neutral()),
            Span::styled(bar, theme::fg(color)),
        ]));
    }
    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (no trades yet)",
            theme::fg_dim(theme::CHROME),
        )));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

// ─── Latency card ─────────────────────────────────────────────────────────

fn render_latency(frame: &mut Frame, area: Rect, app: &App) {
    let block = pane_block("Latency · ns");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let snap = app.metrics.snapshot();
    let header = Line::from(vec![
        Span::styled("  op    ", theme::fg_dim(theme::CHROME)),
        Span::styled("samples ", theme::fg_dim(theme::CHROME)),
        Span::styled("    p50 ", theme::fg_dim(theme::CHROME)),
        Span::styled("    p99 ", theme::fg_dim(theme::CHROME)),
        Span::styled("   p999 ", theme::fg_dim(theme::CHROME)),
        Span::styled("  p9999 ", theme::fg_dim(theme::CHROME)),
        Span::styled("    max ", theme::fg_dim(theme::CHROME)),
        Span::styled("budget", theme::fg_dim(theme::CHROME)),
    ]);
    let row = |op: &str, s: LatencySnapshot| -> Line {
        let (budget_glyph, budget_color) = budget_for(s.p99_ns);
        Line::from(vec![
            Span::styled(format!("  {op:<6}"), theme::bold()),
            Span::styled(format!("{:>7} ", s.count), theme::fg_dim(theme::CHROME)),
            Span::styled(format!("{:>8}", format_latency_ns(s.p50_ns)), theme::neutral()),
            Span::styled(
                format!("{:>8}", format_latency_ns(s.p99_ns)),
                theme::fg(theme::ACCENT),
            ),
            Span::styled(format!("{:>8}", format_latency_ns(s.p999_ns)), theme::neutral()),
            Span::styled(
                format!("{:>8}", format_latency_ns(s.p9999_ns)),
                theme::neutral(),
            ),
            Span::styled(
                format!("{:>8}", format_latency_ns(s.max_ns)),
                theme::fg(theme::ACCENT),
            ),
            Span::styled(format!(" {budget_glyph}"), theme::fg(budget_color)),
        ])
    };

    // Distribution-bar row per op — visualises tail shape on a log
    // 1ns..100µs axis with marks at p50/p99/p999/p9999/max. Numbers
    // remain (precision); the bar tells you "is the tail clustered or
    // spread" at a glance.
    let dist_width = (inner.width as usize).saturating_sub(8).max(20);
    let dist = |op: &str, s: LatencySnapshot| -> Line {
        let bar = theme::distribution_bar(
            s.p50_ns,
            s.p99_ns,
            s.p999_ns,
            s.p9999_ns,
            s.max_ns,
            100_000, // 100µs right edge
            dist_width,
        );
        Line::from(vec![
            Span::styled(format!("  {op:<6}"), theme::fg_dim(theme::CHROME)),
            Span::styled(bar, theme::fg(theme::ACCENT)),
        ])
    };

    let mut lines = vec![
        header,
        row("submit", snap.submit),
        dist("submit", snap.submit),
        row("match ", snap.match_op),
        dist("match ", snap.match_op),
        row("cancel", snap.cancel),
        dist("cancel", snap.cancel),
    ];

    let width = inner.width.saturating_sub(2) as usize;
    if app.selected_state().mid_history.len() > 1 && width > 2 {
        let recent: Vec<u64> = app
            .selected_state()
            .mid_history
            .iter()
            .copied()
            .rev()
            .take(width)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let deltas: Vec<u64> = recent
            .windows(2)
            .map(|w| (w[1] as i64 - w[0] as i64).unsigned_abs())
            .collect();
        if !deltas.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  mid Δ over recent samples",
                theme::fg_dim(theme::CHROME),
            )));
            let used = lines.len() as u16;
            frame.render_widget(Paragraph::new(lines.clone()), inner);
            let spark_area = Rect {
                x: inner.x + 2,
                y: inner.y + used,
                width: inner.width.saturating_sub(2),
                height: 1,
            };
            let sparkline = Sparkline::default()
                .data(&deltas)
                .style(Style::default().fg(theme::ACCENT));
            frame.render_widget(sparkline, spark_area);
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  legend ", theme::fg_dim(theme::CHROME)),
                Span::styled("● ", theme::fg(theme::GOOD)),
                Span::styled("p99 < 10µs ", theme::fg_dim(theme::CHROME)),
                Span::styled("● ", theme::fg(theme::WARN)),
                Span::styled("< 50µs ", theme::fg_dim(theme::CHROME)),
                Span::styled("● ", theme::fg(theme::ALERT)),
                Span::styled("≥ 50µs", theme::fg_dim(theme::CHROME)),
            ]));
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  legend ", theme::fg_dim(theme::CHROME)),
                        Span::styled("● ", theme::fg(theme::GOOD)),
                        Span::styled("p99 < 10µs ", theme::fg_dim(theme::CHROME)),
                        Span::styled("● ", theme::fg(theme::WARN)),
                        Span::styled("< 50µs ", theme::fg_dim(theme::CHROME)),
                        Span::styled("● ", theme::fg(theme::ALERT)),
                        Span::styled("≥ 50µs", theme::fg_dim(theme::CHROME)),
                    ]),
                ]),
                Rect {
                    x: inner.x,
                    y: inner.y + used + 1,
                    width: inner.width,
                    height: inner.height.saturating_sub(used + 1),
                },
            );
            return;
        }
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn budget_for(p99_ns: u64) -> (&'static str, Color) {
    if p99_ns == 0 {
        ("—  ", theme::CHROME)
    } else if p99_ns < 10_000 {
        ("●  ", theme::GOOD)
    } else if p99_ns < 50_000 {
        ("●  ", theme::WARN)
    } else {
        ("●  ", theme::ALERT)
    }
}

// ─── Mid-price chart ─────────────────────────────────────────────────────

fn render_mid_chart(frame: &mut Frame, area: Rect, app: &App) {
    let symbol = app.selected_symbol();
    let title = format!("Mid Price — {} · OU walk", symbol);
    let block = pane_block(&title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mid_history = &app.selected_state().mid_history;
    if mid_history.len() < 2 {
        let p = Paragraph::new(Line::from(Span::styled(
            "  (warming up...)",
            theme::fg_dim(theme::CHROME),
        )));
        frame.render_widget(p, inner);
        return;
    }

    let series: Vec<(f64, f64)> = mid_history
        .iter()
        .enumerate()
        .map(|(i, c)| (i as f64, *c as f64 / 100.0))
        .collect();
    let (lo, hi) = series.iter().map(|(_, y)| *y).fold(
        (f64::INFINITY, f64::NEG_INFINITY),
        |(lo, hi), y| (lo.min(y), hi.max(y)),
    );
    let pad = ((hi - lo).abs() * 0.1).max(0.05);
    let y_min = lo - pad;
    let y_max = hi + pad;
    let x_max = (series.len() - 1) as f64;

    let dataset = Dataset::default()
        .name("mid")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme::ACCENT))
        .data(&series);

    let chart = Chart::new(vec![dataset])
        .x_axis(
            Axis::default()
                .style(theme::fg_dim(theme::CHROME))
                .bounds([0.0, x_max]),
        )
        .y_axis(
            Axis::default()
                .style(theme::fg_dim(theme::CHROME))
                .labels(vec![
                    Span::styled(format!("${y_min:.2}"), theme::fg_dim(theme::CHROME)),
                    Span::styled(format!("${y_max:.2}"), theme::fg_dim(theme::CHROME)),
                ])
                .bounds([y_min, y_max]),
        );
    frame.render_widget(chart, inner);
}

// ─── Throughput card ──────────────────────────────────────────────────────

fn render_throughput(frame: &mut Frame, area: Rect, app: &App) {
    let block = pane_block("Throughput");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let s = app.metrics.snapshot().counters;
    let spark_width = (inner.width as usize).saturating_sub(40).max(12);
    let header = Line::from(vec![
        Span::styled("  metric   ", theme::fg_dim(theme::CHROME)),
        Span::styled("   1s ", theme::fg_dim(theme::CHROME)),
        Span::styled("  10s ", theme::fg_dim(theme::CHROME)),
        Span::styled(" 1min ", theme::fg_dim(theme::CHROME)),
        Span::styled(" 5min ", theme::fg_dim(theme::CHROME)),
        Span::styled(" trend (60s)", theme::fg_dim(theme::CHROME)),
    ]);
    let row = |label: &str,
               w: crate::metrics::windows::WindowSnapshot,
               ring: &std::collections::VecDeque<u64>,
               color: Color|
     -> Line {
        let samples: Vec<u64> = ring.iter().copied().collect();
        let spark = theme::sparkline(&samples, spark_width);
        Line::from(vec![
            Span::styled(format!("  {label:<8}"), theme::bold()),
            Span::styled(format!("{:>5}", w.last_1s), theme::fg(color)),
            Span::styled(format!("{:>6}", w.last_10s), theme::neutral()),
            Span::styled(format!("{:>6}", w.last_1min), theme::neutral()),
            Span::styled(format!("{:>6}", w.last_5min), theme::fg_dim(theme::CHROME)),
            Span::styled(format!("  {spark}"), theme::fg(color)),
        ])
    };

    let order_to_fill = if s.fills.last_10s == 0 {
        format!("  {:>5}", "—")
    } else {
        let ratio = s.orders.last_10s as f64 / s.fills.last_10s as f64;
        format!("  {:>5.2}", ratio)
    };

    let lines = vec![
        header,
        row("orders", s.orders, &app.rate_rings.orders, theme::ACCENT),
        row("fills", s.fills, &app.rate_rings.fills, theme::GOOD),
        row("cancels", s.cancels, &app.rate_rings.cancels, Color::Reset),
        row("rejects", s.rejects, &app.rate_rings.rejects, theme::ALERT),
        row("quotes", s.quotes, &app.rate_rings.quotes, theme::ACCENT),
        Line::from(""),
        Line::from(vec![
            Span::styled("  order/fill 10s ", theme::fg_dim(theme::CHROME)),
            Span::styled(order_to_fill, theme::fg(theme::ACCENT)),
            Span::styled(
                "   (high = cancel-heavy flow)",
                theme::fg_dim(theme::CHROME),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines), inner);
}

// ─── Engine summary ──────────────────────────────────────────────────────

fn render_engine_summary(frame: &mut Frame, area: Rect, app: &App) {
    let symbol = app.selected_symbol();
    let title = format!("Engine — {}", symbol);
    let block = pane_block(&title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let book = app.selected_book();
    let state: &SymbolState = app.selected_state();

    let bid = book
        .and_then(|b| b.best_bid())
        .map(|(p, _)| format_price_cents(p.cents()))
        .unwrap_or_else(|| "—".into());
    let ask = book
        .and_then(|b| b.best_ask())
        .map(|(p, _)| format_price_cents(p.cents()))
        .unwrap_or_else(|| "—".into());
    let bid_qty = book
        .and_then(|b| b.best_bid())
        .map(|(_, q)| q.value().to_string())
        .unwrap_or_else(|| "—".into());
    let ask_qty = book
        .and_then(|b| b.best_ask())
        .map(|(_, q)| q.value().to_string())
        .unwrap_or_else(|| "—".into());

    let spread = book
        .and_then(|b| b.spread_cents())
        .map(|c| {
            let bps = book
                .and_then(|b| b.best_ask())
                .map(|(a, _)| c as f64 / a.cents() as f64 * 10_000.0)
                .unwrap_or(0.0);
            format!("{} ({:.1} bp)", format_price_cents(c), bps)
        })
        .unwrap_or_else(|| "—".into());

    let microprice = book
        .and_then(|b| b.microprice())
        .map(|cents| format!("${:.4}", cents / 100.0))
        .unwrap_or_else(|| "—".into());

    let ofi = book.map(|b| b.ofi(10)).unwrap_or(0.0);
    let ofi_color = if ofi > 0.2 {
        theme::BID
    } else if ofi < -0.2 {
        theme::ASK
    } else {
        theme::ACCENT
    };
    let ofi_glyph = ofi_gauge(ofi);

    let (n_bid, n_ask) = book.map(|b| b.level_counts()).unwrap_or((0, 0));
    let (depth_bid, depth_ask) = book
        .map(|b| b.depth(10))
        .unwrap_or((Qty::ZERO, Qty::ZERO));

    let kv_styled = |k: &str, v: String, c: Color| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("  {k:<13}"), theme::fg_dim(theme::CHROME)),
            Span::styled(v, theme::fg(c)),
        ])
    };
    let kv = |k: &str, v: String| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("  {k:<13}"), theme::fg_dim(theme::CHROME)),
            Span::styled(v, theme::neutral()),
        ])
    };

    // Microprice axis: bid on left, ask on right, microprice marker
    // positioned by linear interpolation. When the book has both sides,
    // the marker visualises which way the book "leans" without us
    // having to read the OFI number.
    let axis_width = (inner.width as usize).saturating_sub(8).max(20);
    let microprice_axis_str = match (book.and_then(|b| b.best_bid()), book.and_then(|b| b.best_ask())) {
        (Some((b, _)), Some((a, _))) => {
            let mp = book.and_then(|b| b.microprice()).map(|f| f.round() as u64).unwrap_or((a.cents() + b.cents()) / 2);
            theme::microprice_axis(b.cents(), a.cents(), mp, axis_width)
        }
        _ => "─".repeat(axis_width),
    };

    // Spread gauge — 1 cell per ~0.5bp, 20 cells = 10bp. Most BTC-USD
    // spreads are 0.1bp (one cell or less) which renders as a tiny
    // sliver = "tight"; meme coins or thin books show much more fill.
    let spread_bps = book
        .and_then(|b| b.spread_cents())
        .and_then(|c| {
            book.and_then(|b| b.best_ask())
                .map(|(a, _)| c as f64 / a.cents() as f64 * 10_000.0)
        })
        .unwrap_or(0.0);
    let spread_gauge_width = 20usize;
    let spread_gauge = theme::block_bar((spread_bps / 10.0).clamp(0.0, 1.0), spread_gauge_width);
    let spread_color = if spread_bps < 1.0 {
        theme::GOOD
    } else if spread_bps < 5.0 {
        theme::WARN
    } else {
        theme::ALERT
    };
    let spread_label = if spread_bps < 1.0 {
        "tight"
    } else if spread_bps < 5.0 {
        "moderate"
    } else {
        "wide"
    };

    // Depth-ratio stacked bar: shows top-10 bid total vs top-10 ask
    // total proportionally. Width fills the pane.
    let depth_width = (inner.width as usize).saturating_sub(8).max(16);
    let (l_bar, r_bar, _) = theme::pressure_bar(
        depth_bid.value() as u64,
        depth_ask.value() as u64,
        depth_width,
    );

    // Twin progress bars for level counts. Reference is `max(n_bid, n_ask)`
    // capped at a soft 250 (typical book on Coinbase L2 with our 50-level
    // cap × N updates).
    let level_bar_width = (inner.width as usize).saturating_sub(20).max(12);
    let level_max = n_bid.max(n_ask).max(1) as u64;
    let bid_levels_bar = theme::twin_progress(n_bid as u64, level_max, level_bar_width);
    let ask_levels_bar = theme::twin_progress(n_ask as u64, level_max, level_bar_width);

    let lines = vec![
        Line::from(Span::styled(" microstructure", theme::bold())),
        kv_styled("best bid", format!("{bid} × {bid_qty}"), theme::BID),
        kv_styled("best ask", format!("{ask} × {ask_qty}"), theme::ASK),
        Line::from(vec![
            Span::styled("  spread       ", theme::fg_dim(theme::CHROME)),
            Span::styled(spread, theme::neutral()),
        ]),
        Line::from(vec![
            Span::styled("               ", theme::neutral()),
            Span::styled(spread_gauge, theme::fg(spread_color)),
            Span::styled(format!(" {spread_label}"), theme::fg(spread_color)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  microprice   ", theme::fg_dim(theme::CHROME)),
            Span::styled(microprice, theme::fg(theme::ACCENT)),
        ]),
        Line::from(vec![
            Span::styled("    bid ", theme::fg(theme::BID)),
            Span::styled(microprice_axis_str, theme::fg(theme::ACCENT)),
            Span::styled(" ask", theme::fg(theme::ASK)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  OFI top-10   ", theme::fg_dim(theme::CHROME)),
            Span::styled(format!("{ofi:>+.3}  "), theme::fg(ofi_color)),
            Span::styled(ofi_glyph, theme::fg(ofi_color)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  depth top-10", theme::fg_dim(theme::CHROME)),
        ]),
        Line::from(vec![
            Span::styled("    bid  ", theme::fg(theme::BID)),
            Span::styled(l_bar, theme::fg(theme::BID)),
            Span::styled(r_bar, theme::fg(theme::ASK)),
            Span::styled(format!("  {} / {}", depth_bid.value(), depth_ask.value()), theme::neutral()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  levels", theme::fg_dim(theme::CHROME)),
        ]),
        Line::from(vec![
            Span::styled("    bid  ", theme::fg(theme::BID)),
            Span::styled(bid_levels_bar, theme::fg(theme::BID)),
            Span::styled(format!("  {n_bid}"), theme::neutral()),
        ]),
        Line::from(vec![
            Span::styled("    ask  ", theme::fg(theme::ASK)),
            Span::styled(ask_levels_bar, theme::fg(theme::ASK)),
            Span::styled(format!("  {n_ask}"), theme::neutral()),
        ]),
        Line::from(""),
        Line::from(Span::styled(" lifetime", theme::bold())),
        kv("submitted", state.total_orders.to_string()),
        kv("filled", state.total_fills.to_string()),
        kv("cancelled", state.total_cancels.to_string()),
        kv("rejected", state.total_rejects.to_string()),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Render OFI as a centered gauge: bar grows left for negative, right for
/// positive, range `[-1, 1]` mapped to 20 cells.
fn ofi_gauge(ofi: f64) -> String {
    let cells = 20i32;
    let half = cells / 2;
    let position = (ofi.clamp(-1.0, 1.0) * half as f64).round() as i32;
    let mut s = String::with_capacity(cells as usize + 2);
    s.push('[');
    for i in -half..half {
        if (i < 0 && i >= position) || (i >= 0 && i < position) {
            s.push('█');
        } else if i == 0 {
            s.push('│');
        } else {
            s.push('·');
        }
    }
    s.push(']');
    s
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn pane_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(PANE_BORDER)
        .border_style(theme::fg_dim(theme::CHROME))
        .title(Span::styled(format!(" {title} "), theme::bold()))
}

fn format_price_cents(cents: u64) -> String {
    let dollars = cents / 100;
    let frac = cents % 100;
    format!("${dollars}.{frac:02}")
}

fn format_latency_ns(ns: u64) -> String {
    if ns == 0 {
        "—".into()
    } else if ns < 1_000 {
        format!("{ns} ns")
    } else if ns < 1_000_000 {
        format!("{:.1} µs", ns as f64 / 1_000.0)
    } else if ns < 1_000_000_000 {
        format!("{:.1} ms", ns as f64 / 1_000_000.0)
    } else {
        format!("{:.2} s", ns as f64 / 1_000_000_000.0)
    }
}

fn format_clock_ns(ns: u64) -> String {
    let secs = ns / 1_000_000_000;
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    let ms = (ns / 1_000_000) % 1_000;
    format!("{h:02}:{m:02}:{s:02}.{ms:03}")
}

fn format_duration(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    let h = secs / 3600;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    if h > 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

// Accessors that book exposes are now used directly; OrderBook import keeps
// public-API discoverability obvious.
#[allow(dead_code)]
fn _typecheck(_: &OrderBook) {}
