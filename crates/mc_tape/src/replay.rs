//! Tape replay — replays a Tape against a fresh World and verifies checkpoints.
//!
//! SPEC-003 section 5 defines the replay contract:
//! - Creates a fresh `World` with the tape's seed.
//! - Steps the world for each entry's tick range (world.step()).
//! - At each checkpoint tick, computes `world.state_hash()` and compares.
//! - Returns the first divergence, if any.

use crate::error::TapeError;
use crate::format::Tape;
use mc_core::command::apply_commands;
use mc_core::world::World;

/// The result of replaying a tape.
#[derive(Debug, Clone)]
pub struct ReplayResult {
    /// The final state hash after all entries have been replayed.
    pub final_hash: [u8; 32],

    /// The first checkpoint where replay diverged, if any.
    ///
    /// `(checkpoint_tick, expected_hash, actual_hash)`
    pub first_divergence: Option<(u64, [u8; 32], [u8; 32])>,

    /// The final World state after replay completes. Used for flag assertions.
    pub final_world: World,
}

/// Replay a tape against a freshly created world with the tape's seed.
///
/// The replay proceeds as follows:
/// 1. Create `World::new(tape.seed)`.
/// 2. Group entries by contiguous tick ranges. For each group:
///    a. Apply the commands via `apply_commands`.
///    b. Step the world.
///    c. If the post-step tick matches a checkpoint tick, verify the hash.
/// 3. After all entries, verify any remaining unchecked checkpoints
///    and compute the final hash.
///
/// Returns the final hash and, if any checkpoint diverged, the first such
/// divergence with the tick, expected hash, and actual hash.
///
/// # Errors
///
/// Returns `TapeError` if the tape data is invalid (e.g. non-monotonic ticks).
pub fn replay(tape: &Tape) -> Result<ReplayResult, TapeError> {
    let mut world = World::new(tape.seed);

    // Build a set of checkpoint ticks for O(1) lookup.
    let mut checkpoint_map: std::collections::BTreeMap<u64, [u8; 32]> =
        std::collections::BTreeMap::new();
    for &(tick, hash) in &tape.checkpoints {
        checkpoint_map.insert(tick, hash);
    }

    let mut first_divergence: Option<(u64, [u8; 32], [u8; 32])> = None;

    // Track which checkpoint ticks we've verified so we can detect missing ones.
    let mut verified_ticks: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();

    // Process each entry.
    for (tick, command) in &tape.entries {
        let cmd_tick = *tick;

        // Step the world forward to match the entry's tick.
        // The world advances one tick per step; we keep stepping until
        // world.tick matches cmd_tick, then apply the command.
        while world.tick < cmd_tick {
            world.step();

            // Check if this post-step tick has a checkpoint that hasn't been
            // verified yet.
            if let Some(&expected_hash) = checkpoint_map.get(&world.tick) {
                if !verified_ticks.contains(&world.tick) {
                    verified_ticks.insert(world.tick);
                    let actual_hash = world.state_hash();
                    if actual_hash.as_bytes() != &expected_hash && first_divergence.is_none() {
                        first_divergence =
                            Some((world.tick, expected_hash, *actual_hash.as_bytes()));
                    }
                }
            }
        }

        // Apply the command at this tick.
        apply_commands(&mut world, &[command.clone()]);

        // Step after command application.
        world.step();

        // Check for checkpoint.
        if let Some(&expected_hash) = checkpoint_map.get(&world.tick) {
            if !verified_ticks.contains(&world.tick) {
                verified_ticks.insert(world.tick);
                let actual_hash = world.state_hash();
                if actual_hash.as_bytes() != &expected_hash && first_divergence.is_none() {
                    first_divergence = Some((world.tick, expected_hash, *actual_hash.as_bytes()));
                }
            }
        }
    }

    // After processing all entries, step the world to the last checkpoint
    // tick if there are any remaining unverified checkpoints.
    let max_entry_tick = tape.entries.last().map(|(t, _)| *t).unwrap_or(0);
    for (&cp_tick, &expected_hash) in &checkpoint_map {
        if cp_tick > max_entry_tick && !verified_ticks.contains(&cp_tick) {
            // Step the world forward to this checkpoint tick.
            while world.tick < cp_tick {
                world.step();
            }
            verified_ticks.insert(cp_tick);
            let actual_hash = world.state_hash();
            if actual_hash.as_bytes() != &expected_hash && first_divergence.is_none() {
                first_divergence = Some((cp_tick, expected_hash, *actual_hash.as_bytes()));
            }
        }
    }

    let final_hash = world.state_hash();
    let final_hash_bytes = *final_hash.as_bytes();

    Ok(ReplayResult {
        final_hash: final_hash_bytes,
        first_divergence,
        final_world: world,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::Tape;
    use mc_core::command::Command;

    #[test]
    fn replay_empty_tape() {
        let tape = Tape::new(
            42,
            crate::format::TapeStart::NewGame,
            vec![],
            vec![],
            [0u8; 32],
        )
        .unwrap();
        let result = replay(&tape).unwrap();
        // Final hash should match a fresh world's state hash.
        let fresh_world = World::new(42);
        let expected_hash = *fresh_world.state_hash().as_bytes();
        assert_eq!(result.final_hash, expected_hash);
        assert!(result.first_divergence.is_none());
    }

    #[test]
    fn replay_single_command() {
        let tape = Tape::new(
            42,
            crate::format::TapeStart::NewGame,
            vec![(0, Command::Interact)],
            vec![],
            [0u8; 32],
        )
        .unwrap();
        let result = replay(&tape).unwrap();
        // Should not diverge (we didn't set checkpoints).
        assert!(result.first_divergence.is_none());
    }

    #[test]
    fn replay_with_matching_checkpoints() {
        // Record some commands and verify that replay matches.
        use crate::record::RecordTape;
        let world = World::new(42);
        let mut recorder = RecordTape::new(world);
        for _ in 0..5 {
            recorder.record_command(Command::Interact).unwrap();
        }
        let tape = recorder.finalize().unwrap();

        let result = replay(&tape).unwrap();
        assert!(
            result.first_divergence.is_none(),
            "expected no divergence, got {:?}",
            result.first_divergence
        );
        assert_eq!(result.final_hash, tape.final_hash);
    }
}
