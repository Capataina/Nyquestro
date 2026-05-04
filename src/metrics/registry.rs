//! `MetricsRegistry` — central observability hub.
//!
//! The registry owns per-operation latency histograms (HDR, autoresize) and
//! a [`CounterSet`] for rate metrics. The matching engine wrapper records
//! each call's wall-clock duration; the dashboard reads
//! [`MetricsRegistry::snapshot`] every render frame.

use std::time::Duration;

use hdrhistogram::Histogram;

use crate::metrics::counters::{CounterSet, CounterSnapshot};

/// Operations the engine reports timings for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    Submit,
    Match,
    Cancel,
}

impl Op {
    pub fn name(self) -> &'static str {
        match self {
            Op::Submit => "submit",
            Op::Match => "match",
            Op::Cancel => "cancel",
        }
    }

    pub const ALL: [Op; 3] = [Op::Submit, Op::Match, Op::Cancel];
}

#[derive(Debug)]
pub struct MetricsRegistry {
    submit_lat: Histogram<u64>,
    match_lat: Histogram<u64>,
    cancel_lat: Histogram<u64>,
    counters: CounterSet,
    started_at: std::time::Instant,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        // 1 ns lower bound, 1 hour upper, 3 sig figs. Autoresizes if overrun.
        let mk = || {
            let mut h = Histogram::<u64>::new_with_bounds(1, 60 * 60 * 1_000_000_000, 3)
                .expect("valid histogram bounds");
            h.auto(true);
            h
        };
        MetricsRegistry {
            submit_lat: mk(),
            match_lat: mk(),
            cancel_lat: mk(),
            counters: CounterSet::new(),
            started_at: std::time::Instant::now(),
        }
    }

    pub fn record_latency(&mut self, op: Op, d: Duration) {
        // Saturate at 1 nanosecond minimum so HDR's 1ns lower bound holds.
        let nanos = d.as_nanos().clamp(1, u64::MAX as u128) as u64;
        let h = match op {
            Op::Submit => &mut self.submit_lat,
            Op::Match => &mut self.match_lat,
            Op::Cancel => &mut self.cancel_lat,
        };
        let _ = h.record(nanos); // saturating record never fails after auto(true)
    }

    pub fn record_orders(&mut self, n: u64) {
        self.counters.orders.record(n);
    }

    pub fn record_fills(&mut self, n: u64) {
        self.counters.fills.record(n);
    }

    pub fn record_cancels(&mut self, n: u64) {
        self.counters.cancels.record(n);
    }

    pub fn record_rejects(&mut self, n: u64) {
        self.counters.rejects.record(n);
    }

    pub fn record_quotes(&mut self, n: u64) {
        self.counters.quotes.record(n);
    }

    pub fn uptime(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub fn snapshot(&self) -> RegistrySnapshot {
        RegistrySnapshot {
            submit: LatencySnapshot::from_hist(&self.submit_lat),
            match_op: LatencySnapshot::from_hist(&self.match_lat),
            cancel: LatencySnapshot::from_hist(&self.cancel_lat),
            counters: self.counters.snapshot(),
            uptime: self.uptime(),
        }
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LatencySnapshot {
    pub count: u64,
    pub p50_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
    pub p999_ns: u64,
    pub p9999_ns: u64,
    pub max_ns: u64,
    pub mean_ns: u64,
}

impl LatencySnapshot {
    fn from_hist(h: &Histogram<u64>) -> Self {
        let count = h.len();
        if count == 0 {
            return LatencySnapshot::default();
        }
        LatencySnapshot {
            count,
            p50_ns: h.value_at_quantile(0.50),
            p95_ns: h.value_at_quantile(0.95),
            p99_ns: h.value_at_quantile(0.99),
            p999_ns: h.value_at_quantile(0.999),
            p9999_ns: h.value_at_quantile(0.9999),
            max_ns: h.max(),
            mean_ns: h.mean() as u64,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RegistrySnapshot {
    pub submit: LatencySnapshot,
    pub match_op: LatencySnapshot,
    pub cancel: LatencySnapshot,
    pub counters: CounterSnapshot,
    pub uptime: Duration,
}
