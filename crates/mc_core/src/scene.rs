//! Scene system — a branching tree of narrative nodes.
//!
//! SPEC-001 §12, §13, §15:
//! - SceneState is a position in a branching tree
//! - SceneAdvance is a linear forward transition
//! - SceneChoose is a branching point with multiple choices
//! - Effects limited to: flags, trust, mask, item grant/consume, and next node
//! - NO hit points, NO turn order, NO meters — the types make this impossible

use crate::flags::{FlagExpr, FlagSet};
use crate::ids::{CharId, FlagId, ItemId, SceneId};
use crate::world::World;
use serde::{Deserialize, Serialize};

// ── SceneEffect ──────────────────────────────────────────────────────────────

/// A narrative effect applied when a scene node is entered.
///
/// Every variant touches only: flags, trust, mask, items, or the scene pointer.
/// No HP, no turn order, no meters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneEffect {
    /// Set a story flag.
    SetFlag(FlagId),
    /// Clear a story flag.
    ClearFlag(FlagId),
    /// Add trust points for a character.
    AddTrust(CharId, i16),
    /// Subtract trust points for a character.
    SubTrust(CharId, i16),
    /// Add to the mask meter.
    AddMask(i16),
    /// Subtract from the mask meter.
    SubMask(i16),
    /// Grant an item (by id, count).
    GrantItem(ItemId, u32),
    /// Consume an item (by id, count). No-op if insufficient.
    ConsumeItem(ItemId, u32),
    /// Jump to a different scene node.
    Goto(SceneId),
}

impl SceneEffect {
    /// Apply this effect to the world.
    pub fn apply(&self, world: &mut World) {
        match self {
            SceneEffect::SetFlag(f) => world.flags.set(*f),
            SceneEffect::ClearFlag(f) => world.flags.clear(*f),
            SceneEffect::AddTrust(c, v) => {
                let entry = world.trust.entry(*c).or_insert(0);
                *entry = entry.saturating_add(*v);
            }
            SceneEffect::SubTrust(c, v) => {
                let entry = world.trust.entry(*c).or_insert(0);
                *entry = entry.saturating_sub(*v);
            }
            SceneEffect::AddMask(v) => {
                world.mask = world.mask.saturating_add(*v);
            }
            SceneEffect::SubMask(v) => {
                world.mask = world.mask.saturating_sub(*v);
            }
            SceneEffect::GrantItem(id, count) => {
                world.inventory.add_item(*id, *count);
            }
            SceneEffect::ConsumeItem(id, count) => {
                world.inventory.remove_item(*id, *count);
            }
            SceneEffect::Goto(_scene) => {
                // Scene pointer updates are handled by the caller via
                // set_current; storing this for reactive dispatchers.
            }
        }
    }
}

// ── SceneState ───────────────────────────────────────────────────────────────

/// A position in the scene branching tree.
///
/// The scene tree itself is content-defined (authored in content packs);
/// this type tracks where the player currently is. No HP, no turns, no meters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneState {
    /// The current scene node.
    pub current: SceneId,
}

impl SceneState {
    /// Create a new scene state at the given starting scene.
    pub fn new(start: SceneId) -> Self {
        SceneState { current: start }
    }

    /// Set the current scene position.
    pub fn set_current(&mut self, scene: SceneId) {
        self.current = scene;
    }
}

// ── SceneAdvance ─────────────────────────────────────────────────────────────

/// A linear, forward-only transition between two scene nodes.
///
/// When the condition is satisfied, the player advances along the path and
/// the listed effects are applied. Exactly one `to` — no branches.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAdvance {
    /// The source scene node.
    pub from: SceneId,
    /// The destination scene node.
    pub to: SceneId,
    /// Condition that must be met to take this advance.
    pub condition: FlagExpr,
    /// Effects applied when traversing this advance.
    pub effects: Vec<SceneEffect>,
}

impl SceneAdvance {
    /// Check whether this advance is available given the world's flags.
    pub fn is_available(&self, flags: &FlagSet) -> bool {
        flags.satisfies(&self.condition)
    }

    /// Apply the advance's effects to the world and return the destination.
    /// Caller is responsible for checking `is_available` first.
    pub fn traverse(&self, state: &mut SceneState, world: &mut World) -> SceneId {
        for effect in &self.effects {
            effect.apply(world);
        }
        state.current = self.to;
        self.to
    }
}

// ── SceneChoice ──────────────────────────────────────────────────────────────

/// A single option within a `SceneChoose` branching node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneChoice {
    /// Display label for this choice.
    pub label: String,
    /// Destination scene node.
    pub to: SceneId,
    /// Condition that must be met for this choice to appear.
    pub condition: FlagExpr,
    /// Effects applied when this choice is selected.
    pub effects: Vec<SceneEffect>,
}

/// A branching scene node where the player chooses from multiple paths.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneChoose {
    /// The source scene node.
    pub from: SceneId,
    /// The available choices.
    pub choices: Vec<SceneChoice>,
    /// Effects applied when the node is entered (regardless of choice).
    pub entry_effects: Vec<SceneEffect>,
}

impl SceneChoose {
    /// Return the subset of choices whose conditions are satisfied.
    pub fn available_choices<'a>(&'a self, flags: &FlagSet) -> Vec<&'a SceneChoice> {
        self.choices
            .iter()
            .filter(|c| flags.satisfies(&c.condition))
            .collect()
    }

    /// Apply entry effects (called once when entering this node).
    pub fn apply_entry_effects(&self, world: &mut World) {
        for effect in &self.entry_effects {
            effect.apply(world);
        }
    }

    /// Select a choice by index. Returns the destination scene after applying
    /// the choice's effects. Panics if index is out of range.
    pub fn select_choice(
        &self,
        index: usize,
        state: &mut SceneState,
        world: &mut World,
    ) -> SceneId {
        let choice = &self.choices[index];
        for effect in &choice.effects {
            effect.apply(world);
        }
        state.current = choice.to;
        choice.to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_state_new() {
        let state = SceneState::new(SceneId::SCN_ARREST);
        assert_eq!(state.current, SceneId::SCN_ARREST);
    }

    #[test]
    fn scene_state_set_current() {
        let mut state = SceneState::new(SceneId::SCN_ARREST);
        state.set_current(SceneId::SCN_FARIA_MEETING);
        assert_eq!(state.current, SceneId::SCN_FARIA_MEETING);
    }

    #[test]
    fn scene_effect_set_flag() {
        let mut world = World::new(0);
        assert!(!world.flags.is_set(FlagId::FLG_ARRESTED));
        SceneEffect::SetFlag(FlagId::FLG_ARRESTED).apply(&mut world);
        assert!(world.flags.is_set(FlagId::FLG_ARRESTED));
    }

    #[test]
    fn scene_effect_clear_flag() {
        let mut world = World::new(0);
        world.flags.set(FlagId::FLG_ARRESTED);
        SceneEffect::ClearFlag(FlagId::FLG_ARRESTED).apply(&mut world);
        assert!(!world.flags.is_set(FlagId::FLG_ARRESTED));
    }

    #[test]
    fn scene_effect_trust() {
        let mut world = World::new(0);
        let mercedes = CharId::CHR_MERCEDES;
        SceneEffect::AddTrust(mercedes, 10).apply(&mut world);
        assert_eq!(world.trust.get(&mercedes), Some(&10));
        SceneEffect::SubTrust(mercedes, 3).apply(&mut world);
        assert_eq!(world.trust.get(&mercedes), Some(&7));
    }

    #[test]
    fn scene_effect_mask() {
        let mut world = World::new(0);
        assert_eq!(world.mask, 100);
        SceneEffect::AddMask(10).apply(&mut world);
        assert_eq!(world.mask, 110);
        SceneEffect::SubMask(20).apply(&mut world);
        assert_eq!(world.mask, 90);
    }

    #[test]
    fn scene_effect_item() {
        let mut world = World::new(0);
        let potion = ItemId::ITM_POTION;
        SceneEffect::GrantItem(potion, 3).apply(&mut world);
        assert!(world
            .inventory
            .items()
            .iter()
            .any(|(id, c)| *id == potion && *c == 3));
        SceneEffect::ConsumeItem(potion, 1).apply(&mut world);
        assert!(world
            .inventory
            .items()
            .iter()
            .any(|(id, c)| *id == potion && *c == 2));
    }

    #[test]
    fn scene_advance_traverse() {
        let mut state = SceneState::new(SceneId::SCN_ARREST);
        let mut world = World::new(0);
        let advance = SceneAdvance {
            from: SceneId::SCN_ARREST,
            to: SceneId::SCN_FARIA_MEETING,
            condition: FlagExpr::Always,
            effects: vec![SceneEffect::SetFlag(FlagId::FLG_FARIA_MET)],
        };
        assert!(advance.is_available(&world.flags));
        let dest = advance.traverse(&mut state, &mut world);
        assert_eq!(dest, SceneId::SCN_FARIA_MEETING);
        assert_eq!(state.current, SceneId::SCN_FARIA_MEETING);
        assert!(world.flags.is_set(FlagId::FLG_FARIA_MET));
    }

    #[test]
    fn scene_advance_condition_gate() {
        let world = World::new(0);
        let advance = SceneAdvance {
            from: SceneId::SCN_ARREST,
            to: SceneId::SCN_FARIA_MEETING,
            condition: FlagExpr::Set(FlagId::FLG_ESCAPED),
            effects: vec![],
        };
        assert!(!advance.is_available(&world.flags));
    }

    #[test]
    fn scene_choose_select() {
        let mut state = SceneState::new(SceneId::SCN_SINDBAD);
        let mut world = World::new(0);
        let choose = SceneChoose {
            from: SceneId::SCN_SINDBAD,
            choices: vec![
                SceneChoice {
                    label: "Reveal your identity".into(),
                    to: SceneId::SCN_MORCERF_REVEAL,
                    condition: FlagExpr::Always,
                    effects: vec![SceneEffect::SetFlag(FlagId::FLG_MORCERF_DOSSIER)],
                },
                SceneChoice {
                    label: "Stay hidden".into(),
                    to: SceneId::SCN_ROMAN_CARNIVAL,
                    condition: FlagExpr::Always,
                    effects: vec![],
                },
            ],
            entry_effects: vec![SceneEffect::SetFlag(FlagId::FLG_SINDBAD_VISITED)],
        };

        choose.apply_entry_effects(&mut world);
        assert!(world.flags.is_set(FlagId::FLG_SINDBAD_VISITED));

        let available = choose.available_choices(&world.flags);
        assert_eq!(available.len(), 2);

        let dest = choose.select_choice(0, &mut state, &mut world);
        assert_eq!(dest, SceneId::SCN_MORCERF_REVEAL);
        assert_eq!(state.current, SceneId::SCN_MORCERF_REVEAL);
        assert!(world.flags.is_set(FlagId::FLG_MORCERF_DOSSIER));
    }

    #[test]
    fn scene_choose_condition_filters_choices() {
        let world = World::new(0);
        let choose = SceneChoose {
            from: SceneId::SCN_SINDBAD,
            choices: vec![
                SceneChoice {
                    label: "Reveal".into(),
                    to: SceneId::SCN_MORCERF_REVEAL,
                    condition: FlagExpr::Set(FlagId::FLG_MORCERF_DOSSIER),
                    effects: vec![],
                },
                SceneChoice {
                    label: "Hide".into(),
                    to: SceneId::SCN_ROMAN_CARNIVAL,
                    condition: FlagExpr::Always,
                    effects: vec![],
                },
            ],
            entry_effects: vec![],
        };
        let available = choose.available_choices(&world.flags);
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].label, "Hide");
    }
}
