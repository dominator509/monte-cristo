//! The World — the single owned game state tree.
//!
//! One owned tree. No interior mutability, no reference cycles, no ECS, no event bus.
//! INV-01: pure function of (seed, content pack, input tape).

use crate::flags::FlagSet;
use crate::fx::Fx;
use crate::ids::{CharId, RegionId};
use crate::rng::Rng;
use crate::step;
use serde::{Deserialize, Serialize};

/// The campaign act.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Act {
    ActIMarseille,
    ActIIIf,
    ActIIIMonteCristo,
    ActIVRome,
    ActVParis,
    ActVIParis,
    ActVIIFinal,
}

/// Party member state.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PartyMember {
    pub char_id: CharId,
    pub hp: Fx,
    pub max_hp: Fx,
    pub attack: Fx,
    pub defense: Fx,
    pub speed: Fx,
    pub level: u16,
}

/// Party — up to 3 active, roster of 11.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Party {
    pub active: Vec<PartyMember>,
    pub roster: Vec<PartyMember>,
}

impl Party {
    pub fn new(roster: Vec<PartyMember>) -> Self {
        let active_count = roster.len().min(3);
        let active: Vec<_> = roster.iter().take(active_count).copied().collect();
        Party { active, roster }
    }
}

/// Inventory — a simple bag of items with counts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Inventory {
    items: Vec<(crate::ids::ItemId, u32)>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory { items: Vec::new() }
    }

    pub fn items(&self) -> &[(crate::ids::ItemId, u32)] {
        &self.items
    }
}

/// An encounter budget for a region/chapter pair.
/// Tracks spent encounters for the anti-grind system.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EncounterBudget {
    pub pool: u32,
    pub spent: u32,
}

/// The World — the entire game state.
///
/// Fields in the order specified by SPEC-001 section 3.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct World {
    pub seed: u128,
    pub tick: u64,
    pub act: Act,
    pub region: RegionId,
    pub party: Party,
    pub flags: FlagSet,
    pub trust: std::collections::BTreeMap<CharId, i16>,
    pub mask: i16,
    // curriculum, inventory, budgets, battle, scene, calendar, season
    // declared as Option/placeholder for now — populated in later milestones
    pub inventory: Inventory,
    pub rng: Rng,
}

impl World {
    /// Create a new World from a seed.
    pub fn new(seed: u128) -> Self {
        let rng = Rng::new(seed);
        let edmond = PartyMember {
            char_id: CharId::CHR_EDMOND,
            hp: Fx::from_int(100),
            max_hp: Fx::from_int(100),
            attack: Fx::from_int(10),
            defense: Fx::from_int(8),
            speed: Fx::from_int(12),
            level: 1,
        };
        World {
            seed,
            tick: 0,
            act: Act::ActIMarseille,
            region: RegionId::R01_MARSEILLE,
            party: Party::new(vec![edmond]),
            flags: FlagSet::new(),
            trust: std::collections::BTreeMap::new(),
            mask: 50,
            inventory: Inventory::new(),
            rng,
        }
    }

    /// Advance the world by one tick.
    /// Dispatches over step::ORDER.
    pub fn step(&mut self) {
        for &system in step::ORDER {
            match system {
                "scene_advance" => { /* EP-002 M9 */ }
                "calendar_advance" => { /* EP-002 M7 */ }
                "season_advance" => { /* EP-002 M7 */ }
                "field_movement" => { /* EP-005 */ }
                "spawn_resolution" => { /* EP-002 M5 */ }
                "encounter_contact" => { /* EP-002 M6 */ }
                "battle_atb" => { /* EP-002 M6 */ }
                "battle_action_resolve" => { /* EP-002 M6 */ }
                "status_tick" => { /* EP-002 M6 */ }
                "poison_tick" => { /* EP-002 M8 */ }
                "budget_decay" => { /* EP-002 M5 */ }
                "flag_reactions" => { /* EP-002 M9 */ }
                "event_flush" => { /* EP-002 M4 */ }
                _ => {}
            }
        }
        self.tick += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::FlagId;

    #[test]
    fn world_creation() {
        let world = World::new(42);
        assert_eq!(world.tick, 0);
        assert_eq!(world.act, Act::ActIMarseille);
        assert_eq!(world.region, RegionId::R01_MARSEILLE);
    }

    #[test]
    fn world_step_advances_tick() {
        let mut world = World::new(42);
        assert_eq!(world.tick, 0);
        world.step();
        assert_eq!(world.tick, 1);
        world.step();
        assert_eq!(world.tick, 2);
    }

    #[test]
    fn flags_work_in_world() {
        let mut world = World::new(42);
        world.flags.set(FlagId::FLG_ARRESTED);
        assert!(world.flags.is_set(FlagId::FLG_ARRESTED));
    }

    #[test]
    fn different_seeds_different_rng() {
        let world_a = World::new(42);
        let world_b = World::new(99);
        assert_ne!(world_a.rng, world_b.rng);
    }

    #[test]
    fn same_seed_same_initial_state() {
        let a = World::new(42);
        let b = World::new(42);
        assert_eq!(a.rng, b.rng);
        assert_eq!(a.party.active.len(), b.party.active.len());
    }

    #[test]
    fn party_creation() {
        let world = World::new(42);
        assert_eq!(world.party.active.len(), 1);
        assert_eq!(world.party.active[0].char_id, CharId::CHR_EDMOND);
    }
}
