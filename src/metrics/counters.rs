//! Monotonic event counters.

use crate::metrics::windows::{WindowSnapshot, WindowedCounter};

#[derive(Debug, Clone, Default)]
pub struct CounterSet {
    pub orders: WindowedCounter,
    pub fills: WindowedCounter,
    pub cancels: WindowedCounter,
    pub rejects: WindowedCounter,
    pub quotes: WindowedCounter,
}

impl CounterSet {
    pub fn new() -> Self {
        CounterSet::default()
    }

    pub fn snapshot(&self) -> CounterSnapshot {
        CounterSnapshot {
            orders: self.orders.snapshot(),
            fills: self.fills.snapshot(),
            cancels: self.cancels.snapshot(),
            rejects: self.rejects.snapshot(),
            quotes: self.quotes.snapshot(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CounterSnapshot {
    pub orders: WindowSnapshot,
    pub fills: WindowSnapshot,
    pub cancels: WindowSnapshot,
    pub rejects: WindowSnapshot,
    pub quotes: WindowSnapshot,
}
