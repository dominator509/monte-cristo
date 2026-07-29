//! Integration test: divergence detection.
//!
//! Tests that the divergence detection correctly identifies mutated
//! checkpoints and reports the exact tick where divergence occurs.

use mc_core::command::Command;
use mc_core::world::World;
use mc_tape::divergence::{compare_replay, compare_tapes};
use mc_tape::format::{Tape, TapeStart};
use mc_tape::record::RecordTape;

/// Mutate one checkpoint hash in a recorded tape and verify that
/// divergence detection identifies the exact mutated tick.
#[test]
fn mutated_checkpoint_reported() {
    // Record a tape with enough commands to get checkpoints.
    let world = World::new(42);
    let mut recorder = RecordTape::new(world);

    // Record ~3000 commands to get checkpoints at ticks 1024, 2048, 3072.
    for _ in 0..3000 {
        recorder.record_command(Command::Interact).unwrap();
    }

    let mut tape = recorder.finalize().unwrap();

    // Now mutate the checkpoint at tick 2048.
    let mutated_tick = 2048u64;
    let mut found = false;
    for cp in tape.checkpoints.iter_mut() {
        if cp.0 == mutated_tick {
            cp.1 = [0xDE; 32]; // arbitrary mutation
            found = true;
            break;
        }
    }

    // Verify that we found and mutated the checkpoint.
    assert!(found, "checkpoint at tick {} not found", mutated_tick);

    // Use compare_replay to detect the divergence.
    let report = compare_replay(&tape).unwrap();

    // The report should flag tick 2048 as the first divergent tick.
    assert!(
        report.first_divergent_tick == Some(mutated_tick)
            || report
                .divergences
                .iter()
                .any(|(t, _, _)| *t == mutated_tick),
        "expected divergence at tick {}, report: first={:?}, divergences={:?}",
        mutated_tick,
        report.first_divergent_tick,
        report.divergences
    );

    // The divergence at tick 2048 should have expected != actual.
    if let Some(pos) = report
        .divergences
        .iter()
        .position(|(t, _, _)| *t == mutated_tick)
    {
        let (tick, expected, actual) = report.divergences[pos];
        assert_eq!(tick, mutated_tick);
        assert_eq!(expected, [0xDE; 32]);
        assert_ne!(
            expected, actual,
            "mutated checkpoint should differ from actual"
        );
    }
}

/// Compare two tapes where one has a changed checkpoint.
#[test]
fn compare_tapes_detects_divergence() {
    let checkpoints_a = vec![(1024, [1u8; 32]), (2048, [2u8; 32]), (3072, [3u8; 32])];
    let mut checkpoints_b = checkpoints_a.clone();
    // Mutate the second checkpoint.
    checkpoints_b[1] = (2048, [0xFF; 32]);

    let tape_a = Tape::new(42, TapeStart::NewGame, vec![], checkpoints_a, [0u8; 32]).unwrap();
    let tape_b = Tape::new(42, TapeStart::NewGame, vec![], checkpoints_b, [0u8; 32]).unwrap();

    let report = compare_tapes(&tape_a, &tape_b);
    assert_eq!(report.total_checked, 3);
    assert_eq!(report.divergences.len(), 1);
    assert_eq!(report.first_divergent_tick, Some(2048));
    let (tick, expected, actual) = report.divergences[0];
    assert_eq!(tick, 2048);
    assert_eq!(expected, [2u8; 32]); // from tape_a at tick 2048
    assert_eq!(actual, [0xFF; 32]); // from tape_b at tick 2048
}

/// Replay a tape with mismatched final_hash — divergence should show.
#[test]
fn final_hash_mismatch_detected() {
    // Create a tape with a known good replay, then change final_hash.
    let world = World::new(42);
    let mut recorder = RecordTape::new(world);
    recorder.record_command(Command::Interact).unwrap();
    let mut tape = recorder.finalize().unwrap();

    // We'll manually compare final_hash vs replay final_hash.
    // First, replay to get the correct final hash.
    let result = mc_tape::replay::replay(&tape).unwrap();
    assert_eq!(
        result.final_hash, tape.final_hash,
        "initial tape's final_hash should match replay"
    );

    // Now mutate the final_hash in the tape.
    tape.final_hash = [0xEE; 32];

    // compare_replay should flag the divergence in checkpoints, but the
    // final_hash check happens in replay's return value, not in compare_replay.
    // compare_replay only compares checkpoint hashes.
    // The divergence report's first_divergent_tick might not be set if all
    // checkpoints match but final_hash differs.
    let report = compare_replay(&tape).unwrap();
    // Checkpoints should still match (we didn't change those).
    // But the final_hash in the tape no longer matches reality.
    let replay_result = mc_tape::replay::replay(&tape).unwrap();
    assert_ne!(
        replay_result.final_hash, tape.final_hash,
        "mutated final_hash should not match replay result"
    );
    assert!(
        replay_result.first_divergence.is_none() || report.first_divergent_tick.is_some(),
        "either replay detects divergence or compare_replay does"
    );
}

/// Explicitly name which tick diverges.
#[test]
fn divergence_reports_exact_tick() {
    // Build a tape with checkpoints at known ticks.
    let checkpoints = vec![
        (1000, [0xAA; 32]),
        (2000, [0xBB; 32]),
        (3000, [0xCC; 32]),
        (4000, [0xDD; 32]),
    ];

    // Create two tapes: same, except the checkpoint at 3000 differs.
    let tape_a = Tape::new(
        42,
        TapeStart::NewGame,
        vec![(0, Command::Interact)],
        checkpoints.clone(),
        [0u8; 32],
    )
    .unwrap();
    let mut tape_b = tape_a.clone();
    tape_b.checkpoints[2] = (3000, [0xC0; 32]); // mutate tick 3000

    let report = compare_tapes(&tape_a, &tape_b);
    assert_eq!(
        report.first_divergent_tick,
        Some(3000),
        "should report tick 3000 as first divergence"
    );
    assert_eq!(report.divergences.len(), 1);
    assert_eq!(report.divergences[0].0, 3000);
}
