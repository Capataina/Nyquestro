//! Dashboard application state + main loop.
//!
//! Single-threaded: the simulators, matching engine, and renderer all run
//! on the main thread. A 50ms tick advances every simulator and applies
//! its output to the engine; a 33ms render tick paints. Both share one
//! `App`. The `Tab` key cycles the symbol the dashboard focuses on.

use std::collections::VecDeque;
use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use std::sync::mpsc::Receiver;

use crate::book::{Market, OrderBook};
use crate::events::{FillEvent, OrderEvent, OrderRejectionReason, QuoteSide};
use crate::feed::FeedAction;
use crate::metrics::{MetricsRegistry, Op};
use crate::order::Order;
use crate::simulator::{MarketSimulator, SimAction, SimConfig};
use crate::telemetry::{TelemetryEvent, TelemetryHandle};
use crate::types::{OrderID, Px, Qty, Side, Symbol, Ts};
use crate::ui::panes;

const RENDER_TICK: Duration = Duration::from_millis(33);
const SIM_TICK: Duration = Duration::from_millis(50);
const POLL_TICK: Duration = Duration::from_millis(10);

/// Default seed for the Reset key.
const RESET_SEED: u64 = 0xC0FFEE;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Quit,
    TogglePause,
    Reset,
    SpeedUp,
    SpeedDown,
    CycleSymbol,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy)]
pub struct TapePrint {
    pub symbol: Symbol,
    pub price: Px,
    pub quantity: Qty,
    pub aggressor: Side,
    pub at: Ts,
}

/// Per-symbol state: simulator + tape ring + mid history + lifetime totals.
pub struct SymbolState {
    pub symbol: Symbol,
    pub sim: MarketSimulator,
    pub tape: VecDeque<TapePrint>,
    pub mid_history: VecDeque<u64>,
    pub total_orders: u64,
    pub total_fills: u64,
    pub total_cancels: u64,
    pub total_rejects: u64,
    pub resting_ids: Vec<OrderID>,
    last_resting_refresh: Instant,
}

impl SymbolState {
    pub fn new(symbol: Symbol, fair_value_cents: u64, seed: u64) -> Self {
        let cfg = SimConfig {
            symbol,
            fair_value_cents,
            ..SimConfig::default()
        };
        SymbolState {
            symbol,
            sim: MarketSimulator::new(cfg, seed),
            tape: VecDeque::with_capacity(200),
            mid_history: VecDeque::with_capacity(600),
            total_orders: 0,
            total_fills: 0,
            total_cancels: 0,
            total_rejects: 0,
            resting_ids: Vec::new(),
            last_resting_refresh: Instant::now(),
        }
    }
}

/// Where the dashboard's flow comes from. The two modes are mutually
/// exclusive: in `Synthetic` the per-symbol simulators tick on the main
/// loop; in `Live` the bridge channel feeds pre-built `FeedAction`s.
pub enum Mode {
    Synthetic,
    Live { feed_rx: Receiver<FeedAction>, status: String },
}

pub struct App {
    pub market: Market,
    pub symbols: Vec<SymbolState>,
    pub selected_idx: usize,
    pub metrics: MetricsRegistry,
    pub state: EngineState,
    pub speed: f64,
    pub mode: Mode,
    pub telemetry: TelemetryHandle,
    /// Last-frame profiling: most-recent (step_us, render_us, actions,
    /// budget_left). Read by render_top_status for the health dot.
    pub last_frame: Option<FrameStats>,
    /// Tracks recent slow-frame events for the health-dot system.
    pub last_slow_frame_at: Option<Instant>,
    /// Per-metric ring of recent 1-second rates for the throughput
    /// sparklines (60 samples = 60 seconds).
    pub rate_rings: RateRings,
    /// Cumulative counters captured at the previous rate-sample moment;
    /// the per-second delta is what feeds the rings.
    rate_baseline: RateBaseline,
    /// 1Hz tick anchor for periodic telemetry snapshots.
    last_snapshot_tick: Instant,
    /// Quote-event sampling counter (1-in-N).
    quote_sample_counter: u32,
    started_at: Instant,
}

/// Snapshot of a single render-tick's profile.
#[derive(Debug, Clone, Copy)]
pub struct FrameStats {
    pub step_us: u64,
    pub render_us: u64,
    pub actions: u32,
    pub budget_left: u32,
}

#[derive(Debug, Clone, Default)]
pub struct RateRings {
    pub orders: VecDeque<u64>,
    pub fills: VecDeque<u64>,
    pub cancels: VecDeque<u64>,
    pub rejects: VecDeque<u64>,
    pub quotes: VecDeque<u64>,
}

#[derive(Debug, Clone, Copy, Default)]
struct RateBaseline {
    orders: u64,
    fills: u64,
    cancels: u64,
    rejects: u64,
}

impl App {
    /// Construct a dashboard for three default synthetic symbols (AAPL,
    /// MSFT, NVDA) with realistic fair values. Each symbol gets a distinct
    /// seed derived from the user-supplied master seed.
    pub fn new(seed: u64, telemetry: TelemetryHandle) -> Self {
        let aapl = Symbol::from_const("AAPL");
        let msft = Symbol::from_const("MSFT");
        let nvda = Symbol::from_const("NVDA");
        let mut market = Market::new();
        market.register(aapl);
        market.register(msft);
        market.register(nvda);
        let symbols_str: Vec<String> = vec!["AAPL".into(), "MSFT".into(), "NVDA".into()];
        telemetry.record(TelemetryEvent::Startup {
            mode: "synthetic",
            symbols: symbols_str,
            term: (0, 0), // populated on first render via Resize event
            seed: Some(seed),
        });
        App {
            market,
            symbols: vec![
                SymbolState::new(aapl, 15_000, seed.wrapping_add(0xA1)),
                SymbolState::new(msft, 30_000, seed.wrapping_add(0xB2)),
                SymbolState::new(nvda, 50_000, seed.wrapping_add(0xC3)),
            ],
            selected_idx: 0,
            metrics: MetricsRegistry::new(),
            state: EngineState::Running,
            speed: 1.0,
            mode: Mode::Synthetic,
            telemetry,
            last_frame: None,
            last_slow_frame_at: None,
            rate_rings: RateRings::default(),
            rate_baseline: RateBaseline::default(),
            last_snapshot_tick: Instant::now(),
            quote_sample_counter: 0,
            started_at: Instant::now(),
        }
    }

    /// Construct a dashboard for live Coinbase-fed symbols. Each symbol
    /// gets its `SymbolState` (without an active simulator — it stays
    /// initialised but unused) so the same render path works for both
    /// modes. The caller is responsible for spawning the WebSocket task
    /// that fills `feed_rx`.
    pub fn new_live(
        symbols: Vec<(Symbol, u64)>,
        feed_rx: Receiver<FeedAction>,
        telemetry: TelemetryHandle,
    ) -> Self {
        let mut market = Market::new();
        let mut states = Vec::with_capacity(symbols.len());
        for (i, (sym, fair)) in symbols.iter().enumerate() {
            market.register(*sym);
            states.push(SymbolState::new(*sym, *fair, (i as u64).wrapping_add(0xFEED)));
        }
        let symbols_str: Vec<String> = symbols.iter().map(|(s, _)| s.to_string()).collect();
        telemetry.record(TelemetryEvent::Startup {
            mode: "live",
            symbols: symbols_str,
            term: (0, 0),
            seed: None,
        });
        App {
            market,
            symbols: states,
            selected_idx: 0,
            metrics: MetricsRegistry::new(),
            state: EngineState::Running,
            speed: 1.0,
            mode: Mode::Live {
                feed_rx,
                status: "starting…".to_string(),
            },
            telemetry,
            last_frame: None,
            last_slow_frame_at: None,
            rate_rings: RateRings::default(),
            rate_baseline: RateBaseline::default(),
            last_snapshot_tick: Instant::now(),
            quote_sample_counter: 0,
            started_at: Instant::now(),
        }
    }

    pub fn uptime(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Compute the engine's combined health level for the top-status dot.
    /// Green = nominal. Yellow = degraded (slow frame in last 10s OR p99
    /// over 10µs). Red = severe (slow frame in last 2s OR p99 over 50µs
    /// OR live feed disconnected).
    pub fn health_level(&self) -> crate::ui::theme::HealthLevel {
        use crate::ui::theme::HealthLevel;
        let now = Instant::now();
        let snap = self.metrics.snapshot();
        let p99 = snap.submit.p99_ns;
        let recent_slow = self
            .last_slow_frame_at
            .map(|t| now.duration_since(t).as_secs_f64())
            .unwrap_or(f64::INFINITY);
        if p99 > 50_000 || recent_slow < 2.0 {
            HealthLevel::Red
        } else if p99 > 10_000 || recent_slow < 10.0 {
            HealthLevel::Yellow
        } else {
            HealthLevel::Green
        }
    }

    /// Per-symbol health for the symbol-selector dots. Currently mirrors
    /// the engine-wide health (we don't yet partition slow-frame events
    /// by symbol). When that lands, this becomes per-symbol.
    pub fn symbol_health(&self, _idx: usize) -> crate::ui::theme::HealthLevel {
        self.health_level()
    }

    #[inline]
    pub fn selected_symbol(&self) -> Symbol {
        self.symbols[self.selected_idx].symbol
    }

    pub fn selected_book(&self) -> Option<&OrderBook> {
        self.market.book(self.selected_symbol())
    }

    pub fn selected_state(&self) -> &SymbolState {
        &self.symbols[self.selected_idx]
    }

    pub fn selected_state_mut(&mut self) -> &mut SymbolState {
        &mut self.symbols[self.selected_idx]
    }

    /// Drive the engine forward by `dt_secs`.
    ///
    /// In `Synthetic` mode this advances every per-symbol simulator and
    /// dispatches its actions. In `Live` mode it drains the feed channel
    /// non-blockingly; `dt_secs` is ignored. Returns the number of
    /// actions dispatched and how much per-frame budget remained — fed to
    /// `record_frame` by the run loop.
    pub fn step(&mut self, dt_secs: f64) -> (u32, u32) {
        if self.state == EngineState::Paused {
            return (0, 500);
        }

        let mut actions_count = 0u32;
        let mut budget_left = 500u32;

        // Drain whichever source is active. We can't pattern-match `mode`
        // and call `&mut self` methods inside the arm, so we replace the
        // mode briefly to extract the receiver, then put it back.
        let mut taken = std::mem::replace(&mut self.mode, Mode::Synthetic);
        match &mut taken {
            Mode::Synthetic => {
                let scaled_dt = dt_secs * self.speed;
                for idx in 0..self.symbols.len() {
                    let actions = self.symbols[idx].sim.step(scaled_dt);
                    for a in actions {
                        self.dispatch(idx, a);
                        actions_count = actions_count.saturating_add(1);
                    }
                    self.bookkeep_per_symbol(idx);
                }
            }
            Mode::Live { feed_rx, status } => {
                const PER_FRAME_BUDGET: usize = 500;
                let mut budget = PER_FRAME_BUDGET;
                while budget > 0 {
                    match feed_rx.try_recv() {
                        Ok(FeedAction::Action { symbol_idx, action }) => {
                            self.dispatch(symbol_idx, action);
                            budget -= 1;
                            actions_count = actions_count.saturating_add(1);
                        }
                        Ok(FeedAction::Status(s)) => {
                            self.telemetry.record(TelemetryEvent::FeedStatus {
                                msg: s.clone(),
                            });
                            *status = s;
                        }
                        Err(_) => break,
                    }
                }
                budget_left = budget as u32;
                // Sample mid from microprice once per frame, per symbol.
                // This replaces the older "track every submit's price"
                // approach which contaminated the chart with off-touch
                // levels (BTC bid at $66k while the book sat at $80k).
                for idx in 0..self.symbols.len() {
                    let symbol = self.symbols[idx].symbol;
                    if let Some(book) = self.market.book(symbol)
                        && let Some(mp) = book.microprice()
                    {
                        let mp_cents = mp.round() as u64;
                        let hist = &mut self.symbols[idx].mid_history;
                        hist.push_back(mp_cents);
                        if hist.len() > 600 {
                            hist.pop_front();
                        }
                    }
                    if self.symbols[idx]
                        .last_resting_refresh
                        .elapsed()
                        > Duration::from_millis(250)
                    {
                        self.refresh_resting_ids(idx);
                        self.symbols[idx].last_resting_refresh = Instant::now();
                    }
                }
            }
        }
        self.mode = taken;
        self.periodic_snapshot();
        (actions_count, budget_left)
    }

    /// Record per-frame profile + slow-frame heuristic. Called by the
    /// run loop after each render tick.
    pub fn record_frame(&mut self, step_us: u64, render_us: u64, actions: u32, budget_left: u32) {
        self.last_frame = Some(FrameStats {
            step_us,
            render_us,
            actions,
            budget_left,
        });
        self.telemetry.record(TelemetryEvent::Frame {
            step_us,
            render_us,
            actions,
            budget_left,
        });
        let total = step_us + render_us;
        if total > 33_000 {
            let reason = if budget_left == 0 {
                "per_frame_budget_exhausted"
            } else if render_us > step_us {
                "render_blocked"
            } else {
                "step_dominated"
            };
            self.last_slow_frame_at = Some(Instant::now());
            self.telemetry.record(TelemetryEvent::FrameSlow {
                step_us,
                render_us,
                actions,
                reason,
            });
        }
    }

    /// 1Hz tick: emit Latency / Throughput / BookState events. Called
    /// once per `step` invocation; gated on a `last_snapshot_tick`
    /// `Instant`.
    fn periodic_snapshot(&mut self) {
        if self.last_snapshot_tick.elapsed() < Duration::from_secs(1) {
            return;
        }
        self.last_snapshot_tick = Instant::now();

        let snap = self.metrics.snapshot();
        for (op_name, lat) in [
            ("submit", snap.submit),
            ("match", snap.match_op),
            ("cancel", snap.cancel),
        ] {
            self.telemetry.record(TelemetryEvent::Latency {
                op: op_name,
                count: lat.count,
                p50_ns: lat.p50_ns,
                p99_ns: lat.p99_ns,
                p999_ns: lat.p999_ns,
                p9999_ns: lat.p9999_ns,
                max_ns: lat.max_ns,
            });
        }
        let c = snap.counters;
        self.telemetry.record(TelemetryEvent::Throughput {
            orders_1s: c.orders.last_1s,
            fills_1s: c.fills.last_1s,
            cancels_1s: c.cancels.last_1s,
            rejects_1s: c.rejects.last_1s,
            quotes_1s: c.quotes.last_1s,
        });

        // Per-symbol book state.
        let symbols: Vec<Symbol> = self.symbols.iter().map(|s| s.symbol).collect();
        for sym in symbols {
            if let Some(book) = self.market.book(sym) {
                let (n_bid, n_ask) = book.level_counts();
                let (depth_bid, depth_ask) = book.depth(10);
                let ofi = book.ofi(10);
                let microprice_c = book.microprice().map(|f| f.round() as u64);
                let spread_c = book.spread_cents();
                self.telemetry.record(TelemetryEvent::BookState {
                    sym: sym.to_string(),
                    levels_bid: n_bid,
                    levels_ask: n_ask,
                    depth_bid: depth_bid.value(),
                    depth_ask: depth_ask.value(),
                    ofi,
                    microprice_c,
                    spread_c,
                });
            }
        }

        // Push 1-second per-metric rates into the rings (for sparklines).
        // Compute deltas vs the baseline so the ring carries per-second
        // counts, not cumulative running totals.
        let cur = RateBaseline {
            orders: self.symbols.iter().map(|s| s.total_orders).sum(),
            fills: self.symbols.iter().map(|s| s.total_fills).sum(),
            cancels: self.symbols.iter().map(|s| s.total_cancels).sum(),
            rejects: self.symbols.iter().map(|s| s.total_rejects).sum(),
        };
        push_ring(
            &mut self.rate_rings.orders,
            cur.orders.saturating_sub(self.rate_baseline.orders),
        );
        push_ring(
            &mut self.rate_rings.fills,
            cur.fills.saturating_sub(self.rate_baseline.fills),
        );
        push_ring(
            &mut self.rate_rings.cancels,
            cur.cancels.saturating_sub(self.rate_baseline.cancels),
        );
        push_ring(
            &mut self.rate_rings.rejects,
            cur.rejects.saturating_sub(self.rate_baseline.rejects),
        );
        push_ring(&mut self.rate_rings.quotes, c.quotes.last_1s);
        self.rate_baseline = cur;
    }

    /// Per-tick bookkeeping for one synthetic-mode symbol: append the
    /// simulator's current mid to the history ring and refresh the
    /// resting-id cache periodically.
    fn bookkeep_per_symbol(&mut self, idx: usize) {
        let mid = self.symbols[idx].sim.mid_cents();
        let hist = &mut self.symbols[idx].mid_history;
        hist.push_back(mid);
        if hist.len() > 600 {
            hist.pop_front();
        }
        if self.symbols[idx]
            .last_resting_refresh
            .elapsed()
            > Duration::from_millis(250)
        {
            self.refresh_resting_ids(idx);
            self.symbols[idx].last_resting_refresh = Instant::now();
        }
    }

    /// Dispatch a single action against the engine. Shared between
    /// synthetic simulator output and live-feed bridge output.
    pub fn dispatch(&mut self, idx: usize, action: SimAction) {
        match action {
            SimAction::Submit(order) => self.handle_submit(order, idx),
            SimAction::CancelHint => self.handle_cancel_hint(idx),
            SimAction::Cancel { symbol, order_id } => {
                self.handle_cancel(symbol, order_id, idx)
            }
        }
    }

    fn handle_cancel(&mut self, symbol: Symbol, order_id: OrderID, idx: usize) {
        let started = Instant::now();
        let ts = Ts::from_nanos(self.uptime().as_nanos() as u64);
        match self.market.cancel(symbol, order_id, ts) {
            Ok(ev) => {
                self.metrics.record_latency(Op::Cancel, started.elapsed());
                self.metrics.record_cancels(1);
                self.symbols[idx].total_cancels =
                    self.symbols[idx].total_cancels.saturating_add(1);
                let remaining = match ev {
                    OrderEvent::Cancelled { remaining, .. } => remaining.value(),
                    _ => 0,
                };
                self.telemetry.record(TelemetryEvent::Cancel {
                    sym: symbol.to_string(),
                    id: order_id.value(),
                    remaining,
                });
            }
            Err(_) => {
                // Cancel-of-unknown-id is benign in live mode (we may
                // race with the venue clearing a level). Don't telemeter.
            }
        }
    }

    fn handle_submit(&mut self, order: Order, idx: usize) {
        let aggressor_side = order.side();
        let symbol = order.symbol();
        let order_id = order.id();
        let order_qty = order.quantity();
        let order_px = order.price();
        let started = Instant::now();
        // Telemetry: record the submit before dispatch so the audit trail
        // captures intent even if the engine rejects.
        self.telemetry.record(TelemetryEvent::Submit {
            sym: symbol.to_string(),
            side: side_str(aggressor_side),
            px_c: order_px.cents(),
            qty: order_qty.value(),
            id: order_id.value(),
        });
        match self.market.submit_limit(order) {
            Ok(res) => {
                let elapsed = started.elapsed();
                self.metrics.record_latency(Op::Submit, elapsed);
                if !res.fills.is_empty() {
                    self.metrics.record_latency(Op::Match, elapsed);
                }
                self.metrics.record_orders(1);
                self.symbols[idx].total_orders =
                    self.symbols[idx].total_orders.saturating_add(1);

                for f in &res.fills {
                    self.metrics.record_fills(1);
                    self.symbols[idx].total_fills =
                        self.symbols[idx].total_fills.saturating_add(1);
                    self.telemetry.record(TelemetryEvent::Fill {
                        sym: f.symbol.to_string(),
                        px_c: f.price.cents(),
                        qty: f.quantity.value(),
                        buyer: f.buyer_order_id.value(),
                        seller: f.seller_order_id.value(),
                    });
                    self.push_print(idx, *f, aggressor_side);
                }
                for ev in &res.lifecycle {
                    if let OrderEvent::Rejected {
                        order_id, reason, ..
                    } = ev
                    {
                        self.metrics.record_rejects(1);
                        self.symbols[idx].total_rejects =
                            self.symbols[idx].total_rejects.saturating_add(1);
                        self.telemetry.record(TelemetryEvent::Reject {
                            sym: ev.symbol().to_string(),
                            id: order_id.value(),
                            reason: rejection_reason_str(*reason),
                        });
                    }
                }
                self.metrics.record_quotes(res.quotes.len() as u64);
                // Sample quotes 1-in-10 — busy live mode produces 1k+/sec.
                for q in &res.quotes {
                    self.quote_sample_counter = self.quote_sample_counter.wrapping_add(1);
                    if self.quote_sample_counter.is_multiple_of(10) {
                        self.telemetry.record(TelemetryEvent::Quote {
                            sym: q.symbol.to_string(),
                            side: quote_side_str(q.side),
                            px_c: q.price.cents(),
                            qty: q.quantity.value(),
                        });
                    }
                }
            }
            Err(_) => {
                self.metrics.record_rejects(1);
                self.symbols[idx].total_rejects =
                    self.symbols[idx].total_rejects.saturating_add(1);
                self.telemetry.record(TelemetryEvent::Reject {
                    sym: symbol.to_string(),
                    id: order_id.value(),
                    reason: "submit_error",
                });
            }
        }
    }

    fn handle_cancel_hint(&mut self, idx: usize) {
        let resting = &self.symbols[idx].resting_ids;
        if resting.is_empty() {
            return;
        }
        let cancels = self.symbols[idx].total_cancels;
        let id = resting[(cancels as usize) % resting.len()];
        let started = Instant::now();
        let ts = Ts::from_nanos(self.uptime().as_nanos() as u64);
        let symbol = self.symbols[idx].symbol;
        if self.market.cancel(symbol, id, ts).is_ok() {
            self.metrics.record_latency(Op::Cancel, started.elapsed());
            self.metrics.record_cancels(1);
            self.symbols[idx].total_cancels =
                self.symbols[idx].total_cancels.saturating_add(1);
        }
    }

    fn push_print(&mut self, idx: usize, f: FillEvent, aggressor: Side) {
        let tape = &mut self.symbols[idx].tape;
        if tape.len() >= 200 {
            tape.pop_back();
        }
        tape.push_front(TapePrint {
            symbol: f.symbol,
            price: f.price,
            quantity: f.quantity,
            aggressor,
            at: f.timestamp,
        });
    }

    fn refresh_resting_ids(&mut self, idx: usize) {
        let symbol = self.symbols[idx].symbol;
        let ids: Vec<OrderID> = match self.market.book(symbol) {
            Some(book) => {
                let mut v: Vec<OrderID> = Vec::new();
                for (_, lvl) in book.bid_levels() {
                    for o in lvl.iter() {
                        v.push(o.id());
                    }
                }
                for (_, lvl) in book.ask_levels() {
                    for o in lvl.iter() {
                        v.push(o.id());
                    }
                }
                v
            }
            None => Vec::new(),
        };
        self.symbols[idx].resting_ids = ids;
    }

    pub fn handle_action(&mut self, action: Action) -> bool {
        self.handle_action_with_raw(format!("{:?}", action), action)
    }

    /// Same as `handle_action` but accepts the raw keystroke string so
    /// telemetry can distinguish e.g. `Tab` from `s` even though both
    /// map to `Action::CycleSymbol`.
    pub fn handle_action_with_raw(&mut self, raw: String, action: Action) -> bool {
        let action_name = action_str(&action);
        let quit_requested = matches!(action, Action::Quit);
        match action {
            Action::Quit => return true,
            Action::TogglePause => {
                self.state = match self.state {
                    EngineState::Running => EngineState::Paused,
                    EngineState::Paused => EngineState::Running,
                };
            }
            Action::Reset => {
                self.market = Market::new();
                let count = self.symbols.len();
                for (i, s) in self.symbols.iter_mut().enumerate() {
                    self.market.register(s.symbol);
                    s.sim.reseed(RESET_SEED.wrapping_add(i as u64));
                    s.tape.clear();
                    s.mid_history.clear();
                    s.total_orders = 0;
                    s.total_fills = 0;
                    s.total_cancels = 0;
                    s.total_rejects = 0;
                    s.resting_ids.clear();
                }
                let _ = count;
                self.metrics = MetricsRegistry::new();
            }
            Action::SpeedUp => self.speed = (self.speed * 1.5).min(50.0),
            Action::SpeedDown => self.speed = (self.speed / 1.5).max(0.1),
            Action::CycleSymbol => {
                self.selected_idx = (self.selected_idx + 1) % self.symbols.len();
            }
            Action::None => {}
        }
        self.telemetry.record(TelemetryEvent::Key {
            raw,
            action: action_name,
            selected_after: Some(self.selected_idx),
            mode_after: Some(mode_str(&self.mode)),
        });
        quit_requested
    }
}

fn map_key(key: KeyEvent) -> Action {
    if key.kind != KeyEventKind::Press {
        return Action::None;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char(' ') => Action::TogglePause,
        KeyCode::Char('r') | KeyCode::Char('R') => Action::Reset,
        KeyCode::Char('+') | KeyCode::Char('=') => Action::SpeedUp,
        KeyCode::Char('-') | KeyCode::Char('_') => Action::SpeedDown,
        KeyCode::Tab | KeyCode::Char('s') | KeyCode::Char('S') => Action::CycleSymbol,
        _ => Action::None,
    }
}

pub fn run(seed: u64) -> Result<(), Box<dyn std::error::Error>> {
    let telemetry = match crate::telemetry::spawn_writer() {
        Ok((handle, path)) => {
            eprintln!("telemetry → {}", path.display());
            handle
        }
        Err(e) => {
            eprintln!("telemetry disabled: {e}");
            TelemetryHandle::noop()
        }
    };
    let mut terminal = setup_terminal()?;
    let result = run_loop(&mut terminal, App::new(seed, telemetry));
    restore_terminal(&mut terminal)?;
    result
}

/// Entry point used by the `--live coinbase` mode. The caller wires the
/// `App` with a feed receiver via `App::new_live` before passing in.
/// The caller is also responsible for spawning the telemetry writer and
/// embedding the handle in the `App`.
pub fn run_with_app(app: App) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = setup_terminal()?;
    let result = run_loop(&mut terminal, app);
    restore_terminal(&mut terminal)?;
    result
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn side_str(s: Side) -> &'static str {
    match s {
        Side::Buy => "Buy",
        Side::Sell => "Sell",
    }
}

fn quote_side_str(s: QuoteSide) -> &'static str {
    match s {
        QuoteSide::Bid => "Bid",
        QuoteSide::Ask => "Ask",
    }
}

fn rejection_reason_str(r: OrderRejectionReason) -> &'static str {
    match r {
        OrderRejectionReason::InvalidQuantity => "InvalidQuantity",
        OrderRejectionReason::InvalidPrice => "InvalidPrice",
        OrderRejectionReason::InvalidOrderId => "InvalidOrderId",
        OrderRejectionReason::SelfMatch => "SelfMatch",
        OrderRejectionReason::DuplicateOrderId => "DuplicateOrderId",
    }
}

fn action_str(a: &Action) -> &'static str {
    match a {
        Action::Quit => "Quit",
        Action::TogglePause => "TogglePause",
        Action::Reset => "Reset",
        Action::SpeedUp => "SpeedUp",
        Action::SpeedDown => "SpeedDown",
        Action::CycleSymbol => "CycleSymbol",
        Action::None => "None",
    }
}

fn mode_str(m: &Mode) -> &'static str {
    match m {
        Mode::Synthetic => "synthetic",
        Mode::Live { .. } => "live",
    }
}

fn push_ring(ring: &mut VecDeque<u64>, sample: u64) {
    ring.push_back(sample);
    if ring.len() > 60 {
        ring.pop_front();
    }
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut app: App,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_render = Instant::now();
    let mut last_sim = Instant::now();
    let mut last_step_us = 0u64;
    let mut last_actions = 0u32;
    let mut last_budget_left = 500u32;
    loop {
        if last_render.elapsed() >= RENDER_TICK {
            let render_start = Instant::now();
            terminal.draw(|frame| panes::render(frame, &app))?;
            let render_us = render_start.elapsed().as_micros() as u64;
            // Emit a Frame event (and FrameSlow if we exceeded budget).
            // The step counters are from the most-recent step() call
            // so the frame's profile is the full step+render pair.
            app.record_frame(last_step_us, render_us, last_actions, last_budget_left);
            last_render = Instant::now();
        }
        let since_sim = last_sim.elapsed();
        if since_sim >= SIM_TICK {
            let step_start = Instant::now();
            let (actions, budget_left) = app.step(since_sim.as_secs_f64());
            last_step_us = step_start.elapsed().as_micros() as u64;
            last_actions = actions;
            last_budget_left = budget_left;
            last_sim = Instant::now();
        }
        if event::poll(POLL_TICK)?
            && let Event::Key(key) = event::read()?
        {
            let raw = format!("{:?}", key.code);
            let action = map_key(key);
            if app.handle_action_with_raw(raw, action) {
                let uptime_ms = app.uptime().as_millis() as u64;
                app.telemetry.record(TelemetryEvent::Shutdown {
                    reason: "user_quit",
                    uptime_ms,
                });
                return Ok(());
            }
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
