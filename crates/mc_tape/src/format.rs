//! Tape format — the canonical binary input tape.
//!
//! SPEC-003 section 4 defines the tape layout:
//! - magic: `b"MCTAPE01"` (8 bytes, verified on deserialize)
//! - seed: u128
//! - content_digest: [u8; 32]
//! - start: TapeStart (NewGame | FromSave)
//! - entries: Vec<(u64, Command)> — strictly ascending by tick
//! - checkpoints: Vec<(u64, [u8; 32])>
//! - final_hash: [u8; 32]

use crate::error::TapeError;
use mc_core::command::Command;
use serde::{Deserialize, Serialize};

/// Maximum number of entries allowed in a tape (10 MiB safety bound).
pub const MAX_TAPE_ENTRIES: usize = 10_485_760;

/// How the tape begins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TapeStart {
    /// Start a new game from the initial state derived from the seed.
    NewGame,
    // Future: FromSave(Save) will be added when save/load is implemented.
}

/// A serialisable, verifiable input tape.
///
/// The tape records every command applied to a deterministic game world
/// together with periodic checkpoints that enable replay verification
/// and divergence detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tape {
    /// Magic bytes — must be `b"MCTAPE01"`.
    pub magic: [u8; 8],

    /// The random seed used to initialise the world.
    pub seed: u128,

    /// blake3 hash of the concatenated entries and checkpoints content.
    /// Computed as `blake3::hash(&postcard::to_allocvec(&(&entries, &checkpoints)).unwrap())`.
    pub content_digest: [u8; 32],

    /// How the tape starts (NewGame or from a saved state).
    pub start: TapeStart,

    /// The sequence of (tick, command) entries, strictly ascending by tick.
    pub entries: Vec<(u64, Command)>,

    /// Checkpoints — snapshot hashes at regular intervals.
    /// Each entry is `(tick, state_hash)`.
    pub checkpoints: Vec<(u64, [u8; 32])>,

    /// The expected blake3 state hash after all entries have been applied.
    pub final_hash: [u8; 32],
}

impl Tape {
    /// Create a new tape with the given components.
    ///
    /// Validates that:
    /// 1. Entries are strictly ascending by tick.
    /// 2. Entry count does not exceed `MAX_TAPE_ENTRIES`.
    /// 3. The `content_digest` matches a blake3 hash of the serialised
    ///    entries and checkpoints.
    pub fn new(
        seed: u128,
        start: TapeStart,
        entries: Vec<(u64, Command)>,
        checkpoints: Vec<(u64, [u8; 32])>,
        final_hash: [u8; 32],
    ) -> Result<Self, TapeError> {
        // Validate entry count bound.
        if entries.len() > MAX_TAPE_ENTRIES {
            return Err(TapeError::BoundsExceeded(format!(
                "entry count {} exceeds maximum {}",
                entries.len(),
                MAX_TAPE_ENTRIES,
            )));
        }

        // Validate strictly ascending ticks.
        if let Some((prev_tick, _)) = entries.first() {
            let mut prev = *prev_tick;
            for (_, (tick, _)) in entries.iter().enumerate().skip(1) {
                if *tick <= prev {
                    return Err(TapeError::NonMonotonicTicks {
                        tick: *tick,
                        prev,
                    });
                }
                prev = *tick;
            }
        }

        // Compute content digest.
        let content_digest = compute_content_digest(&entries, &checkpoints);

        let tape = Tape {
            magic: *b"MCTAPE01",
            seed,
            content_digest,
            start,
            entries,
            checkpoints,
            final_hash,
        };

        Ok(tape)
    }

    /// Deserialise a tape from postcard-encoded bytes.
    ///
    /// Validates:
    /// 1. Magic matches `b"MCTAPE01"`.
    /// 2. Entry ticks are strictly ascending.
    /// 3. Entry count does not exceed `MAX_TAPE_ENTRIES`.
    /// 4. Content digest matches a hash of the serialised entries/checkpoints.
    pub fn from_bytes(data: &[u8]) -> Result<Self, TapeError> {
        let tape: Tape =
            postcard::from_bytes(data).map_err(|e| map_postcard_err(e))?;

        // Validate magic.
        if &tape.magic != b"MCTAPE01" {
            return Err(TapeError::BadMagic);
        }

        // Validate entry count bound.
        if tape.entries.len() > MAX_TAPE_ENTRIES {
            return Err(TapeError::BoundsExceeded(format!(
                "entry count {} exceeds maximum {}",
                tape.entries.len(),
                MAX_TAPE_ENTRIES,
            )));
        }

        // Validate strictly ascending ticks.
        if let Some((prev_tick, _)) = tape.entries.first() {
            let mut prev = *prev_tick;
            for (tick, _) in tape.entries.iter().skip(1) {
                if *tick <= prev {
                    return Err(TapeError::NonMonotonicTicks {
                        tick: *tick,
                        prev,
                    });
                }
                prev = *tick;
            }
        }

        // Validate content digest.
        let expected_digest = compute_content_digest(&tape.entries, &tape.checkpoints);
        if tape.content_digest != expected_digest {
            return Err(TapeError::ContentDigestMismatch);
        }

        Ok(tape)
    }

    /// Serialise the tape to postcard-encoded bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, TapeError> {
        postcard::to_allocvec(self).map_err(|e| {
            TapeError::Io(format!("serialization failed: {}", e))
        })
    }

    /// Return the number of entries in the tape.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return `true` if the tape has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Compute the content digest for a set of entries and checkpoints.
///
/// The digest is `blake3::hash(postcard::to_allocvec(&(&entries, &checkpoints)))`.
fn compute_content_digest(
    entries: &[(u64, Command)],
    checkpoints: &[(u64, [u8; 32])],
) -> [u8; 32] {
    let encoded = postcard::to_allocvec(&(&entries, &checkpoints))
        .expect("serialization of entries and checkpoints should never fail");
    let hash = blake3::hash(&encoded);
    *hash.as_bytes()
}

/// Map a postcard deserialization error to a TapeError.
fn map_postcard_err(e: postcard::Error) -> TapeError {
    match e {
        postcard::Error::DeserializeUnexpectedEnd => TapeError::Truncated,
        postcard::Error::DeserializeBadVarint => TapeError::Truncated,
        postcard::Error::DeserializeBadBool => TapeError::Deserialize(e.to_string()),
        postcard::Error::DeserializeBadChar => TapeError::Deserialize(e.to_string()),
        postcard::Error::DeserializeBadUtf8 => TapeError::Deserialize(e.to_string()),
        postcard::Error::DeserializeBadOption => TapeError::Deserialize(e.to_string()),
        postcard::Error::DeserializeBadEnum => TapeError::Deserialize(e.to_string()),
        postcard::Error::DeserializeBadCrc => TapeError::Deserialize(e.to_string()),
        postcard::Error::DeserializeBadEncoding => TapeError::Deserialize(e.to_string()),
        postcard::Error::SerializeBufferFull => TapeError::Deserialize(e.to_string()),
        postcard::Error::SerdeSerCustom => TapeError::Deserialize("serde serialization error".into()),
        postcard::Error::SerdeDeCustom => TapeError::Deserialize("serde deserialization error".into()),
        _ => TapeError::Deserialize(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tape_new_empty_valid() {
        let tape = Tape::new(42, TapeStart::NewGame, vec![], vec![], [0u8; 32]).unwrap();
        assert_eq!(tape.magic, *b"MCTAPE01");
        assert_eq!(tape.seed, 42);
        assert!(tape.is_empty());
    }

    #[test]
    fn tape_new_ascending_ticks() {
        let entries = vec![
            (0, Command::Interact),
            (1, Command::Move(mc_core::command::Dir::North)),
            (5, Command::OpenMenu),
        ];
        let tape = Tape::new(42, TapeStart::NewGame, entries, vec![], [0u8; 32]).unwrap();
        assert_eq!(tape.len(), 3);
    }

    #[test]
    fn tape_new_non_monotonic_ticks_rejected() {
        let entries = vec![
            (0, Command::Interact),
            (0, Command::Move(mc_core::command::Dir::North)),
        ];
        let err = Tape::new(42, TapeStart::NewGame, entries, vec![], [0u8; 32]).unwrap_err();
        assert!(
            matches!(&err, TapeError::NonMonotonicTicks { tick: 0, prev: 0 }),
            "expected NonMonotonicTicks, got {:?}",
            err
        );
    }

    #[test]
    fn tape_new_too_many_entries_rejected() {
        let entries = vec![(0u64, Command::Interact); MAX_TAPE_ENTRIES + 1];
        let err = Tape::new(42, TapeStart::NewGame, entries, vec![], [0u8; 32]).unwrap_err();
        assert!(matches!(&err, TapeError::BoundsExceeded(_)));
    }

    #[test]
    fn tape_roundtrip() {
        let entries = vec![
            (0, Command::Interact),
            (1, Command::Move(mc_core::command::Dir::East)),
        ];
        let tape = Tape::new(42, TapeStart::NewGame, entries, vec![], [0u8; 32]).unwrap();
        let bytes = tape.to_bytes().unwrap();
        let deserialized = Tape::from_bytes(&bytes).unwrap();
        assert_eq!(tape, deserialized);
    }

    #[test]
    fn bad_magic_rejected() {
        let entries = vec![(0, Command::Interact)];
        let tape = Tape::new(42, TapeStart::NewGame, entries, vec![], [0u8; 32]).unwrap();
        let mut bytes = tape.to_bytes().unwrap();
        bytes[0] = 0xFF; // corrupt magic
        let err = Tape::from_bytes(&bytes).unwrap_err();
        assert!(matches!(&err, TapeError::BadMagic));
    }

    #[test]
    fn truncated_data_rejected() {
        let entries = vec![(0, Command::Interact)];
        let tape = Tape::new(42, TapeStart::NewGame, entries, vec![], [0u8; 32]).unwrap();
        let bytes = tape.to_bytes().unwrap();
        let truncated = &bytes[..bytes.len() / 2];
        let err = Tape::from_bytes(truncated).unwrap_err();
        // Should be Truncated or Deserialize.
        assert!(
            matches!(&err, TapeError::Truncated | TapeError::Deserialize(_)),
            "expected truncation error, got {:?}",
            err
        );
    }
}
