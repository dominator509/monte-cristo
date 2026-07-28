//! Tests for status effect rules — stacking, tick effects, Terror immunity,
//! duration management.

use mc_core::battle::status::{
    poison_tick_damage, terror_applicable, StatusEffect, StatusKind, StatusList,
};
use mc_core::bestiary::Family;
use mc_core::fx::Fx;
use mc_core::ids::PoisonId;

#[test]
fn empty_status_list_is_empty() {
    let list = StatusList::new();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
}

#[test]
fn add_single_status() {
    let mut list = StatusList::new();
    assert!(list.add(StatusEffect::Bleeding { duration: 5 }));
    assert_eq!(list.len(), 1);
    assert!(list.has(StatusKind::Bleeding));
}

#[test]
fn stacking_same_kind_shorter_duration_noop() {
    let mut list = StatusList::new();
    list.add(StatusEffect::Bleeding { duration: 5 });
    assert!(!list.add(StatusEffect::Bleeding { duration: 3 }));
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(StatusKind::Bleeding).unwrap().duration(), 5);
}

#[test]
fn stacking_same_kind_longer_duration_refreshes() {
    let mut list = StatusList::new();
    list.add(StatusEffect::Bleeding { duration: 3 });
    assert!(list.add(StatusEffect::Bleeding { duration: 5 }));
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(StatusKind::Bleeding).unwrap().duration(), 5);
}

#[test]
fn multiple_different_statuses_coexist() {
    let mut list = StatusList::new();
    list.add(StatusEffect::Bleeding { duration: 3 });
    list.add(StatusEffect::Fever { duration: 5 });
    list.add(StatusEffect::Winded { duration: 2 });
    assert_eq!(list.len(), 3);
    assert!(list.has(StatusKind::Bleeding));
    assert!(list.has(StatusKind::Fever));
    assert!(list.has(StatusKind::Winded));
}

#[test]
fn all_eight_statuses_can_be_added_independently() {
    let mut list = StatusList::new();
    list.add(StatusEffect::Bleeding { duration: 3 });
    list.add(StatusEffect::Fever { duration: 3 });
    list.add(StatusEffect::FouledPowder { duration: 3 });
    list.add(StatusEffect::Winded { duration: 3 });
    list.add(StatusEffect::Blinded { duration: 3 });
    list.add(StatusEffect::Poisoned {
        poison_id: PoisonId::PSN_BRUCINE,
        duration: 3,
    });
    list.add(StatusEffect::BrokenGuard { duration: 3 });
    list.add(StatusEffect::Terror { duration: 3 });
    assert_eq!(list.len(), 8);
}

#[test]
fn remove_status() {
    let mut list = StatusList::new();
    list.add(StatusEffect::Bleeding { duration: 3 });
    assert!(list.remove(StatusKind::Bleeding));
    assert!(!list.has(StatusKind::Bleeding));
    assert_eq!(list.len(), 0);
}

#[test]
fn remove_nonexistent_status_returns_false() {
    let mut list = StatusList::new();
    assert!(!list.remove(StatusKind::Bleeding));
}

#[test]
fn tick_reduces_duration_by_one() {
    let mut list = StatusList::new();
    list.add(StatusEffect::Bleeding { duration: 3 });
    let expired = list.tick();
    assert!(expired.is_empty());
    assert_eq!(list.get(StatusKind::Bleeding).unwrap().duration(), 2);
}

#[test]
fn tick_expires_status_when_duration_reaches_zero() {
    let mut list = StatusList::new();
    list.add(StatusEffect::Bleeding { duration: 1 });
    let expired = list.tick();
    assert_eq!(expired, vec![StatusKind::Bleeding]);
    assert!(!list.has(StatusKind::Bleeding));
    assert!(list.is_empty());
}

#[test]
fn multiple_ticks_expire_multiple_statuses() {
    let mut list = StatusList::new();
    list.add(StatusEffect::Bleeding { duration: 1 });
    list.add(StatusEffect::Fever { duration: 2 });
    list.add(StatusEffect::Winded { duration: 3 });

    // Tick 1: Bleeding expires
    let expired1 = list.tick();
    assert_eq!(expired1, vec![StatusKind::Bleeding]);
    assert_eq!(list.len(), 2);

    // Tick 2: Fever expires
    let expired2 = list.tick();
    assert_eq!(expired2, vec![StatusKind::Fever]);
    assert_eq!(list.len(), 1);

    // Tick 3: Winded expires
    let expired3 = list.tick();
    assert_eq!(expired3, vec![StatusKind::Winded]);
    assert!(list.is_empty());
}

#[test]
fn bleeding_tick_damage_deals_one_sixteenth_max_hp() {
    let mut list = StatusList::new();
    list.add(StatusEffect::Bleeding { duration: 3 });
    let dmg = list.apply_tick_effects(Fx::from_int(80));
    assert_eq!(dmg, Fx::from_int(5)); // 80 / 16 = 5
}

#[test]
fn poison_tick_damage_varies_by_type() {
    // Brucine: 1/12
    assert_eq!(
        poison_tick_damage(PoisonId::PSN_BRUCINE, Fx::from_int(96)),
        Fx::from_int(8)
    );
    // Aconite: 1/16
    assert_eq!(
        poison_tick_damage(PoisonId::PSN_ACONITE, Fx::from_int(80)),
        Fx::from_int(5)
    );
    // Belladonna: 1/20
    assert_eq!(
        poison_tick_damage(PoisonId::PSN_BELLADONNA, Fx::from_int(100)),
        Fx::from_int(5)
    );
    // Arsenic: 1/32
    assert_eq!(
        poison_tick_damage(PoisonId::PSN_ARSENIC, Fx::from_int(96)),
        Fx::from_int(3)
    );
    // Hydrocyanic: 1/8 (deadly)
    assert_eq!(
        poison_tick_damage(PoisonId::PSN_HYDROCYANIC, Fx::from_int(80)),
        Fx::from_int(10)
    );
}

#[test]
fn fever_tick_damage_deals_one_thirtysecond_max_hp() {
    let mut list = StatusList::new();
    list.add(StatusEffect::Fever { duration: 3 });
    let dmg = list.apply_tick_effects(Fx::from_int(64));
    assert_eq!(dmg, Fx::from_int(2)); // 64 / 32 = 2
}

#[test]
fn multiple_tick_effects_combine() {
    let mut list = StatusList::new();
    list.add(StatusEffect::Bleeding { duration: 3 }); // 80/16 = 5
    list.add(StatusEffect::Fever { duration: 3 }); // 80/32 = 2
    list.add(StatusEffect::Poisoned {
        // 80/12 ≈ 6
        poison_id: PoisonId::PSN_BRUCINE,
        duration: 3,
    });
    let dmg = list.apply_tick_effects(Fx::from_int(80));
    // 80/16 + 80/32 + 80/12 = 5.0 + 2.5 + 6.666... = ~14.1667
    // Compare against a range rather than an exact value
    assert!(dmg > Fx::from_int(14) && dmg < Fx::from_int(15));
}

#[test]
fn non_tick_statuses_deal_no_tick_damage() {
    let mut list = StatusList::new();
    list.add(StatusEffect::Winded { duration: 3 });
    list.add(StatusEffect::Blinded { duration: 3 });
    list.add(StatusEffect::FouledPowder { duration: 3 });
    list.add(StatusEffect::BrokenGuard { duration: 3 });
    list.add(StatusEffect::Terror { duration: 3 });
    let dmg = list.apply_tick_effects(Fx::from_int(100));
    assert_eq!(dmg, Fx::ZERO);
}

#[test]
fn terror_not_applicable_to_beast() {
    assert!(!terror_applicable(Family::Beast));
}

#[test]
fn terror_not_applicable_to_vermin() {
    assert!(!terror_applicable(Family::Vermin));
}

#[test]
fn terror_applicable_to_all_other_families() {
    let applicable: Vec<Family> = Family::ALL
        .iter()
        .copied()
        .filter(|f| terror_applicable(*f))
        .collect();
    // 10 families total, 2 immune (Beast, Vermin) = 8 applicable
    assert_eq!(applicable.len(), 8);
    assert!(!applicable.contains(&Family::Beast));
    assert!(!applicable.contains(&Family::Vermin));
}

#[test]
fn status_kind_all_has_eight_variants() {
    assert_eq!(StatusKind::ALL.len(), 8);
}

#[test]
fn status_name_returns_correct_label() {
    assert_eq!(StatusEffect::Bleeding { duration: 1 }.name(), "Bleeding");
    assert_eq!(StatusEffect::Fever { duration: 1 }.name(), "Fever");
    assert_eq!(
        StatusEffect::FouledPowder { duration: 1 }.name(),
        "Fouled Powder"
    );
    assert_eq!(StatusEffect::Winded { duration: 1 }.name(), "Winded");
    assert_eq!(StatusEffect::Blinded { duration: 1 }.name(), "Blinded");
    assert_eq!(
        StatusEffect::Poisoned {
            poison_id: PoisonId::PSN_BRUCINE,
            duration: 1,
        }
        .name(),
        "Poisoned"
    );
    assert_eq!(
        StatusEffect::BrokenGuard { duration: 1 }.name(),
        "Broken Guard"
    );
    assert_eq!(StatusEffect::Terror { duration: 1 }.name(), "Terror");
}

#[test]
fn status_duration_accessor() {
    assert_eq!(StatusEffect::Bleeding { duration: 7 }.duration(), 7);
    assert_eq!(
        StatusEffect::Poisoned {
            poison_id: PoisonId::PSN_BRUCINE,
            duration: 4,
        }
        .duration(),
        4
    );
}

#[test]
fn has_tick_effect_bleeding() {
    assert!(StatusEffect::Bleeding { duration: 3 }.has_tick_effect());
}

#[test]
fn has_tick_effect_poisoned() {
    assert!(StatusEffect::Poisoned {
        poison_id: PoisonId::PSN_BRUCINE,
        duration: 3,
    }
    .has_tick_effect());
}

#[test]
fn has_tick_effect_fever() {
    assert!(StatusEffect::Fever { duration: 3 }.has_tick_effect());
}

#[test]
fn has_no_tick_effect_winded() {
    assert!(!StatusEffect::Winded { duration: 3 }.has_tick_effect());
}

#[test]
fn status_kind_equality() {
    assert_eq!(StatusKind::Bleeding, StatusKind::Bleeding);
    assert_ne!(StatusKind::Bleeding, StatusKind::Fever);
}

#[test]
fn from_vec_constructor() {
    let list = StatusList::from_vec(vec![
        StatusEffect::Bleeding { duration: 3 },
        StatusEffect::Fever { duration: 5 },
    ]);
    assert_eq!(list.len(), 2);
    assert!(list.has(StatusKind::Bleeding));
    assert!(list.has(StatusKind::Fever));
}

#[test]
fn poison_kind_has_5_variants() {
    assert_eq!(PoisonId::COUNT, 5);
}
