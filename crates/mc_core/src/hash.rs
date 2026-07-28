//! State hashing for determinism verification.
//!
//! Uses blake3 over a canonical postcard encoding with sorted map iteration.
//! Every test that asserts determinism uses this function.

use crate::world::World;

impl World {
    /// Compute a deterministic hash of the current world state.
    ///
    /// Uses blake3 over canonical postcard encoding. This is the function that
    /// proves or disproves INV-01 in tests.
    pub fn state_hash(&self) -> blake3::Hash {
        let encoded = postcard::to_allocvec(self).expect("World serialization should never fail");
        blake3::hash(&encoded)
    }
}

#[cfg(test)]
mod tests {
    use crate::world::World;

    #[test]
    fn hash_is_deterministic() {
        let a = World::new(42);
        let b = World::new(42);
        assert_eq!(a.state_hash(), b.state_hash());
    }

    #[test]
    fn different_seeds_different_hashes() {
        let a = World::new(42);
        let b = World::new(99);
        let ha = a.state_hash();
        let hb = b.state_hash();
        assert_ne!(ha, hb, "different seeds must produce different hashes");
    }

    #[test]
    fn step_changes_hash() {
        let mut a = World::new(42);
        let h0 = a.state_hash();
        a.step();
        let h1 = a.state_hash();
        assert_ne!(h0, h1, "stepping the world must change its hash");
    }

    #[test]
    fn same_steps_same_hash() {
        let mut a = World::new(42);
        let mut b = World::new(42);
        for _ in 0..10 {
            a.step();
            b.step();
            assert_eq!(
                a.state_hash(),
                b.state_hash(),
                "hash divergence at tick {}",
                a.tick
            );
        }
    }
}
