//! Spawn resolution — picks an eligible enemy for an encounter.
//!
//! Uses `bestiary::eligible` to filter enemies by region and flag gate, then
//! picks uniformly from the eligible set. When a ContentDb is available, the
//! pick will be weighted by authored spawn weights. For now, uniform.

use crate::bestiary::{eligible, Enemy};
use crate::flags::FlagSet;
use crate::ids::EnemyId;
use crate::ids::RegionId;
use crate::rng::Rng;

/// Resolve one spawn: pick an eligible enemy uniformly at random.
///
/// Returns `None` when no enemy is eligible for the given region and flags.
///
/// # Determinism
///
/// The result is a pure function of `(rng, region, flags, enemies)` — identical
/// inputs always produce the same output. The RNG is advanced exactly once
/// (one `next_range` call) per successful resolution.
pub fn resolve_spawn(
    rng: &mut Rng,
    region: RegionId,
    flags: &FlagSet,
    enemies: &[Enemy],
) -> Option<EnemyId> {
    let pool = eligible(region, flags, enemies);
    if pool.is_empty() {
        return None;
    }
    let idx = rng.next_range(0, pool.len() as u32 - 1);
    Some(pool[idx as usize])
}

/// Resolve multiple spawns, picking without replacement from the eligible set.
///
/// Returns up to `count` unique enemies. Fewer may be returned if the eligible
/// set is smaller than `count` or if the budget is exhausted (checked by caller).
pub fn resolve_spawns(
    rng: &mut Rng,
    region: RegionId,
    flags: &FlagSet,
    enemies: &[Enemy],
    count: usize,
) -> Vec<EnemyId> {
    let mut pool = eligible(region, flags, enemies);
    if pool.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(count.min(pool.len()));
    while result.len() < count && !pool.is_empty() {
        let idx = rng.next_range(0, pool.len() as u32 - 1) as usize;
        result.push(pool[idx]);
        pool.swap_remove(idx);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bestiary::Family;
    use crate::flags::FlagExpr;
    use crate::flags::FlagSet;
    use crate::ids::FlagId;
    use crate::rng::Rng;

    fn sample_enemies() -> Vec<Enemy> {
        vec![
            Enemy {
                id: EnemyId::ENM_BANDIT,
                family: Family::Bandit,
                region_affinity: vec![RegionId::R01_MARSEILLE],
                gate: FlagExpr::Always,
            },
            Enemy {
                id: EnemyId::ENM_SOLDIER,
                family: Family::ManAtArms,
                region_affinity: vec![RegionId::R01_MARSEILLE],
                gate: FlagExpr::Always,
            },
            Enemy {
                id: EnemyId::ENM_GUARD,
                family: Family::ManAtArms,
                region_affinity: vec![RegionId::R02_CHATEAU_DIF],
                gate: FlagExpr::Set(FlagId::FLG_ARRESTED),
            },
        ]
    }

    #[test]
    fn resolve_returns_some_when_eligible() {
        let mut rng = Rng::new(42);
        let flags = FlagSet::new();
        let enemies = sample_enemies();
        let result = resolve_spawn(&mut rng, RegionId::R01_MARSEILLE, &flags, &enemies);
        assert!(result.is_some());
        let id = result.unwrap();
        // Should be one of the R01-eligible enemies: BANDIT or SOLDIER
        assert!(id == EnemyId::ENM_BANDIT || id == EnemyId::ENM_SOLDIER);
    }

    #[test]
    fn resolve_returns_none_when_no_eligible() {
        let mut rng = Rng::new(42);
        let flags = FlagSet::new(); // FLG_ARRESTED not set
        let enemies = sample_enemies();
        // Guard requires FLG_ARRESTED and region R02
        let result = resolve_spawn(&mut rng, RegionId::R02_CHATEAU_DIF, &flags, &enemies);
        assert!(result.is_none());
    }

    #[test]
    fn resolve_empty_enemies() {
        let mut rng = Rng::new(42);
        let flags = FlagSet::new();
        let result = resolve_spawn(&mut rng, RegionId::R01_MARSEILLE, &flags, &[]);
        assert!(result.is_none());
    }

    #[test]
    fn resolve_deterministic() {
        let enemies = sample_enemies();
        let flags = FlagSet::new();
        let mut rng1 = Rng::new(42);
        let mut rng2 = Rng::new(42);
        let a = resolve_spawn(&mut rng1, RegionId::R01_MARSEILLE, &flags, &enemies);
        let b = resolve_spawn(&mut rng2, RegionId::R01_MARSEILLE, &flags, &enemies);
        assert_eq!(a, b);
    }

    #[test]
    fn resolve_spawns_returns_unique() {
        let mut rng = Rng::new(42);
        let flags = FlagSet::new();
        let enemies = sample_enemies();
        // Only 2 eligible (BANDIT, SOLDIER), request 5, should get 2 unique
        let result = resolve_spawns(&mut rng, RegionId::R01_MARSEILLE, &flags, &enemies, 5);
        assert_eq!(result.len(), 2);
        // Verify uniqueness
        let mut sorted = result.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), result.len());
    }
}
