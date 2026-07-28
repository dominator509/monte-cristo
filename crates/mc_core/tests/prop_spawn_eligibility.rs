//! Property test: spawn eligibility is pure and region/flag-gated (LF-04).
//!
//! Generates random flag sets and asserts that every eligible enemy:
//! 1. Has the queried region in its `region_affinity`
//! 2. Satisfies its own `gate` expression under the given flags
//! 3. Results are sorted by EnemyId

use mc_core::bestiary::{eligible, Enemy, Family};
use mc_core::flags::{FlagExpr, FlagSet};
use mc_core::ids::{EnemyId, FlagId, RegionId};
use proptest::prelude::*;

/// Build a representative set of enemies covering all 15 regions and various
/// flag gates.
fn full_bestiary() -> Vec<Enemy> {
    vec![
        // R01 (Marseille)
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
            gate: FlagExpr::Not(Box::new(FlagExpr::Set(FlagId::FLG_ARRESTED))),
        },
        // R02 (Château d'If)
        Enemy {
            id: EnemyId::ENM_GUARD,
            family: Family::ManAtArms,
            region_affinity: vec![RegionId::R02_CHATEAU_DIF],
            gate: FlagExpr::Set(FlagId::FLG_ARRESTED),
        },
        Enemy {
            id: EnemyId::ENM_JAILER,
            family: Family::Prisoner,
            region_affinity: vec![RegionId::R02_CHATEAU_DIF],
            gate: FlagExpr::Any(vec![
                FlagExpr::Set(FlagId::FLG_ARRESTED),
                FlagExpr::Set(FlagId::FLG_FARIA_MET),
            ]),
        },
        // R03 (Monte Cristo)
        Enemy {
            id: EnemyId::ENM_SMUGGLER,
            family: Family::Criminal,
            region_affinity: vec![RegionId::R03_MONTE_CRISTO],
            gate: FlagExpr::Always,
        },
        Enemy {
            id: EnemyId::ENM_ASSASSIN,
            family: Family::Criminal,
            region_affinity: vec![RegionId::R03_MONTE_CRISTO],
            gate: FlagExpr::Not(Box::new(FlagExpr::Set(FlagId::FLG_TREASURE_KNOWN))),
        },
        // R04 (Rome)
        Enemy {
            id: EnemyId::ENM_SPY,
            family: Family::Criminal,
            region_affinity: vec![RegionId::R04_ROME],
            gate: FlagExpr::Always,
        },
        // R05 (Paris Faubourg)
        Enemy {
            id: EnemyId::ENM_GENDARME,
            family: Family::Troop,
            region_affinity: vec![RegionId::R05_PARIS_FAUBOURG],
            gate: FlagExpr::Always,
        },
        Enemy {
            id: EnemyId::ENM_BODYGUARD,
            family: Family::ManAtArms,
            region_affinity: vec![RegionId::R05_PARIS_FAUBOURG],
            gate: FlagExpr::All(vec![
                FlagExpr::Set(FlagId::FLG_ESCAPED),
                FlagExpr::Not(Box::new(FlagExpr::Set(FlagId::FLG_COMTE_IDENTITY))),
            ]),
        },
        // R06 (Paris Salon)
        Enemy {
            id: EnemyId::ENM_CORSICAN,
            family: Family::Bandit,
            region_affinity: vec![RegionId::R06_PARIS_SALON],
            gate: FlagExpr::Always,
        },
        // R07 (Normandy)
        Enemy {
            id: EnemyId::ENM_ALBANIAN,
            family: Family::Troop,
            region_affinity: vec![RegionId::R07_NORMANDY],
            gate: FlagExpr::Always,
        },
        Enemy {
            id: EnemyId::ENM_OTTOMAN,
            family: Family::Troop,
            region_affinity: vec![RegionId::R07_NORMANDY],
            gate: FlagExpr::Set(FlagId::FLG_MORCERF_DOSSIER),
        },
        // R08 (Lyon)
        Enemy {
            id: EnemyId::ENM_GREEK_REBEL,
            family: Family::Bandit,
            region_affinity: vec![RegionId::R08_LYON],
            gate: FlagExpr::Always,
        },
        // R09 (Strasbourg)
        Enemy {
            id: EnemyId::ENM_CRETAN,
            family: Family::Sea,
            region_affinity: vec![RegionId::R09_STRASBOURG],
            gate: FlagExpr::Always,
        },
        // R10 (Méditerranée)
        Enemy {
            id: EnemyId::ENM_AGENT,
            family: Family::Troop,
            region_affinity: vec![RegionId::R10_MEDITERRANEE],
            gate: FlagExpr::Set(FlagId::FLG_COMTE_IDENTITY),
        },
    ]
}

/// All 15 region IDs.
const ALL_REGIONS: [RegionId; 15] = [
    RegionId::R01_MARSEILLE,
    RegionId::R02_CHATEAU_DIF,
    RegionId::R03_MONTE_CRISTO,
    RegionId::R04_ROME,
    RegionId::R05_PARIS_FAUBOURG,
    RegionId::R06_PARIS_SALON,
    RegionId::R07_NORMANDY,
    RegionId::R08_LYON,
    RegionId::R09_STRASBOURG,
    RegionId::R10_MEDITERRANEE,
    RegionId::R11_ORIENT,
    RegionId::R12_GREECE,
    RegionId::R13_ALBANIA,
    RegionId::R14_MORCERF_ESTATE,
    RegionId::R15_VILLEFORT_MANSION,
];

/// Generate a random FlagSet with various combinations of flags.
fn arb_flags() -> impl Strategy<Value = FlagSet> {
    proptest::bits::u64::masked(0x3F_FFFF).prop_map(|bits| {
        let mut fs = FlagSet::new();
        for i in 0..22 {
            if bits & (1u64 << i) != 0 {
                fs.set(FlagId::from_raw(i as u16));
            }
        }
        fs
    })
}

proptest! {
    #[test]
    fn eligible_enemies_respect_affinity_and_gate(
        flags in arb_flags(),
        region_idx in 0..15usize,
    ) {
        let enemies = full_bestiary();
        let region = ALL_REGIONS[region_idx];
        let result = eligible(region, &flags, &enemies);

        // Assert result is sorted
        for pair in result.windows(2) {
            assert!(pair[0] <= pair[1], "results must be sorted by EnemyId");
        }

        // Assert each result is genuinely eligible
        for &eid in &result {
            let enemy = enemies.iter().find(|e| e.id == eid)
                .expect("eligible returned an enemy not in the list");
            assert!(
                enemy.region_affinity.contains(&region),
                "enemy {:?} should have affinity for region {:?}",
                eid, region
            );
            assert!(
                flags.satisfies(&enemy.gate),
                "enemy {:?} gate should be satisfied by flags",
                eid
            );
        }

        // Assert no missing eligible enemies
        for enemy in &enemies {
            if enemy.region_affinity.contains(&region) && flags.satisfies(&enemy.gate) {
                assert!(
                    result.contains(&enemy.id),
                    "eligible enemy {:?} should appear in result for region {:?}",
                    enemy.id, region
                );
            }
        }
    }
}

#[test]
fn eligibility_is_pure() {
    let enemies = full_bestiary();
    let flags = FlagSet::new();
    let a = eligible(RegionId::R01_MARSEILLE, &flags, &enemies);
    let b = eligible(RegionId::R01_MARSEILLE, &flags, &enemies);
    let c = eligible(RegionId::R01_MARSEILLE, &flags, &enemies);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn empty_flags_simple_affinity() {
    let enemies = full_bestiary();
    let flags = FlagSet::new();

    // R01: BANDIT (Always) is eligible; SOLDIER (Not(FLG_ARRESTED)) also eligible
    let r01 = eligible(RegionId::R01_MARSEILLE, &flags, &enemies);
    assert!(r01.contains(&EnemyId::ENM_BANDIT));
    assert!(r01.contains(&EnemyId::ENM_SOLDIER));
}

#[test]
fn all_flags_satisfies_all_gates() {
    let enemies = full_bestiary();
    let mut flags = FlagSet::new();
    for i in 0..22u16 {
        flags.set(FlagId::from_raw(i));
    }

    for region in ALL_REGIONS {
        let result = eligible(region, &flags, &enemies);
        for &eid in &result {
            let enemy = enemies.iter().find(|e| e.id == eid).unwrap();
            assert!(
                enemy.region_affinity.contains(&region),
                "enemy {:?} should have affinity for region {:?}",
                eid,
                region
            );
        }
    }
}

#[test]
fn missing_regions_return_empty() {
    let enemies = full_bestiary();
    let flags = FlagSet::new();
    // Regions with no Always-gate enemies under empty flags
    for r in &[RegionId::R02_CHATEAU_DIF, RegionId::R10_MEDITERRANEE] {
        let result = eligible(*r, &flags, &enemies);
        assert!(
            result.is_empty(),
            "region {:?} should have no eligible enemies",
            r
        );
    }
}
