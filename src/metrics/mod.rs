//! Engine observability — latency histograms and rate counters.
//!
//! Every observable operation (`submit`, `match`, `cancel`) records its
//! wall-clock duration into an [`hdrhistogram::Histogram`]. Order/fill/cancel
//! rates are tracked as monotonic counters with rolling 1s/10s/1min/5min
//! windows derived on snapshot.
//!
//! The whole registry is single-threaded — the TUI runs the simulator and
//! the engine on one thread, so we don't need atomics for the MVP.

pub mod counters;
pub mod registry;
pub mod windows;

pub use counters::CounterSnapshot;
pub use registry::{MetricsRegistry, Op, RegistrySnapshot};
pub use windows::WindowedCounter;
