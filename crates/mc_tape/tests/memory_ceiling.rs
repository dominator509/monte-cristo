//! Resident-memory ceiling tests for tape replay.
//!
//! SPEC-008 §3: peak resident memory stays below 512 MiB and repeated replay does not show
//! unbounded growth.

use mc_core::command::Command;
use mc_core::world::World;
use mc_tape::format::Tape;
use mc_tape::record::RecordTape;

const TAPES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tapes");
const MEMORY_CEILING_BYTES: u64 = 512 * 1024 * 1024;
const LATE_GROWTH_TOLERANCE_BYTES: u64 = 16 * 1024 * 1024;

#[cfg(target_os = "linux")]
fn resident_bytes() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").expect("read /proc/self/status");
    let kib = status
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmRSS:")
                .and_then(|rest| rest.split_whitespace().next())
                .and_then(|value| value.parse::<u64>().ok())
        })
        .expect("VmRSS is present in /proc/self/status");
    kib * 1024
}

#[cfg(target_os = "windows")]
fn resident_bytes() -> u64 {
    let script = format!("(Get-Process -Id {}).WorkingSet64", std::process::id());
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script.as_str()])
        .output()
        .expect("query current-process WorkingSet64 with PowerShell");
    assert!(
        output.status.success(),
        "WorkingSet64 query failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("WorkingSet64 output is UTF-8")
        .trim()
        .parse::<u64>()
        .expect("WorkingSet64 output is an integer")
}

#[cfg(target_os = "macos")]
fn resident_bytes() -> u64 {
    let pid = std::process::id().to_string();
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", pid.as_str()])
        .output()
        .expect("query current-process RSS with ps");
    assert!(
        output.status.success(),
        "ps RSS query failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let kib = String::from_utf8(output.stdout)
        .expect("ps RSS output is UTF-8")
        .trim()
        .parse::<u64>()
        .expect("ps RSS output is an integer");
    kib * 1024
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
compile_error!("memory_ceiling requires Linux, Windows, or macOS RSS support");

fn sample_under_ceiling(label: &str, peak: &mut u64) -> u64 {
    let resident = resident_bytes();
    *peak = (*peak).max(resident);
    assert!(
        resident < MEMORY_CEILING_BYTES,
        "{label} resident memory {resident} exceeds {MEMORY_CEILING_BYTES}"
    );
    resident
}

/// Warm up, replay the golden campaign 100 times, and assert both the hard ceiling and that
/// the second half does not grow materially faster than the first.
#[test]
fn memory_ceiling_golden_full_replay() {
    let tape_path = format!("{}/golden-full.tape", TAPES_DIR);
    let data = std::fs::read(&tape_path)
        .expect("golden-full.tape should exist — run gen-golden-tapes first");
    let tape = Tape::from_bytes(&data).expect("deserialize tape");

    for i in 0..5 {
        let result = mc_tape::replay::replay(&tape)
            .unwrap_or_else(|_| panic!("warm-up replay iteration {i} should succeed"));
        assert_eq!(result.final_hash, tape.final_hash, "warm-up hash {i}");
    }

    let mut peak = 0;
    let baseline = sample_under_ceiling("baseline", &mut peak);
    let mut checkpoints = Vec::with_capacity(10);
    for i in 0..100 {
        let result = mc_tape::replay::replay(&tape)
            .unwrap_or_else(|_| panic!("replay iteration {i} should succeed"));
        assert!(
            result.first_divergence.is_none(),
            "replay divergence at iter {i}: {:?}",
            result.first_divergence
        );
        assert_eq!(result.final_hash, tape.final_hash, "hash match at iter {i}");
        if i % 10 == 9 {
            checkpoints.push(sample_under_ceiling("golden replay", &mut peak));
        }
    }

    let midpoint = checkpoints[4];
    let final_resident = checkpoints[9];
    let early_growth = midpoint.saturating_sub(baseline);
    let late_growth = final_resident.saturating_sub(midpoint);
    assert!(
        late_growth <= early_growth.saturating_add(LATE_GROWTH_TOLERANCE_BYTES),
        "resident memory grows without bound: baseline={baseline}, midpoint={midpoint}, \
         final={final_resident}, early_growth={early_growth}, late_growth={late_growth}"
    );
    println!(
        "memory ceiling: peak={peak} baseline={baseline} final={final_resident} \
         late_growth={late_growth}"
    );
}

/// Record and replay a 10,000-command tape while sampling the real process working set.
#[test]
fn memory_ceiling_long_tape() {
    let mut peak = 0;
    sample_under_ceiling("before long tape", &mut peak);

    let world = World::new(42);
    let mut recorder = RecordTape::new(world);

    for i in 0..10_000 {
        recorder
            .record_command(Command::Interact)
            .unwrap_or_else(|_| panic!("record command {}", i));
    }

    let tape = recorder.finalize().expect("finalize tape");
    sample_under_ceiling("recorded long tape", &mut peak);

    let result = mc_tape::replay::replay(&tape).expect("replay long tape");
    assert!(result.first_divergence.is_none(), "long tape divergence");
    assert_eq!(result.final_hash, tape.final_hash, "long tape hash match");
    sample_under_ceiling("replayed long tape", &mut peak);
    println!("memory ceiling long tape: peak={peak}");
}
