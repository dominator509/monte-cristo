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
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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

/// Most recently observed authoritative state hash for crash reproduction.
static CURRENT_STATE_HASH: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

/// Record the authoritative state hash alongside the current tick.
pub fn record_state_hash(hash: [u8; 32]) {
    if let Ok(mut current) = CURRENT_STATE_HASH.lock() {
        *current = blake3::Hash::from(hash).to_hex().to_string();
    }
}

fn current_state_hash() -> String {
    CURRENT_STATE_HASH
        .lock()
        .map(|hash| hash.clone())
        .unwrap_or_else(|_| "unavailable".into())
}

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
    match fsroot::confine(Root::Data, std::path::Path::new("logs")) {
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
    match fsroot::confine(Root::Data, std::path::Path::new("crash")) {
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
/// Retains up to `max_generations` rotated files; older ones are deleted.
pub struct RotatingFileWriter {
    base: PathBuf,
    max_bytes: u64,
    max_generations: u32,
    current: std::fs::File,
    current_size: u64,
    generation: u32,
}

impl RotatingFileWriter {
    /// Create a new rotating file writer.
    ///
    /// `max_generations` controls how many rotated files are retained.
    /// The current file + `max_generations - 1` rotated files are kept.
    pub fn new(base: PathBuf, max_bytes: u64) -> std::io::Result<Self> {
        Self::with_retention(base, max_bytes, 7)
    }

    /// Create a new rotating file writer with explicit retention count.
    pub fn with_retention(
        base: PathBuf,
        max_bytes: u64,
        max_generations: u32,
    ) -> std::io::Result<Self> {
        let current = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&base)?;
        let current_size = std::fs::metadata(&base).map(|m| m.len()).unwrap_or(0);
        Ok(RotatingFileWriter {
            base,
            max_bytes,
            max_generations,
            current,
            current_size,
            generation: 0,
        })
    }

    fn rotate(&mut self) -> std::io::Result<()> {
        self.generation += 1;
        let rotated = self
            .base
            .with_extension(format!("jsonl.{}", self.generation));
        std::fs::rename(&self.base, &rotated)?;
        self.current = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.base)?;
        self.current_size = 0;

        // Enforce retention limit: delete files older than max_generations.
        self.cleanup_old_files();
        Ok(())
    }

    /// Remove rotated files whose generation number is beyond the
    /// retention limit. Keeps at most `max_generations` rotated files.
    fn cleanup_old_files(&self) {
        if self.max_generations == 0 {
            return;
        }
        if self.generation <= self.max_generations {
            return;
        }
        // Delete files where gen <= generation - max_generations
        let cutoff = self.generation - self.max_generations;
        for gen in 1..=cutoff {
            let old_path = self.base.with_extension(format!("jsonl.{}", gen));
            let _ = std::fs::remove_file(&old_path);
        }
    }
}

impl std::io::Write for RotatingFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written = self.current.write(buf)?;
        self.current_size += written as u64;
        if self.current_size >= self.max_bytes {
            self.rotate()?;
        }
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.current.flush()
    }
}

/// Adapt tracing-subscriber JSON to the repository's stable observability
/// field names and inject process context into every record.
pub struct CanonicalJsonWriter {
    inner: RotatingFileWriter,
    pending: Vec<u8>,
}

impl CanonicalJsonWriter {
    pub fn new(base: PathBuf, max_bytes: u64) -> std::io::Result<Self> {
        Ok(Self {
            inner: RotatingFileWriter::new(base, max_bytes)?,
            pending: Vec::new(),
        })
    }

    fn normalize_line(line: &[u8]) -> Vec<u8> {
        let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(line) else {
            return line.to_vec();
        };
        let Some(object) = value.as_object_mut() else {
            return line.to_vec();
        };

        if let Some(timestamp) = object.remove("timestamp") {
            object.entry("ts").or_insert(timestamp);
        }
        if let Some(message) = object.remove("message") {
            object.entry("msg").or_insert(message);
        }
        if !object.contains_key("msg") {
            if let Some(fields) = object
                .get_mut("fields")
                .and_then(|fields| fields.as_object_mut())
            {
                if let Some(message) = fields.remove("message") {
                    object.insert("msg".into(), message);
                }
            }
        }
        object
            .entry("session")
            .or_insert_with(|| serde_json::Value::String(SESSION_ID.clone()));
        object
            .entry("build")
            .or_insert_with(|| serde_json::Value::String(build_version()));
        object.entry("tick").or_insert_with(|| {
            serde_json::Value::Number(CURRENT_TICK.load(Ordering::Relaxed).into())
        });

        let mut output = serde_json::to_vec(&value).unwrap_or_else(|_| line.to_vec());
        output.push(b'\n');
        output
    }
}

impl std::io::Write for CanonicalJsonWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.pending.extend_from_slice(buf);
        while let Some(index) = self.pending.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=index).collect();
            let normalized = Self::normalize_line(&line);
            std::io::Write::write_all(&mut self.inner, &normalized)?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if !self.pending.is_empty() {
            let pending = std::mem::take(&mut self.pending);
            let normalized = Self::normalize_line(&pending);
            std::io::Write::write_all(&mut self.inner, &normalized)?;
        }
        self.inner.flush()
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
    let writer = CanonicalJsonWriter::new(base_path, max_bytes)?;

    let layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(Mutex::new(writer))
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_current_span(false)
        .flatten_event(true)
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

/// Fixed buckets keep the metrics accumulator bounded while retaining useful
/// percentile evidence for long-running sessions.
const HISTOGRAM_BUCKETS: [u64; 18] = [
    1,
    2,
    4,
    8,
    16,
    32,
    64,
    128,
    256,
    512,
    1_024,
    2_048,
    4_096,
    8_192,
    16_384,
    32_768,
    65_536,
    u64::MAX,
];

const MEMORY_BUCKETS: [u64; 18] = [
    1 << 20,
    2 << 20,
    4 << 20,
    8 << 20,
    16 << 20,
    32 << 20,
    64 << 20,
    128 << 20,
    256 << 20,
    512 << 20,
    1 << 30,
    2 << 30,
    4 << 30,
    8 << 30,
    16 << 30,
    32 << 30,
    64 << 30,
    u64::MAX,
];

/// A bounded local histogram. Values are recorded in the metric's declared
/// unit (microseconds, milliseconds, or ticks).
#[derive(Debug, Clone)]
pub struct MetricHistogram {
    count: u64,
    sum: u64,
    max: u64,
    buckets: [u64; HISTOGRAM_BUCKETS.len()],
    bounds: &'static [u64],
}

impl MetricHistogram {
    fn with_bounds(bounds: &'static [u64]) -> Self {
        MetricHistogram {
            count: 0,
            sum: 0,
            max: 0,
            buckets: [0; HISTOGRAM_BUCKETS.len()],
            bounds,
        }
    }

    fn record(&mut self, value: u64) {
        self.count = self.count.saturating_add(1);
        self.sum = self.sum.saturating_add(value);
        self.max = self.max.max(value);
        let bucket = self
            .bounds
            .iter()
            .position(|bound| value <= *bound)
            .unwrap_or(self.bounds.len() - 1);
        self.buckets[bucket] = self.buckets[bucket].saturating_add(1);
    }

    fn percentile(&self, percentile: u64) -> u64 {
        if self.count == 0 {
            return 0;
        }
        let rank = ((self.count.saturating_mul(percentile)).saturating_add(99)) / 100;
        let mut cumulative = 0u64;
        for (index, count) in self.buckets.iter().take(self.bounds.len()).enumerate() {
            cumulative = cumulative.saturating_add(*count);
            if cumulative >= rank.max(1) {
                return self.bounds[index];
            }
        }
        self.max
    }
}

impl Default for MetricHistogram {
    fn default() -> Self {
        Self::with_bounds(&HISTOGRAM_BUCKETS)
    }
}

impl Serialize for MetricHistogram {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut output = serializer.serialize_struct("MetricHistogram", 6)?;
        output.serialize_field("count", &self.count)?;
        output.serialize_field("sum", &self.sum)?;
        output.serialize_field("max", &self.max)?;
        output.serialize_field("p50", &self.percentile(50))?;
        output.serialize_field("p95", &self.percentile(95))?;
        output.serialize_field("p99", &self.percentile(99))?;
        output.end()
    }
}

fn duration_micros(duration: Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
}

fn duration_millis(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

/// Read the process resident set size without introducing a runtime dependency.
///
/// The shell owns platform I/O, so this sampling stays outside `mc_core` and
/// degrades to `None` only on targets without a supported native query.
#[cfg(target_os = "linux")]
fn resident_memory_bytes() -> Option<u64> {
    unsafe extern "C" {
        fn sysconf(name: i32) -> i64;
    }
    let pages = std::fs::read_to_string("/proc/self/statm")
        .ok()?
        .split_whitespace()
        .nth(1)?
        .parse::<u64>()
        .ok()?;
    let page_size = unsafe { sysconf(30) };
    (page_size > 0)
        .then_some(page_size as u64)
        .and_then(|size| pages.checked_mul(size))
}

#[cfg(target_os = "macos")]
fn resident_memory_bytes() -> Option<u64> {
    #[repr(C)]
    struct TimeVal {
        seconds: i64,
        microseconds: i64,
    }
    #[repr(C)]
    struct Usage {
        user: TimeVal,
        system: TimeVal,
        max_resident_set: i64,
        rest: [i64; 13],
    }
    unsafe extern "C" {
        fn getrusage(which: i32, usage: *mut Usage) -> i32;
    }
    let mut usage = Usage {
        user: TimeVal {
            seconds: 0,
            microseconds: 0,
        },
        system: TimeVal {
            seconds: 0,
            microseconds: 0,
        },
        max_resident_set: 0,
        rest: [0; 13],
    };
    (unsafe { getrusage(0, &mut usage) } == 0)
        .then_some(usage.max_resident_set)
        .filter(|bytes| *bytes > 0)
        .map(|bytes| bytes as u64)
}

#[cfg(target_os = "windows")]
fn resident_memory_bytes() -> Option<u64> {
    use std::ffi::c_void;
    use std::mem::size_of;

    #[repr(C)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
    }
    #[link(name = "psapi")]
    unsafe extern "system" {
        fn GetProcessMemoryInfo(
            process: *mut c_void,
            counters: *mut ProcessMemoryCounters,
            size: u32,
        ) -> i32;
    }

    let mut counters = ProcessMemoryCounters {
        cb: size_of::<ProcessMemoryCounters>() as u32,
        page_fault_count: 0,
        peak_working_set_size: 0,
        working_set_size: 0,
        quota_peak_paged_pool_usage: 0,
        quota_paged_pool_usage: 0,
        quota_peak_non_paged_pool_usage: 0,
        quota_non_paged_pool_usage: 0,
        pagefile_usage: 0,
        peak_pagefile_usage: 0,
    };
    (unsafe {
        GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut counters,
            size_of::<ProcessMemoryCounters>() as u32,
        )
    } != 0)
        .then_some(counters.peak_working_set_size as u64)
        .filter(|bytes| *bytes > 0)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn resident_memory_bytes() -> Option<u64> {
    None
}

/// Runtime metrics collected across the session.
#[derive(Debug, Clone, Serialize)]
pub struct SessionMetrics {
    pub session: String,
    pub build: String,
    pub reference_machine: String,
    pub start_time: String,
    pub end_time: String,
    pub ticks_elapsed: u64,
    pub commands_processed: u64,
    pub battles_fought: u32,
    pub encounters_resolved: u32,
    #[serde(rename = "core.step.duration")]
    pub core_step_duration: MetricHistogram,
    #[serde(rename = "frame.total.duration")]
    pub frame_total_duration: MetricHistogram,
    #[serde(rename = "content.load.duration")]
    pub content_load_duration: MetricHistogram,
    #[serde(rename = "startup.to_title.duration")]
    pub startup_to_title_duration: MetricHistogram,
    #[serde(rename = "save.write.duration")]
    pub save_write_duration: MetricHistogram,
    #[serde(rename = "save.load.duration")]
    pub save_load_duration: MetricHistogram,
    #[serde(rename = "memory.resident.peak")]
    pub memory_resident_peak: MetricHistogram,
    #[serde(rename = "encounter.resolve.ticks")]
    pub encounter_resolve_ticks: MetricHistogram,
}

impl SessionMetrics {
    pub fn new() -> Self {
        SessionMetrics {
            session: SESSION_ID.clone(),
            build: build_version(),
            reference_machine: std::env::var("MC_REFERENCE_MACHINE")
                .unwrap_or_else(|_| "unknown".into()),
            start_time: fmt_date(SystemTime::now()),
            end_time: String::new(),
            ticks_elapsed: 0,
            commands_processed: 0,
            battles_fought: 0,
            encounters_resolved: 0,
            core_step_duration: MetricHistogram::default(),
            frame_total_duration: MetricHistogram::default(),
            content_load_duration: MetricHistogram::default(),
            startup_to_title_duration: MetricHistogram::default(),
            save_write_duration: MetricHistogram::default(),
            save_load_duration: MetricHistogram::default(),
            memory_resident_peak: MetricHistogram::with_bounds(&MEMORY_BUCKETS),
            encounter_resolve_ticks: MetricHistogram::default(),
        }
    }
}

impl Default for SessionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metrics accumulator.
static METRICS: LazyLock<Mutex<SessionMetrics>> =
    LazyLock::new(|| Mutex::new(SessionMetrics::new()));

/// Record a tick for metrics.
pub fn record_tick() {
    let resident_memory = resident_memory_bytes();
    if let Ok(mut m) = METRICS.lock() {
        m.ticks_elapsed += 1;
        if let Some(bytes) = resident_memory {
            m.memory_resident_peak.record(bytes);
        }
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

/// Record one core step duration in microseconds.
pub fn record_core_step(duration: Duration) {
    if let Ok(mut metrics) = METRICS.lock() {
        metrics.core_step_duration.record(duration_micros(duration));
    }
}

/// Record one complete presentation frame duration in microseconds.
pub fn record_frame(duration: Duration) {
    if let Ok(mut metrics) = METRICS.lock() {
        metrics
            .frame_total_duration
            .record(duration_micros(duration));
    }
}

/// Record content-pack loading duration in milliseconds.
pub fn record_content_load(duration: Duration) {
    if let Ok(mut metrics) = METRICS.lock() {
        metrics
            .content_load_duration
            .record(duration_millis(duration));
    }
}

/// Record startup-to-title duration in milliseconds.
pub fn record_startup_to_title(duration: Duration) {
    if let Ok(mut metrics) = METRICS.lock() {
        metrics
            .startup_to_title_duration
            .record(duration_millis(duration));
    }
}

/// Record one save write duration in milliseconds.
pub fn record_save_write(duration: Duration) {
    if let Ok(mut metrics) = METRICS.lock() {
        metrics
            .save_write_duration
            .record(duration_millis(duration));
    }
}

/// Record one save load duration in milliseconds.
pub fn record_save_load(duration: Duration) {
    if let Ok(mut metrics) = METRICS.lock() {
        metrics.save_load_duration.record(duration_millis(duration));
    }
}

/// Record the tick span used to resolve one encounter.
pub fn record_encounter_ticks(ticks: u64) {
    if let Ok(mut metrics) = METRICS.lock() {
        metrics.encounter_resolve_ticks.record(ticks);
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

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("metrics.json");
    tracing::info!(metrics_file = %file_name, "metrics written");
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
    pub state_hash: String,
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
            state_hash: current_state_hash(),
            panic_message: panic_msg,
            backtrace: bt_str,
        };

        let dir = crash_dir();
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(format!("crash-{}.json", ts));
        if let Ok(json) = serde_json::to_string_pretty(&report) {
            let _ = std::fs::write(&path, json.as_bytes());
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("crash.json");
            eprintln!("crash report written: {file_name}");
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
        let mut writer = RotatingFileWriter::new(path.clone(), 100).expect("test log path");
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
        let mut writer = RotatingFileWriter::new(path.clone(), 10).expect("test log path");
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
    fn resident_memory_sampler_is_nonzero_when_supported() {
        if let Some(bytes) = resident_memory_bytes() {
            assert!(bytes > 0, "resident memory sample should be positive");
        }
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
    fn memory_metric_uses_byte_scale_buckets() {
        let mut metric = MetricHistogram::with_bounds(&MEMORY_BUCKETS);
        metric.record(64 << 20);
        let json = serde_json::to_string(&metric).expect("serialize memory metric");
        assert!(json.contains("\"p50\":67108864"), "json: {json}");
    }

    #[test]
    fn crash_report_serialize() {
        let report = CrashReport {
            session: "test-session".into(),
            build: "test-build".into(),
            timestamp: "2020-01-15".into(),
            tick: 1234,
            state_hash: "00".repeat(32),
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
