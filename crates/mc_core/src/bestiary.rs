//! The bestiary model — Family, Enemy, and the pure eligibility function.
//!
//! INV-11: Enemy eligibility is a pure function of `(region, flags)` with no
//! other inputs — no randomness, no special cases, no exceptions.
//! INV-06: Enemy definitions (region_affinity, gate) live in content, not code.
//! This module defines the *types and mechanism* only.

use crate::flags::FlagExpr;
use crate::flags::FlagSet;
use crate::ids::EnemyId;
use crate::ids::RegionId;
use serde::{Deserialize, Serialize};

/// Closed set of enemy families (SPEC-009 section 2 — design law L1).
///
/// Exactly these ten. Adding a variant requires changing SPEC-009 and an ADR.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Family {
    Vermin,
    Beast,
    Sea,
    ManAtArms,
    Criminal,
    Prisoner,
    Troop,
    Bandit,
    Hazard,
    Boss,
}

impl Family {
    /// All ten family variants in definition order.
    pub const ALL: &[Family] = &[
        Family::Vermin,
        Family::Beast,
        Family::Sea,
        Family::ManAtArms,
        Family::Criminal,
        Family::Prisoner,
        Family::Troop,
        Family::Bandit,
        Family::Hazard,
        Family::Boss,
    ];

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            Family::Vermin => "Vermin",
            Family::Beast => "Beast",
            Family::Sea => "Sea",
            Family::ManAtArms => "Man-at-Arms",
            Family::Criminal => "Criminal",
            Family::Prisoner => "Prisoner",
            Family::Troop => "Troop",
            Family::Bandit => "Bandit",
            Family::Hazard => "Hazard",
            Family::Boss => "Boss",
        }
    }
}

/// An entry in the bestiary — authored in content, used by `eligible`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enemy {
    /// The enemy's locked identifier.
    pub id: EnemyId,
    /// The enemy's family (determines status immunity rules, Terror, etc.).
    pub family: Family,
    /// Regions where this enemy may spawn.
    pub region_affinity: Vec<RegionId>,
    /// Flag gate that must be satisfied for this enemy to be eligible.
    pub gate: FlagExpr,
}

/// Pure eligibility check (INV-11).
///
/// Returns every `EnemyId` whose `region_affinity` contains `region` AND whose
/// `gate` is satisfied by `flags`. Results are sorted by `EnemyId` and deduplicated.
///
/// # Invariants
///
/// - No randomness: same `(region, flags, enemies)` always returns the same result.
/// - No special cases: every enemy follows exactly the same rule.
/// - No I/O: operates entirely on in-memory arguments.
pub fn eligible(region: RegionId, flags: &FlagSet, enemies: &[Enemy]) -> Vec<EnemyId> {
    let mut result: Vec<EnemyId> = enemies
        .iter()
        .filter(|e| e.region_affinity.contains(&region) && flags.satisfies(&e.gate))
        .map(|e| e.id)
        .collect();
    result.sort();
    result.dedup();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flags::FlagSet;
    use crate::ids::FlagId;

    fn sample_enemies() -> Vec<Enemy> {
        vec![
            Enemy {
                id: EnemyId::ENM_BANDIT,
                family: Family::Bandit,
                region_affinity: vec![RegionId::R01_MARSEILLE, RegionId::R05_PARIS_FAUBOURG],
                gate: FlagExpr::Always,
            },
            Enemy {
                id: EnemyId::ENM_SOLDIER,
                family: Family::ManAtArms,
                region_affinity: vec![RegionId::R01_MARSEILLE],
                gate: FlagExpr::Not(Box::new(FlagExpr::Set(FlagId::FLG_ARRESTED))),
            },
            Enemy {
                id: EnemyId::ENM_GUARD,
                family: Family::ManAtArms,
                region_affinity: vec![RegionId::R02_CHATEAU_DIF, RegionId::R05_PARIS_FAUBOURG],
                gate: FlagExpr::Set(FlagId::FLG_ARRESTED),
            },
        ]
    }

    #[test]
    fn eligible_empty_set() {
        let flags = FlagSet::new();
        let result = eligible(RegionId::R01_MARSEILLE, &flags, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn eligible_region_affinity_matches() {
        let flags = FlagSet::new();
        let enemies = sample_enemies();
        // Bandit has affinity for R01 and R05, gate Always
        let result = eligible(RegionId::R01_MARSEILLE, &flags, &enemies);
        assert!(result.contains(&EnemyId::ENM_BANDIT));
    }

    #[test]
    fn eligible_region_mismatch() {
        let mut flags = FlagSet::new();
        flags.set(FlagId::FLG_ARRESTED);
        let enemies = sample_enemies();
        // Soldier only has affinity for R01, not R02
        let result = eligible(RegionId::R02_CHATEAU_DIF, &flags, &enemies);
        assert!(!result.contains(&EnemyId::ENM_SOLDIER));
    }

    #[test]
    fn eligible_gate_satisfied() {
        let mut flags = FlagSet::new();
        flags.set(FlagId::FLG_ARRESTED);
        let enemies = sample_enemies();
        // Guard requires FLG_ARRESTED and has affinity for R02
        let result = eligible(RegionId::R02_CHATEAU_DIF, &flags, &enemies);
        assert!(result.contains(&EnemyId::ENM_GUARD));
    }

    #[test]
    fn eligible_gate_not_satisfied() {
        let flags = FlagSet::new();
        let enemies = sample_enemies();
        // Soldier requires FLG_ARRESTED to NOT be set (gate Not(Set(ARRESTED)))
        let result = eligible(RegionId::R01_MARSEILLE, &flags, &enemies);
        assert!(result.contains(&EnemyId::ENM_SOLDIER));
    }

    #[test]
    fn eligible_sorted_result() {
        let flags = FlagSet::new();
        let enemies = sample_enemies();
        let result = eligible(RegionId::R01_MARSEILLE, &flags, &enemies);
        // Should contain Bandit (id=0) and Soldier (id=1) — sorted ascending
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], EnemyId::ENM_BANDIT);
        assert_eq!(result[1], EnemyId::ENM_SOLDIER);
    }

    #[test]
    fn eligible_no_duplicates() {
        let enemies = vec![
            Enemy {
                id: EnemyId::ENM_BANDIT,
                family: Family::Bandit,
                region_affinity: vec![
                    RegionId::R01_MARSEILLE,
                    RegionId::R01_MARSEILLE, // duplicate affinity
                ],
                gate: FlagExpr::Always,
            },
            Enemy {
                id: EnemyId::ENM_BANDIT,
                family: Family::Bandit,
                region_affinity: vec![RegionId::R01_MARSEILLE],
                gate: FlagExpr::Always,
            },
        ];
        let flags = FlagSet::new();
        let result = eligible(RegionId::R01_MARSEILLE, &flags, &enemies);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn family_names() {
        assert_eq!(Family::Vermin.name(), "Vermin");
        assert_eq!(Family::Boss.name(), "Boss");
        assert_eq!(Family::Hazard.name(), "Hazard");
    }

    #[test]
    fn all_families_count() {
        assert_eq!(Family::ALL.len(), 10);
    }
}
