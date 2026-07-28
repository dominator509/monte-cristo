//! Divergence detection — compares checkpoints between two tapes or between
//! a tape and a live replay, localised to a tick range.

use crate::format::Tape;
use crate::replay::{replay, ReplayResult};

/// A report of divergences found between two sources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DivergenceReport {
    /// The tick at which the first divergence was detected.
    pub first_divergent_tick: Option<u64>,

    /// All divergences found, in tick order.
    /// Each entry is `(tick, expected_hash, actual_hash)`.
    pub divergences: Vec<(u64, [u8; 32], [u8; 32])>,

    /// Total number of checkpoints compared.
    pub total_checked: usize,
}

/// Compare the checkpoints of two tapes directly (without replaying).
///
/// Returns all checkpoints where the two tapes differ, limited to the
/// intersection of their tick ranges.
pub fn compare_tapes(a: &Tape, b: &Tape) -> DivergenceReport {
    let mut divergences = Vec::new();
    let mut first_divergent_tick = None;

    // Build a map from tick -> hash for each tape.
    let a_map: std::collections::BTreeMap<u64, [u8; 32]> =
        a.checkpoints.iter().cloned().collect();
    let b_map: std::collections::BTreeMap<u64, [u8; 32]> =
        b.checkpoints.iter().cloned().collect();

    // Compare at all ticks present in both tapes.
    let mut total_checked = 0;
    for (&tick, &a_hash) in &a_map {
        if let Some(&b_hash) = b_map.get(&tick) {
            total_checked += 1;
            if a_hash != b_hash {
                if first_divergent_tick.is_none() {
                    first_divergent_tick = Some(tick);
                }
                divergences.push((tick, a_hash, b_hash));
            }
        }
    }

    // Also check for ticks present in b but not a (symmetry).
    for (&_tick, &_b_hash) in &b_map {
        if !a_map.contains_key(&_tick) {
            // This tick exists only in b — not a divergence per se,
            // but we could report it. For now we skip — compare only
            // shared ticks.
        }
    }

    DivergenceReport {
        first_divergent_tick,
        divergences,
        total_checked,
    }
}

/// Compare the checkpoints of a tape against a live replay of the same tape.
///
/// Replays the tape from scratch and compares each checkpoint hash
/// against the tape's recorded checkpoint at the same tick.
///
/// # Errors
///
/// Returns `TapeError` if replay fails (e.g. invalid tape data).
pub fn compare_replay(tape: &Tape) -> Result<DivergenceReport, crate::error::TapeError> {
    let ReplayResult {
        first_divergence, ..
    } = replay(tape)?;

    let mut divergences = Vec::new();
    let mut first_divergent_tick = None;
    let mut total_checked = 0;

    // Replay doesn't give us every checkpoint checked — only the first divergence.
    // So we rebuild by checking each checkpoint manually.
    let mut world = mc_core::world::World::new(tape.seed);

    // Build checkpoint map.
    let cp_map: std::collections::BTreeMap<u64, [u8; 32]> =
        tape.checkpoints.iter().cloned().collect();

    // Process entries the same way as replay, checking each checkpoint.
    for (tick, command) in &tape.entries {
        let cmd_tick = *tick;

        while world.tick < cmd_tick {
            world.step();
            if let Some(&expected) = cp_map.get(&world.tick) {
                total_checked += 1;
                let actual = *world.state_hash().as_bytes();
                if actual != expected {
                    if first_divergent_tick.is_none() {
                        first_divergent_tick = Some(world.tick);
                    }
                    divergences.push((world.tick, expected, actual));
                }
            }
        }

        mc_core::command::apply_commands(&mut world, &[command.clone()]);
        world.step();

        if let Some(&expected) = cp_map.get(&world.tick) {
            total_checked += 1;
            let actual = *world.state_hash().as_bytes();
            if actual != expected {
                if first_divergent_tick.is_none() {
                    first_divergent_tick = Some(world.tick);
                }
                divergences.push((world.tick, expected, actual));
            }
        }
    }

    // Check remaining checkpoints after the last entry.
    let max_entry_tick = tape.entries.last().map(|(t, _)| *t).unwrap_or(0);
    for (&cp_tick, &expected) in &cp_map {
        if cp_tick > max_entry_tick {
            while world.tick < cp_tick {
                world.step();
            }
            total_checked += 1;
            let actual = *world.state_hash().as_bytes();
            if actual != expected {
                if first_divergent_tick.is_none() {
                    first_divergent_tick = Some(cp_tick);
                }
                divergences.push((cp_tick, expected, actual));
            }
        }
    }

    // If replay found a divergence, use its tick as authoritative first.
    if let Some((tick, _, _)) = first_divergence {
        // Make sure our first_divergent_tick also reflects this.
        if first_divergent_tick.map_or(true, |ft| tick < ft) {
            first_divergent_tick = Some(tick);
        }
    }

    Ok(DivergenceReport {
        first_divergent_tick,
        divergences,
        total_checked,
    })
}

/// Compare the checkpoints of a tape against a specific set of expected
/// checkpoints, within a specific tick range `[range_start, range_end]`.
///
/// This is useful for focused divergence detection when you know the
/// approximate location of the divergence.
pub fn compare_in_range(
    tape: &Tape,
    expected_checkpoints: &[(u64, [u8; 32])],
    range_start: u64,
    range_end: u64,
) -> DivergenceReport {
    let mut divergences = Vec::new();
    let mut first_divergent_tick = None;
    let mut total_checked = 0;

    let expected_map: std::collections::BTreeMap<u64, [u8; 32]> =
        expected_checkpoints.iter().cloned().collect();
    let tape_map: std::collections::BTreeMap<u64, [u8; 32]> =
        tape.checkpoints.iter().cloned().collect();

    for (&tick, &expected) in &expected_map {
        if tick < range_start || tick > range_end {
            continue;
        }
        if let Some(&actual) = tape_map.get(&tick) {
            total_checked += 1;
            if actual != expected {
                if first_divergent_tick.is_none() {
                    first_divergent_tick = Some(tick);
                }
                divergences.push((tick, expected, actual));
            }
        }
    }

    DivergenceReport {
        first_divergent_tick,
        divergences,
        total_checked,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::TapeStart;
    use mc_core::command::Command;

    #[test]
    fn compare_identical_tapes() {
        let tape = Tape::new(42, TapeStart::NewGame, vec![(0, Command::Interact)], vec![], [0u8; 32]).unwrap();
        let report = compare_tapes(&tape, &tape);
        assert!(report.divergences.is_empty());
        assert!(report.first_divergent_tick.is_none());
    }

    #[test]
    fn compare_different_checkpoints() {
        let tape_a = Tape::new(
            42,
            TapeStart::NewGame,
            vec![(0, Command::Interact)],
            vec![(1024, [1u8; 32])],
            [0u8; 32],
        )
        .unwrap();
        let tape_b = Tape::new(
            42,
            TapeStart::NewGame,
            vec![(0, Command::Interact)],
            vec![(1024, [2u8; 32])],
            [0u8; 32],
        )
        .unwrap();
        let report = compare_tapes(&tape_a, &tape_b);
        assert_eq!(report.divergences.len(), 1);
        assert_eq!(report.first_divergent_tick, Some(1024));
    }
}
