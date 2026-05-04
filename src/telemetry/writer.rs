//! Writer thread + handle for the telemetry recorder.
//!
//! The handle is a cheap-clone struct holding a `SyncSender`. Emitting
//! code calls `record(event)` which `try_send`s into a bounded channel.
//! On `Full`, the event is dropped and an atomic counter is incremented;
//! the writer periodically flushes a `DroppedEvents` summary so the
//! file records when the channel was saturated.
//!
//! The main loop never blocks on telemetry. This is the structural
//! guarantee that the dashboard cannot freeze because of disk I/O — a
//! lesson from the Coinbase-snapshot incident where unbounded action
//! processing froze the input poll.

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use chrono::Utc;
use serde::Serialize;

use crate::telemetry::events::TelemetryEvent;

/// Bounded-channel capacity. ~10s of bursty 1k-events/sec headroom; well
/// above realistic synchronous throughput. Tuned to fit one Coinbase L2
/// snapshot's worth of bridge actions plus normal flow.
const CHANNEL_CAPACITY: usize = 8192;

/// BufWriter capacity. 64 KiB lets a typical batch (~hundreds of small
/// JSON lines) flush in one syscall.
const BUFFER_CAPACITY: usize = 64 * 1024;

/// Schema version. Bump on breaking changes (renamed fields, removed
/// variants). Additive changes (new variants, new fields) keep `v: 1`.
const SCHEMA_VERSION: u8 = 1;

/// Writer-side flush cadence; ensures the file is at most this stale on
/// disk when reading mid-run.
const FLUSH_INTERVAL: Duration = Duration::from_millis(200);

/// Writer-side dropped-events summary cadence.
const DROPPED_REPORT_INTERVAL: Duration = Duration::from_secs(1);

/// Cheap-clone handle to the writer thread. Construct once via
/// [`spawn_writer`] and pass copies wherever telemetry needs to flow.
#[derive(Clone)]
pub struct TelemetryHandle {
    tx: SyncSender<TelemetryEvent>,
    dropped: Arc<AtomicU64>,
}

impl TelemetryHandle {
    /// Record an event. Non-blocking — drops the event if the channel
    /// is full and increments the dropped-counter for periodic summary.
    pub fn record(&self, event: TelemetryEvent) {
        if self.tx.try_send(event).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// A no-op handle for tests and for binaries that explicitly opt out
    /// of telemetry. Both `record` and `clone` work without panicking;
    /// no file is written.
    pub fn noop() -> Self {
        let (tx, rx) = sync_channel::<TelemetryEvent>(1);
        thread::spawn(move || {
            // Drain forever, drop on the floor.
            while rx.recv().is_ok() {}
        });
        TelemetryHandle {
            tx,
            dropped: Arc::new(AtomicU64::new(0)),
        }
    }
}

/// Spawn the writer thread. Returns the handle plus the resolved file
/// path (so the caller can log "telemetry → /path/to/last-run.jsonl" on
/// startup).
///
/// On any I/O failure (cannot create directory, cannot truncate file)
/// the function returns `Err` with the underlying error. Callers should
/// fall back to `TelemetryHandle::noop()` rather than crashing.
pub fn spawn_writer() -> std::io::Result<(TelemetryHandle, PathBuf)> {
    let path = resolve_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // O_TRUNC | O_CREATE | O_WRONLY: wipe the previous run.
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;

    let (tx, rx) = sync_channel::<TelemetryEvent>(CHANNEL_CAPACITY);
    let dropped = Arc::new(AtomicU64::new(0));
    let dropped_writer = Arc::clone(&dropped);

    thread::Builder::new()
        .name("nyq-telemetry".to_string())
        .spawn(move || writer_loop(file, rx, dropped_writer))
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    Ok((TelemetryHandle { tx, dropped }, path))
}

fn resolve_path() -> std::io::Result<PathBuf> {
    match dirs::data_local_dir() {
        Some(base) => Ok(base.join("Nyquestro").join("last-run.jsonl")),
        None => Err(std::io::Error::other(
            "could not resolve platform data-local directory",
        )),
    }
}

fn writer_loop(file: File, rx: Receiver<TelemetryEvent>, dropped: Arc<AtomicU64>) {
    let mut writer = BufWriter::with_capacity(BUFFER_CAPACITY, file);
    let mut last_flush = Instant::now();
    let mut last_dropped_report = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                let _ = write_event(&mut writer, &event);
            }
            Err(RecvTimeoutError::Timeout) => { /* fall through to flush + dropped check */ }
            Err(RecvTimeoutError::Disconnected) => break,
        }

        if last_flush.elapsed() >= FLUSH_INTERVAL {
            let _ = writer.flush();
            last_flush = Instant::now();
        }
        if last_dropped_report.elapsed() >= DROPPED_REPORT_INTERVAL {
            let n = dropped.swap(0, Ordering::Relaxed);
            if n > 0 {
                let _ = write_event(&mut writer, &TelemetryEvent::DroppedEvents { count: n });
            }
            last_dropped_report = Instant::now();
        }
    }
    let _ = writer.flush();
}

#[derive(Serialize)]
struct LoggedEvent<'a> {
    v: u8,
    t: String,
    #[serde(flatten)]
    inner: &'a TelemetryEvent,
}

fn write_event(writer: &mut BufWriter<File>, event: &TelemetryEvent) -> std::io::Result<()> {
    let logged = LoggedEvent {
        v: SCHEMA_VERSION,
        t: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
        inner: event,
    };
    serde_json::to_writer(&mut *writer, &logged)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    writer.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_handle_accepts_records_without_panicking() {
        let h = TelemetryHandle::noop();
        h.record(TelemetryEvent::Startup {
            mode: "test",
            symbols: vec!["TEST".to_string()],
            term: (200, 50),
            seed: Some(42),
        });
        h.record(TelemetryEvent::Frame {
            step_us: 100,
            render_us: 200,
            actions: 5,
            budget_left: 495,
        });
    }

    #[test]
    fn handle_clones_cheaply() {
        let h = TelemetryHandle::noop();
        let h2 = h.clone();
        let h3 = h2.clone();
        // Each clone independently records.
        h.record(TelemetryEvent::Resize { cols: 200, rows: 50 });
        h2.record(TelemetryEvent::Resize { cols: 100, rows: 30 });
        h3.record(TelemetryEvent::Resize { cols: 80, rows: 24 });
    }

    #[test]
    fn writer_truncates_and_writes() {
        let tmp = std::env::temp_dir().join(format!("nyq-test-{}.jsonl", std::process::id()));
        // Pre-fill with garbage to verify truncation.
        std::fs::write(&tmp, b"old garbage line\n").unwrap();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)
            .unwrap();
        let mut writer = BufWriter::new(file);
        write_event(
            &mut writer,
            &TelemetryEvent::Startup {
                mode: "test",
                symbols: vec!["AAPL".to_string()],
                term: (200, 50),
                seed: Some(7),
            },
        )
        .unwrap();
        writer.flush().unwrap();
        let contents = std::fs::read_to_string(&tmp).unwrap();
        assert!(contents.contains("\"v\":1"));
        assert!(contents.contains("\"kind\":\"startup\""));
        assert!(contents.contains("\"mode\":\"test\""));
        assert!(!contents.contains("garbage"));
        let _ = std::fs::remove_file(&tmp);
    }
}
