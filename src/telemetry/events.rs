//! `TelemetryEvent` — every kind of structured event the dashboard
//! emits. Discriminated by `kind`; serde produces JSON with a flat
//! per-variant payload.

use serde::Serialize;

/// A telemetry event. Serialised as JSON with `kind` as the discriminant.
///
/// New variants are additive; bumping the schema version is reserved for
/// breaking changes (renaming fields, removing variants). See
/// `notes/telemetry-policy.md`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TelemetryEvent {
    /// Emitted once on app start.
    Startup {
        mode: &'static str,
        symbols: Vec<String>,
        term: (u16, u16),
        seed: Option<u64>,
    },

    /// Emitted on every keypress that resolves to an `Action`. `raw` is the
    /// stringified `KeyCode`; `action` is the discriminant of the parsed
    /// `Action`. Includes `selected_after` and `mode_after` so the
    /// resulting state is captured atomically with the cause.
    Key {
        raw: String,
        action: &'static str,
        selected_after: Option<usize>,
        mode_after: Option<&'static str>,
    },

    /// Emitted for every order submitted to the engine.
    Submit {
        sym: String,
        side: &'static str,
        px_c: u64,
        qty: u32,
        id: u64,
    },

    /// Emitted for every `FillEvent` produced.
    Fill {
        sym: String,
        px_c: u64,
        qty: u32,
        buyer: u64,
        seller: u64,
    },

    /// Emitted for every successful cancel.
    Cancel {
        sym: String,
        id: u64,
        remaining: u32,
    },

    /// Emitted for every `OrderEvent::Rejected`.
    Reject {
        sym: String,
        id: u64,
        reason: &'static str,
    },

    /// Emitted (sampled at 1-in-10) for `QuoteEvent`s. Sampled because
    /// busy live mode produces 1000+/sec; full audit isn't required for
    /// quote events, the periodic state snapshot covers the relevant
    /// information.
    Quote {
        sym: String,
        side: &'static str,
        px_c: u64,
        qty: u32,
    },

    /// Per render-tick profiling.
    Frame {
        step_us: u64,
        render_us: u64,
        actions: u32,
        budget_left: u32,
    },

    /// Emitted when `step_us + render_us > 33_000`. `reason` is a heuristic
    /// classification: `"per_frame_budget_exhausted"` (action drain hit
    /// the 500-cap), `"render_blocked"` (render dominated), or
    /// `"step_dominated"` (engine work dominated).
    FrameSlow {
        step_us: u64,
        render_us: u64,
        actions: u32,
        reason: &'static str,
    },

    /// Per-pane render timing, sampled at 1Hz.
    PaneRender {
        pane: &'static str,
        us: u64,
    },

    /// Emitted every 1 second per `Op`.
    Latency {
        op: &'static str,
        count: u64,
        p50_ns: u64,
        p99_ns: u64,
        p999_ns: u64,
        p9999_ns: u64,
        max_ns: u64,
    },

    /// Emitted every 1 second.
    Throughput {
        orders_1s: u64,
        fills_1s: u64,
        cancels_1s: u64,
        rejects_1s: u64,
        quotes_1s: u64,
    },

    /// Emitted every 1 second per symbol.
    BookState {
        sym: String,
        levels_bid: usize,
        levels_ask: usize,
        depth_bid: u32,
        depth_ask: u32,
        ofi: f64,
        microprice_c: Option<u64>,
        spread_c: Option<u64>,
    },

    /// Emitted on every Coinbase L2 snapshot received. `raw_*` are the
    /// counts before our cap; `capped` is the per-side cap value.
    Snapshot {
        sym: String,
        raw_bids: usize,
        raw_asks: usize,
        capped: usize,
    },

    /// Live-feed status messages (connect, subscribe, reconnect, etc.).
    FeedStatus { msg: String },

    /// Live-feed errors (parse failures, connection drops).
    FeedError { msg: String },

    /// Emitted periodically when the channel has dropped events.
    DroppedEvents { count: u64 },

    /// Emitted on terminal resize.
    Resize { cols: u16, rows: u16 },

    /// Emitted on clean shutdown.
    Shutdown {
        reason: &'static str,
        uptime_ms: u64,
    },
}
