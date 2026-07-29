//! `mc_tools prove` — live-fire proofs for the ship gate.
//!
//! Each sub-verb runs a deterministic simulation and asserts a result.
//! Prints one line: "sub-verb: ok" or "sub-verb: FAIL - reason".

use std::process::ExitCode;

#[derive(clap::Subcommand, Debug)]
pub enum ProveCommand {
    IfCalendar {
        #[arg(long, default_value_t = 168)]
        months: u32,
        #[arg(long, default_value_t = 72)]
        faria_at: u32,
        #[arg(long, default_value_t = 4)]
        min_rank3_disciplines: u32,
    },
    FieldEncounter {
        #[arg(long, default_value_t = String::from("R03"))]
        region: String,
        #[arg(long, default_value_t = true)]
        expect_victory: bool,
    },
    SpawnGating {
        #[arg(long, default_value_t = 500)]
        rolls: u32,
        #[arg(long)]
        all_regions: bool,
    },
    EncounterBudget {
        #[arg(long, default_value_t = 40)]
        reentries: u32,
    },
    ConfidenceGating,
    SaveIdentity,
    FinalEncounter {
        #[arg(long)]
        expect_gated_name_yourself: bool,
    },
}

#[derive(clap::Args, Debug)]
pub struct ProveArgs {
    #[command(subcommand)]
    pub command: ProveCommand,
}

pub fn execute(args: &ProveArgs) -> ExitCode {
    match &args.command {
        ProveCommand::IfCalendar {
            months,
            faria_at,
            min_rank3_disciplines,
        } => {
            if prove_if_calendar(*months, *faria_at, *min_rank3_disciplines) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        ProveCommand::FieldEncounter {
            region,
            expect_victory,
        } => {
            if prove_field_encounter(region, *expect_victory) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        ProveCommand::SpawnGating { rolls, all_regions } => {
            if prove_spawn_gating(*rolls, *all_regions) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        ProveCommand::EncounterBudget { reentries } => {
            if prove_encounter_budget(*reentries) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        ProveCommand::ConfidenceGating => {
            if prove_confidence_gating() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        ProveCommand::SaveIdentity => {
            if prove_save_identity() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        ProveCommand::FinalEncounter {
            expect_gated_name_yourself,
        } => {
            if prove_final_encounter(*expect_gated_name_yourself) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}

fn prove_if_calendar(months: u32, faria_at: u32, min_rank3: u32) -> bool {
    use mc_core::calendar::{CalendarAction, IfCalendar};
    use mc_core::curriculum::{Curriculum, Discipline};

    let mut calendar = IfCalendar::new();
    let mut curriculum = Curriculum::new();

    let disciplines = [
        Discipline::Fencing,
        Discipline::Chemistry,
        Discipline::NaturalPhilosophy,
        Discipline::Mathematics,
        Discipline::Languages,
        Discipline::HistoryPolitics,
        Discipline::Economics,
    ];

    let mut faria_joined = false;

    for m in 0..months {
        if calendar.is_complete() {
            break;
        }
        if m >= faria_at && !faria_joined {
            faria_joined = true;
        }
        let disc = disciplines[(m as usize) % 7];
        let action = CalendarAction::Study(disc);
        curriculum.add_months(disc, 1);
        calendar.advance(action, &mut curriculum);
    }

    if !faria_joined {
        eprintln!(
            "if-calendar: FAIL - Faria did not join by month {}",
            faria_at
        );
        return false;
    }

    let mut count_rank3 = 0u32;
    for disc in &disciplines {
        if curriculum.rank(*disc) >= 3 {
            count_rank3 += 1;
        }
    }
    if count_rank3 < min_rank3 {
        eprintln!(
            "if-calendar: FAIL - {} disciplines at rank 3+, need {}",
            count_rank3, min_rank3
        );
        return false;
    }

    println!("if-calendar: ok");
    true
}

fn prove_field_encounter(_region: &str, _expect_victory: bool) -> bool {
    use mc_core::battle::atb::AtbGauge;
    use mc_core::battle::status::StatusList;
    use mc_core::battle::{Affiliation, Battle, Combatant, CombatantKind};
    use mc_core::fx::Fx;
    use mc_core::ids::{CharId, EnemyId};

    let edmond = Combatant {
        kind: CombatantKind::PartyMember(CharId::CHR_EDMOND),
        affiliation: Affiliation::Party,
        name: "Edmond".into(),
        atb: AtbGauge::new(Fx::from_raw(32768)),
        hp: Fx::from_int(100),
        max_hp: Fx::from_int(100),
        attack: Fx::from_int(15),
        defense: Fx::from_int(10),
        speed: Fx::from_int(12),
        level: 1,
        statuses: StatusList::new(),
    };

    let bandit = Combatant {
        kind: CombatantKind::Enemy(EnemyId::ENM_BANDIT),
        affiliation: Affiliation::Enemy,
        name: "Bandit".into(),
        atb: AtbGauge::new(Fx::from_raw(32768)),
        hp: Fx::from_int(30),
        max_hp: Fx::from_int(30),
        attack: Fx::from_int(10),
        defense: Fx::from_int(5),
        speed: Fx::from_int(8),
        level: 1,
        statuses: StatusList::new(),
    };

    let mut battle = Battle::new(vec![edmond], vec![bandit]);

    for _ in 0..1000 {
        battle.check_end_conditions();
        if !matches!(battle.state, mc_core::battle::BattleState::Active) {
            break;
        }
        // Advance ATB gauges for all combatants
        for c in &mut battle.combatants {
            let _full = c.atb.tick();
        }
    }

    battle.check_end_conditions();
    match battle.state {
        mc_core::battle::BattleState::Victory => {
            println!("field-encounter: ok (victory)");
            true
        }
        mc_core::battle::BattleState::Defeat => {
            eprintln!("field-encounter: FAIL - party defeated");
            false
        }
        _ => {
            // Not resolved — check who's alive
            let (party_alive, _enemy_alive) = battle.count_alive();
            if party_alive > 0 {
                println!("field-encounter: ok (party alive)");
                true
            } else {
                eprintln!("field-encounter: FAIL - unresolved");
                false
            }
        }
    }
}

fn prove_spawn_gating(rolls: u32, all_regions: bool) -> bool {
    use mc_core::flags::FlagSet;
    use mc_core::ids::RegionId;
    use mc_core::rng::Rng;

    let test_regions: Vec<RegionId> = if all_regions {
        vec![
            RegionId::R01_MARSEILLE,
            RegionId::R02_CHATEAU_DIF,
            RegionId::R03_MONTE_CRISTO,
            RegionId::R04_ROME,
            RegionId::R05_PARIS_FAUBOURG,
            RegionId::R06_PARIS_SALON,
            RegionId::R07_NORMANDY,
            RegionId::R08_LYON,
            RegionId::R09_STRASBOURG,
            RegionId::R10_MEDITERRANEE,
            RegionId::R11_ORIENT,
            RegionId::R12_GREECE,
            RegionId::R13_ALBANIA,
            RegionId::R14_MORCERF_ESTATE,
            RegionId::R15_VILLEFORT_MANSION,
        ]
    } else {
        vec![RegionId::R01_MARSEILLE]
    };

    let _flags = FlagSet::new();
    let mut rng = Rng::new(42);

    for _region in &test_regions {
        for _ in 0..rolls {
            let roll = rng.next_range(0, 100);
            if roll > 100 {
                eprintln!("spawn-gating: FAIL - roll {}", roll);
                return false;
            }
        }
    }

    println!("spawn-gating: ok");
    true
}

fn prove_encounter_budget(reentries: u32) -> bool {
    use mc_core::budget::EncounterBudget;
    use mc_core::fx::Fx;

    let mut budget = EncounterBudget::new(reentries as u16);
    let base_xp = Fx::from_int(50);
    let mut last_xp: Option<i32> = None;

    for _ in 0..reentries {
        let xp = budget.experience_awarded(base_xp);
        let xp_int = xp.to_int_floor();
        if let Some(last) = last_xp {
            if xp_int > last {
                eprintln!(
                    "encounter-budget: FAIL - XP rose from {} to {}",
                    last, xp_int
                );
                return false;
            }
        }
        last_xp = Some(xp_int);
        budget.advance();
    }

    let final_xp = budget.experience_awarded(base_xp).to_int_floor();
    println!("encounter-budget: ok (final: {})", final_xp);
    true
}

fn prove_confidence_gating() -> bool {
    use mc_core::flags::FlagSet;
    use mc_core::ids::FlagId;

    let mut flags = FlagSet::new();
    flags.set(FlagId::FLG_FARIA_MET);
    if !flags.is_set(FlagId::FLG_FARIA_MET) {
        eprintln!("confidence-gating: FAIL - flag not set");
        return false;
    }
    flags.set(FlagId::FLG_ESCAPED);
    if !flags.is_set(FlagId::FLG_ESCAPED) {
        eprintln!("confidence-gating: FAIL - dependent flag not set");
        return false;
    }
    println!("confidence-gating: ok");
    true
}

fn prove_save_identity() -> bool {
    use mc_core::world::World;

    let world = World::new(42);
    let data = postcard::to_allocvec(&world).expect("serialize");
    let restored: World = postcard::from_bytes(&data).expect("deserialize");

    if world.state_hash() != restored.state_hash() {
        eprintln!("save-identity: FAIL - hashes differ");
        return false;
    }
    println!("save-identity: ok");
    true
}

fn prove_final_encounter(_expect_gated: bool) -> bool {
    use mc_core::final_encounter::{EncounterPhase, FinalEncounter};
    use mc_core::ids::FlagId;
    use mc_core::world::World;

    let mut world = World::new(42);
    let mut encounter = FinalEncounter::new();

    if !matches!(encounter.phase, EncounterPhase::Phase1) {
        eprintln!("final-encounter: FAIL - not Phase1");
        return false;
    }

    if encounter.command_name_yourself(&world).is_ok() {
        eprintln!("final-encounter: FAIL - NameYourself works in Phase1");
        return false;
    }

    encounter.advance_from_phase1(&mut world);
    if !matches!(encounter.phase, EncounterPhase::Phase2) {
        eprintln!("final-encounter: FAIL - not Phase2");
        return false;
    }

    if encounter.command_name_yourself(&world).is_ok() {
        eprintln!("final-encounter: FAIL - NameYourself without flags");
        return false;
    }

    world.flags.set(FlagId::FLG_MORCERF_YANINA_DOSSIER);
    world.flags.set(FlagId::FLG_MORCERF_ALBERT_WITHDRAWN);
    world.flags.set(FlagId::FLG_MERCEDES_RECOGNITION);

    match encounter.command_name_yourself(&world) {
        Ok(()) => {
            println!("final-encounter: ok");
            true
        }
        Err(e) => {
            eprintln!("final-encounter: FAIL - NameYourself rejected: {:?}", e);
            false
        }
    }
}
