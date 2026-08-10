//! Command bus — the only channel between shell and core (INV-04).
//!
//! SPEC-003 is authoritative. `Command` variants have explicit discriminants and
//! are append-only. `apply_commands` never panics; invalid commands produce a
//! `CoreEvent::Rejected` instead.

use crate::battle::{self, Affiliation, BattleState};
use crate::ids::{ItemId, RegionId};
use crate::world::{Party, World};
use serde::{Deserialize, Serialize};

// ── Supporting types ─────────────────────────────────────────────────────────

/// Cardinal direction for movement commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dir {
    North,
    South,
    East,
    West,
}

/// Identifier for an actor (combatant index) in battle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorId(pub usize);

/// Target identifier for battle actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetId(pub usize);

/// A dialogue/scene choice index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceIdx(pub u32);

/// A save slot identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSlot(pub u8);

/// The identity persona Edmond adopts (SPEC-010 §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonaId {
    Edmond,
    MonteCristo,
    Sinbad,
    Busoni,
    Wilmore,
}

/// A campaign identifier for Act VI operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignId(pub String);

/// Actions available during a Paris season campaign in Act VI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CampaignAction {
    Advance,
    Investigate,
    Confront,
    Rest,
}

/// A high-level action taken by the player in combat.
/// Maps onto mc_core's internal BattleAction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Attack {
        target: TargetId,
    },
    Tech {
        tech_id: crate::ids::TechId,
        target: TargetId,
    },
    Item {
        item_id: ItemId,
        target: TargetId,
    },
    Guard,
    Flee,
}

// ── Command ──────────────────────────────────────────────────────────────────

/// Every action a player can take.
///
/// Explicit discriminants are append-only: new variants get the next free
/// discriminant and are never removed or reordered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum Command {
    /// Move one tile in a direction.
    Move(Dir) = 0,
    /// Interact with the current tile or target.
    Interact = 1,
    /// Open the main menu.
    OpenMenu = 2,
    /// Close the main menu.
    CloseMenu = 3,
    /// Select a battle action for an actor.
    SelectAction(ActorId, Action) = 4,
    /// Confirm a battle target selection.
    ConfirmTarget(TargetId) = 5,
    /// Cancel the current selection.
    CancelSelection = 6,
    /// Set ATB wait mode on or off.
    SetWaitMode(bool) = 7,
    /// Advance a linear scene node.
    SceneAdvance = 8,
    /// Choose an option at a scene branch.
    SceneChoose(ChoiceIdx) = 9,
    /// Take a calendar action in Act II (Château d'If).
    CalendarAct(crate::calendar::CalendarAction) = 10,
    /// Take a season action in Act VI (Paris).
    SeasonAct(CampaignId, CampaignAction) = 11,
    /// Swap to a different persona (affects mask, available scenes).
    SwapPersona(PersonaId) = 12,
    /// Fast travel to a visited region.
    FastTravel(RegionId) = 13,
    /// Name yourself at the final encounter (gated on Phase2 + flags).
    NameYourself = 14,
    /// Save the game to a slot.
    Save(SaveSlot) = 15,
    /// Load a saved game from a slot.
    Load(SaveSlot) = 16,
}

// ── CoreEvent ────────────────────────────────────────────────────────────────

/// An event produced by the core in response to a command or a tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreEvent {
    /// The command was accepted and produced an effect.
    Applied { command: Command },
    /// The command was rejected with a reason.
    Rejected { command: Command, reason: String },
    /// A tick advanced the world (one frame).
    TickAdvanced { tick: u64 },
}

// ── StateView ────────────────────────────────────────────────────────────────

/// A read-only projection of the world state produced once per frame.
///
/// The shell may read this and may not mutate through it. No callback into
/// the shell and no second channel (INV-04).
#[derive(Debug, Clone, PartialEq)]
pub struct StateView<'a> {
    /// The current tick number.
    pub tick: u64,
    /// The current campaign act.
    pub act: crate::world::Act,
    /// The current region.
    pub region: RegionId,
    /// The player's party.
    pub party: &'a Party,
    /// Curriculum progress used by Act II and authored scene gates.
    pub curriculum: &'a crate::curriculum::Curriculum,
    /// Inventory projection for menu and scene rendering.
    pub inventory: &'a crate::world::Inventory,
    /// Active battle, when the encounter system has opened one.
    pub battle: Option<&'a crate::battle::Battle>,
    /// Current narrative scene position, when a scene is active.
    pub scene: Option<&'a crate::scene::SceneState>,
    /// Act II calendar, when the campaign is in Château d'If.
    pub calendar: Option<&'a crate::calendar::IfCalendar>,
    /// Act VI season clock, when the campaign is in Paris.
    pub season: Option<&'a crate::season::SeasonClock>,
    /// The set of story flags.
    pub flags: &'a crate::flags::FlagSet,
    /// Events produced since the last frame.
    pub events: &'a [CoreEvent],
    /// State hash, only Some at checkpoint ticks (every 1024 ticks).
    pub state_hash: Option<[u8; 32]>,
}

impl<'a> StateView<'a> {
    /// Produce a StateView from the current world.
    pub fn from_world(world: &'a World, events: &'a [CoreEvent]) -> Self {
        let hash = if world.tick % 1024 == 0 {
            let h = world.state_hash();
            Some(*h.as_bytes())
        } else {
            None
        };
        StateView {
            tick: world.tick,
            act: world.act,
            region: world.region,
            party: &world.party,
            curriculum: &world.curriculum,
            inventory: &world.inventory,
            battle: world.battle.as_ref(),
            scene: world.scene.as_ref(),
            calendar: world.calendar.as_ref(),
            season: world.season.as_ref(),
            flags: &world.flags,
            events,
            state_hash: hash,
        }
    }
}

// ── apply_commands ───────────────────────────────────────────────────────────

/// Apply a batch of commands to the world and return the resulting events.
///
/// Each command is validated against the current state. Invalid commands
/// produce `CoreEvent::Rejected` rather than panicking. Valid commands
/// mutate the world and produce `CoreEvent::Applied`.
///
/// NameYourself is rejected unless the final encounter is in Phase2 AND
/// FLG_MORCERF_YANINA_DOSSIER, FLG_MORCERF_ALBERT_WITHDRAWN, and
/// FLG_MERCEDES_RECOGNITION are all set (INV-14).
pub fn apply_commands(world: &mut World, commands: &[Command]) -> Vec<CoreEvent> {
    let mut events = Vec::with_capacity(commands.len());

    for cmd in commands {
        let event = validate_and_apply(world, cmd);
        events.push(event);
    }

    events
}

/// Validate a single command and apply it if valid, returning a CoreEvent.
fn validate_and_apply(world: &mut World, cmd: &Command) -> CoreEvent {
    match cmd {
        // Navigation — always valid
        Command::Move(_) => CoreEvent::Applied {
            command: cmd.clone(),
        },

        // Interaction — always valid
        Command::Interact => CoreEvent::Applied {
            command: cmd.clone(),
        },

        // Menu — always valid
        Command::OpenMenu | Command::CloseMenu => CoreEvent::Applied {
            command: cmd.clone(),
        },

        // Battle commands — valid only during an active battle. The selected
        // attack is resolved immediately against the authoritative battle tree.
        Command::SelectAction(actor, action) => {
            if let Err(reason) = resolve_battle_action(world, *actor, action) {
                return CoreEvent::Rejected {
                    command: cmd.clone(),
                    reason,
                };
            }
            CoreEvent::Applied {
                command: cmd.clone(),
            }
        }
        Command::ConfirmTarget(target) => {
            let Some(battle) = world.battle.as_ref() else {
                return CoreEvent::Rejected {
                    command: cmd.clone(),
                    reason: "No active battle".into(),
                };
            };
            if battle.state != BattleState::Active
                || battle
                    .combatants
                    .get(target.0)
                    .map_or(true, |c| !c.is_alive())
            {
                return CoreEvent::Rejected {
                    command: cmd.clone(),
                    reason: "Target is not available in the active battle".into(),
                };
            }
            CoreEvent::Applied {
                command: cmd.clone(),
            }
        }
        Command::CancelSelection => CoreEvent::Applied {
            command: cmd.clone(),
        },
        Command::SetWaitMode(wait) => {
            if let Some(battle) = world.battle.as_mut() {
                battle.wait_mode = *wait;
            }
            CoreEvent::Applied {
                command: cmd.clone(),
            }
        }

        // Scene commands — always valid
        Command::SceneAdvance | Command::SceneChoose(_) => CoreEvent::Applied {
            command: cmd.clone(),
        },

        // Calendar — valid only during Act II
        Command::CalendarAct(action) => {
            if world.act != crate::world::Act::ActIIIf {
                return CoreEvent::Rejected {
                    command: cmd.clone(),
                    reason: "Calendar actions are only available during Act II (Château d'If)"
                        .into(),
                };
            }
            let calendar = world
                .calendar
                .get_or_insert_with(crate::calendar::IfCalendar::new);
            calendar.advance(*action, &mut world.curriculum);
            CoreEvent::Applied {
                command: cmd.clone(),
            }
        }

        // Season — valid only during Act VI
        Command::SeasonAct(_, _) => {
            if world.act != crate::world::Act::ActVIParis {
                return CoreEvent::Rejected {
                    command: cmd.clone(),
                    reason: "Season actions are only available during Act VI (Paris)".into(),
                };
            }
            world
                .season
                .get_or_insert_with(|| crate::season::SeasonClock::new(Vec::new()))
                .advance();
            CoreEvent::Applied {
                command: cmd.clone(),
            }
        }

        // Persona swap — always valid
        Command::SwapPersona(_) => CoreEvent::Applied {
            command: cmd.clone(),
        },

        // Fast travel — valid only if region exists
        Command::FastTravel(rid) => {
            if rid.raw() >= crate::ids::RegionId::COUNT as u16 {
                return CoreEvent::Rejected {
                    command: cmd.clone(),
                    reason: format!("Unknown region: {:?}", rid),
                };
            }
            world.region = *rid;
            CoreEvent::Applied {
                command: cmd.clone(),
            }
        }

        // NameYourself — gated on Phase2 + three dossier flags
        Command::NameYourself => {
            let phase2 = world.flags.is_set(crate::ids::FlagId::FLG_FINAL_PHASE2);
            let yanina = world
                .flags
                .is_set(crate::ids::FlagId::FLG_MORCERF_YANINA_DOSSIER);
            let albert = world
                .flags
                .is_set(crate::ids::FlagId::FLG_MORCERF_ALBERT_WITHDRAWN);
            let mercedes = world
                .flags
                .is_set(crate::ids::FlagId::FLG_MERCEDES_RECOGNITION);

            if !phase2 || !yanina || !albert || !mercedes {
                let mut missing = Vec::new();
                if !phase2 {
                    missing.push("Phase2 not reached");
                }
                if !yanina {
                    missing.push("FLG_MORCERF_YANINA_DOSSIER not set");
                }
                if !albert {
                    missing.push("FLG_MORCERF_ALBERT_WITHDRAWN not set");
                }
                if !mercedes {
                    missing.push("FLG_MERCEDES_RECOGNITION not set");
                }
                return CoreEvent::Rejected {
                    command: cmd.clone(),
                    reason: format!("NameYourself denied: {}", missing.join(", ")),
                };
            }
            CoreEvent::Applied {
                command: cmd.clone(),
            }
        }

        // Save/Load — always valid (actual I/O happens in the shell)
        Command::Save(_) | Command::Load(_) => CoreEvent::Applied {
            command: cmd.clone(),
        },
    }
}

/// Resolve one player battle action against the live World battle.
fn resolve_battle_action(world: &mut World, actor: ActorId, action: &Action) -> Result<(), String> {
    let mut rng = world.rng;
    let battle = world
        .battle
        .as_mut()
        .ok_or_else(|| "No active battle".to_string())?;
    if battle.state != BattleState::Active {
        return Err("Battle is not active".into());
    }
    let attacker = battle
        .combatants
        .get(actor.0)
        .ok_or_else(|| "Actor is not present in the active battle".to_string())?;
    if attacker.affiliation != Affiliation::Party || !attacker.is_alive() {
        return Err("Actor is not a living party combatant".into());
    }
    if !attacker.is_atb_full() {
        return Err("Actor ATB gauge is not full".into());
    }

    match action {
        Action::Attack { target } => {
            let target_index = target.0;
            let target_ref = battle
                .combatants
                .get(target_index)
                .ok_or_else(|| "Target is not present in the active battle".to_string())?;
            if target_ref.affiliation != Affiliation::Enemy || !target_ref.is_alive() {
                return Err("Attack target must be a living enemy".into());
            }
            let attacker = battle.combatants[actor.0].clone();
            let defender = battle.combatants[target_index].clone();
            let damage =
                battle::damage::compute_damage(attacker.attack, &attacker, &defender, &mut rng)
                    .mitigated;
            battle::damage::apply_damage(&mut battle.combatants[target_index], damage);
        }
        Action::Guard => {}
        Action::Flee => {
            battle.state = BattleState::Fleeing;
        }
        Action::Tech { .. } | Action::Item { .. } => {
            return Err("This battle action requires an authored ability or item resolver".into())
        }
    }

    battle.combatants[actor.0].atb.reset();
    battle.check_end_conditions();
    world.rng = rng;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::FlagId;
    use crate::world::World;

    #[test]
    fn name_yourself_rejected_without_flags() {
        let mut world = World::new(42);
        let events = apply_commands(&mut world, &[Command::NameYourself]);
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::Rejected { command, reason } => {
                assert_eq!(command, &Command::NameYourself);
                assert!(reason.contains("Phase2 not reached"), "reason: {reason}");
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    #[test]
    fn name_yourself_accepted_with_all_flags() {
        let mut world = World::new(42);
        world.flags.set(FlagId::FLG_FINAL_PHASE2);
        world.flags.set(FlagId::FLG_MORCERF_YANINA_DOSSIER);
        world.flags.set(FlagId::FLG_MORCERF_ALBERT_WITHDRAWN);
        world.flags.set(FlagId::FLG_MERCEDES_RECOGNITION);
        let events = apply_commands(&mut world, &[Command::NameYourself]);
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::Applied { command } => {
                assert_eq!(command, &Command::NameYourself);
            }
            other => panic!("expected Applied, got {other:?}"),
        }
    }

    #[test]
    fn calendar_action_rejected_outside_act2() {
        let mut world = World::new(42);
        // Default act is ActIMarseille
        let events = apply_commands(
            &mut world,
            &[Command::CalendarAct(crate::calendar::CalendarAction::Dig)],
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::Rejected { reason, .. } => {
                assert!(reason.contains("Act II"), "reason: {reason}");
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    #[test]
    fn unknown_fast_travel_rejected() {
        let mut world = World::new(42);
        let bad_region = crate::ids::RegionId::from_raw(255);
        let events = apply_commands(&mut world, &[Command::FastTravel(bad_region)]);
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::Rejected { .. } => {} // expected
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    #[test]
    fn move_command_is_applied() {
        let mut world = World::new(42);
        let events = apply_commands(&mut world, &[Command::Move(Dir::North)]);
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::Applied { command } => {
                assert!(matches!(command, Command::Move(Dir::North)));
            }
            other => panic!("expected Applied, got {other:?}"),
        }
    }

    #[test]
    fn multiple_commands_produce_correct_count() {
        let mut world = World::new(42);
        let cmds = vec![
            Command::Interact,
            Command::SceneAdvance,
            Command::OpenMenu,
            Command::CloseMenu,
        ];
        let events = apply_commands(&mut world, &cmds);
        assert_eq!(events.len(), 4);
        for ev in &events {
            assert!(matches!(ev, CoreEvent::Applied { .. }));
        }
    }
}
