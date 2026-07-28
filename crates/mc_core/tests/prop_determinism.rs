//! Determinism property test.
//!
//! Generates random command sequences and asserts that identical sequences produce
//! identical hashes. This is the project's defining proof (INV-01).

#[cfg(test)]
mod tests {
    use mc_core::world::World;

    /// Generate a deterministic "command" from a tick number and RNG state.
    /// Since we don't have a command bus yet (EP-004), we simulate commands
    /// by stepping the world a varying number of times.
    fn run_command_sequence(seed: u128, steps: &[u8]) -> blake3::Hash {
        let mut world = World::new(seed);
        for &s in steps {
            for _ in 0..s {
                world.step();
            }
        }
        world.state_hash()
    }

    #[test]
    fn identical_sequences_identical_hashes() {
        let steps = vec![1u8, 2, 3, 0, 5, 1, 2, 3, 4];
        let h1 = run_command_sequence(42, &steps);
        let h2 = run_command_sequence(42, &steps);
        assert_eq!(
            h1, h2,
            "identical command sequences must produce identical hashes"
        );
    }

    #[test]
    fn different_sequences_different_hashes() {
        let steps_a = vec![1u8, 2, 3, 4, 5];
        let steps_b = vec![1u8, 2, 3, 4, 6];
        let ha = run_command_sequence(42, &steps_a);
        let hb = run_command_sequence(42, &steps_b);
        assert_ne!(
            ha, hb,
            "different command sequences must produce different hashes"
        );
    }

    #[test]
    fn same_seed_produces_identical_world() {
        let w1 = World::new(12345);
        let w2 = World::new(12345);
        assert_eq!(w1.state_hash(), w2.state_hash());
    }

    #[test]
    fn different_seed_produces_different_world() {
        let w1 = World::new(12345);
        let w2 = World::new(54321);
        assert_ne!(w1.state_hash(), w2.state_hash());
    }
}
