//! Additional coverage tests for mc_core modules.
//!
//! Exercises uncovered branches in core logic to help reach
//! the 85% line-coverage floor (TESTING.md §8, SPEC-008 §3).
//! These tests target the modules flagged as undercovered by
//! `cargo llvm-cov --workspace --summary-only`.

use mc_core::battle::atb::AtbGauge;
use mc_core::battle::status::StatusList;
use mc_core::battle::{Affiliation, Battle, BattleState, Combatant, CombatantKind};
use mc_core::bestiary::Family;
use mc_core::flags::FlagSet;
use mc_core::fx::Fx;
use mc_core::ids::{CharId, EnemyId, FlagId, ItemId, RegionId, SceneId};
use mc_core::poison::lookup;

// ── ids.rs coverage ──────────────────────────────────────────────────

#[test]
fn region_id_from_raw_roundtrip() {
    for i in 0..RegionId::COUNT as u16 {
        let id = RegionId::from_raw(i);
        assert_eq!(id.raw(), i, "RegionId({}) roundtrip", i);
        let name = id.name();
        assert!(!name.is_empty(), "RegionId({}) should have a name", i);
    }
}

#[test]
fn char_id_from_raw_roundtrip() {
    for i in 0..CharId::COUNT as u16 {
        let id = CharId::from_raw(i);
        assert_eq!(id.raw(), i, "CharId({}) roundtrip", i);
    }
}

#[test]
fn enemy_id_from_raw_roundtrip() {
    for i in 0..EnemyId::COUNT as u16 {
        let id = EnemyId::from_raw(i);
        assert_eq!(id.raw(), i, "EnemyId({}) roundtrip", i);
    }
}

#[test]
fn flag_id_from_raw_roundtrip() {
    for i in 0..FlagId::COUNT as u16 {
        let id = FlagId::from_raw(i);
        assert_eq!(id.raw(), i, "FlagId({}) roundtrip", i);
    }
}

#[test]
fn item_id_from_raw_roundtrip() {
    for i in 0..ItemId::COUNT as u16 {
        let id = ItemId::from_raw(i);
        assert_eq!(id.raw(), i, "ItemId({}) roundtrip", i);
    }
}

#[test]
fn scene_id_from_raw_roundtrip() {
    for i in 0..SceneId::COUNT as u16 {
        let id = SceneId::from_raw(i);
        assert_eq!(id.raw(), i, "SceneId({}) roundtrip", i);
    }
}

#[test]
fn unknown_ids_return_default_names() {
    let rid = RegionId::from_raw(999);
    assert!(
        rid.name().contains("UNKNOWN"),
        "unknown RegionId name: {}",
        rid.name()
    );
    let eid = EnemyId::from_raw(999);
    assert!(
        eid.name().contains("UNKNOWN"),
        "unknown EnemyId name: {}",
        eid.name()
    );
    let fid = FlagId::from_raw(999);
    assert!(
        fid.name().contains("UNKNOWN"),
        "unknown FlagId name: {}",
        fid.name()
    );
}

// ── flag.rs coverage ─────────────────────────────────────────────────

#[test]
fn flags_set_and_clear_multiple() {
    let mut fs = FlagSet::new();
    // Initially all clear
    for i in 0..5 {
        assert!(
            !fs.is_set(FlagId::from_raw(i)),
            "flag {} should start clear",
            i
        );
    }
    // Set some
    fs.set(FlagId::from_raw(1));
    fs.set(FlagId::from_raw(3));
    assert!(fs.is_set(FlagId::from_raw(1)));
    assert!(!fs.is_set(FlagId::from_raw(0)));
    assert!(fs.is_set(FlagId::from_raw(3)));
    // Clear one
    fs.clear(FlagId::from_raw(1));
    assert!(!fs.is_set(FlagId::from_raw(1)));
    assert!(fs.is_set(FlagId::from_raw(3)));
}

// ── bestiary.rs coverage ─────────────────────────────────────────────

#[test]
fn family_discriminants_are_distinct() {
    let all = Family::ALL;
    assert!(
        all.len() >= 8,
        "expected at least 8 families, got {}",
        all.len()
    );
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_ne!(
                std::mem::discriminant(&all[i]),
                std::mem::discriminant(&all[j]),
                "families at {} and {} should be distinct",
                i,
                j
            );
        }
    }
}

#[test]
fn family_name_is_non_empty() {
    for family in Family::ALL {
        let s = format!("{:?}", family);
        assert!(!s.is_empty(), "Family should have debug name");
        assert!(!family.name().is_empty(), "Family should have text name");
    }
}

// ── poison.rs coverage ───────────────────────────────────────────────

#[test]
fn poison_lookup_all_ids() {
    for i in 0..5 {
        let pid = mc_core::ids::PoisonId::from_raw(i);
        let def = lookup(pid);
        assert!(def.is_some(), "PoisonId({}) should have a definition", i);
        let def = def.unwrap();
        assert!(
            def.lethal_dose > Fx::ZERO,
            "PoisonId({}) should have positive lethal_dose",
            i
        );
        assert!(def.onset > 0, "PoisonId({}) should have positive onset", i);
    }
}

#[test]
fn poison_lookup_unknown_returns_none() {
    let pid = mc_core::ids::PoisonId::from_raw(999);
    assert!(lookup(pid).is_none(), "unknown PoisonId should return None");
}

// ── battle/mod.rs coverage ───────────────────────────────────────────

fn make_combatant(name: &str, kind: CombatantKind, affiliation: Affiliation, hp: i32) -> Combatant {
    Combatant {
        kind,
        affiliation,
        name: name.into(),
        atb: AtbGauge::new(Fx::from_raw(32768)),
        hp: Fx::from_int(hp),
        max_hp: Fx::from_int(hp),
        attack: Fx::from_int(15),
        defense: Fx::from_int(10),
        speed: Fx::from_int(12),
        level: 1,
        statuses: StatusList::new(),
    }
}

#[test]
fn combatant_display_check() {
    let c = make_combatant(
        "Edmond",
        CombatantKind::PartyMember(CharId::CHR_EDMOND),
        Affiliation::Party,
        100,
    );
    let repr = format!("{:?}", c);
    assert!(
        repr.contains("Edmond"),
        "Combatant debug should contain name"
    );
}

#[test]
fn battle_alive_counts() {
    let party = vec![
        make_combatant(
            "Edmond",
            CombatantKind::PartyMember(CharId::CHR_EDMOND),
            Affiliation::Party,
            100,
        ),
        make_combatant(
            "Abbé",
            CombatantKind::PartyMember(CharId::CHR_ABBE_FARIA),
            Affiliation::Party,
            80,
        ),
    ];
    let enemies = vec![
        make_combatant(
            "Bandit",
            CombatantKind::Enemy(EnemyId::ENM_BANDIT),
            Affiliation::Enemy,
            30,
        ),
        make_combatant(
            "Thug",
            CombatantKind::Enemy(EnemyId::ENM_SOLDIER),
            Affiliation::Enemy,
            40,
        ),
        make_combatant(
            "Boss",
            CombatantKind::Enemy(EnemyId::ENM_ASSASSIN),
            Affiliation::Enemy,
            60,
        ),
    ];

    let mut battle = Battle::new(party, enemies);
    let (party_alive, enemy_alive) = battle.count_alive();
    assert_eq!(party_alive, 2);
    assert_eq!(enemy_alive, 3);

    // Kill one enemy
    battle.combatants[3].hp = Fx::ZERO;
    let (party_alive, enemy_alive) = battle.count_alive();
    assert_eq!(party_alive, 2);
    assert_eq!(enemy_alive, 2);
}

#[test]
fn battle_victory_detected() {
    let party = vec![make_combatant(
        "Edmond",
        CombatantKind::PartyMember(CharId::CHR_EDMOND),
        Affiliation::Party,
        100,
    )];
    let enemies = vec![make_combatant(
        "Bandit",
        CombatantKind::Enemy(EnemyId::ENM_BANDIT),
        Affiliation::Enemy,
        30,
    )];

    let mut battle = Battle::new(party, enemies);
    assert!(matches!(battle.state, BattleState::Active));

    // Kill the enemy
    battle.combatants[1].hp = Fx::ZERO;
    battle.check_end_conditions();
    assert!(matches!(battle.state, BattleState::Victory));
}

#[test]
fn battle_defeat_detected() {
    let party = vec![make_combatant(
        "Edmond",
        CombatantKind::PartyMember(CharId::CHR_EDMOND),
        Affiliation::Party,
        100,
    )];
    let enemies = vec![make_combatant(
        "Bandit",
        CombatantKind::Enemy(EnemyId::ENM_BANDIT),
        Affiliation::Enemy,
        30,
    )];

    let mut battle = Battle::new(party, enemies);
    assert!(matches!(battle.state, BattleState::Active));

    // Kill the party member
    battle.combatants[0].hp = Fx::ZERO;
    battle.check_end_conditions();
    assert!(matches!(battle.state, BattleState::Defeat));
}

#[test]
fn battle_first_enemy_index() {
    let party = vec![make_combatant(
        "Edmond",
        CombatantKind::PartyMember(CharId::CHR_EDMOND),
        Affiliation::Party,
        100,
    )];
    let enemies = vec![
        make_combatant(
            "Bandit",
            CombatantKind::Enemy(EnemyId::ENM_BANDIT),
            Affiliation::Enemy,
            30,
        ),
        make_combatant(
            "Thug",
            CombatantKind::Enemy(EnemyId::ENM_SOLDIER),
            Affiliation::Enemy,
            40,
        ),
    ];
    let battle = Battle::new(party, enemies);
    assert_eq!(battle.first_enemy_index(), Some(1));
    assert_eq!(battle.first_party_index(), Some(0));
}
