//! Integration test: tape round-trip serialization/deserialization.
//!
//! Validates that a tape with valid data survives a
//! serialize -> deserialize -> compare round trip.

use mc_core::command::{Command, Dir};
use mc_tape::format::{Tape, TapeStart};
use mc_tape::record::RecordTape;
use mc_core::world::World;

/// Build a tape with a small set of commands, serialize it,
/// deserialize it, and assert they are identical.
#[test]
fn roundtrip_empty_tape() {
    let tape = Tape::new(42, TapeStart::NewGame, vec![], vec![], [0u8; 32]).unwrap();
    let bytes = tape.to_bytes().unwrap();
    let deserialized = Tape::from_bytes(&bytes).unwrap();
    assert_eq!(tape, deserialized);
}

#[test]
fn roundtrip_single_entry() {
    let tape = Tape::new(
        42,
        TapeStart::NewGame,
        vec![(0, Command::Interact)],
        vec![],
        [0u8; 32],
    )
    .unwrap();
    let bytes = tape.to_bytes().unwrap();
    let deserialized = Tape::from_bytes(&bytes).unwrap();
    assert_eq!(tape, deserialized);
}

#[test]
fn roundtrip_multiple_entries() {
    let entries = vec![
        (0, Command::Move(Dir::North)),
        (1, Command::Interact),
        (2, Command::Move(Dir::East)),
        (3, Command::OpenMenu),
        (4, Command::CloseMenu),
    ];
    let tape = Tape::new(42, TapeStart::NewGame, entries, vec![], [0u8; 32]).unwrap();
    let bytes = tape.to_bytes().unwrap();
    let deserialized = Tape::from_bytes(&bytes).unwrap();
    assert_eq!(tape, deserialized);
}

#[test]
fn roundtrip_with_checkpoints() {
    let checkpoints = vec![
        (1024, [1u8; 32]),
        (2048, [2u8; 32]),
        (3072, [3u8; 32]),
    ];
    let entries = vec![
        (0, Command::Interact),
        (1, Command::Move(Dir::West)),
    ];
    let tape = Tape::new(42, TapeStart::NewGame, entries, checkpoints, [0u8; 32]).unwrap();
    let bytes = tape.to_bytes().unwrap();
    let deserialized = Tape::from_bytes(&bytes).unwrap();
    assert_eq!(tape, deserialized);
}

#[test]
fn roundtrip_all_command_variants() {
    // Exercise all Command variants to ensure serde handles them all.
    let entries = vec![
        (0, Command::Move(Dir::North)),
        (1, Command::Move(Dir::South)),
        (2, Command::Move(Dir::East)),
        (3, Command::Move(Dir::West)),
        (4, Command::Interact),
        (5, Command::OpenMenu),
        (6, Command::CloseMenu),
        (7, Command::SelectAction(
            mc_core::command::ActorId(0),
            mc_core::command::Action::Attack {
                target: mc_core::command::TargetId(1),
            },
        )),
        (8, Command::ConfirmTarget(mc_core::command::TargetId(0))),
        (9, Command::CancelSelection),
        (10, Command::SetWaitMode(true)),
        (11, Command::SceneAdvance),
        (12, Command::SceneChoose(mc_core::command::ChoiceIdx(0))),
        (13, Command::SwapPersona(mc_core::command::PersonaId::MonteCristo)),
        (14, Command::FastTravel(mc_core::ids::RegionId::R01_MARSEILLE)),
        (15, Command::NameYourself),
        (16, Command::Save(mc_core::command::SaveSlot(0))),
        (17, Command::Load(mc_core::command::SaveSlot(1))),
    ];
    let tape = Tape::new(42, TapeStart::NewGame, entries, vec![], [0u8; 32]).unwrap();
    let bytes = tape.to_bytes().unwrap();
    let deserialized = Tape::from_bytes(&bytes).unwrap();
    assert_eq!(tape, deserialized);
}

#[test]
fn record_then_replay_roundtrip() {
    // Record a sequence, convert to tape, serialize, deserialize, replay.
    let world = World::new(12345);
    let mut recorder = RecordTape::new(world);

    // Record ~100 commands so we get at least one checkpoint (every 1024 ticks).
    for _ in 0..1050 {
        recorder.record_command(Command::Interact).unwrap();
    }

    let original_tape = recorder.finalize().unwrap();

    // Serialize and deserialize.
    let bytes = original_tape.to_bytes().unwrap();
    let deserialized_tape = Tape::from_bytes(&bytes).unwrap();

    // The tapes should be identical.
    assert_eq!(original_tape, deserialized_tape);

    // Replay the deserialized tape — should produce the same final hash.
    let result = mc_tape::replay::replay(&deserialized_tape).unwrap();
    assert_eq!(result.final_hash, deserialized_tape.final_hash);
    assert!(
        result.first_divergence.is_none(),
        "replay diverged at {:?}",
        result.first_divergence
    );
}

#[test]
fn roundtrip_with_1024_step_gaps() {
    // Test entries with tick gaps that cross checkpoint boundaries.
    let entries = vec![
        (0, Command::Interact),
        (500, Command::Move(Dir::South)),
        (1500, Command::OpenMenu),
        (3000, Command::CloseMenu),
    ];
    let checkpoints = vec![
        (1024, [0xAA; 32]),
        (2048, [0xBB; 32]),
    ];
    let tape = Tape::new(42, TapeStart::NewGame, entries, checkpoints, [0u8; 32]).unwrap();
    let bytes = tape.to_bytes().unwrap();
    let deserialized = Tape::from_bytes(&bytes).unwrap();
    assert_eq!(tape, deserialized);
}
