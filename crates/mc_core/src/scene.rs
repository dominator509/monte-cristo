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
use std::collections::BTreeMap;

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
                *entry = entry.saturating_add(*v).clamp(-50, 50);
            }
            SceneEffect::SubTrust(c, v) => {
                let entry = world.trust.entry(*c).or_insert(0);
                *entry = entry.saturating_sub(*v).clamp(-50, 50);
            }
            SceneEffect::AddMask(v) => {
                world.mask = world.mask.saturating_add(*v).clamp(0, 100);
            }
            SceneEffect::SubMask(v) => {
                world.mask = world.mask.saturating_sub(*v).clamp(0, 100);
            }
            SceneEffect::GrantItem(id, count) => {
                world.inventory.add_item(*id, *count);
            }
            SceneEffect::ConsumeItem(id, count) => {
                world.inventory.remove_item(*id, *count);
            }
            SceneEffect::Goto(scene) => {
                world.scene = Some(SceneState::new(*scene));
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

/// Failure returned when a scene choice cannot be selected safely.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SceneChoiceError {
    /// The requested index does not exist in the authored choice list.
    #[error("scene choice index {0} is out of range")]
    InvalidIndex(usize),
    /// The choice exists but its authored condition is not satisfied.
    #[error("scene choice index {0} is not available")]
    Unavailable(usize),
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

    /// Select an available choice by index and return its destination.
    ///
    /// The index and authored condition are checked before any effect is
    /// applied, so malformed input cannot panic or partially mutate the world.
    pub fn select_choice(
        &self,
        index: usize,
        state: &mut SceneState,
        world: &mut World,
    ) -> Result<SceneId, SceneChoiceError> {
        let choice = self
            .choices
            .get(index)
            .ok_or(SceneChoiceError::InvalidIndex(index))?;
        if !world.flags.satisfies(&choice.condition) {
            return Err(SceneChoiceError::Unavailable(index));
        }
        for effect in &choice.effects {
            effect.apply(world);
        }
        state.current = choice.to;
        Ok(choice.to)
    }
}

// ── Authored scene catalog ──────────────────────────────────────────────────

/// A content-defined scene before its node identifiers are assigned.
///
/// The content crate converts its RON schema into these domain-only types. The
/// catalog then assigns stable node identifiers in input order, keeping the
/// runtime independent from any I/O or content-parser dependency.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredSceneDefinition {
    /// Stable authored scene identifier (for example `SCN_ARREST`).
    pub id: String,
    /// Gate required before the scene can begin.
    pub requires: FlagExpr,
    /// Authored nodes in deterministic source order.
    pub nodes: Vec<AuthoredNodeDefinition>,
    /// Effects applied when the scene reaches its final node and advances.
    pub on_exit: Vec<SceneEffect>,
    /// Whether this scene is the terminal ending.
    pub terminal: bool,
}

/// A content-defined scene node before destination IDs are resolved.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredNodeDefinition {
    /// Stable node identifier local to its scene (for example `n0`).
    pub id: String,
    /// Localisation key used by the shell.
    pub text_key: String,
    /// Choices from this node.
    pub choices: Vec<AuthoredChoiceDefinition>,
}

/// A content-defined choice before its destination ID is resolved.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredChoiceDefinition {
    /// Display/localisation key for the choice.
    pub label: String,
    /// Destination node identifier local to the same scene.
    pub to: String,
    /// Flag gate for this choice.
    pub condition: FlagExpr,
    /// Effects applied when the choice is selected.
    pub effects: Vec<SceneEffect>,
}

/// A deterministic catalog of authored scene nodes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuthoredSceneCatalog {
    scenes: BTreeMap<String, AuthoredScene>,
    nodes: BTreeMap<SceneId, AuthoredSceneNode>,
}

/// A resolved authored scene.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredScene {
    /// Stable authored scene identifier.
    pub id: String,
    /// Gate required before the scene can begin or advance.
    pub requires: FlagExpr,
    /// First node in the scene.
    pub start: SceneId,
    /// Effects applied when the scene exits.
    pub on_exit: Vec<SceneEffect>,
    /// Whether this scene is the terminal ending.
    pub terminal: bool,
}

/// A resolved authored node.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredSceneNode {
    /// Parent scene identifier.
    pub scene_id: String,
    /// Local authored node identifier.
    pub node_id: String,
    /// Localisation key used by the shell.
    pub text_key: String,
    /// Resolved choices.
    pub choices: Vec<SceneChoice>,
}

/// Errors found while resolving an authored scene catalog.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SceneCatalogError {
    /// A scene identifier was repeated.
    #[error("duplicate authored scene {0}")]
    DuplicateScene(String),
    /// A scene did not contain any nodes.
    #[error("authored scene {0} has no nodes")]
    EmptyScene(String),
    /// A node identifier was repeated within one scene.
    #[error("duplicate node {node} in authored scene {scene}")]
    DuplicateNode { scene: String, node: String },
    /// A choice points to a node that does not exist in its scene.
    #[error("authored scene {scene} node {node} points to missing node {destination}")]
    UnknownDestination {
        scene: String,
        node: String,
        destination: String,
    },
    /// The compact runtime identifier space is exhausted.
    #[error("authored scene catalog contains too many nodes")]
    TooManyNodes,
}

/// Failure returned when beginning an authored scene.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SceneStartError {
    /// The requested scene is not in the loaded catalog.
    #[error("unknown authored scene {0}")]
    UnknownScene(String),
    /// The current world flags do not satisfy the scene gate.
    #[error("authored scene {0} is not available")]
    Unavailable(String),
}

impl AuthoredSceneCatalog {
    /// Resolve authored definitions into a deterministic runtime catalog.
    pub fn from_definitions(
        definitions: Vec<AuthoredSceneDefinition>,
    ) -> Result<Self, SceneCatalogError> {
        let mut scenes = BTreeMap::new();
        let mut nodes = BTreeMap::new();
        let mut next_raw = SceneId::COUNT as u16;

        for definition in definitions {
            if scenes.contains_key(&definition.id) {
                return Err(SceneCatalogError::DuplicateScene(definition.id));
            }
            if definition.nodes.is_empty() {
                return Err(SceneCatalogError::EmptyScene(definition.id));
            }

            let scene_id = definition.id.clone();
            let mut local_ids = BTreeMap::new();
            let mut resolved_ids = Vec::with_capacity(definition.nodes.len());
            for node in &definition.nodes {
                if local_ids.contains_key(&node.id) {
                    return Err(SceneCatalogError::DuplicateNode {
                        scene: scene_id.clone(),
                        node: node.id.clone(),
                    });
                }
                let node_id = SceneId::from_raw(next_raw);
                next_raw = next_raw
                    .checked_add(1)
                    .ok_or(SceneCatalogError::TooManyNodes)?;
                local_ids.insert(node.id.clone(), node_id);
                resolved_ids.push(node_id);
            }

            for (node, node_id) in definition.nodes.iter().zip(resolved_ids.iter()) {
                let mut choices = Vec::with_capacity(node.choices.len());
                for choice in &node.choices {
                    let Some(&destination) = local_ids.get(&choice.to) else {
                        return Err(SceneCatalogError::UnknownDestination {
                            scene: scene_id.clone(),
                            node: node.id.clone(),
                            destination: choice.to.clone(),
                        });
                    };
                    choices.push(SceneChoice {
                        label: choice.label.clone(),
                        to: destination,
                        condition: choice.condition.clone(),
                        effects: choice.effects.clone(),
                    });
                }
                nodes.insert(
                    *node_id,
                    AuthoredSceneNode {
                        scene_id: scene_id.clone(),
                        node_id: node.id.clone(),
                        text_key: node.text_key.clone(),
                        choices,
                    },
                );
            }

            scenes.insert(
                scene_id.clone(),
                AuthoredScene {
                    id: scene_id,
                    requires: definition.requires,
                    start: resolved_ids[0],
                    on_exit: definition.on_exit,
                    terminal: definition.terminal,
                },
            );
        }

        Ok(AuthoredSceneCatalog { scenes, nodes })
    }

    /// Return the number of authored scenes.
    pub fn scene_count(&self) -> usize {
        self.scenes.len()
    }

    /// Return the number of authored nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Look up a scene by its authored identifier.
    pub fn scene(&self, id: &str) -> Option<&AuthoredScene> {
        self.scenes.get(id)
    }

    /// Resolve the scene owning a runtime node identifier.
    pub fn scene_for_node(&self, node: SceneId) -> Option<&AuthoredScene> {
        let node = self.nodes.get(&node)?;
        self.scenes.get(&node.scene_id)
    }

    /// Resolve a runtime node identifier.
    pub fn node(&self, node: SceneId) -> Option<&AuthoredSceneNode> {
        self.nodes.get(&node)
    }

    /// Begin an authored scene if its gate is satisfied.
    pub fn begin(&self, world: &mut World, id: &str) -> Result<SceneId, SceneStartError> {
        let scene = self
            .scenes
            .get(id)
            .ok_or_else(|| SceneStartError::UnknownScene(id.to_string()))?;
        if !world.flags.satisfies(&scene.requires) {
            return Err(SceneStartError::Unavailable(id.to_string()));
        }
        world.scene = Some(SceneState::new(scene.start));
        Ok(scene.start)
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
        SceneEffect::AddTrust(mercedes, 100).apply(&mut world);
        assert_eq!(world.trust.get(&mercedes), Some(&50));
        SceneEffect::SubTrust(mercedes, 100).apply(&mut world);
        assert_eq!(world.trust.get(&mercedes), Some(&-50));
    }

    #[test]
    fn scene_effect_mask() {
        let mut world = World::new(0);
        assert_eq!(world.mask, 100);
        SceneEffect::AddMask(10).apply(&mut world);
        assert_eq!(world.mask, 100);
        SceneEffect::SubMask(20).apply(&mut world);
        assert_eq!(world.mask, 80);
        SceneEffect::SubMask(100).apply(&mut world);
        assert_eq!(world.mask, 0);
        SceneEffect::AddMask(100).apply(&mut world);
        assert_eq!(world.mask, 100);
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
    fn scene_effect_goto_updates_authoritative_scene_pointer() {
        let mut world = World::new(0);
        assert!(world.scene.is_none());

        SceneEffect::Goto(SceneId::SCN_FARIA_MEETING).apply(&mut world);

        assert_eq!(
            world.scene.as_ref().map(|state| state.current),
            Some(SceneId::SCN_FARIA_MEETING)
        );
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

        let dest = choose
            .select_choice(0, &mut state, &mut world)
            .expect("available scene choice should select");
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

    #[test]
    fn scene_choose_rejects_invalid_index_without_mutating() {
        let mut state = SceneState::new(SceneId::SCN_SINDBAD);
        let mut world = World::new(0);
        let choose = SceneChoose {
            from: SceneId::SCN_SINDBAD,
            choices: vec![SceneChoice {
                label: "Reveal".into(),
                to: SceneId::SCN_MORCERF_REVEAL,
                condition: FlagExpr::Always,
                effects: vec![SceneEffect::SetFlag(FlagId::FLG_MORCERF_DOSSIER)],
            }],
            entry_effects: vec![],
        };

        let result = choose.select_choice(1, &mut state, &mut world);
        assert_eq!(result, Err(SceneChoiceError::InvalidIndex(1)));
        assert_eq!(state.current, SceneId::SCN_SINDBAD);
        assert!(!world.flags.is_set(FlagId::FLG_MORCERF_DOSSIER));
    }

    #[test]
    fn scene_choose_rejects_locked_choice_without_mutating() {
        let mut state = SceneState::new(SceneId::SCN_SINDBAD);
        let mut world = World::new(0);
        let choose = SceneChoose {
            from: SceneId::SCN_SINDBAD,
            choices: vec![SceneChoice {
                label: "Reveal".into(),
                to: SceneId::SCN_MORCERF_REVEAL,
                condition: FlagExpr::Set(FlagId::FLG_MORCERF_DOSSIER),
                effects: vec![SceneEffect::SetFlag(FlagId::FLG_MORCERF_DOSSIER)],
            }],
            entry_effects: vec![],
        };

        let result = choose.select_choice(0, &mut state, &mut world);
        assert_eq!(result, Err(SceneChoiceError::Unavailable(0)));
        assert_eq!(state.current, SceneId::SCN_SINDBAD);
        assert!(!world.flags.is_set(FlagId::FLG_MORCERF_DOSSIER));
    }
}
