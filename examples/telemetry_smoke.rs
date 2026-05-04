//! Verifies the telemetry writer end-to-end. Spawns the writer, emits
//! one of every event variant, sleeps long enough for the writer's flush
//! cadence to land them on disk, then prints the file path + first 5
//! lines + line count.

use std::thread;
use std::time::Duration;

use nyquestro::telemetry::{spawn_writer, TelemetryEvent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (handle, path) = spawn_writer()?;
    println!("telemetry → {}", path.display());

    handle.record(TelemetryEvent::Startup {
        mode: "smoke",
        symbols: vec!["BTC-USD".into(), "ETH-USD".into()],
        term: (200, 50),
        seed: Some(42),
    });
    handle.record(TelemetryEvent::Key {
        raw: "Tab".into(),
        action: "CycleSymbol",
        selected_after: Some(1),
        mode_after: Some("live"),
    });
    handle.record(TelemetryEvent::Submit {
        sym: "BTC-USD".into(),
        side: "Buy",
        px_c: 8_012_345,
        qty: 50_000,
        id: 1_000_000_000_001,
    });
    handle.record(TelemetryEvent::Fill {
        sym: "BTC-USD".into(),
        px_c: 8_012_345,
        qty: 12_500,
        buyer: 1_000_000_000_001,
        seller: 1_000_000_000_002,
    });
    handle.record(TelemetryEvent::Frame {
        step_us: 2300,
        render_us: 4100,
        actions: 12,
        budget_left: 488,
    });
    handle.record(TelemetryEvent::FrameSlow {
        step_us: 47200,
        render_us: 300,
        actions: 500,
        reason: "per_frame_budget_exhausted",
    });
    handle.record(TelemetryEvent::Latency {
        op: "submit",
        count: 13_038,
        p50_ns: 1600,
        p99_ns: 9100,
        p999_ns: 18_600,
        p9999_ns: 39_300,
        max_ns: 41_800,
    });
    handle.record(TelemetryEvent::FeedStatus {
        msg: "subscribed: BTC-USD on level2".into(),
    });
    handle.record(TelemetryEvent::Shutdown {
        reason: "smoke_test",
        uptime_ms: 60_000,
    });

    // Wait for the writer to flush. The writer flushes every 200ms and
    // also on disconnection; we drop the handle by ending the function.
    thread::sleep(Duration::from_millis(500));
    drop(handle);
    thread::sleep(Duration::from_millis(200));

    let contents = std::fs::read_to_string(&path)?;
    let line_count = contents.lines().count();
    println!("\nlines written: {line_count}");
    println!("\n── first 3 lines ──");
    for line in contents.lines().take(3) {
        println!("{line}");
    }
    println!("\n── last 2 lines ──");
    for line in contents.lines().rev().take(2).collect::<Vec<_>>().iter().rev() {
        println!("{line}");
    }

    if line_count == 9 {
        println!("\n✅ all 9 events recorded");
    } else {
        println!("\n⚠ expected 9 events, got {line_count}");
    }
    Ok(())
}
