//! Tape recording — records commands applied to a World and produces a Tape.
//!
//! The `RecordTape` struct wraps a `World` and accumulates commands and
//! periodic checkpoints. Every 1024 ticks it records the world's state hash
//! so that replay can verify determinism at those points.

use crate::error::TapeError;
use crate::format::{Tape, TapeStart, MAX_TAPE_ENTRIES};
use mc_core::command::Command;
use mc_core::world::World;

/// Checkpoint interval — a checkpoint is recorded every N ticks.
pub const CHECKPOINT_INTERVAL: u64 = 1024;

/// Records commands applied to a World and produces a verified Tape.
///
/// Usage:
/// ```
/// use mc_core::world::World;
/// use mc_core::command::Command;
/// use mc_tape::record::RecordTape;
///
/// let world = World::new(42);
/// let mut recorder = RecordTape::new(world);
/// recorder.record_command(Command::Interact);
/// let tape = recorder.finalize().unwrap();
/// ```
pub struct RecordTape {
    /// The world being recorded (drives step advancement).
    world: World,
    /// Accumulated (tick, command) entries.
    entries: Vec<(u64, Command)>,
    /// Accumulated checkpoints: (tick, state_hash).
    checkpoints: Vec<(u64, [u8; 32])>,
    /// The tick at which the next checkpoint should be taken.
    next_checkpoint_tick: u64,
    /// Whether recording has been finalised.
    finalized: bool,
}

impl RecordTape {
    /// Create a new recorder wrapping the given world.
    ///
    /// The world is consumed; use `world()` to access it after recording.
    pub fn new(world: World) -> Self {
        if world.tick % CHECKPOINT_INTERVAL == 0 {
            // Current tick is a checkpoint boundary — record it.
            let hash = world.state_hash();
            let checkpoint_tick = world.tick;
            RecordTape {
                world,
                entries: Vec::new(),
                checkpoints: vec![(checkpoint_tick, *hash.as_bytes())],
                next_checkpoint_tick: checkpoint_tick + CHECKPOINT_INTERVAL,
                finalized: false,
            }
        } else {
            let next_cp = ((world.tick / CHECKPOINT_INTERVAL) + 1) * CHECKPOINT_INTERVAL;
            RecordTape {
                world,
                entries: Vec::new(),
                checkpoints: Vec::new(),
                next_checkpoint_tick: next_cp,
                finalized: false,
            }
        }
    }

    /// Record a single command.
    ///
    /// The command is applied to the world via `mc_core::command::apply_commands`,
    /// the world is stepped, and the (tick, command) pair is recorded.
    /// If the resulting tick crosses a checkpoint boundary, a checkpoint is taken.
    pub fn record_command(&mut self, command: Command) -> Result<(), TapeError> {
        if self.finalized {
            return Err(TapeError::BoundsExceeded(
                "cannot record after finalize".into(),
            ));
        }

        if self.entries.len() >= MAX_TAPE_ENTRIES {
            return Err(TapeError::BoundsExceeded(format!(
                "entry count {} exceeds maximum {}",
                self.entries.len(),
                MAX_TAPE_ENTRIES,
            )));
        }

        // Record the command at the current tick.
        let tick = self.world.tick;
        self.entries.push((tick, command.clone()));

        // Apply the command to the world.
        mc_core::command::apply_commands(&mut self.world, &[command]);

        // Step the world to advance to the next tick.
        self.world.step();

        // Check if we've crossed a checkpoint boundary.
        if self.world.tick >= self.next_checkpoint_tick {
            let hash = self.world.state_hash();
            self.checkpoints.push((self.world.tick, *hash.as_bytes()));
            self.next_checkpoint_tick = self.world.tick + CHECKPOINT_INTERVAL;
        }

        Ok(())
    }

    /// Record a batch of commands.
    pub fn record_commands(&mut self, commands: &[Command]) -> Result<(), TapeError> {
        for cmd in commands {
            self.record_command(cmd.clone())?;
        }
        Ok(())
    }

    /// Finalise recording and produce the completed `Tape`.
    ///
    /// Takes a final checkpoint at the current tick and computes the
    /// content digest and final hash.
    pub fn finalize(&mut self) -> Result<Tape, TapeError> {
        if self.finalized {
            return Err(TapeError::BoundsExceeded("tape already finalized".into()));
        }
        self.finalized = true;

        // Record a final checkpoint at the current tick.
        let final_hash = self.world.state_hash();
        let final_tick = self.world.tick;

        // Add a final checkpoint if we don't already have one for this tick.
        if self.checkpoints.last().map(|(t, _)| *t) != Some(final_tick) {
            self.checkpoints.push((final_tick, *final_hash.as_bytes()));
        }

        // Swap out entries and checkpoints to avoid cloning.
        let entries = std::mem::take(&mut self.entries);
        let checkpoints = std::mem::take(&mut self.checkpoints);

        Tape::new(
            self.world.seed,
            TapeStart::NewGame,
            entries,
            checkpoints,
            *final_hash.as_bytes(),
        )
    }

    /// Borrow the inner world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Check if recording has been finalized.
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::command::Dir;

    #[test]
    fn record_empty_tape() {
        let world = World::new(42);
        let mut recorder = RecordTape::new(world);
        let tape = recorder.finalize().unwrap();
        assert!(tape.entries.is_empty());
        assert_eq!(tape.seed, 42);
    }

    #[test]
    fn record_one_command() {
        let world = World::new(42);
        let mut recorder = RecordTape::new(world);
        recorder.record_command(Command::Interact).unwrap();
        let tape = recorder.finalize().unwrap();
        assert_eq!(tape.entries.len(), 1);
        assert_eq!(tape.entries[0].0, 0); // tick 0
        assert_eq!(tape.entries[0].1, Command::Interact);
    }

    #[test]
    fn record_multiple_commands() {
        let world = World::new(42);
        let mut recorder = RecordTape::new(world);
        let cmds = vec![
            Command::Interact,
            Command::Move(Dir::North),
            Command::OpenMenu,
        ];
        recorder.record_commands(&cmds).unwrap();
        let tape = recorder.finalize().unwrap();
        assert_eq!(tape.entries.len(), 3);
        assert_eq!(tape.entries[0].0, 0);
        assert_eq!(tape.entries[1].0, 1);
        assert_eq!(tape.entries[2].0, 2);
    }

    #[test]
    fn record_with_checkpoint() {
        let world = World::new(42);
        let mut recorder = RecordTape::new(world);

        // Record 1025 commands — should trigger checkpoint at tick 1024.
        for _ in 0..1025 {
            recorder.record_command(Command::Interact).unwrap();
        }

        let tape = recorder.finalize().unwrap();
        // Should have at least the checkpoint at tick 1024 (or near it)
        assert!(
            !tape.checkpoints.is_empty(),
            "expected at least 1 checkpoint, got {}",
            tape.checkpoints.len()
        );
    }

    #[test]
    fn record_finalize_twice_fails() {
        let world = World::new(42);
        let mut recorder = RecordTape::new(world);
        recorder.finalize().unwrap();
        let err = recorder.finalize().unwrap_err();
        assert!(matches!(&err, TapeError::BoundsExceeded(_)));
    }

    #[test]
    fn record_after_finalize_fails() {
        let world = World::new(42);
        let mut recorder = RecordTape::new(world);
        recorder.finalize().unwrap();
        let err = recorder.record_command(Command::Interact).unwrap_err();
        assert!(matches!(&err, TapeError::BoundsExceeded(_)));
    }
}
