//! EP-004 M1: Command validation tests.
//!
//! Every invalid command must produce a `CoreEvent::Rejected` rather than
//! panicking. The validation table below enumerates every invalid
//! combination the current domain knows about.

use mc_core::battle::atb::AtbGauge;
use mc_core::battle::status::StatusList;
use mc_core::battle::{Affiliation, Battle, Combatant, CombatantKind};
use mc_core::command::{
    apply_commands, Action, CampaignAction, CampaignId, ChoiceIdx, Command, CoreEvent, Dir,
    PersonaId, SaveSlot, TargetId,
};
use mc_core::fx::Fx;
use mc_core::ids::{CharId, EnemyId, FlagId, RegionId};
use mc_core::world::{Act, World};

/// Every Command variant should be accepted in its valid context.
#[test]
fn move_accepts_all_directions() {
    let mut world = World::new(42);
    for dir in [Dir::North, Dir::South, Dir::East, Dir::West] {
        let events = apply_commands(&mut world, &[Command::Move(dir)]);
        assert_valid(&events[0], &Command::Move(dir));
    }
}

#[test]
fn interact_accepted() {
    let mut world = World::new(42);
    let events = apply_commands(&mut world, &[Command::Interact]);
    assert_valid(&events[0], &Command::Interact);
}

#[test]
fn menu_commands_accepted() {
    let mut world = World::new(42);
    for cmd in &[Command::OpenMenu, Command::CloseMenu] {
        let events = apply_commands(&mut world, &[cmd.clone()]);
        assert_valid(&events[0], cmd);
    }
}

#[test]
fn scene_commands_accepted() {
    let mut world = World::new(42);
    for cmd in &[
        Command::SceneAdvance,
        Command::SceneChoose(ChoiceIdx(0)),
        Command::SceneChoose(ChoiceIdx(99)),
    ] {
        let events = apply_commands(&mut world, &[cmd.clone()]);
        assert_valid(&events[0], cmd);
    }
}

#[test]
fn battle_commands_rejected_outside_battle() {
    let mut world = World::new(42);
    let events = apply_commands(
        &mut world,
        &[Command::SelectAction(
            mc_core::command::ActorId(0),
            Action::Attack {
                target: TargetId(0),
            },
        )],
    );
    assert_rejected(&events[0]);
}

#[test]
fn attack_command_resolves_against_the_authoritative_battle() {
    let mut world = World::new(42);
    let mut party = Combatant {
        kind: CombatantKind::PartyMember(CharId::CHR_EDMOND),
        affiliation: Affiliation::Party,
        name: "Edmond".into(),
        atb: AtbGauge::new(Fx::from_int(12)),
        hp: Fx::from_int(100),
        max_hp: Fx::from_int(100),
        attack: Fx::from_int(10),
        defense: Fx::from_int(8),
        speed: Fx::from_int(12),
        level: 1,
        statuses: StatusList::new(),
    };
    let enemy = Combatant {
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
    };
    party.atb.force_full();
    world.battle = Some(Battle::new(vec![party], vec![enemy]));
    let events = apply_commands(
        &mut world,
        &[Command::SelectAction(
            mc_core::command::ActorId(0),
            Action::Attack {
                target: TargetId(1),
            },
        )],
    );
    assert_valid(
        &events[0],
        &Command::SelectAction(
            mc_core::command::ActorId(0),
            Action::Attack {
                target: TargetId(1),
            },
        ),
    );
    let battle = world.battle.as_ref().unwrap();
    assert!(battle.combatants[1].hp < Fx::from_int(30));
    assert!(!battle.combatants[0].atb.is_full());
}

#[test]
fn set_wait_mode_accepted() {
    let mut world = World::new(42);
    let rejected = apply_commands(&mut world, &[Command::SetWaitMode(true)]);
    assert_rejected(&rejected[0]);

    let party = Combatant {
        kind: CombatantKind::PartyMember(CharId::CHR_EDMOND),
        affiliation: Affiliation::Party,
        name: "Edmond".into(),
        atb: AtbGauge::new(Fx::from_int(12)),
        hp: Fx::from_int(100),
        max_hp: Fx::from_int(100),
        attack: Fx::from_int(10),
        defense: Fx::from_int(8),
        speed: Fx::from_int(12),
        level: 1,
        statuses: StatusList::new(),
    };
    let enemy = Combatant {
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
    };
    world.battle = Some(Battle::new(vec![party], vec![enemy]));
    for wait in &[true, false] {
        let command = Command::SetWaitMode(*wait);
        let events = apply_commands(&mut world, &[command.clone()]);
        assert_valid(&events[0], &command);
        assert_eq!(world.battle.as_ref().unwrap().wait_mode, *wait);
    }
}

/// Calendar actions rejected outside Act II.
#[test]
fn calendar_act_rejected_in_act_i() {
    let mut world = World::new(42);
    assert_eq!(world.act, Act::ActIMarseille);
    let events = apply_commands(
        &mut world,
        &[Command::CalendarAct(mc_core::calendar::CalendarAction::Dig)],
    );
    assert_rejected(&events[0]);
}

/// Season actions rejected outside Act VI.
#[test]
fn season_act_rejected_in_act_i() {
    let mut world = World::new(42);
    let events = apply_commands(
        &mut world,
        &[Command::SeasonAct(
            CampaignId("test".into()),
            CampaignAction::Advance,
        )],
    );
    assert_rejected(&events[0]);
}

/// Fast travel to an unknown region rejected.
#[test]
fn fast_travel_unknown_region_rejected() {
    let mut world = World::new(42);
    let events = apply_commands(&mut world, &[Command::FastTravel(RegionId::from_raw(255))]);
    assert_rejected(&events[0]);
}

/// Fast travel to a valid region accepted.
#[test]
fn fast_travel_valid_region_accepted() {
    let mut world = World::new(42);
    let events = apply_commands(
        &mut world,
        &[Command::FastTravel(RegionId::R02_CHATEAU_DIF)],
    );
    assert_valid(&events[0], &Command::FastTravel(RegionId::R02_CHATEAU_DIF));
    assert_eq!(world.region, RegionId::R02_CHATEAU_DIF);
}

#[test]
fn calendar_action_updates_world_curriculum() {
    let mut world = World::new(42);
    world.set_act(Act::ActIIIf);
    let events = apply_commands(
        &mut world,
        &[Command::CalendarAct(
            mc_core::calendar::CalendarAction::Study(mc_core::curriculum::Discipline::Fencing),
        )],
    );
    assert_valid(
        &events[0],
        &Command::CalendarAct(mc_core::calendar::CalendarAction::Study(
            mc_core::curriculum::Discipline::Fencing,
        )),
    );
    assert_eq!(world.calendar.as_ref().unwrap().month, 1);
    assert_eq!(
        world
            .curriculum
            .months_for(mc_core::curriculum::Discipline::Fencing),
        1
    );
}

#[test]
fn calendar_endure_restores_party_wounds() {
    let mut world = World::new(42);
    world.set_act(Act::ActIIIf);
    world.party.active[0].hp = Fx::from_int(23);
    world.party.roster[0].hp = Fx::from_int(23);

    let command = Command::CalendarAct(mc_core::calendar::CalendarAction::Endure);
    let events = apply_commands(&mut world, &[command.clone()]);

    assert_valid(&events[0], &command);
    assert_eq!(world.party.active[0].hp, world.party.active[0].max_hp);
    assert_eq!(world.party.roster[0].hp, world.party.roster[0].max_hp);
    assert_eq!(world.calendar.as_ref().unwrap().month, 1);
}

#[test]
fn season_action_advances_the_act_vi_clock() {
    let mut world = World::new(42);
    world.set_act(Act::ActVIParis);
    let events = apply_commands(
        &mut world,
        &[Command::SeasonAct(
            CampaignId("paris".into()),
            CampaignAction::Advance,
        )],
    );
    assert_valid(
        &events[0],
        &Command::SeasonAct(CampaignId("paris".into()), CampaignAction::Advance),
    );
    assert_eq!(world.season.as_ref().unwrap().fortnight, 1);
}

/// SwapPersona accepted.
#[test]
fn swap_persona_accepted() {
    let mut world = World::new(42);
    for persona in &[PersonaId::Edmond, PersonaId::MonteCristo, PersonaId::Sinbad] {
        let events = apply_commands(&mut world, &[Command::SwapPersona(*persona)]);
        assert_valid(&events[0], &Command::SwapPersona(*persona));
    }
}

/// Save and Load accepted.
#[test]
fn save_load_accepted() {
    let mut world = World::new(42);
    for cmd in &[
        Command::Save(SaveSlot(0)),
        Command::Save(SaveSlot(3)),
        Command::Load(SaveSlot(1)),
    ] {
        let events = apply_commands(&mut world, &[cmd.clone()]);
        assert_valid(&events[0], cmd);
    }
}

/// NameYourself rejected without Phase2 and flags.
#[test]
fn name_yourself_rejected_without_flags() {
    let mut world = World::new(42);
    let events = apply_commands(&mut world, &[Command::NameYourself]);
    assert_rejected(&events[0]);
}

/// NameYourself accepted with Phase2 and all three dossier flags.
#[test]
fn name_yourself_accepted_with_all_flags() {
    let mut world = World::new(42);
    world.flags.set(FlagId::FLG_FINAL_PHASE2);
    world.flags.set(FlagId::FLG_MORCERF_YANINA_DOSSIER);
    world.flags.set(FlagId::FLG_MORCERF_ALBERT_WITHDRAWN);
    world.flags.set(FlagId::FLG_MERCEDES_RECOGNITION);
    let events = apply_commands(&mut world, &[Command::NameYourself]);
    assert_valid(&events[0], &Command::NameYourself);
}

/// NameYourself rejected when only some flags are set.
#[test]
fn name_yourself_rejected_with_partial_flags() {
    let mut world = World::new(42);
    // Set Phase2 but only one flag
    world.flags.set(FlagId::FLG_FINAL_PHASE2);
    world.flags.set(FlagId::FLG_MORCERF_YANINA_DOSSIER);
    let events = apply_commands(&mut world, &[Command::NameYourself]);
    assert_rejected(&events[0]);
}

/// Multiple commands produce the correct number of events.
#[test]
fn multiple_commands_all_applied() {
    let mut world = World::new(42);
    let cmds = vec![
        Command::Interact,
        Command::SceneAdvance,
        Command::OpenMenu,
        Command::CloseMenu,
        Command::Move(Dir::North),
    ];
    let events = apply_commands(&mut world, &cmds);
    assert_eq!(events.len(), cmds.len());
    for ev in &events {
        assert!(matches!(ev, CoreEvent::Applied { .. }));
    }
}

/// StateView from_world works correctly.
#[test]
fn state_view_from_world() {
    let mut world = World::new(42);
    let events = apply_commands(&mut world, &[Command::Interact]);
    let view = mc_core::command::StateView::from_world(&world, &events);
    assert_eq!(view.tick, 0);
    assert_eq!(view.act, Act::ActIMarseille);
    assert_eq!(view.region, RegionId::R01_MARSEILLE);
    assert_eq!(view.events.len(), 1);
}

/// StateView hash is Some at checkpoint ticks.
#[test]
fn state_view_hash_at_checkpoint() {
    let mut world = World::new(42);
    // Advance to tick 1024 (first checkpoint)
    for _ in 0..1024 {
        world.step();
    }
    assert_eq!(world.tick, 1024);
    let events = [];
    let view = mc_core::command::StateView::from_world(&world, &events);
    assert!(
        view.state_hash.is_some(),
        "expected hash at checkpoint tick"
    );
}

/// StateView hash is None at non-checkpoint ticks.
#[test]
fn state_view_no_hash_outside_checkpoint() {
    let mut world = World::new(42);
    world.step(); // tick 1
    let events = [];
    let view = mc_core::command::StateView::from_world(&world, &events);
    assert!(
        view.state_hash.is_none(),
        "expected no hash outside checkpoint tick"
    );
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn assert_valid(event: &CoreEvent, expected: &Command) {
    match event {
        CoreEvent::Applied { command } => {
            assert_eq!(command, expected, "command mismatch");
        }
        other => panic!(
            "expected Applied {{ command: {:?} }}, got {other:?}",
            expected
        ),
    }
}

fn assert_rejected(event: &CoreEvent) {
    match event {
        CoreEvent::Rejected { .. } => {} // expected
        other => panic!("expected Rejected, got {other:?}"),
    }
}
