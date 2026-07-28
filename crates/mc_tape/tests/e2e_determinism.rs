//! EP-004 M7: Cross-run determinism — verifies that replaying the same
//! tape twice produces the same hash, across multiple seeds.

use mc_core::command::{Command, Dir};
use mc_core::world::World;
use mc_tape::record::RecordTape;
use mc_tape::replay;

/// Build a tape from a seed with a fixed command pattern.
fn build_test_tape(seed: u128) -> mc_tape::format::Tape {
    let world = World::new(seed);
    let mut recorder = RecordTape::new(world);

    // Record a deterministic sequence of commands.
    recorder.record_command(Command::Interact).unwrap();
    recorder.record_command(Command::Move(Dir::North)).unwrap();
    recorder.record_command(Command::Move(Dir::East)).unwrap();
    recorder.record_command(Command::Interact).unwrap();
    recorder.record_command(Command::SceneAdvance).unwrap();
    recorder.record_command(Command::OpenMenu).unwrap();
    recorder.record_command(Command::CloseMenu).unwrap();
    recorder.record_command(Command::Move(Dir::West)).unwrap();
    recorder.record_command(Command::Move(Dir::South)).unwrap();
    recorder.record_command(Command::Interact).unwrap();

    recorder.finalize().unwrap()
}

/// Replay a tape twice and assert the hashes match.
fn assert_deterministic_replay(tape: &mc_tape::format::Tape) {
    let result_a = replay::replay(tape).expect("first replay should succeed");
    let result_b = replay::replay(tape).expect("second replay should succeed");

    assert_eq!(
        result_a.final_hash, result_b.final_hash,
        "determinism violation: two replays of the same tape produced different final hashes"
    );
    assert_eq!(
        result_a.first_divergence, result_b.first_divergence,
        "determinism violation: two replays produced different divergence patterns"
    );

    // Also verify that the final hash matches the tape's recorded final_hash.
    assert_eq!(
        result_a.final_hash, tape.final_hash,
        "replayed hash must match tape's recorded final_hash"
    );
}

#[test]
fn determinsitic_replay_seed_42() {
    let tape = build_test_tape(42);
    assert_deterministic_replay(&tape);
}

#[test]
fn determinsitic_replay_seed_12345() {
    let tape = build_test_tape(12345);
    assert_deterministic_replay(&tape);
}

#[test]
fn determinsitic_replay_seed_999() {
    let tape = build_test_tape(999);
    assert_deterministic_replay(&tape);
}

#[test]
fn determinsitic_replay_seed_zero() {
    let tape = build_test_tape(0);
    assert_deterministic_replay(&tape);
}

#[test]
fn determinsitic_replay_seed_u128_max() {
    let tape = build_test_tape(u128::MAX);
    assert_deterministic_replay(&tape);
}

#[test]
fn determinsitic_replay_empty_tape() {
    let world = World::new(42);
    let mut recorder = RecordTape::new(world);
    let tape = recorder.finalize().unwrap();
    assert_deterministic_replay(&tape);
}
