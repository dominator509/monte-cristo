//! Damage formula and resolution.
//!
//! SPEC-001 section 6: base * atk / (atk + def) in Q16.16, then status
//! multipliers, then crit roll. Wounds persist after battle (L6).

use crate::battle::status::{StatusEffect, StatusList};
use crate::battle::Combatant;
use crate::fx::Fx;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

/// The result of a damage calculation.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DamageResult {
    pub raw: Fx,
    pub mitigated: Fx,
    pub is_critical: bool,
    pub multiplier: Fx,
}

/// Status effect damage multipliers (applied multiplicatively).
///
/// These are applied in order and multiplied together.
pub fn status_damage_multiplier(statuses: &StatusList) -> Fx {
    let mut mult = Fx::ONE;

    for status in statuses.iter() {
        match status {
            StatusEffect::Bleeding { .. } => {
                // Bleeding: no direct damage multiplier, but deals per-tick damage
                // (handled in status module)
            }
            StatusEffect::Fever { .. } => {
                // Fever: 1.5x damage dealt AND received
                mult = mult.saturating_mul(Fx::from_raw(98304)); // 1.5
            }
            StatusEffect::FouledPowder { .. } => {
                // Fouled Powder: 0.75x attack
                mult = mult.saturating_mul(Fx::from_raw(49152)); // 0.75
            }
            StatusEffect::Winded { .. } => {
                // Winded: cannot act for the duration, no damage mult
            }
            StatusEffect::Blinded { .. } => {
                // Blinded: 0.5x accuracy, implemented as 0.8x damage for simplicity
                mult = mult.saturating_mul(Fx::from_raw(52429)); // 0.8
            }
            StatusEffect::Poisoned { .. } => {
                // Poisoned: per-tick damage, no direct damage multiplier
            }
            StatusEffect::BrokenGuard { .. } => {
                // Broken Guard: 1.5x damage received (applied on the defender side)
                mult = mult.saturating_mul(Fx::from_raw(98304)); // 1.5
            }
            StatusEffect::Terror { .. } => {
                // Terror: 0.5x attack
                mult = mult.saturating_mul(Fx::HALF);
            }
        }
    }
    mult
}

/// Compute the base damage before mitigation.
///
/// Formula: `base * atk / (atk + def)`
///
/// All values are Q16.16 Fx. The base damage is a design constant (typically
/// the attacker's attack stat or a tech-specific base).
pub fn base_damage(base: Fx, atk: Fx, def: Fx) -> Fx {
    let denom = atk.saturating_add(def);
    if denom == Fx::ZERO {
        return base;
    }
    let ratio = atk.saturating_div(denom);
    base.saturating_mul(ratio)
}

/// Roll for a critical hit.
///
/// Base crit chance is level-based: `level / 100` capped at 25%.
/// Returns (is_critical, crit_multiplier) where crit_multiplier is 2.0 on crit.
pub fn roll_critical(level: u16, rng: &mut Rng) -> (bool, Fx) {
    let crit_chance = Fx::from_int(level as i32).saturating_div(Fx::from_int(100));
    let crit_chance = if crit_chance > Fx::from_int(25) {
        Fx::from_int(25)
    } else {
        crit_chance
    };
    // Convert to u32 range for RNG roll
    let threshold = (crit_chance * Fx::from_int(100)).to_int_floor().max(0) as u32;
    let roll = rng.next_range(0, 99);
    if roll < threshold {
        (true, Fx::from_int(2))
    } else {
        (false, Fx::ONE)
    }
}

/// Compute final damage dealt from one combatant to another.
///
/// Steps:
/// 1. Compute base mitigated damage: `base * atk / (atk + def)`
/// 2. Apply attacker status multipliers
/// 3. Apply defender status multipliers
/// 4. Roll for critical hit
pub fn compute_damage(
    base: Fx,
    attacker: &Combatant,
    defender: &Combatant,
    rng: &mut Rng,
) -> DamageResult {
    let raw = base_damage(base, attacker.attack, defender.defense);

    // Attacker multipliers
    let atk_mult = status_damage_multiplier(&attacker.statuses);

    // Defender multipliers (BrokenGuard makes defender take more damage)
    let def_mult = status_damage_multiplier(&defender.statuses);

    let combined_mult = atk_mult.saturating_mul(def_mult);
    let mitigated = raw.saturating_mul(combined_mult);

    let (is_critical, crit_mult) = roll_critical(attacker.level, rng);
    let final_damage = mitigated.saturating_mul(crit_mult);

    DamageResult {
        raw,
        mitigated: final_damage,
        is_critical,
        multiplier: combined_mult.saturating_mul(crit_mult),
    }
}

/// Apply damage to a combatant, clamping to 0.
/// Returns the actual damage dealt.
pub fn apply_damage(target: &mut Combatant, damage: Fx) -> Fx {
    if damage >= target.hp {
        let dealt = target.hp;
        target.hp = Fx::ZERO;
        dealt
    } else {
        target.hp = target.hp.saturating_sub(damage);
        damage
    }
}

/// Heal a combatant, clamping to max_hp.
/// Returns the actual HP recovered.
pub fn apply_heal(target: &mut Combatant, amount: Fx) -> Fx {
    let before = target.hp;
    target.hp = target.hp.saturating_add(amount);
    if target.hp > target.max_hp {
        target.hp = target.max_hp;
    }
    target.hp.saturating_sub(before)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::status::StatusEffect;
    use crate::battle::{Affiliation, AtbGauge, CombatantKind};
    use crate::ids::CharId;
    use crate::ids::EnemyId;

    fn make_attacker() -> Combatant {
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

    fn make_defender() -> Combatant {
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

    #[test]
    fn base_damage_formula() {
        // base=10, atk=10, def=4 => 10 * 10 / (10+4) = 100/14 ≈ 7.14
        let dmg = base_damage(Fx::from_int(10), Fx::from_int(10), Fx::from_int(4));
        assert!(dmg > Fx::from_int(7));
        assert!(dmg < Fx::from_int(8));
    }

    #[test]
    fn base_damage_equal_stats() {
        // atk == def => ratio = 0.5, so base * 0.5
        let dmg = base_damage(Fx::from_int(10), Fx::from_int(10), Fx::from_int(10));
        assert_eq!(dmg, Fx::from_int(5));
    }

    #[test]
    fn base_damage_zero_defense() {
        // def=0 => ratio = 1.0, so full base damage
        let dmg = base_damage(Fx::from_int(10), Fx::from_int(10), Fx::ZERO);
        assert_eq!(dmg, Fx::from_int(10));
    }

    #[test]
    fn critical_roll_low_level() {
        let mut rng = Rng::new(42);
        let (is_crit, mult) = roll_critical(1, &mut rng);
        // Level 1 = 1% crit chance, very unlikely but possible
        if is_crit {
            assert_eq!(mult, Fx::from_int(2));
        } else {
            assert_eq!(mult, Fx::ONE);
        }
    }

    #[test]
    fn critical_roll_high_level() {
        let mut rng = Rng::new(12345);
        let (is_crit, mult) = roll_critical(50, &mut rng);
        if is_crit {
            assert_eq!(mult, Fx::from_int(2));
        } else {
            assert_eq!(mult, Fx::ONE);
        }
    }

    #[test]
    fn compute_damage_no_statuses() {
        let attacker = make_attacker();
        let defender = make_defender();
        let mut rng = Rng::new(42);

        // Use a deterministic seed where we won't crit
        let result = compute_damage(Fx::from_int(10), &attacker, &defender, &mut rng);
        // base=10, atk=10, def=4 => ratio = 10/14 ≈ 0.714 => raw ≈ 7.14
        assert!(result.raw > Fx::from_int(7));
        assert!(result.raw < Fx::from_int(8));
    }

    #[test]
    fn fever_multiplier() {
        let mut attacker = make_attacker();
        attacker.statuses.add(StatusEffect::Fever { duration: 3 });
        let defender = make_defender();
        let mut rng = Rng::new(42);
        let result = compute_damage(Fx::from_int(10), &attacker, &defender, &mut rng);
        // Fever = 1.5x multiplier
        assert!(result.multiplier >= Fx::from_raw(98304)); // >= 1.5
    }

    #[test]
    fn terror_multiplier() {
        let mut attacker = make_attacker();
        attacker.statuses.add(StatusEffect::Terror { duration: 3 });
        let defender = make_defender();
        let mut rng = Rng::new(42);
        let result = compute_damage(Fx::from_int(10), &attacker, &defender, &mut rng);
        // Terror = 0.5x multiplier
        assert!(result.multiplier <= Fx::HALF);
    }

    #[test]
    fn apply_damage_clamps_to_zero() {
        let mut target = make_defender();
        target.hp = Fx::from_int(10);
        let dealt = apply_damage(&mut target, Fx::from_int(20));
        assert_eq!(target.hp, Fx::ZERO);
        assert_eq!(dealt, Fx::from_int(10));
    }

    #[test]
    fn apply_damage_partial() {
        let mut target = make_defender();
        target.hp = Fx::from_int(30);
        let dealt = apply_damage(&mut target, Fx::from_int(10));
        assert_eq!(target.hp, Fx::from_int(20));
        assert_eq!(dealt, Fx::from_int(10));
    }

    #[test]
    fn apply_heal_clamps_to_max() {
        let mut target = make_defender();
        target.hp = Fx::from_int(25);
        let recovered = apply_heal(&mut target, Fx::from_int(10));
        assert_eq!(target.hp, Fx::from_int(30)); // max_hp
        assert_eq!(recovered, Fx::from_int(5));
    }

    #[test]
    fn apply_heal_partial() {
        let mut target = make_defender();
        target.hp = Fx::from_int(20);
        let recovered = apply_heal(&mut target, Fx::from_int(5));
        assert_eq!(target.hp, Fx::from_int(25));
        assert_eq!(recovered, Fx::from_int(5));
    }

    #[test]
    fn status_damage_multiplier_combines() {
        let mut list = StatusList::new();
        list.add(StatusEffect::Fever { duration: 3 }); // 1.5x
        list.add(StatusEffect::Terror { duration: 3 }); // 0.5x

        let mult = status_damage_multiplier(&list);
        // 1.5 * 0.5 = 0.75
        assert_eq!(mult, Fx::from_raw(49152)); // 0.75 in Q16.16
    }
}
