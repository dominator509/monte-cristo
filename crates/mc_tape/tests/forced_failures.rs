//! Integration test: forced failure cases.
//!
//! Tests that the tape format correctly rejects:
//! 1. Bad magic bytes
//! 2. Non-monotonic ticks
//! 3. Truncated data
//! 4. Unknown command discriminant
//! 5. Content digest mismatch

use mc_core::command::{Command, Dir};
use mc_tape::error::TapeError;
use mc_tape::format::{Tape, TapeStart};

/// Corrupt the magic bytes and verify BadMagic.
#[test]
fn bad_magic_rejected() {
    let tape = Tape::new(
        42,
        TapeStart::NewGame,
        vec![(0, Command::Interact)],
        vec![],
        [0u8; 32],
    )
    .unwrap();
    let mut bytes = tape.to_bytes().unwrap();
    // Overwrite the first magic byte.
    bytes[0] = 0xFF;
    let err = Tape::from_bytes(&bytes).unwrap_err();
    assert!(
        matches!(err, TapeError::BadMagic),
        "expected BadMagic, got {:?}",
        err
    );
}

/// Completely wrong magic header.
#[test]
fn wrong_magic_header_rejected() {
    let mut bytes = vec![0u8; 64];
    bytes[..8].copy_from_slice(b"NOTTAPE0"); // exactly 8 bytes
    let err = Tape::from_bytes(&bytes).unwrap_err();
    assert!(
        matches!(
            err,
            TapeError::BadMagic | TapeError::Truncated | TapeError::Deserialize(_)
        ),
        "expected an error for bad magic, got {:?}",
        err
    );
}

/// Duplicate ticks in entries should be rejected.
#[test]
fn duplicate_ticks_rejected() {
    let entries = vec![
        (0, Command::Interact),
        (0, Command::Move(Dir::North)), // same tick
    ];
    let err = Tape::new(42, TapeStart::NewGame, entries, vec![], [0u8; 32]).unwrap_err();
    assert!(
        matches!(err, TapeError::NonMonotonicTicks { tick: 0, prev: 0 }),
        "expected NonMonotonicTicks(0, 0), got {:?}",
        err
    );
}

/// Decreasing ticks should be rejected.
#[test]
fn decreasing_ticks_rejected() {
    let entries = vec![
        (5, Command::Interact),
        (3, Command::Move(Dir::East)), // earlier tick
    ];
    let err = Tape::new(42, TapeStart::NewGame, entries, vec![], [0u8; 32]).unwrap_err();
    assert!(
        matches!(err, TapeError::NonMonotonicTicks { tick: 3, prev: 5 }),
        "expected NonMonotonicTicks(3, 5), got {:?}",
        err
    );
}

/// Truncated binary data should produce a Truncated or Deserialize error.
#[test]
fn truncated_data_rejected() {
    let tape = Tape::new(
        42,
        TapeStart::NewGame,
        vec![(0, Command::Interact)],
        vec![],
        [0u8; 32],
    )
    .unwrap();
    let bytes = tape.to_bytes().unwrap();
    // Take only a prefix.
    let truncated = &bytes[..bytes.len().min(10)];
    let err = Tape::from_bytes(truncated).unwrap_err();
    assert!(
        matches!(err, TapeError::Truncated | TapeError::Deserialize(_)),
        "expected truncation/deserialize error, got {:?}",
        err
    );
}

/// Completely empty data.
#[test]
fn empty_bytes_rejected() {
    let err = Tape::from_bytes(&[]).unwrap_err();
    assert!(
        matches!(err, TapeError::Truncated | TapeError::Deserialize(_)),
        "expected truncation/deserialize error, got {:?}",
        err
    );
}

/// Content digest mismatch after mutation.
#[test]
fn content_digest_mismatch_rejected() {
    let tape = Tape::new(
        42,
        TapeStart::NewGame,
        vec![(0, Command::Interact)],
        vec![],
        [0u8; 32],
    )
    .unwrap();
    let mut bad_tape = tape.clone();
    // Corrupt the content_digest.
    bad_tape.content_digest = [0xFFu8; 32];
    let bad_bytes = bad_tape.to_bytes().unwrap();
    let err = Tape::from_bytes(&bad_bytes).unwrap_err();
    assert!(
        matches!(err, TapeError::ContentDigestMismatch),
        "expected ContentDigestMismatch, got {:?}",
        err
    );
}

/// Unknown command discriminant is handled by serde as a deserialize error.
///
/// The tape's byte layout for a 1-entry tape with Interact at tick 0:
///   magic[8] | seed[1varint] | content_digest[32] | TapeStart[1] |
///   entries_len[1] | tick0[1] | cmd_variant[1] | checkpoints_len[1] | final_hash[32]
///   = 8 + 1 + 32 + 1 + 1 + 1 + 1 + 1 + 32 = 78 bytes
///   Command variant is at byte 44 (0-based).
#[test]
fn unknown_command_discriminant_rejected() {
    // Use a tape with 1 entry (Interact) so we know the command variant is at byte 44.
    let tape = Tape::new(
        42,
        TapeStart::NewGame,
        vec![(0, Command::Interact)],
        vec![],
        [0u8; 32],
    )
    .unwrap();
    let bytes = tape.to_bytes().unwrap();

    // Command variant for Interact is at byte 44 (0x01).
    // postcard uses serde variant index, not repr(u16) discriminant.
    // Interact is variant 1, encoded as varint [0x01].
    let cmd_variant_offset = 44;

    if bytes.len() > cmd_variant_offset {
        let mut modified = bytes.clone();
        // Set to 99 — serde has no variant with index 99.
        modified[cmd_variant_offset] = 99;
        let err = Tape::from_bytes(&modified).unwrap_err();
        assert!(
            matches!(err, TapeError::Deserialize(_)),
            "expected Deserialize error for unknown discriminant, got {:?}",
            err
        );
    }
}

/// Exceeding MAX_TAPE_ENTRIES should produce BoundsExceeded.
#[test]
fn too_many_entries_rejected() {
    let entries: Vec<(u64, Command)> = (0..=mc_tape::format::MAX_TAPE_ENTRIES)
        .map(|i| (i as u64, Command::Interact))
        .collect();
    let err = Tape::new(42, TapeStart::NewGame, entries, vec![], [0u8; 32]).unwrap_err();
    assert!(
        matches!(err, TapeError::BoundsExceeded(_)),
        "expected BoundsExceeded, got {:?}",
        err
    );
}

/// Verify that content digest is checked after serialization round-trip
/// even if we manually modify it.
#[test]
fn content_digest_checked_after_manual_modification() {
    let tape = Tape::new(
        42,
        TapeStart::NewGame,
        vec![(0, Command::Interact)],
        vec![],
        [0u8; 32],
    )
    .unwrap();
    let bytes = tape.to_bytes().unwrap();

    // Content digest is at bytes[9..40] (32 bytes starting after the seed varint)
    let digest_start = 9;
    let digest_end = digest_start + 32;

    if bytes.len() > digest_end {
        let mut modified = bytes.clone();
        modified[digest_start] ^= 0xFF; // flip bits in the first byte of content_digest
        let err = Tape::from_bytes(&modified).unwrap_err();
        assert!(
            matches!(err, TapeError::ContentDigestMismatch),
            "expected ContentDigestMismatch, got {:?}",
            err
        );
    }
}

/// Non-monotonic detection in from_bytes (not just in new).
#[test]
fn non_monotonic_detected_in_from_bytes() {
    // Create a tape with 2 entries (Interact at tick 0, Move(North) at tick 1).
    // Then modify the second entry's tick to be the same as the first.
    //
    // Byte layout:
    //   magic[8] | seed[1varint] | content_digest[32] | TapeStart[1] |
    //   entries_len[1] | tick0[1] | cmd0[1] | tick1[1] | cmd1[1] | cmd1_data[1] |
    //   checkpoints_len[1] | final_hash[32]
    //   = 8 + 1 + 32 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 32 = 81 bytes
    //   tick1 is at byte 45 (0-based).
    let tape = Tape::new(
        42,
        TapeStart::NewGame,
        vec![(0, Command::Interact), (1, Command::Move(Dir::North))],
        vec![],
        [0u8; 32],
    )
    .unwrap();
    let bytes = tape.to_bytes().unwrap();

    // tick1 is at byte 45 for this 2-entry tape.
    let second_tick_offset = 45;

    if bytes.len() > second_tick_offset {
        let mut modified = bytes.clone();
        // Set the second entry's tick to 0 (same as first entry's tick).
        modified[second_tick_offset] = 0;
        let err = Tape::from_bytes(&modified).unwrap_err();
        assert!(
            matches!(err, TapeError::NonMonotonicTicks { .. }),
            "expected NonMonotonicTicks, got {:?}",
            err
        );
    }
}
