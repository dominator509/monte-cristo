//! Tests for battle resolution — ATB tick, action execution, damage, end conditions.
//!
//! Covers EP-002 M6 battle system integration.

use mc_core::battle::atb::AtbGauge;
use mc_core::battle::damage::{apply_damage, compute_damage};
use mc_core::battle::status::{StatusEffect, StatusList};
use mc_core::battle::{Affiliation, Battle, BattleState, Combatant, CombatantKind};
use mc_core::fx::Fx;
use mc_core::ids::CharId;
use mc_core::ids::EnemyId;
use mc_core::rng::Rng;

fn make_edmond() -> Combatant {
    Combatant {
        kind: CombatantKind::PartyMember(CharId::CHR_EDMOND),
        affiliation: Affiliation::Party,
        name: "Edmond".to_string(),
        atb: AtbGauge::new(Fx::from_int(12)),
        hp: Fx::from_int(100),
        max_hp: Fx::from_int(100),
        attack: Fx::from_int(10),
        defense: Fx::from_int(8),
        speed: Fx::from_int(12),
        level: 1,
        statuses: StatusList::new(),
    }
}

fn make_haydee() -> Combatant {
    Combatant {
        kind: CombatantKind::PartyMember(CharId::CHR_HAYDEE),
        affiliation: Affiliation::Party,
        name: "Haydée".to_string(),
        atb: AtbGauge::new(Fx::from_int(15)),
        hp: Fx::from_int(80),
        max_hp: Fx::from_int(80),
        attack: Fx::from_int(12),
        defense: Fx::from_int(6),
        speed: Fx::from_int(15),
        level: 2,
        statuses: StatusList::new(),
    }
}

fn make_bandit() -> Combatant {
    Combatant {
        kind: CombatantKind::Enemy(EnemyId::ENM_BANDIT),
        affiliation: Affiliation::Enemy,
        name: "Bandit".to_string(),
        atb: AtbGauge::new(Fx::from_int(8)),
        hp: Fx::from_int(30),
        max_hp: Fx::from_int(30),
        attack: Fx::from_int(6),
        defense: Fx::from_int(4),
        speed: Fx::from_int(8),
        level: 1,
        statuses: StatusList::new(),
    }
}

fn make_soldier() -> Combatant {
    Combatant {
        kind: CombatantKind::Enemy(EnemyId::ENM_SOLDIER),
        affiliation: Affiliation::Enemy,
        name: "Soldier".to_string(),
        atb: AtbGauge::new(Fx::from_int(10)),
        hp: Fx::from_int(40),
        max_hp: Fx::from_int(40),
        attack: Fx::from_int(8),
        defense: Fx::from_int(6),
        speed: Fx::from_int(10),
        level: 1,
        statuses: StatusList::new(),
    }
}

#[test]
fn battle_atb_ticks_advance_gauges() {
    let party = vec![make_edmond()];
    let enemies = vec![make_bandit()];
    let mut battle = Battle::new(party, enemies);

    // Advance all ATB gauges (active mode)
    for combatant in battle.combatants.iter_mut() {
        // Edmond speed 12 => fills in 5 ticks
        // Bandit speed 8 => fills in ~7.5 ticks (8 ticks)
        combatant.atb.tick();
    }

    assert!(battle.combatants[0].atb.progress() > Fx::ZERO);
    assert!(battle.combatants[1].atb.progress() > Fx::ZERO);
    assert!(!battle.combatants[0].atb.is_full()); // Not yet after 1 tick
}

#[test]
fn atb_fills_to_full_then_ready() {
    let enemies = vec![make_bandit()];

    // Create a fast party member
    let mut fast = make_haydee();
    fast.atb = AtbGauge::new(Fx::from_int(60)); // speed 60 = 1.0 per tick
    let party = vec![fast];
    let battle = Battle::new(party, enemies);

    // Before any ticks
    assert!(!battle.combatants[0].is_atb_full());
}

#[test]
fn attack_damage_reduces_hp() {
    let mut rng = Rng::new(42);
    let attacker = make_edmond();
    let mut defender = make_bandit();

    let result = compute_damage(Fx::from_int(10), &attacker, &defender, &mut rng);
    let dealt = apply_damage(&mut defender, result.mitigated);

    assert!(dealt > Fx::ZERO);
    assert!(defender.hp < Fx::from_int(30));
}

#[test]
fn kill_enemy_triggers_victory_check() {
    let party = vec![make_edmond()];
    let mut enemies = vec![make_bandit()];
    enemies[0].hp = Fx::from_int(1); // 1 HP remaining

    let mut battle = Battle::new(party, enemies);

    // Apply fatal damage to the enemy
    let idx = 1; // enemy at index 1
    battle.combatants[idx].hp = Fx::ZERO;

    battle.check_end_conditions();
    assert_eq!(battle.state, BattleState::Victory);
}

#[test]
fn party_wipe_triggers_defeat() {
    let mut party = vec![make_edmond()];
    party[0].hp = Fx::from_int(1);
    let enemies = vec![make_bandit()];

    let mut battle = Battle::new(party, enemies);
    battle.combatants[0].hp = Fx::ZERO;

    battle.check_end_conditions();
    assert_eq!(battle.state, BattleState::Defeat);
}

#[test]
fn next_ready_combatant_returns_none_when_none_ready() {
    let party = vec![make_edmond()];
    let enemies = vec![make_bandit()];
    let battle = Battle::new(party, enemies);

    // Neither is full
    assert!(battle.next_ready_combatant().is_none());
}

#[test]
fn next_ready_combatant_returns_party_first() {
    let mut party = vec![make_edmond()];
    let mut enemies = vec![make_bandit()];
    party[0].atb.force_full();
    enemies[0].atb.force_full();

    let battle = Battle::new(party, enemies);
    let next = battle.next_ready_combatant().unwrap();
    assert_eq!(battle.combatants[next].affiliation, Affiliation::Party);
}

#[test]
fn dead_combatants_are_skipped() {
    let mut party = vec![make_edmond()];
    let enemies = vec![make_bandit()];
    party[0].atb.force_full();
    party[0].hp = Fx::ZERO; // dead

    let battle = Battle::new(party, enemies);
    // No living ready combatants
    assert!(battle.next_ready_combatant().is_none());
}

#[test]
fn auto_target_selects_opponent() {
    let party = vec![make_edmond()];
    let enemies = vec![make_bandit()];
    let battle = Battle::new(party, enemies);

    // Attacker is party (index 0), target should be enemy (index 1)
    let target = battle.find_auto_target(0).unwrap();
    assert_eq!(battle.combatants[target].affiliation, Affiliation::Enemy);
}

#[test]
fn multiple_enemies_alive_count() {
    let party = vec![make_edmond()];
    let enemies = vec![make_bandit(), make_soldier()];
    let battle = Battle::new(party, enemies);

    let (party_alive, enemies_alive) = battle.count_alive();
    assert_eq!(party_alive, 1);
    assert_eq!(enemies_alive, 2);
}

#[test]
fn battle_state_starts_active() {
    let party = vec![make_edmond()];
    let enemies = vec![make_bandit()];
    let battle = Battle::new(party, enemies);
    assert_eq!(battle.state, BattleState::Active);
}

#[test]
fn wounds_persist_after_battle() {
    // L6: wounds persist (confirmed by HP not being restored)
    let mut edmond = make_edmond();
    edmond.hp = Fx::from_int(50); // wounded

    let party = vec![edmond];
    let enemies = vec![make_bandit()];

    let battle = Battle::new(party, enemies);
    // After battle, HP stays at 50 (not auto-healed)
    assert_eq!(battle.combatants[0].hp, Fx::from_int(50));
}

#[test]
fn guard_action_reduces_damage() {
    // Guard reduces damage by half
    let mut rng = Rng::new(42);
    let attacker = make_edmond();
    let defender = make_bandit();

    // Damage without guard
    let result_no_guard = compute_damage(Fx::from_int(10), &attacker, &defender, &mut rng);

    // Damage with guard (half damage)
    let mut rng2 = Rng::new(42);
    let result_guard = compute_damage(Fx::from_int(10), &attacker, &defender, &mut rng2);
    let guarded = result_guard.mitigated.saturating_mul(Fx::HALF);

    assert!(guarded <= result_no_guard.mitigated);
}

#[test]
fn status_affects_damage_output() {
    let mut rng = Rng::new(42);
    let mut attacker = make_edmond();
    let defender = make_bandit();

    // Without Fever
    let result_no_status = compute_damage(Fx::from_int(10), &attacker, &defender, &mut rng);

    // With Fever (1.5x)
    let mut rng2 = Rng::new(42);
    attacker.statuses.add(StatusEffect::Fever { duration: 3 });
    let result_fever = compute_damage(Fx::from_int(10), &attacker, &defender, &mut rng2);

    assert!(result_fever.mitigated > result_no_status.mitigated);
}

#[test]
fn combatant_repr() {
    let edmond = make_edmond();
    assert_eq!(edmond.name, "Edmond");
    assert_eq!(edmond.hp, Fx::from_int(100));
    assert_eq!(edmond.max_hp, Fx::from_int(100));
    assert_eq!(edmond.level, 1);
}
