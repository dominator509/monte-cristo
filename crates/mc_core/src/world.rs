//! The World — the single owned game state tree.
//!
//! One owned tree. No interior mutability, no reference cycles, no ECS, no event bus.
//! INV-01: pure function of (seed, content pack, input tape).

use crate::flags::FlagSet;
use crate::fx::Fx;
use crate::ids::{CharId, RegionId};
use crate::rng::Rng;
use crate::step;
use crate::{
    battle::Battle, calendar::IfCalendar, curriculum::Curriculum, scene::SceneState,
    season::SeasonClock,
};
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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Inventory { items: Vec::new() }
    }

    pub fn items(&self) -> &[(crate::ids::ItemId, u32)] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<(crate::ids::ItemId, u32)> {
        &mut self.items
    }

    /// Add `count` copies of an item. Returns the new total count.
    pub fn add_item(&mut self, id: crate::ids::ItemId, count: u32) -> u32 {
        if let Some((_, c)) = self.items.iter_mut().find(|(i, _)| *i == id) {
            *c = c.saturating_add(count);
            *c
        } else {
            self.items.push((id, count));
            count
        }
    }

    /// Remove `count` copies of an item. Returns `false` if the item is not
    /// present or the requested count exceeds the held amount.
    pub fn remove_item(&mut self, id: crate::ids::ItemId, count: u32) -> bool {
        if let Some(idx) = self.items.iter().position(|(i, _)| *i == id) {
            let (_, c) = self.items[idx];
            if c <= count {
                self.items.swap_remove(idx);
            } else {
                self.items[idx].1 = c - count;
            }
            true
        } else {
            false
        }
    }
}

/// The authored encounter-budget implementation is shared with the world map.
pub use crate::budget::EncounterBudget;

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
    pub curriculum: Curriculum,
    pub inventory: Inventory,
    /// Encounter budgets keyed by region and authored chapter stage.
    pub budgets: std::collections::BTreeMap<(RegionId, u32), EncounterBudget>,
    pub battle: Option<Battle>,
    pub scene: Option<SceneState>,
    pub calendar: Option<IfCalendar>,
    pub season: Option<SeasonClock>,
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
            mask: 100,
            curriculum: Curriculum::new(),
            inventory: Inventory::new(),
            budgets: std::collections::BTreeMap::new(),
            battle: None,
            scene: None,
            calendar: None,
            season: None,
            rng,
        }
    }

    /// Move the campaign into an authored act and initialise its act-local
    /// calendar/season state. This is the only supported way for adapters and
    /// tests to change acts without leaving stale clocks behind.
    pub fn set_act(&mut self, act: Act) {
        self.act = act;
        self.calendar = (act == Act::ActIIIf).then(IfCalendar::new);
        self.season = (act == Act::ActVIParis).then(|| SeasonClock::new(Vec::new()));
    }

    /// Persist battle wounds into the authoritative party roster when a battle
    /// reaches a terminal state. Battle combatants are a temporary projection;
    /// the party remains the source of truth between encounters.
    fn sync_party_wounds(&mut self) {
        let Some(battle) = self.battle.as_ref() else {
            return;
        };
        if !matches!(
            battle.state,
            crate::battle::BattleState::Victory
                | crate::battle::BattleState::Defeat
                | crate::battle::BattleState::Fleeing
        ) {
            return;
        }

        for combatant in &battle.combatants {
            let crate::battle::CombatantKind::PartyMember(char_id) = combatant.kind else {
                continue;
            };
            let hp = combatant.hp.min(combatant.max_hp).max(Fx::ZERO);
            if let Some(member) = self
                .party
                .active
                .iter_mut()
                .find(|member| member.char_id == char_id)
            {
                member.hp = hp;
            }
            if let Some(member) = self
                .party
                .roster
                .iter_mut()
                .find(|member| member.char_id == char_id)
            {
                member.hp = hp;
            }
        }
    }

    /// Advance the world by one tick.
    /// Dispatches over step::ORDER.
    pub fn step(&mut self) {
        for &system in step::ORDER {
            match system {
                "scene_advance"
                | "calendar_advance"
                | "season_advance"
                | "field_movement"
                | "spawn_resolution"
                | "encounter_contact"
                | "battle_action_resolve"
                | "poison_tick"
                | "budget_decay"
                | "flag_reactions"
                | "event_flush" => {}
                "battle_atb" => {
                    if let Some(battle) = self.battle.as_mut() {
                        if battle.state == crate::battle::BattleState::Active {
                            let mut party_gauges = Vec::new();
                            let mut enemy_gauges = Vec::new();
                            for combatant in &mut battle.combatants {
                                match combatant.affiliation {
                                    crate::battle::Affiliation::Party => {
                                        party_gauges.push(&mut combatant.atb)
                                    }
                                    crate::battle::Affiliation::Enemy => {
                                        enemy_gauges.push(&mut combatant.atb)
                                    }
                                }
                            }
                            if battle.wait_mode {
                                for gauge in enemy_gauges {
                                    gauge.tick();
                                }
                            } else {
                                for gauge in party_gauges.into_iter().chain(enemy_gauges) {
                                    gauge.tick();
                                }
                            }
                        }
                    }
                }
                "status_tick" => {
                    if let Some(battle) = self.battle.as_mut() {
                        if battle.state == crate::battle::BattleState::Active {
                            for combatant in &mut battle.combatants {
                                combatant.statuses.tick();
                                let damage =
                                    combatant.statuses.apply_tick_effects(combatant.max_hp);
                                combatant.hp = combatant.hp.saturating_sub(damage);
                            }
                            battle.check_end_conditions();
                        }
                    }
                }
                _ => {}
            }
        }
        self.sync_party_wounds();
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
    fn terminal_battle_persists_party_wounds() {
        use crate::battle::atb::AtbGauge;
        use crate::battle::status::StatusList;
        use crate::battle::{Affiliation, Battle, Combatant, CombatantKind};
        use crate::ids::EnemyId;

        let mut world = World::new(42);
        let party = Combatant {
            kind: CombatantKind::PartyMember(CharId::CHR_EDMOND),
            affiliation: Affiliation::Party,
            name: "Edmond".into(),
            atb: AtbGauge::new(Fx::from_int(12)),
            hp: Fx::from_int(37),
            max_hp: Fx::from_int(100),
            attack: Fx::from_int(10),
            defense: Fx::from_int(8),
            speed: Fx::from_int(12),
            level: 1,
            statuses: StatusList::new(),
        };
        let battle = Battle::new(
            vec![party],
            vec![Combatant {
                kind: CombatantKind::Enemy(EnemyId::ENM_BANDIT),
                affiliation: Affiliation::Enemy,
                name: "Bandit".into(),
                atb: AtbGauge::new(Fx::from_int(8)),
                hp: Fx::from_int(30),
                max_hp: Fx::from_int(30),
                attack: Fx::from_int(6),
                defense: Fx::from_int(4),
                speed: Fx::from_int(8),
                level: 1,
                statuses: StatusList::new(),
            }],
        );
        world.battle = Some(battle);

        world.step();
        assert_eq!(world.party.active[0].hp, Fx::from_int(100));

        let battle = world.battle.as_mut().unwrap();
        battle.combatants[1].hp = Fx::ZERO;
        battle.check_end_conditions();
        world.step();

        assert_eq!(world.party.active[0].hp, Fx::from_int(37));
        assert_eq!(world.party.roster[0].hp, Fx::from_int(37));
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
