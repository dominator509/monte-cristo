//! EP-007 M2: Property tests for combat and damage systems.
//!
//! Uses proptest to verify invariants over generated inputs.
//! All Fx values are Q16.16; from_int saturates at i32 limits.
//! Use values <= 1000 for atk/def/hp to avoid fixed-point overflow.

use mc_core::battle::atb::AtbGauge;
use mc_core::battle::damage::{apply_damage, apply_heal, base_damage, compute_damage};
use mc_core::battle::status::terror_applicable;
use mc_core::battle::{Affiliation, Combatant, CombatantKind};
use mc_core::fx::Fx;
use mc_core::ids::{CharId, EnemyId};

proptest::proptest! {
    /// Damage is never negative for valid inputs (atk, def, base <= 1000 to avoid Fx overflow).
    #[test]
    fn damage_never_negative(atk in 0i32..1000i32, def in 0i32..1000i32, base in 0i32..1000i32, atk_level in 1u16..100u16) {
        let mut rng = mc_core::rng::Rng::new(42);
        let attacker = Combatant {
            kind: CombatantKind::PartyMember(CharId::CHR_EDMOND),
            affiliation: Affiliation::Party,
            name: "Attacker".into(),
            atb: AtbGauge::new(Fx::from_int(12)),
            hp: Fx::from_int(100),
            max_hp: Fx::from_int(100),
            attack: Fx::from_int(atk),
            defense: Fx::from_int(def),
            speed: Fx::from_int(12),
            level: atk_level,
            statuses: mc_core::battle::status::StatusList::new(),
        };
        let defender = Combatant {
            kind: CombatantKind::Enemy(EnemyId::ENM_BANDIT),
            affiliation: Affiliation::Enemy,
            name: "Defender".into(),
            atb: AtbGauge::new(Fx::from_int(8)),
            hp: Fx::from_int(50),
            max_hp: Fx::from_int(50),
            attack: Fx::from_int(6),
            defense: Fx::from_int(4),
            speed: Fx::from_int(8),
            level: 1,
            statuses: mc_core::battle::status::StatusList::new(),
        };
        let result = compute_damage(Fx::from_int(base), &attacker, &defender, &mut rng);
        assert!(result.mitigated >= Fx::ZERO, "mitigated damage must be >= 0");
        assert!(result.raw >= Fx::ZERO, "raw damage must be >= 0");
    }

    /// Damage application never brings HP below zero.
    #[test]
    fn apply_damage_clamps_non_negative(current_hp in 0i32..1000i32, damage in 0i32..1000i32) {
        let mut defender = Combatant {
            kind: CombatantKind::Enemy(EnemyId::ENM_BANDIT),
            affiliation: Affiliation::Enemy,
            name: "Defender".into(),
            atb: AtbGauge::new(Fx::from_int(8)),
            hp: Fx::from_int(current_hp),
            max_hp: Fx::from_int(1000),
            attack: Fx::from_int(6),
            defense: Fx::from_int(4),
            speed: Fx::from_int(8),
            level: 1,
            statuses: mc_core::battle::status::StatusList::new(),
        };
        apply_damage(&mut defender, Fx::from_int(damage));
        assert!(defender.hp >= Fx::ZERO, "HP must never go below zero");
    }

    /// base_damage is always non-negative for non-negative inputs.
    #[test]
    fn base_damage_non_negative(base in 0i32..1000i32, atk in 0i32..1000i32, def in 0i32..1000i32) {
        let dmg = base_damage(Fx::from_int(base), Fx::from_int(atk), Fx::from_int(def));
        assert!(dmg >= Fx::ZERO, "base_damage must be >= 0 for non-negative inputs");
    }

    /// Terror never applies to BEAST or VERMIN families.
    #[test]
    fn terror_applies_non_beast_non_vermin(family in 0u8..10u8) {
        use mc_core::bestiary::Family;
        let all = Family::ALL;
        let idx = (family as usize).min(all.len() - 1);
        let fam = all[idx];
        let applicable = terror_applicable(fam);
        assert_eq!(
            applicable,
            fam != Family::Beast && fam != Family::Vermin,
            "Terror applicability mismatch for family {:?}",
            fam
        );
    }

    /// Healing never brings HP above max_hp.
    #[test]
    fn heal_does_not_exceed_max(current_hp in 0i32..1000i32, heal_amt in 0i32..1000i32) {
        let max_hp_val = 1000i32;
        let mut defender = Combatant {
            kind: CombatantKind::Enemy(EnemyId::ENM_BANDIT),
            affiliation: Affiliation::Enemy,
            name: "Defender".into(),
            atb: AtbGauge::new(Fx::from_int(8)),
            hp: Fx::from_int(current_hp),
            max_hp: Fx::from_int(max_hp_val),
            attack: Fx::from_int(6),
            defense: Fx::from_int(4),
            speed: Fx::from_int(8),
            level: 1,
            statuses: mc_core::battle::status::StatusList::new(),
        };
        apply_heal(&mut defender, Fx::from_int(heal_amt));
        assert!(defender.hp <= defender.max_hp, "HP must not exceed max HP after heal");
        assert!(defender.hp >= Fx::ZERO, "HP must never go below zero");
    }
}
