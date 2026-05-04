//! Sliding-window event counter.
//!
//! Records `(timestamp, count)` pairs in a deque. Snapshots return totals
//! for the last 1s / 10s / 60s / 300s. Older entries are aged out lazily on
//! every record/snapshot to keep memory bounded.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct WindowedCounter {
    /// (timestamp, count). Oldest at the front.
    samples: VecDeque<(Instant, u64)>,
    /// Anything older than this is pruned on every operation.
    retention: Duration,
}

impl WindowedCounter {
    pub fn new() -> Self {
        WindowedCounter {
            samples: VecDeque::new(),
            // 5 minutes — the longest window we display.
            retention: Duration::from_secs(300),
        }
    }

    pub fn record(&mut self, count: u64) {
        let now = Instant::now();
        self.prune(now);
        self.samples.push_back((now, count));
    }

    /// Sum of counts within the last `window` from `now`.
    pub fn sum_within(&self, now: Instant, window: Duration) -> u64 {
        let cutoff = now.checked_sub(window).unwrap_or(now);
        self.samples
            .iter()
            .rev()
            .take_while(|(t, _)| *t >= cutoff)
            .map(|(_, c)| *c)
            .sum()
    }

    pub fn snapshot(&self) -> WindowSnapshot {
        let now = Instant::now();
        WindowSnapshot {
            last_1s: self.sum_within(now, Duration::from_secs(1)),
            last_10s: self.sum_within(now, Duration::from_secs(10)),
            last_1min: self.sum_within(now, Duration::from_secs(60)),
            last_5min: self.sum_within(now, Duration::from_secs(300)),
        }
    }

    fn prune(&mut self, now: Instant) {
        let cutoff = now.checked_sub(self.retention).unwrap_or(now);
        while let Some((t, _)) = self.samples.front() {
            if *t < cutoff {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }
}

impl Default for WindowedCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WindowSnapshot {
    pub last_1s: u64,
    pub last_10s: u64,
    pub last_1min: u64,
    pub last_5min: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_then_snapshot_within_1s() {
        let mut c = WindowedCounter::new();
        c.record(3);
        c.record(2);
        let s = c.snapshot();
        assert_eq!(s.last_1s, 5);
        assert_eq!(s.last_10s, 5);
    }

    #[test]
    fn empty_snapshot_is_zero() {
        let c = WindowedCounter::new();
        let s = c.snapshot();
        assert_eq!(s.last_1s, 0);
        assert_eq!(s.last_5min, 0);
    }
}
