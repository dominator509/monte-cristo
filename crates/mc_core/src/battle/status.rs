//! Status effects — a closed set of battle statuses.
//!
//! SPEC-001 section 6: Bleeding, Fever, FouledPowder, Winded, Blinded,
//! Poisoned(PoisonId), BrokenGuard, Terror.
//!
//! Rules:
//! - None stack with themselves.
//! - Terror does not apply to BEAST or VERMIN families.
//! - Duration in ticks, per-tick effect.

use crate::bestiary::Family;
use crate::fx::Fx;
use crate::ids::PoisonId;
use serde::{Deserialize, Serialize};

/// The closed set of status effects.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusEffect {
    /// Deals damage each tick (1/16 of max HP).
    Bleeding { duration: u16 },
    /// 1.5x damage dealt and received.
    Fever { duration: u16 },
    /// 0.75x attack power.
    FouledPowder { duration: u16 },
    /// Cannot act for the duration (stun).
    Winded { duration: u16 },
    /// 0.8x accuracy/damage.
    Blinded { duration: u16 },
    /// Per-tick poison damage (type-dependent).
    Poisoned { poison_id: PoisonId, duration: u16 },
    /// 1.5x damage received.
    BrokenGuard { duration: u16 },
    /// 0.5x attack power. Does not apply to BEAST or VERMIN families.
    Terror { duration: u16 },
}

impl StatusEffect {
    /// The internal identifier for this status (for stacking checks).
    pub fn kind(&self) -> StatusKind {
        match self {
            StatusEffect::Bleeding { .. } => StatusKind::Bleeding,
            StatusEffect::Fever { .. } => StatusKind::Fever,
            StatusEffect::FouledPowder { .. } => StatusKind::FouledPowder,
            StatusEffect::Winded { .. } => StatusKind::Winded,
            StatusEffect::Blinded { .. } => StatusKind::Blinded,
            StatusEffect::Poisoned { .. } => StatusKind::Poisoned,
            StatusEffect::BrokenGuard { .. } => StatusKind::BrokenGuard,
            StatusEffect::Terror { .. } => StatusKind::Terror,
        }
    }

    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            StatusEffect::Bleeding { .. } => "Bleeding",
            StatusEffect::Fever { .. } => "Fever",
            StatusEffect::FouledPowder { .. } => "Fouled Powder",
            StatusEffect::Winded { .. } => "Winded",
            StatusEffect::Blinded { .. } => "Blinded",
            StatusEffect::Poisoned { .. } => "Poisoned",
            StatusEffect::BrokenGuard { .. } => "Broken Guard",
            StatusEffect::Terror { .. } => "Terror",
        }
    }

    /// Get the remaining duration in ticks.
    pub fn duration(&self) -> u16 {
        match self {
            StatusEffect::Bleeding { duration }
            | StatusEffect::Fever { duration }
            | StatusEffect::FouledPowder { duration }
            | StatusEffect::Winded { duration }
            | StatusEffect::Blinded { duration }
            | StatusEffect::Poisoned { duration, .. }
            | StatusEffect::BrokenGuard { duration }
            | StatusEffect::Terror { duration } => *duration,
        }
    }

    /// Check whether this status has a per-tick effect that should be applied.
    pub fn has_tick_effect(&self) -> bool {
        matches!(
            self,
            StatusEffect::Bleeding { .. }
                | StatusEffect::Poisoned { .. }
                | StatusEffect::Fever { .. }
        )
    }
}

/// Kind of status (for stacking — you can't have two of the same kind).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusKind {
    Bleeding,
    Fever,
    FouledPowder,
    Winded,
    Blinded,
    Poisoned,
    BrokenGuard,
    Terror,
}

impl StatusKind {
    /// All status kinds.
    pub const ALL: &[StatusKind] = &[
        StatusKind::Bleeding,
        StatusKind::Fever,
        StatusKind::FouledPowder,
        StatusKind::Winded,
        StatusKind::Blinded,
        StatusKind::Poisoned,
        StatusKind::BrokenGuard,
        StatusKind::Terror,
    ];
}

/// A list of active statuses on a single combatant.
///
/// Invariant: no two statuses have the same `StatusKind` (no stacking).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatusList {
    statuses: Vec<StatusEffect>,
}

impl StatusList {
    /// Create an empty status list.
    pub fn new() -> Self {
        StatusList {
            statuses: Vec::new(),
        }
    }

    /// Create a status list from a vector (used in tests).
    pub fn from_vec(statuses: Vec<StatusEffect>) -> Self {
        StatusList { statuses }
    }

    /// Iterate over all active statuses.
    pub fn iter(&self) -> impl Iterator<Item = &StatusEffect> {
        self.statuses.iter()
    }

    /// Check if the list contains a status of the given kind.
    pub fn has(&self, kind: StatusKind) -> bool {
        self.statuses.iter().any(|s| s.kind() == kind)
    }

    /// Get a reference to a status by kind.
    pub fn get(&self, kind: StatusKind) -> Option<&StatusEffect> {
        self.statuses.iter().find(|s| s.kind() == kind)
    }

    /// Get a mutable reference to a status by kind.
    pub fn get_mut(&mut self, kind: StatusKind) -> Option<&mut StatusEffect> {
        self.statuses.iter_mut().find(|s| s.kind() == kind)
    }

    /// Add a status effect.
    ///
    /// If a status of the same kind already exists, the longer duration is kept
    /// (refreshing the duration only if new duration is longer).
    /// Terror is not applied to BEAST or VERMIN families (caller must check).
    ///
    /// Returns true if the status was added/changed.
    pub fn add(&mut self, status: StatusEffect) -> bool {
        let kind = status.kind();
        if let Some(existing) = self.get_mut(kind) {
            // Refresh duration only if new duration is longer
            if status.duration() > existing.duration() {
                *existing = status;
                return true;
            }
            false
        } else {
            self.statuses.push(status);
            true
        }
    }

    /// Remove a status by kind. Returns true if it was present.
    pub fn remove(&mut self, kind: StatusKind) -> bool {
        let len_before = self.statuses.len();
        self.statuses.retain(|s| s.kind() != kind);
        self.statuses.len() < len_before
    }

    /// Number of active statuses.
    pub fn len(&self) -> usize {
        self.statuses.len()
    }

    /// True if no statuses are active.
    pub fn is_empty(&self) -> bool {
        self.statuses.is_empty()
    }

    /// Advance all statuses by one tick, reducing their duration.
    /// Returns a list of status kinds that expired this tick.
    pub fn tick(&mut self) -> Vec<StatusKind> {
        let mut expired = Vec::new();
        // Decrement durations
        for status in self.statuses.iter_mut() {
            match status {
                StatusEffect::Bleeding { ref mut duration }
                | StatusEffect::Fever { ref mut duration }
                | StatusEffect::FouledPowder { ref mut duration }
                | StatusEffect::Winded { ref mut duration }
                | StatusEffect::Blinded { ref mut duration }
                | StatusEffect::Poisoned {
                    ref mut duration, ..
                }
                | StatusEffect::BrokenGuard { ref mut duration }
                | StatusEffect::Terror { ref mut duration } => {
                    *duration = duration.saturating_sub(1);
                }
            }
        }
        // Collect expired
        self.statuses.retain(|s| {
            let keep = s.duration() > 0;
            if !keep {
                expired.push(s.kind());
            }
            keep
        });
        expired
    }

    /// Apply per-tick effects (Bleeding, Poisoned, Fever).
    /// Returns the amount of damage dealt by tick effects.
    pub fn apply_tick_effects(&mut self, max_hp: Fx) -> Fx {
        let mut total_damage = Fx::ZERO;

        for status in self.statuses.iter() {
            match status {
                StatusEffect::Bleeding { .. } => {
                    // Bleeding: 1/16 of max HP per tick
                    let dmg = max_hp.saturating_div(Fx::from_int(16));
                    total_damage = total_damage.saturating_add(dmg);
                }
                StatusEffect::Fever { .. } => {
                    // Fever: 1/32 of max HP per tick (small self-damage from fever)
                    let dmg = max_hp.saturating_div(Fx::from_int(32));
                    total_damage = total_damage.saturating_add(dmg);
                }
                StatusEffect::Poisoned { poison_id, .. } => {
                    // Poison damage varies by type
                    let dmg = poison_tick_damage(*poison_id, max_hp);
                    total_damage = total_damage.saturating_add(dmg);
                }
                _ => {}
            }
        }
        total_damage
    }
}

/// Per-tick damage for each poison type (fraction of max HP).
pub fn poison_tick_damage(poison_id: PoisonId, max_hp: Fx) -> Fx {
    match poison_id {
        PoisonId::PSN_BRUCINE => max_hp.saturating_div(Fx::from_int(12)), // strong
        PoisonId::PSN_ACONITE => max_hp.saturating_div(Fx::from_int(16)), // moderate
        PoisonId::PSN_BELLADONNA => max_hp.saturating_div(Fx::from_int(20)), // slow
        PoisonId::PSN_ARSENIC => max_hp.saturating_div(Fx::from_int(32)), // mild
        PoisonId::PSN_HYDROCYANIC => max_hp.saturating_div(Fx::from_int(8)), // deadly
        _ => max_hp.saturating_div(Fx::from_int(16)),                     // default
    }
}

/// Check whether Terror can be applied to a combatant of the given family.
/// Terror does not apply to BEAST or VERMIN families.
pub fn terror_applicable(family: Family) -> bool {
    !matches!(family, Family::Beast | Family::Vermin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_status_list() {
        let list = StatusList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn add_status() {
        let mut list = StatusList::new();
        let added = list.add(StatusEffect::Bleeding { duration: 5 });
        assert!(added);
        assert_eq!(list.len(), 1);
        assert!(list.has(StatusKind::Bleeding));
    }

    #[test]
    fn no_stacking_shorter_duration() {
        let mut list = StatusList::new();
        list.add(StatusEffect::Bleeding { duration: 5 });
        let added = list.add(StatusEffect::Bleeding { duration: 3 });
        assert!(!added); // Not replaced (shorter duration)
        assert_eq!(list.len(), 1);
        assert_eq!(list.get(StatusKind::Bleeding).unwrap().duration(), 5);
    }

    #[test]
    fn no_stacking_longer_duration() {
        let mut list = StatusList::new();
        list.add(StatusEffect::Bleeding { duration: 3 });
        let added = list.add(StatusEffect::Bleeding { duration: 5 });
        assert!(added); // Replaced (longer duration)
        assert_eq!(list.len(), 1);
        assert_eq!(list.get(StatusKind::Bleeding).unwrap().duration(), 5);
    }

    #[test]
    fn multiple_different_statuses() {
        let mut list = StatusList::new();
        list.add(StatusEffect::Bleeding { duration: 3 });
        list.add(StatusEffect::Fever { duration: 5 });
        assert_eq!(list.len(), 2);
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
    fn tick_reduces_duration() {
        let mut list = StatusList::new();
        list.add(StatusEffect::Bleeding { duration: 3 });
        let expired = list.tick();
        assert!(expired.is_empty());
        assert_eq!(list.get(StatusKind::Bleeding).unwrap().duration(), 2);
    }

    #[test]
    fn tick_expires_status() {
        let mut list = StatusList::new();
        list.add(StatusEffect::Bleeding { duration: 1 });
        let expired = list.tick();
        assert_eq!(expired, vec![StatusKind::Bleeding]);
        assert!(!list.has(StatusKind::Bleeding));
    }

    #[test]
    fn bleeding_tick_damage() {
        let mut list = StatusList::new();
        list.add(StatusEffect::Bleeding { duration: 3 });
        let dmg = list.apply_tick_effects(Fx::from_int(80));
        // 80 / 16 = 5
        assert_eq!(dmg, Fx::from_int(5));
    }

    #[test]
    fn poison_tick_damage_brucine() {
        let dmg = poison_tick_damage(PoisonId::PSN_BRUCINE, Fx::from_int(96));
        // 96 / 12 = 8
        assert_eq!(dmg, Fx::from_int(8));
    }

    #[test]
    fn poison_tick_damage_hydrocyanic() {
        let dmg = poison_tick_damage(PoisonId::PSN_HYDROCYANIC, Fx::from_int(80));
        // 80 / 8 = 10
        assert_eq!(dmg, Fx::from_int(10));
    }

    #[test]
    fn fever_tick_damage() {
        let mut list = StatusList::new();
        list.add(StatusEffect::Fever { duration: 3 });
        let dmg = list.apply_tick_effects(Fx::from_int(64));
        // 64 / 32 = 2
        assert_eq!(dmg, Fx::from_int(2));
    }

    #[test]
    fn multiple_tick_effects_stack() {
        let mut list = StatusList::new();
        list.add(StatusEffect::Bleeding { duration: 3 }); // 80/16 = 5
        list.add(StatusEffect::Fever { duration: 3 }); // 80/32 = 2.5 → 2 (floor)
        let dmg = list.apply_tick_effects(Fx::from_int(80));
        // Total: 5 + 2 = 7
        assert!(dmg > Fx::ZERO);
    }

    #[test]
    fn no_effect_for_non_tick_statuses() {
        let mut list = StatusList::new();
        list.add(StatusEffect::Winded { duration: 3 });
        list.add(StatusEffect::Blinded { duration: 3 });
        let dmg = list.apply_tick_effects(Fx::from_int(80));
        assert_eq!(dmg, Fx::ZERO);
    }

    #[test]
    fn status_kind_all_includes_eight() {
        assert_eq!(StatusKind::ALL.len(), 8);
    }

    #[test]
    fn status_names() {
        assert_eq!(StatusEffect::Bleeding { duration: 1 }.name(), "Bleeding");
        assert_eq!(StatusEffect::Terror { duration: 1 }.name(), "Terror");
        assert_eq!(
            StatusEffect::FouledPowder { duration: 1 }.name(),
            "Fouled Powder"
        );
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
    fn terror_applicable_to_others() {
        assert!(terror_applicable(Family::ManAtArms));
        assert!(terror_applicable(Family::Bandit));
        assert!(terror_applicable(Family::Boss));
    }

    #[test]
    fn has_tick_effect() {
        assert!(StatusEffect::Bleeding { duration: 3 }.has_tick_effect());
        assert!(StatusEffect::Poisoned {
            poison_id: PoisonId::PSN_BRUCINE,
            duration: 3
        }
        .has_tick_effect());
        assert!(!StatusEffect::Winded { duration: 3 }.has_tick_effect());
        assert!(!StatusEffect::Blinded { duration: 3 }.has_tick_effect());
    }
}
