//! EP-004 M6: Act I tape — end-to-end test that records a tape,
//! writes it to disk, replays it, and verifies determinism.
//!
//! This test constructs a programmatic RecordTape simulating a portion
//! of Act I. The tape captures the sequence of commands and their
//! deterministic effect on the world state. Replay produces the same
//! final state hash, proving determinism.
//!
//! When scene effects are wired in later milestones, flag transitions
//! will be captured as part of the recorded command sequence. For now,
//! the test verifies the full record → serialize → deserialize → replay
//! pipeline with commands that are valid for the initial game state.

use mc_core::command::{Command, Dir};
use mc_core::world::World;
use mc_tape::format::Tape;
use mc_tape::record::RecordTape;
use mc_tape::replay;

/// Path to the tapes directory relative to the workspace root.
const TAPES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tapes");

/// Compute a hex string from a [u8; 32] hash.
fn hex_fmt(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[test]
fn e2e_act1_tape() {
    // ── Phase 1: Seed and world ──────────────────────────────────────
    let seed: u128 = 42;
    let world = World::new(seed);

    // ── Phase 2: Record a sequence of Act I commands ─────────────────
    let mut recorder = RecordTape::new(world);

    // Scene: Edmond Dantes at the Marseille docks
    recorder.record_command(Command::Interact).unwrap();

    // Move north
    recorder.record_command(Command::Move(Dir::North)).unwrap();

    // Scene advances
    recorder.record_command(Command::SceneAdvance).unwrap();

    // Interaction
    recorder.record_command(Command::Interact).unwrap();

    // Menu operations
    recorder.record_command(Command::OpenMenu).unwrap();
    recorder.record_command(Command::CloseMenu).unwrap();

    // Movement
    recorder.record_command(Command::Move(Dir::East)).unwrap();
    recorder.record_command(Command::Move(Dir::East)).unwrap();

    // More scene/interaction
    recorder.record_command(Command::SceneAdvance).unwrap();
    recorder.record_command(Command::Interact).unwrap();

    // ── Phase 3: Finalize and produce the tape ───────────────────────
    let tape = recorder.finalize().unwrap();

    // Verify basic structure
    assert_eq!(tape.seed, seed);
    assert_eq!(tape.entries.len(), 10, "expected 10 recorded entries");
    assert!(!tape.is_empty(), "tape should not be empty");

    // ── Phase 4: Serialize to bytes and write to file ────────────────
    let bytes = tape.to_bytes().expect("serialization should succeed");
    let tape_path = format!("{}/act1.tape", TAPES_DIR);
    std::fs::write(&tape_path, &bytes)
        .expect("should write act1.tape to disk");

    // ── Phase 5: Read back from file and deserialize ─────────────────
    let read_bytes = std::fs::read(&tape_path)
        .expect("should read act1.tape from disk");
    let deserialized = Tape::from_bytes(&read_bytes)
        .expect("deserialization of written tape should succeed");

    // Verify roundtrip identity
    assert_eq!(tape, deserialized, "tape roundtrip should preserve all data");

    // ── Phase 6: Replay and verify hash matches tape's final_hash ────
    let result = replay::replay(&deserialized)
        .expect("replay should succeed");

    assert_eq!(
        result.final_hash, tape.final_hash,
        "replayed final hash must match tape's final_hash"
    );

    // There should be no divergence
    assert!(
        result.first_divergence.is_none(),
        "expected no divergence, got {:?}",
        result.first_divergence
    );

    // ── Phase 7: Record the hash in HASHES.txt ───────────────────────
    let hash_hex = hex_fmt(&tape.final_hash);
    let hash_line = format!("act1.tape {}\n", hash_hex);
    let hashes_path = format!("{}/HASHES.txt", TAPES_DIR);
    std::fs::write(&hashes_path, &hash_line)
        .expect("should write HASHES.txt");

    eprintln!("act1.tape {}", hash_hex);
    eprintln!("e2e_act1: PASS");
}
