//! Battle state machine — ATB-driven combat resolution.
//!
//! SPEC-001 sections 5, 6. INV-01: pure, deterministic, no I/O.
//! Chrono Trigger ATB model with wait mode.

pub mod atb;
pub mod damage;
pub mod status;

use crate::battle::atb::AtbGauge;
use crate::battle::status::StatusList;
use crate::fx::Fx;
use crate::ids::{CharId, EnemyId, ItemId, TechId};
use serde::{Deserialize, Serialize};

/// Which side a combatant belongs to.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Affiliation {
    Party,
    Enemy,
}

/// The type of combatant — either a party member or an enemy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CombatantKind {
    PartyMember(CharId),
    Enemy(EnemyId),
}

/// A single combatant in battle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Combatant {
    pub kind: CombatantKind,
    pub affiliation: Affiliation,
    pub name: String,
    pub atb: AtbGauge,
    pub hp: Fx,
    pub max_hp: Fx,
    pub attack: Fx,
    pub defense: Fx,
    pub speed: Fx,
    pub level: u16,
    pub statuses: StatusList,
}

impl Combatant {
    pub fn is_alive(&self) -> bool {
        self.hp > Fx::ZERO
    }

    pub fn is_atb_full(&self) -> bool {
        self.atb.is_full()
    }
}

/// An action a combatant can take.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BattleAction {
    Attack { target: usize },
    Tech { tech_id: TechId, target: usize },
    Item { item_id: ItemId, target: usize },
    Guard,
    Flee,
}

/// The overall battle state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BattleState {
    /// No battle is active.
    Idle,
    /// Battle is running.
    Active,
    /// Player has won.
    Victory,
    /// Party has been defeated.
    Defeat,
    /// Party fled successfully.
    Fleeing,
}

/// The full battle instance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Battle {
    pub state: BattleState,
    pub combatants: Vec<Combatant>,
    pub wait_mode: bool,
    /// One-hit guard latches indexed like `combatants`.
    ///
    /// The save loader migrates pre-guard saves with an empty latch vector.
    #[serde(default)]
    pub guarding: Vec<bool>,
}

impl Battle {
    /// Create a new battle from party combatants and enemy combatants.
    pub fn new(party: Vec<Combatant>, enemies: Vec<Combatant>) -> Self {
        let mut combatants = party;
        combatants.extend(enemies);
        let guarding = vec![false; combatants.len()];
        Battle {
            state: BattleState::Active,
            combatants,
            wait_mode: false,
            guarding,
        }
    }

    /// Return whether a combatant has an active one-hit guard latch.
    pub fn is_guarding(&self, index: usize) -> bool {
        self.guarding.get(index).copied().unwrap_or(false)
    }

    /// Set a combatant's one-hit guard latch, growing a legacy vector if needed.
    pub fn set_guarding(&mut self, index: usize, guarding: bool) {
        if index >= self.guarding.len() {
            self.guarding.resize(self.combatants.len(), false);
        }
        if let Some(latch) = self.guarding.get_mut(index) {
            *latch = guarding;
        }
    }

    /// Index of first enemy combatant.
    pub fn first_enemy_index(&self) -> Option<usize> {
        self.combatants
            .iter()
            .position(|c| c.affiliation == Affiliation::Enemy && c.is_alive())
    }

    /// Index of first alive party combatant.
    pub fn first_party_index(&self) -> Option<usize> {
        self.combatants
            .iter()
            .position(|c| c.affiliation == Affiliation::Party && c.is_alive())
    }

    /// Number of alive combatants on each side.
    pub fn count_alive(&self) -> (usize, usize) {
        let party = self
            .combatants
            .iter()
            .filter(|c| c.affiliation == Affiliation::Party && c.is_alive())
            .count();
        let enemies = self
            .combatants
            .iter()
            .filter(|c| c.affiliation == Affiliation::Enemy && c.is_alive())
            .count();
        (party, enemies)
    }

    /// Check for battle end conditions.
    pub fn check_end_conditions(&mut self) {
        let (party_alive, enemies_alive) = self.count_alive();
        if party_alive == 0 {
            self.state = BattleState::Defeat;
        } else if enemies_alive == 0 {
            self.state = BattleState::Victory;
        }
    }

    /// Get the index of the combatant whose ATB gauge is full and ready to act,
    /// preferring party members for player control.
    pub fn next_ready_combatant(&self) -> Option<usize> {
        self.combatants
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_alive() && c.is_atb_full())
            .min_by_key(|(_, c)| {
                // Party members first (player chooses action), then enemies (auto-act)
                match c.affiliation {
                    Affiliation::Party => 0,
                    Affiliation::Enemy => 1,
                }
            })
            .map(|(i, _)| i)
    }

    /// Find a living target for an action. Returns the first living enemy for
    /// party actions, or a random party member for enemy actions.
    pub fn find_auto_target(&self, attacker: usize) -> Option<usize> {
        let attacker_affiliation = self.combatants.get(attacker)?.affiliation;
        let target_affiliation = match attacker_affiliation {
            Affiliation::Party => Affiliation::Enemy,
            Affiliation::Enemy => Affiliation::Party,
        };
        self.combatants
            .iter()
            .enumerate()
            .find(|(_, c)| c.affiliation == target_affiliation && c.is_alive())
            .map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::atb::AtbGauge;
    fn make_test_party() -> Vec<Combatant> {
        vec![Combatant {
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
        }]
    }

    fn make_test_enemy() -> Vec<Combatant> {
        vec![Combatant {
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
        }]
    }

    #[test]
    fn battle_creation() {
        let party = make_test_party();
        let enemies = make_test_enemy();
        let battle = Battle::new(party, enemies);
        assert_eq!(battle.state, BattleState::Active);
        assert_eq!(battle.combatants.len(), 2);
    }

    #[test]
    fn battle_end_conditions_victory() {
        let party = make_test_party();
        let mut enemy = make_test_enemy();
        enemy[0].hp = Fx::ZERO;
        let mut battle = Battle::new(party, enemy);
        battle.check_end_conditions();
        assert_eq!(battle.state, BattleState::Victory);
    }

    #[test]
    fn battle_end_conditions_defeat() {
        let mut party = make_test_party();
        party[0].hp = Fx::ZERO;
        let enemies = make_test_enemy();
        let mut battle = Battle::new(party, enemies);
        battle.check_end_conditions();
        assert_eq!(battle.state, BattleState::Defeat);
    }

    #[test]
    fn next_ready_combatant_prefers_party() {
        let mut party = make_test_party();
        let mut enemies = make_test_enemy();
        // Both have full ATB
        party[0].atb.force_full();
        enemies[0].atb.force_full();
        let battle = Battle::new(party, enemies);
        let next = battle.next_ready_combatant();
        assert!(next.is_some());
        assert_eq!(
            battle.combatants[next.unwrap()].affiliation,
            Affiliation::Party
        );
    }

    #[test]
    fn count_alive() {
        let party = make_test_party();
        let enemies = make_test_enemy();
        let battle = Battle::new(party, enemies);
        assert_eq!(battle.count_alive(), (1, 1));
    }

    #[test]
    fn find_auto_target() {
        let party = make_test_party();
        let enemies = make_test_enemy();
        let battle = Battle::new(party, enemies);
        let target = battle.find_auto_target(0).unwrap();
        assert_eq!(battle.combatants[target].affiliation, Affiliation::Enemy);
    }
}
