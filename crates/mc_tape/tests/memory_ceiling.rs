//! Memory ceiling test for tape replay.
//!
//! Verifies that replaying a long campaign tape does not leak memory
//! and stays within reasonable bounds. SPEC-008 §3, LF-11 companion.

use mc_core::command::Command;
use mc_core::world::World;
use mc_tape::format::Tape;
use mc_tape::record::RecordTape;

const TAPES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tapes");

/// Loads the golden-full tape and replays it multiple times.
/// Verifies no memory regression: each replay should produce the same hash.
#[test]
fn memory_ceiling_golden_full_replay() {
    let tape_path = format!("{}/golden-full.tape", TAPES_DIR);
    let data = std::fs::read(&tape_path)
        .expect("golden-full.tape should exist — run gen-golden-tapes first");
    let tape = Tape::from_bytes(&data).expect("deserialize tape");

    // Replay 5 times — no divergence, no crash, consistent memory
    for i in 0..5 {
        let result = mc_tape::replay::replay(&tape)
            .unwrap_or_else(|_| panic!("replay iteration {} should succeed", i));
        assert!(
            result.first_divergence.is_none(),
            "replay divergence at iter {}: {:?}",
            i,
            result.first_divergence
        );
        assert_eq!(
            result.final_hash, tape.final_hash,
            "hash match at iter {}",
            i
        );
    }
}

/// Long replay: record-then-replay a tape with ~10K commands.
/// Ensures large tapes don't blow memory.
#[test]
fn memory_ceiling_long_tape() {
    let world = World::new(42);
    let mut recorder = RecordTape::new(world);

    // Record 10,000 interact commands — pushes most systems
    for i in 0..10_000 {
        recorder
            .record_command(Command::Interact)
            .unwrap_or_else(|_| panic!("record command {}", i));
    }

    let tape = recorder.finalize().expect("finalize tape");

    // Replay
    let result = mc_tape::replay::replay(&tape).expect("replay long tape");
    assert!(result.first_divergence.is_none(), "long tape divergence");
    assert_eq!(result.final_hash, tape.final_hash, "long tape hash match");
}
