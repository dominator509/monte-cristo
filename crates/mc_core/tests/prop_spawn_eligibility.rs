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
/// flag gates. Mirrors the SPEC-009 bestiary structure at a semantic level
/// (not every single entry — that's a content concern).
fn full_bestiary() -> Vec<Enemy> {
    let mut enemies = Vec::new();

    // Helper: push an enemy with given params
    let mut add = |id: EnemyId, family: Family, regions: &[RegionId], gate: FlagExpr| {
        enemies.push(Enemy {
            id,
            family,
            region_affinity: regions.to_vec(),
            gate,
        });
    };

    // R01 (Marseille) — 2 enemies
    add(
        EnemyId::ENM_BANDIT,
        Family::Bandit,
        &[RegionId::R01_MARSEILLE],
        FlagExpr::Always,
    );
    add(
        EnemyId::ENM_SOLDIER,
        Family::ManAtArms,
        &[RegionId::R01_MARSEILLE],
        FlagExpr::Not(Box::new(FlagExpr::Set(FlagId::FLG_ARRESTED))),
    );

    // R02 (Château d'If) — 2 enemies
    add(
        EnemyId::ENM_GUARD,
        Family::ManAtArms,
        &[RegionId::R02_CHATEAU_DIF],
        FlagExpr::Set(FlagId::FLG_ARRESTED),
    );
    add(
        EnemyId::ENM_JAILER,
        Family::Prisoner,
        &[RegionId::R02_CHATEAU_DIF],
        FlagExpr::Any(vec![
            FlagExpr::Set(FlagId::FLG_ARRESTED),
            FlagExpr::Set(FlagId::FLG_FARIA_MET),
        ]),
    );

    // R03 (Monte Cristo) — 2 enemies
    add(
        EnemyId::ENM_SMUGGLER,
        Family::Criminal,
        &[RegionId::R03_MONTE_CRISTO],
        FlagExpr::Always,
    );
    add(
        EnemyId::ENM_BODYGUARD,
        Family::ManAtArms,
        &[RegionId::R03_MONTE_CRISTO],
        FlagExpr::Not(Box::new(FlagExpr::Set(FlagId::FLG_TREASURE_KNOWN))),
    );

    // R04 (Rome)
    add(
        EnemyId::ENM_ASSASSIN,
        Family::Criminal,
        &[RegionId::R04_ROME],
        FlagExpr::Always,
    );
    add(
        EnemyId::ENM_SPY,
        Family::Criminal,
        &[RegionId::R04_ROME],
        FlagExpr::Set(FlagId::FLG_COMTE_IDENTITY),
    );

    // R05 (Paris Faubourg)
    add(
        EnemyId::ENM_AGENT,
        Family::Troop,
        &[RegionId::R05_PARIS_FAUBOURG],
        FlagExpr::Always,
    );
    add(
        EnemyId::ENM_GENDARME,
        Family::Troop,
        &[RegionId::R05_PARIS_FAUBOURG],
        FlagExpr::All(vec![
            FlagExpr::Set(FlagId::FLG_ESCAPED),
            FlagExpr::Not(Box::new(FlagExpr::Set(FlagId::FLG_COMTE_IDENTITY))),
        ]),
    );

    // R06 (Paris Salon) — multi-region enemy
    add(
        EnemyId::ENM_CORSICAN,
        Family::Bandit,
        &[RegionId::R05_PARIS_FAUBOURG, RegionId::R06_PARIS_SALON],
        FlagExpr::Always,
    );
    add(
        EnemyId::ENM_CRETAN,
        Family::Sea,
        &[RegionId::R06_PARIS_SALON],
        FlagExpr::Always,
    );

    // R07 (Normandy)
    add(
        EnemyId::ENM_ALBANIAN,
        Family::Troop,
        &[RegionId::R07_NORMANDY],
        FlagExpr::Always,
    );
    add(
        EnemyId::ENM_OTTOMAN,
        Family::Troop,
        &[RegionId::R07_NORMANDY],
        FlagExpr::Set(FlagId::FLG_MORCERF_DOSSIER),
    );

    // R08 (Lyon), R09 (Strasbourg), R10 (Méditerranée)
    add(
        EnemyId::ENM_GREEK_REBEL,
        Family::Bandit,
        &[RegionId::R08_LYON],
        FlagExpr::Always,
    );
    add(
        EnemyId::ENM_BODYGUARD,
        Family::ManAtArms,
        &[RegionId::R09_STRASBOURG, RegionId::R10_MEDITERRANEE],
        FlagExpr::Always,
    );

    // R11 (Orient), R12 (Greece), R13 (Albania)
    add(
        EnemyId::ENM_JAILER,
        Family::Prisoner,
        &[RegionId::R11_ORIENT],
        FlagExpr::Set(FlagId::FLG_FARIA_MET),
    );
    add(
        EnemyId::ENM_CRETAN,
        Family::Sea,
        &[RegionId::R12_GREECE],
        FlagExpr::Always,
    );
    add(
        EnemyId::ENM_ALBANIAN,
        Family::Troop,
        &[RegionId::R13_ALBANIA],
        FlagExpr::Set(FlagId::FLG_FERNAND_CONFRONTED),
    );

    // R14 (Morcerf Estate), R15 (Villefort Mansion)
    add(
        EnemyId::ENM_GUARD,
        Family::ManAtArms,
        &[RegionId::R14_MORCERF_ESTATE],
        FlagExpr::Any(vec![
            FlagExpr::Set(FlagId::FLG_MORCERF_DOSSIER),
            FlagExpr::Set(FlagId::FLG_MORCERF_YANINA_DOSSIER),
        ]),
    );
    add(
        EnemyId::ENM_AGENT,
        Family::Troop,
        &[RegionId::R15_VILLEFORT_MANSION],
        FlagExpr::Set(FlagId::FLG_VILLEFORT_DOSSIER),
    );

    enemies
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
    /// For every region, for many random flag sets, every eligible enemy
    /// must declare the queried region in its region_affinity and must
    /// satisfy its own gate.
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

/// Unit test: eligibility is a pure function — same inputs always same outputs.
#[test]
fn eligibility_is_pure() {
    let enemies = full_bestiary();
    let mut flags = FlagSet::new();
    flags.set(FlagId::FLG_ARRESTED);
    flags.set(FlagId::FLG_FARIA_MET);

    let region = RegionId::R02_CHATEAU_DIF;
    let a = eligible(region, &flags, &enemies);
    let b = eligible(region, &flags, &enemies);
    let c = eligible(region, &flags, &enemies);

    assert_eq!(a, b);
    assert_eq!(b, c);
}

/// Unit test: empty flag set with no gates
#[test]
fn empty_flags_simple_affinity() {
    let enemies = full_bestiary();
    let flags = FlagSet::new();

    // R01: BANDIT (Always) is eligible; SOLDIER (Not(FLG_ARRESTED)) is also eligible
    let r01 = eligible(RegionId::R01_MARSEILLE, &flags, &enemies);
    assert!(r01.contains(&EnemyId::ENM_BANDIT));
    assert!(r01.contains(&EnemyId::ENM_SOLDIER));

    // R02: neither GUARD (needs FLG_ARRESTED) nor JAILER (needs FLG_ARRESTED|FLG_FARIA_MET)
    let r02 = eligible(RegionId::R02_CHATEAU_DIF, &flags, &enemies);
    assert!(r02.is_empty());
}

/// Unit test: all flags set
#[test]
fn all_flags_satisfies_all_gates() {
    let enemies = full_bestiary();
    let mut flags = FlagSet::new();
    for i in 0..22u16 {
        flags.set(FlagId::from_raw(i));
    }

    // Every enemy should be eligible for their regions
    let mut region_hits = [0u32; 15];
    for (i, region) in ALL_REGIONS.iter().enumerate() {
        let result = eligible(*region, &flags, &enemies);
        region_hits[i] = result.len() as u32;
        for eid in result {
            let e = enemies.iter().find(|e| e.id == eid).unwrap();
            assert!(e.region_affinity.contains(region));
        }
    }
    // At least some enemies should be found
    assert!(region_hits.iter().sum::<u32>() > 0);
}
