//! Local-only flight recorder for the dashboard run.
//!
//! Every keystroke, engine event, frame timing, and periodic state
//! snapshot is recorded to a JSONL file in the platform-canonical
//! application-data directory:
//!
//! - macOS:   `~/Library/Application Support/Nyquestro/last-run.jsonl`
//! - Linux:   `~/.local/share/nyquestro/last-run.jsonl`
//! - Windows: `%LOCALAPPDATA%\Nyquestro\last-run.jsonl`
//!
//! Resolved at runtime via [`dirs::data_local_dir`]. Truncated on every
//! startup so there is exactly one run on disk at any moment. Never
//! uploaded, never aggregated, never analytics. The user's own audit
//! trail.
//!
//! Threading: emitting code calls [`TelemetryHandle::record`] from any
//! thread. The handle pushes events through a `std::sync::mpsc::SyncSender`
//! into a dedicated writer thread that batches writes through a
//! `BufWriter<File>`. Backpressure: when the bounded channel fills, the
//! sender drops the event and increments an atomic counter; the writer
//! periodically flushes a `dropped_events` summary so reviewers know
//! when the channel was saturated.
//!
//! Telemetry must never block the dashboard's main loop. Drop-on-full
//! is the structural guarantee.

pub mod events;
pub mod writer;

pub use events::TelemetryEvent;
pub use writer::{spawn_writer, TelemetryHandle};
