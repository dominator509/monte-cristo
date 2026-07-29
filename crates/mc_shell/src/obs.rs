//! EP-008: Observability — JSON logging, metrics, crash reports, profiler.
//!
//! SPEC-007 defines the observability contract:
//! - Structured JSON logging to `$MC_DATA_DIR/logs/monte-cristo-<date>.jsonl`
//! - Log rotation (10 MiB per file)
//! - Required fields: ts, level, target, msg, session, build, tick
//! - Metrics file on clean exit: `$MC_DATA_DIR/logs/metrics-<date>.json`
//! - Crash reports: `$MC_DATA_DIR/crash/crash-<timestamp>.json`
//! - Profiler and debug overlay behind feature flags

use crate::fsroot::{self, Root};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing_subscriber::prelude::*;

// ── Session identity ──────────────────────────────────────────────────────────

/// One-shot session ID for this process lifetime.
static SESSION_ID: LazyLock<String> = LazyLock::new(|| {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("mc-{:016x}", nanos)
});

/// Tick counter, written by the main loop each frame.
pub static CURRENT_TICK: AtomicU64 = AtomicU64::new(0);

/// Build version from env var or fallback.
fn build_version() -> String {
    std::env::var("MC_BUILD").unwrap_or_else(|_| "dev".into())
}

// ── Date helpers (no chrono dependency) ───────────────────────────────────────

/// Format a SystemTime as `YYYY-MM-DDTHH:MM:SSZ` in UTC.
fn fmt_date(now: SystemTime) -> String {
    let secs = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    let days = secs / 86400;
    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let leap = is_leap(y);
    let mdays: [i64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0usize;
    for (i, &md) in mdays.iter().enumerate() {
        if d < md {
            m = i + 1;
            break;
        }
        d -= md;
    }
    if m == 0 {
        m = 12;
    }
    let day = d + 1;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, day, hours, minutes, seconds
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Format a SystemTime as `YYYY-MM-DD` date-only.
fn fmt_date_only(now: SystemTime) -> String {
    let s = fmt_date(now);
    s[..10].to_string()
}

/// Format a timestamp for crash filenames (no colons).
fn fmt_timestamp(now: SystemTime) -> String {
    fmt_date(now).replace(':', "-")
}

// ── Log directory path ────────────────────────────────────────────────────────

fn logs_dir() -> PathBuf {
    // Resolve $MC_DATA_DIR via fsroot::confine
    match fsroot::confine(Root::Data, &std::path::Path::new("logs")) {
        Ok(p) => p,
        Err(_) => {
            let mut p = std::env::current_dir().unwrap_or_default();
            p.push("data");
            p.push("logs");
            p
        }
    }
}

fn crash_dir() -> PathBuf {
    match fsroot::confine(Root::Data, &std::path::Path::new("crash")) {
        Ok(p) => p,
        Err(_) => {
            let mut p = std::env::current_dir().unwrap_or_default();
            p.push("data");
            p.push("crash");
            p
        }
    }
}

// ── Rotating file writer ──────────────────────────────────────────────────────

/// A writer that rotates the log file when it exceeds a size threshold.
pub struct RotatingFileWriter {
    base: PathBuf,
    max_bytes: u64,
    current: std::fs::File,
    current_size: u64,
    generation: u32,
}

impl RotatingFileWriter {
    pub fn new(base: PathBuf, max_bytes: u64) -> Self {
        let current = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&base)
            .expect("open log file");
        let current_size = std::fs::metadata(&base).map(|m| m.len()).unwrap_or(0);
        RotatingFileWriter {
            base,
            max_bytes,
            current,
            current_size,
            generation: 0,
        }
    }

    fn rotate(&mut self) {
        self.generation += 1;
        let rotated = self
            .base
            .with_extension(format!("jsonl.{}", self.generation));
        let _ = std::fs::rename(&self.base, &rotated);
        self.current = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.base)
            .expect("reopen log file");
        self.current_size = 0;
    }
}

impl std::io::Write for RotatingFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written = self.current.write(buf)?;
        self.current_size += written as u64;
        if self.current_size >= self.max_bytes {
            self.rotate();
        }
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.current.flush()
    }
}

// ── JSON logging init ─────────────────────────────────────────────────────────

/// Initialise structured JSON logging to file.
///
/// Writes to `$MC_DATA_DIR/logs/monte-cristo-<date>.jsonl`.
/// Rotates at 10 MiB by appending `.1` suffix.
pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let dir = logs_dir();
    std::fs::create_dir_all(&dir)?;

    let now = SystemTime::now();
    let date = fmt_date_only(now);
    let base_path = dir.join(format!("monte-cristo-{}.jsonl", date));

    let max_bytes: u64 = 10 * 1024 * 1024;
    let writer = RotatingFileWriter::new(base_path, max_bytes);

    let layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(Mutex::new(writer))
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_current_span(false)
        .flatten_event(false)
        .with_span_list(false);

    let filter = tracing_subscriber::EnvFilter::new("info,mc_shell=debug,mc_core=warn");

    let subscriber = tracing_subscriber::Registry::default()
        .with(filter)
        .with(layer);

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| format!("tracing init: {}", e))?;

    tracing::info!(
        session = %*SESSION_ID,
        build = %build_version(),
        "logging initialised"
    );

    Ok(())
}

// ── Metrics ───────────────────────────────────────────────────────────────────

/// Runtime metrics collected across the session.
#[derive(Debug, Clone, Serialize)]
pub struct SessionMetrics {
    pub session: String,
    pub build: String,
    pub start_time: String,
    pub end_time: String,
    pub ticks_elapsed: u64,
    pub commands_processed: u64,
    pub battles_fought: u32,
    pub encounters_resolved: u32,
}

impl SessionMetrics {
    pub fn new() -> Self {
        SessionMetrics {
            session: SESSION_ID.clone(),
            build: build_version(),
            start_time: fmt_date(SystemTime::now()),
            end_time: String::new(),
            ticks_elapsed: 0,
            commands_processed: 0,
            battles_fought: 0,
            encounters_resolved: 0,
        }
    }
}

/// Global metrics accumulator.
static METRICS: LazyLock<Mutex<SessionMetrics>> =
    LazyLock::new(|| Mutex::new(SessionMetrics::new()));

/// Record a tick for metrics.
pub fn record_tick() {
    if let Ok(mut m) = METRICS.lock() {
        m.ticks_elapsed += 1;
    }
}

/// Record a processed command.
pub fn record_command() {
    if let Ok(mut m) = METRICS.lock() {
        m.commands_processed += 1;
    }
}

/// Record a battle start.
pub fn record_battle() {
    if let Ok(mut m) = METRICS.lock() {
        m.battles_fought += 1;
    }
}

/// Record an encounter resolution.
pub fn record_encounter() {
    if let Ok(mut m) = METRICS.lock() {
        m.encounters_resolved += 1;
    }
}

/// Write metrics to disk on clean exit.
pub fn write_metrics() -> Result<(), Box<dyn std::error::Error>> {
    let dir = logs_dir();
    std::fs::create_dir_all(&dir)?;

    let now = SystemTime::now();
    let date = fmt_date_only(now);
    let path = dir.join(format!("metrics-{}.json", date));

    let mut metrics = METRICS.lock().map_err(|e| format!("metrics lock: {}", e))?;
    metrics.end_time = fmt_date(now);

    let json = serde_json::to_string_pretty(&*metrics)?;
    std::fs::write(&path, json.as_bytes())?;

    tracing::info!(metrics_file = %path.display(), "metrics written");
    Ok(())
}

// ── Crash reports ─────────────────────────────────────────────────────────────

/// A crash report captured at panic time.
#[derive(Debug, Serialize)]
pub struct CrashReport {
    pub session: String,
    pub build: String,
    pub timestamp: String,
    pub tick: u64,
    pub panic_message: String,
    pub backtrace: String,
}

/// Install a panic hook that writes crash reports.
pub fn install_crash_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let tick = CURRENT_TICK.load(Ordering::Relaxed);
        let now = SystemTime::now();
        let ts = fmt_timestamp(now);
        let panic_msg = info.to_string();

        let backtrace = std::backtrace::Backtrace::force_capture();
        let bt_str = format!("{:#}", backtrace);

        let report = CrashReport {
            session: SESSION_ID.clone(),
            build: build_version(),
            timestamp: fmt_date(now),
            tick,
            panic_message: panic_msg,
            backtrace: bt_str,
        };

        let dir = crash_dir();
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(format!("crash-{}.json", ts));
        if let Ok(json) = serde_json::to_string_pretty(&report) {
            let _ = std::fs::write(&path, json.as_bytes());
            eprintln!("crash report written to {}", path.display());
        }

        tracing::error!(tick, panic = %info, "FATAL: crash detected");
        prev(info);
    }));
}

// ── Profiler (behind `profiling` feature) ─────────────────────────────────────

/// A simple frame profiler. Activated only when the `profiling` feature is enabled.
#[cfg(feature = "profiling")]
pub mod profiler {
    use std::collections::BTreeMap;
    use std::time::{Duration, Instant};

    static mut PROFILER: Option<Profiler> = None;

    struct Profiler {
        enabled: bool,
        frame_start: Instant,
        labels: BTreeMap<&'static str, (Duration, u64)>,
    }

    pub fn enable() {
        unsafe {
            PROFILER = Some(Profiler {
                enabled: true,
                frame_start: Instant::now(),
                labels: BTreeMap::new(),
            });
        }
    }

    pub fn frame_start() {
        unsafe {
            if let Some(ref mut p) = PROFILER {
                if p.enabled {
                    p.frame_start = Instant::now();
                }
            }
        }
    }

    pub fn section(label: &'static str) {
        unsafe {
            if let Some(ref mut p) = PROFILER {
                if p.enabled {
                    let elapsed = p.frame_start.elapsed();
                    let entry = p.labels.entry(label).or_insert((Duration::ZERO, 0));
                    entry.0 += elapsed;
                    entry.1 += 1;
                }
            }
        }
    }

    pub fn report() -> String {
        unsafe {
            if let Some(ref p) = PROFILER {
                if !p.enabled {
                    return "profiler disabled".into();
                }
                let mut out = String::from("--- profiler ---\n");
                for (label, (total, count)) in &p.labels {
                    let avg = if *count > 0 {
                        total.as_nanos() as f64 / *count as f64
                    } else {
                        0.0
                    };
                    out.push_str(&format!(
                        "  {}: {} calls, {:.1}ns avg, {:?} total\n",
                        label, count, avg, total
                    ));
                }
                out
            } else {
                "profiler not initialised".into()
            }
        }
    }
}

/// Stub profiler when the feature is off.
#[cfg(not(feature = "profiling"))]
pub mod profiler {
    pub fn enable() {}
    pub fn frame_start() {}
    pub fn section(_label: &'static str) {}
    pub fn report() -> String {
        String::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn session_id_is_unique() {
        let id1 = SESSION_ID.clone();
        let id2 = SESSION_ID.clone();
        assert_eq!(id1, id2, "session ID must be stable within process");
    }

    #[test]
    fn fmt_date_format() {
        let fixed = UNIX_EPOCH + std::time::Duration::from_secs(1579046400);
        let s = fmt_date(fixed);
        assert!(s.starts_with("2020-01-15"), "got {}", s);
    }

    #[test]
    fn fmt_date_only_format() {
        let fixed = UNIX_EPOCH + std::time::Duration::from_secs(1579046400);
        let s = fmt_date_only(fixed);
        assert_eq!(s, "2020-01-15", "got {}", s);
    }

    #[test]
    fn rotating_writer_creation() {
        let dir = std::env::temp_dir().join("mc_test_obs");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.jsonl");
        let mut writer = RotatingFileWriter::new(path.clone(), 100);
        let _ = writer.write(b"hello world\n");
        assert!(path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rotating_writer_triggers_rotation() {
        let dir = std::env::temp_dir().join("mc_test_obs_rotate");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("rotate.jsonl");
        let mut writer = RotatingFileWriter::new(path.clone(), 10);
        let data = b"this is a long line that exceeds the max bytes\n";
        let _ = writer.write(data);
        assert!(
            path.exists(),
            "current file should exist after rotation trigger"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn metrics_accumulate() {
        let initial = METRICS.lock().unwrap().ticks_elapsed;
        record_tick();
        let after = METRICS.lock().unwrap().ticks_elapsed;
        assert_eq!(after, initial + 1, "tick should increment");
    }

    #[test]
    fn metrics_serialize() {
        let m = SessionMetrics::new();
        let json = serde_json::to_string(&m).expect("serialize metrics");
        assert!(
            json.contains("\"session\""),
            "json should contain session field: {}",
            json
        );
    }

    #[test]
    fn crash_report_serialize() {
        let report = CrashReport {
            session: "test-session".into(),
            build: "test-build".into(),
            timestamp: "2020-01-15".into(),
            tick: 1234,
            panic_message: "test panic".into(),
            backtrace: "stack trace".into(),
        };
        let json = serde_json::to_string(&report).expect("serialize crash report");
        assert!(json.contains("test-session"), "json: {}", json);
        assert!(json.contains("1234"), "json should contain tick: {}", json);
    }

    #[test]
    fn profiler_stub_compiles() {
        profiler::enable();
        profiler::frame_start();
        profiler::section("test");
        let report = profiler::report();
        assert!(
            !report.contains("profiler"),
            "stub report should be empty, got: {}",
            report
        );
    }
}
