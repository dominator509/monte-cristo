//! Poison model with tolerance tracking (SPEC-001 §14, SPEC-009 §7).
//!
//! Five compounds, each with onset (ticks), potency (`Fx` per tick), tolerance
//! step, and lethal dose. Tolerance accrues per character per compound when a
//! sub-lethal dose is survived and decays slowly over time. Valentine's
//! survival in the narrative is exactly this mechanism: Noirtier administers
//! rising brucine doses across authored days until her tolerance exceeds
//! Heloise's lethal dose — it is simulated, not scripted.

use crate::fx::Fx;
use crate::ids::{CharId, PoisonId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ── Q16.16 raw constants ────────────────────────────────────────────────────
// Computed as (value * 65536) rounded to nearest integer.
const RAW_0_25: i32 = 16_384; // 0.25 × 65536
const RAW_0_10: i32 = 6_554; // 0.10 × 65536
const RAW_4_0: i32 = 262_144; // 4.0 × 65536
const RAW_0_60: i32 = 39_322; // 0.60 × 65536
const RAW_0_04: i32 = 2_621; // 0.04 × 65536
const RAW_3_0: i32 = 196_608; // 3.0 × 65536
const RAW_0_30: i32 = 19_661; // 0.30 × 65536
const RAW_0_06: i32 = 3_932; // 0.06 × 65536
const RAW_3_5: i32 = 229_376; // 3.5 × 65536
const RAW_0_15: i32 = 9_830; // 0.15 × 65536
const RAW_0_08: i32 = 5_243; // 0.08 × 65536
const RAW_6_0: i32 = 393_216; // 6.0 × 65536
const RAW_1_50: i32 = 98_304; // 1.50 × 65536

/// A single compound's properties (SPEC-009 §7).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoisonDef {
    /// Ticks until the poison takes effect.
    pub onset: u64,
    /// Damage (`Fx`) per tick once active.
    pub potency: Fx,
    /// Tolerance accrued per sub-lethal dose survived.
    pub tolerance_step: Fx,
    /// Cumulative dose that is lethal (expressed as `Fx`).
    pub lethal_dose: Fx,
}

/// The complete poison compound table, indexed by `PoisonId::raw()`.
///
/// Values from SPEC-009 §7. The offset matches the [`PoisonId`] discriminant:
///
/// | Index | Constant         | Onset | Potency | Step  | Lethal |
/// |-------|------------------|-------|---------|-------|--------|
/// | 0     | PSN_BRUCINE      | 240   | 0.25    | 0.10  | 4.0    |
/// | 1     | PSN_ACONITE      | 120   | 0.60    | 0.04  | 3.0    |
/// | 2     | PSN_BELLADONNA   | 300   | 0.30    | 0.06  | 3.5    |
/// | 3     | PSN_ARSENIC      | 900   | 0.15    | 0.08  | 6.0    |
/// | 4     | PSN_HYDROCYANIC  | 30    | 1.50    | 0.00  | 1.5    |
pub const POISON_TABLE: [PoisonDef; 5] = [
    // PSN_BRUCINE
    PoisonDef {
        onset: 240,
        potency: Fx::from_raw(RAW_0_25),
        tolerance_step: Fx::from_raw(RAW_0_10),
        lethal_dose: Fx::from_raw(RAW_4_0),
    },
    // PSN_ACONITE
    PoisonDef {
        onset: 120,
        potency: Fx::from_raw(RAW_0_60),
        tolerance_step: Fx::from_raw(RAW_0_04),
        lethal_dose: Fx::from_raw(RAW_3_0),
    },
    // PSN_BELLADONNA
    PoisonDef {
        onset: 300,
        potency: Fx::from_raw(RAW_0_30),
        tolerance_step: Fx::from_raw(RAW_0_06),
        lethal_dose: Fx::from_raw(RAW_3_5),
    },
    // PSN_ARSENIC
    PoisonDef {
        onset: 900,
        potency: Fx::from_raw(RAW_0_15),
        tolerance_step: Fx::from_raw(RAW_0_08),
        lethal_dose: Fx::from_raw(RAW_6_0),
    },
    // PSN_HYDROCYANIC
    PoisonDef {
        onset: 30,
        potency: Fx::from_raw(RAW_1_50),
        tolerance_step: Fx::ZERO,
        lethal_dose: Fx::from_raw(RAW_1_50),
    },
];

/// Look up a [`PoisonDef`] by [`PoisonId`].
///
/// Returns `None` if the id's raw value is out of range.
#[inline]
pub fn lookup(id: PoisonId) -> Option<&'static PoisonDef> {
    POISON_TABLE.get(id.raw() as usize)
}

/// Per-character tolerance state for a single compound.
///
/// Tolerance is the cumulative `Fx` value accrued from surviving sub-lethal
/// doses. It decays slowly over time.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToleranceState {
    /// Current accumulated tolerance level.
    pub value: Fx,
    /// Tick when tolerance was last updated (for decay computation).
    pub last_tick: u64,
}

/// Active poison instance on a character.
///
/// Tracks a single administration of a compound that is ticking toward its
/// onset or currently dealing damage.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivePoison {
    /// The compound being administered.
    pub poison_id: PoisonId,
    /// The dose administered.
    pub dose: Fx,
    /// Ticks remaining before onset (0 means active).
    pub ticks_remaining: u64,
    /// Current cumulative damage dealt by this instance.
    pub damage_dealt: Fx,
}

/// The complete poison subsystem state.
///
/// Attached to [`World`](crate::world::World) to track active poison instances
/// and per-character tolerance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoisonState {
    /// Tolerance per character, per compound.
    ///
    /// Keyed as `(CharId::raw(), PoisonId::raw())` for efficient lookup.
    pub tolerance: BTreeMap<(u16, u16), ToleranceState>,
    /// Active poison instances currently affecting characters.
    pub active: Vec<(CharId, ActivePoison)>,
    /// Tolerance decay per tick as a fraction of current tolerance.
    /// Default: 0.001 (1/1000) of current tolerance lost per tick.
    pub decay_rate: Fx,
    /// Ticks between tolerance decay applications.
    pub decay_interval: u64,
}

impl PoisonState {
    /// Create a new empty poison state with default decay parameters.
    ///
    /// Default decay: 0.001 per tick, applied every tick.
    pub fn new() -> Self {
        PoisonState {
            tolerance: BTreeMap::new(),
            active: Vec::new(),
            decay_rate: Fx::from_raw(66),   // ≈ 0.001 in Q16.16
            decay_interval: 1,
        }
    }

    /// Return the current tolerance value for `(char, poison)`.
    pub fn tolerance_for(&self, char_id: CharId, poison_id: PoisonId) -> Fx {
        self.tolerance
            .get(&(char_id.raw(), poison_id.raw()))
            .map_or(Fx::ZERO, |s| s.value)
    }

    /// Administer a dose of a compound to a character.
    ///
    /// If the dose is sub-lethal (i.e. dose < lethal_dose + tolerance), the
    /// character survives and gains tolerance. If the dose meets or exceeds
    /// the lethal threshold, the character dies (returns `true` for lethal).
    ///
    /// Returns `true` if the administered dose was lethal (character dies).
    ///
    /// This is the core simulation for Valentine's arc: Noirtier administers
    /// rising brucine doses; each sub-lethal dose raises tolerance until it
    /// exceeds Heloise's lethal dose.
    pub fn administer(
        &mut self,
        char_id: CharId,
        poison_id: PoisonId,
        dose: Fx,
        current_tick: u64,
    ) -> bool {
        let Some(def) = lookup(poison_id) else {
            return false;
        };

        let current_tolerance = self.tolerance_for(char_id, poison_id);

        // The effective lethal threshold = lethal_dose + current_tolerance
        let effective_lethal = def.lethal_dose.saturating_add(current_tolerance);

        if dose >= effective_lethal {
            return true; // lethal
        }

        // Sub-lethal — gain tolerance, start an active poison instance.
        let new_tolerance = current_tolerance.saturating_add(def.tolerance_step);

        self.tolerance.insert(
            (char_id.raw(), poison_id.raw()),
            ToleranceState {
                value: new_tolerance,
                last_tick: current_tick,
            },
        );

        self.active.push((
            char_id,
            ActivePoison {
                poison_id,
                dose,
                ticks_remaining: def.onset,
                damage_dealt: Fx::ZERO,
            },
        ));

        false
    }

    /// Advance all active poison instances by one tick.
    ///
    /// Returns the total damage (`Fx`) dealt to each character this tick.
    pub fn tick(&mut self, current_tick: u64) -> Vec<(CharId, Fx)> {
        let mut damage_this_tick: BTreeMap<CharId, Fx> = BTreeMap::new();

        self.active.retain(|(char_id, poison)| {
            let Some(def) = lookup(poison.poison_id) else {
                return false;
            };

            if poison.ticks_remaining > 0 {
                return true;
            }

            // Active — apply potency damage.
            let dmg = def.potency;
            let entry = damage_this_tick.entry(*char_id).or_insert(Fx::ZERO);
            *entry = entry.saturating_add(dmg);

            true
        });

        // Decrement ticks_remaining for all active instances.
        for (_char_id, active) in self.active.iter_mut() {
            if active.ticks_remaining > 0 {
                active.ticks_remaining -= 1;
            }
        }

        // Apply tolerance decay at intervals.
        if current_tick % self.decay_interval == 0 {
            self.decay_tolerance(current_tick);
        }

        damage_this_tick.into_iter().collect()
    }

    /// Decay all tolerances by `decay_rate` fraction.
    fn decay_tolerance(&mut self, current_tick: u64) {
        let rate = self.decay_rate;
        for state in self.tolerance.values_mut() {
            let elapsed = current_tick.saturating_sub(state.last_tick);
            if elapsed == 0 {
                continue;
            }
            // Decay: value = value - (value * decay_rate * elapsed)
            let decay = state
                .value
                .saturating_mul(rate)
                .saturating_mul(Fx::from_int(elapsed as i32));
            state.value = state.value.saturating_sub(decay);
            if state.value < Fx::ZERO {
                state.value = Fx::ZERO;
            }
            state.last_tick = current_tick;
        }
    }

    /// Clear all active poison instances for a character (e.g. after
    /// treatment with an antidote).
    pub fn cure(&mut self, char_id: CharId) {
        self.active.retain(|(cid, _)| *cid != char_id);
    }
}

impl Default for PoisonState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poison_table_lookup_all() {
        for raw in 0..5u16 {
            let id = PoisonId::from_raw(raw);
            let def = lookup(id).expect("every valid PoisonId must have a def");
            assert!(def.onset > 0, "onset must be positive");
            assert!(def.potency > Fx::ZERO, "potency must be positive");
            assert!(def.lethal_dose > Fx::ZERO, "lethal dose must be positive");
        }
    }

    #[test]
    fn tolerance_starts_zero() {
        let state = PoisonState::new();
        assert_eq!(
            state.tolerance_for(CharId::CHR_VALENTINE, PoisonId::PSN_BRUCINE),
            Fx::ZERO
        );
    }

    #[test]
    fn sub_lethal_dose_grants_tolerance() {
        let mut state = PoisonState::new();
        let char_id = CharId::CHR_VALENTINE;
        let poison_id = PoisonId::PSN_BRUCINE;

        // Dose is well below lethal (4.0) and tolerance starts at 0.
        let lethal = state.administer(char_id, poison_id, Fx::from_int(1), 0);
        assert!(!lethal, "dose of 1 should not be lethal");

        let tol = state.tolerance_for(char_id, poison_id);
        // tolerance_step for brucine is 0.10
        assert_eq!(tol, Fx::from_raw(RAW_0_10));
    }

    #[test]
    fn lethal_dose_kills() {
        let mut state = PoisonState::new();
        let char_id = CharId::CHR_VALENTINE;
        let poison_id = PoisonId::PSN_BRUCINE;

        // Dose of 5.0 > lethal_dose of 4.0 — should be lethal.
        let lethal = state.administer(char_id, poison_id, Fx::from_int(5), 0);
        assert!(lethal, "dose of 5 should be lethal");
    }

    #[test]
    fn tolerance_allows_surviving_higher_dose() {
        let mut state = PoisonState::new();
        let char_id = CharId::CHR_VALENTINE;
        let poison_id = PoisonId::PSN_BRUCINE;

        // Administer sub-lethal dose to build tolerance.
        let lethal = state.administer(char_id, poison_id, Fx::ONE, 0);
        assert!(!lethal);

        // Now tolerance is 0.10. Lethal threshold = 4.0 + 0.10 = 4.10.
        // Dose of 4.05 should now be sub-lethal.
        let lethal = state.administer(
            char_id,
            poison_id,
            Fx::from_raw(265_421), // 4.05 × 65536
            100,
        );
        assert!(
            !lethal,
            "tolerance should allow surviving slightly above base lethal"
        );

        let tol = state.tolerance_for(char_id, poison_id);
        // tolerance_step was applied twice: 0.10 + 0.10 = 0.20
        assert_eq!(tol, Fx::from_raw(RAW_0_10 * 2));
    }

    #[test]
    fn active_poison_damages_over_time() {
        let mut state = PoisonState::new();
        let char_id = CharId::CHR_VALENTINE;
        let poison_id = PoisonId::PSN_BRUCINE;

        state.administer(char_id, poison_id, Fx::from_int(1), 0);

        // Brucine onset is 240 ticks. During the onset period, no damage.
        for t in 1..=240 {
            let damage = state.tick(t);
            assert!(
                damage.is_empty(),
                "no damage expected during onset at tick {}",
                t
            );
        }

        // At tick 241 (onset has elapsed), potency should apply.
        let damage = state.tick(241);
        assert!(!damage.is_empty(), "damage should apply after onset");
        // potency = 0.25
        assert_eq!(damage[0].1, Fx::from_raw(RAW_0_25));
    }
}
