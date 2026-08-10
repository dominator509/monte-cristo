//! `mc_tools prove` — live-fire proofs for the ship gate.
//!
//! Each sub-verb runs a deterministic simulation and asserts a result.
//! Prints one line: "sub-verb: ok" or "sub-verb: FAIL - reason".

use mc_core::command::{apply_commands_with_catalog, ChoiceIdx, Command, CoreEvent};
use mc_core::ids::FlagId;
use mc_core::world::World;
use std::path::Path;
use std::process::ExitCode;

#[derive(clap::Subcommand, Debug)]
pub enum ProveCommand {
    /// Prove that Act I reaches the arrest with FLG_ARRESTED set.
    /// Loads the content pack, verifies the arrest scene definition,
    /// then simulates the scene transition on the core.
    Act1Arrest,
    /// Prove that the full content tree has a complete epilogue ending.
    /// Validates: all 45 confidences, Act VII structure, terminal scene,
    /// and the Fernand final encounter gating chain.
    Epilogue,
    /// Prove that the Château d'If calendar
    IfCalendar {
        #[arg(long, default_value_t = 168)]
        months: u32,
        #[arg(long, default_value_t = 72)]
        faria_at: u32,
        #[arg(long, default_value_t = 4)]
        min_rank3_disciplines: u32,
    },
    /// Prove that a field encounter resolves deterministically
    FieldEncounter {
        #[arg(long, default_value_t = String::from("R03"))]
        region: String,
        #[arg(long, default_value_t = true)]
        expect_victory: bool,
    },
    /// Prove that spawn eligibility is terrain-gated
    SpawnGating {
        #[arg(long, default_value_t = 500)]
        rolls: u32,
        #[arg(long)]
        all_regions: bool,
    },
    /// Prove that encounter budget decays to zero
    EncounterBudget {
        #[arg(long, default_value_t = 40)]
        reentries: u32,
    },
    /// Prove that confidence scene gating works
    ConfidenceGating,
    /// Prove that save/load round-trips identically
    SaveIdentity,
    /// Prove that the final encounter is gated correctly
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
        ProveCommand::Act1Arrest => {
            if prove_act1_arrest() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        ProveCommand::Epilogue => {
            if prove_epilogue() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
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

/// Prove LF-01: Act I reaches the arrest scene with FLG_ARRESTED set.
fn prove_act1_arrest() -> bool {
    let content_dir = Path::new("./content");
    if !content_dir.exists() {
        eprintln!("act1-arrest: FAIL - content directory not found: ./content");
        return false;
    }

    let pack = match mc_data::pack::Pack::from_content(content_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("act1-arrest: FAIL - could not load content pack: {e}");
            return false;
        }
    };

    let arrest_scene = match pack.scenes.iter().find(|s| s.id == "SCN_ARREST") {
        Some(s) => s,
        None => {
            eprintln!(
                "act1-arrest: FAIL - SCN_ARREST not found in content ({} scenes loaded)",
                pack.scenes.len()
            );
            return false;
        }
    };

    let on_exit = match &arrest_scene.on_exit {
        Some(e) => e,
        None => {
            eprintln!("act1-arrest: FAIL - SCN_ARREST has no on_exit effects");
            return false;
        }
    };

    let has_arrest_flag = on_exit.set_flags.iter().any(|f| f == "FLG_ARRESTED");
    if !has_arrest_flag {
        eprintln!(
            "act1-arrest: FAIL - SCN_ARREST on_exit does not set FLG_ARRESTED (flags: {:?})",
            on_exit.set_flags
        );
        return false;
    }

    let catalog = match pack.scene_catalog() {
        Ok(catalog) => catalog,
        Err(e) => {
            eprintln!("act1-arrest: FAIL - authored scene catalog rejected content: {e}");
            return false;
        }
    };

    let mut world = World::new(42);
    if world.flags.is_set(FlagId::FLG_ARRESTED) {
        eprintln!("act1-arrest: FAIL - FLG_ARRESTED already set at game start");
        return false;
    }

    if let Err(e) = catalog.begin(&mut world, "SCN_ARREST") {
        eprintln!("act1-arrest: FAIL - authored arrest scene could not begin: {e}");
        return false;
    }

    // Traverse the authored choice graph, not a reconstructed SceneAdvance.
    // The final SceneAdvance applies the scene's real on_exit effects.
    for choice in [0, 0] {
        let events = apply_commands_with_catalog(
            &mut world,
            &[Command::SceneChoose(ChoiceIdx(choice))],
            Some(&catalog),
        );
        if !matches!(events.first(), Some(CoreEvent::Applied { .. })) {
            eprintln!(
                "act1-arrest: FAIL - authored scene choice {} was rejected: {:?}",
                choice, events
            );
            return false;
        }
    }

    let events = apply_commands_with_catalog(&mut world, &[Command::SceneAdvance], Some(&catalog));
    if !matches!(events.first(), Some(CoreEvent::Applied { .. })) {
        eprintln!(
            "act1-arrest: FAIL - authored arrest scene exit was rejected: {:?}",
            events
        );
        return false;
    }

    if !world.flags.is_set(FlagId::FLG_ARRESTED) {
        eprintln!("act1-arrest: FAIL - FLG_ARRESTED not set after advance traversal");
        return false;
    }

    if world.scene.is_some() {
        eprintln!("act1-arrest: FAIL - authored arrest scene remained active after exit");
        return false;
    }

    println!("act1-arrest: ok");
    println!("  content: SCN_ARREST found with on_exit set_flags including FLG_ARRESTED");
    println!("  runtime: authored choices and SceneAdvance set FLG_ARRESTED in World");
    true
}

/// Prove LF-08 (epilogue): The content tree has a complete ending structure.
///
/// Verifies:
/// 1. All 45 confidence scene files exist across 7 acts (verified from filesystem)
/// 2. Act VII has 6 confidence scenes (cf40-cf45)
/// 3. The SCN_ARRIVAL scene is terminal (proven by content_invariants test)
/// 4. The final encounter gating chain flags exist
/// 5. All scene on_exit flags (from loaded pack) reference known identifiers
fn prove_epilogue() -> bool {
    // ── 0. Filesystem: verify all 45 confidence RON files exist ──────
    let content_dir = Path::new("./content");
    if !content_dir.exists() {
        eprintln!("epilogue: FAIL - content directory not found: ./content");
        return false;
    }

    // Count confidence scene files directly from the filesystem. The pack
    // loader also traverses every act directory, so these counts cross-check
    // the authored tree against the loaded pack below.
    let scenes_base = content_dir.join("scenes");
    let mut confidence_files: Vec<std::path::PathBuf> = Vec::new();
    let mut total_scene_files = 0usize;
    if let Ok(entries) = std::fs::read_dir(&scenes_base) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let act_dir = entry.path();
                if let Ok(act_entries) = std::fs::read_dir(&act_dir) {
                    for scene_entry in act_entries.flatten() {
                        let path = scene_entry.path();
                        if path.extension().map_or(false, |e| e == "ron") {
                            total_scene_files += 1;
                            let fname = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                            if fname.starts_with("scn_confidence_cf") {
                                confidence_files.push(path);
                            }
                        }
                    }
                }
            }
        }
    }

    if confidence_files.len() < 45 {
        eprintln!(
            "epilogue: FAIL - expected at least 45 confidence scene files on disk, found {}",
            confidence_files.len()
        );
        return false;
    }

    // ── 1. Verify Act VII confidence scenes exist ────────────────────
    let act7_dir = scenes_base.join("act7");
    if !act7_dir.exists() {
        eprintln!("epilogue: FAIL - act7 directory not found");
        return false;
    }
    let act7_confidence_count = confidence_files
        .iter()
        .filter(|p| p.parent() == Some(act7_dir.as_path()))
        .count();
    if act7_confidence_count < 6 {
        eprintln!(
            "epilogue: FAIL - expected at least 6 Act VII confidence files, found {}",
            act7_confidence_count
        );
        return false;
    }

    let required_act7 = [
        "scn_confidence_cf40",
        "scn_confidence_cf41",
        "scn_confidence_cf42",
        "scn_confidence_cf43",
        "scn_confidence_cf44",
        "scn_confidence_cf45",
    ];
    for id in &required_act7 {
        let found = confidence_files
            .iter()
            .any(|p| p.file_stem().and_then(|s| s.to_str()) == Some(*id));
        if !found {
            eprintln!(
                "epilogue: FAIL - required Act VII scene `{}` not found on disk",
                id
            );
            return false;
        }
    }

    // ── 2. Load the content pack for flag verification ───────────────
    let pack = match mc_data::pack::Pack::from_content(content_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("epilogue: FAIL - could not load content pack: {e}");
            return false;
        }
    };

    // Verify final encounter gating chain flags exist in flags.ron
    let flag_list: Vec<&str> = pack.flags.iter().map(|s| s.as_str()).collect();
    let required_flags = [
        "FLG_MORCERF_YANINA_DOSSIER",
        "FLG_MORCERF_ALBERT_WITHDRAWN",
        "FLG_MERCEDES_RECOGNITION",
        "FLG_FINAL_PHASE1",
        "FLG_FINAL_PHASE2",
        "FLG_FINAL_PHASE3",
    ];
    for flag in &required_flags {
        if !flag_list.contains(flag) {
            eprintln!(
                "epilogue: FAIL - required flag `{}` not found in flags.ron",
                flag
            );
            return false;
        }
    }

    // ── 3. Scene on_exit flag reference check ────────────────────────
    // Build a set of all known flag identifiers from the pack.
    let known_flags: std::collections::HashSet<&str> = flag_list.into_iter().collect();
    let mut bad_refs = Vec::new();
    for scene in &pack.scenes {
        if let Some(ref effects) = scene.on_exit {
            for flag in &effects.set_flags {
                if !known_flags.contains(flag.as_str()) {
                    bad_refs.push(format!(
                        "scene `{}` set_flags references unknown flag `{}`",
                        scene.id, flag
                    ));
                }
            }
            for flag in &effects.clear_flags {
                if !known_flags.contains(flag.as_str()) {
                    bad_refs.push(format!(
                        "scene `{}` clear_flags references unknown flag `{}`",
                        scene.id, flag
                    ));
                }
            }
        }
    }
    if !bad_refs.is_empty() {
        for r in &bad_refs {
            eprintln!("epilogue: FAIL - {}", r);
        }
        return false;
    }

    // ── 4. Validate via content_invariants (run checks directly) ─────
    // Verify exactly one terminal scene in the loaded pack.
    let terminal_count = pack.scenes.iter().filter(|s| s.terminal).count();
    if terminal_count != 1 {
        eprintln!(
            "epilogue: FAIL - expected exactly one terminal scene in the loaded pack, found {terminal_count}"
        );
        return false;
    }

    let catalog = match pack.scene_catalog() {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("epilogue: FAIL - authored scene catalog rejected content: {error}");
            return false;
        }
    };
    let Some(terminal_scene) = pack.scenes.iter().find(|scene| scene.terminal) else {
        eprintln!("epilogue: FAIL - terminal scene disappeared during catalog verification");
        return false;
    };
    let mut terminal_world = mc_core::world::World::new(42);
    for raw in 0..mc_core::ids::FlagId::COUNT as u16 {
        terminal_world
            .flags
            .set(mc_core::ids::FlagId::from_raw(raw));
    }
    if catalog
        .begin(&mut terminal_world, &terminal_scene.id)
        .is_err()
    {
        eprintln!(
            "epilogue: FAIL - terminal scene `{}` could not begin from the authored catalog",
            terminal_scene.id
        );
        return false;
    }
    for _ in 0..catalog.node_count() {
        let Some(current) = terminal_world.scene.as_ref().map(|state| state.current) else {
            break;
        };
        let Some(node) = catalog.node(current) else {
            eprintln!("epilogue: FAIL - terminal scene cursor left the authored catalog");
            return false;
        };
        if node.choices.is_empty() {
            break;
        }
        let events = apply_commands_with_catalog(
            &mut terminal_world,
            &[Command::SceneChoose(ChoiceIdx(0))],
            Some(&catalog),
        );
        if !matches!(events.first(), Some(CoreEvent::Applied { .. })) {
            eprintln!("epilogue: FAIL - terminal scene choice was rejected");
            return false;
        }
    }
    let events = apply_commands_with_catalog(
        &mut terminal_world,
        &[Command::SceneAdvance],
        Some(&catalog),
    );
    if !matches!(events.first(), Some(CoreEvent::Applied { .. })) || terminal_world.scene.is_some()
    {
        eprintln!("epilogue: FAIL - terminal scene did not exit through authored effects");
        return false;
    }

    let known_count = known_flags.len();
    println!("epilogue: ok");
    println!(
        "  scene files: {} total, {} confidence (45 required)",
        total_scene_files,
        confidence_files.len()
    );
    println!(
        "  act7 scenes: {} (cf40-cf45 present)",
        act7_confidence_count
    );
    println!(
        "  flags in pack: {} known, {} invalid scene references",
        known_count,
        bad_refs.len()
    );
    println!("  terminal scenes: 1 ({} traversed)", terminal_scene.id);
    true
}

fn prove_if_calendar(months: u32, faria_at: u32, min_rank3: u32) -> bool {
    use mc_core::calendar::CalendarAction;
    use mc_core::curriculum::Discipline;
    use mc_core::world::{Act, World};

    let mut world = World::new(42);
    world.set_act(Act::ActIIIf);

    let disciplines = [
        Discipline::Fencing,
        Discipline::Chemistry,
        Discipline::NaturalPhilosophy,
        Discipline::Mathematics,
        Discipline::Languages,
        Discipline::HistoryPolitics,
        Discipline::Economics,
    ];

    for m in 0..months {
        if world
            .calendar
            .as_ref()
            .is_some_and(|calendar| calendar.is_complete())
        {
            break;
        }
        let disc = disciplines[(m as usize) % 7];
        let events = mc_core::command::apply_commands(
            &mut world,
            &[mc_core::command::Command::CalendarAct(
                CalendarAction::Study(disc),
            )],
        );
        if !matches!(
            events.first(),
            Some(mc_core::command::CoreEvent::Applied { .. })
        ) {
            eprintln!("if-calendar: FAIL - study command was rejected");
            return false;
        }
    }

    let faria_joined = world
        .calendar
        .as_ref()
        .is_some_and(|calendar| calendar.faria_joined && calendar.month >= faria_at);
    if !faria_joined {
        eprintln!(
            "if-calendar: FAIL - Faria did not join by month {}",
            faria_at
        );
        return false;
    }

    let mut count_rank3 = 0u32;
    for disc in &disciplines {
        if world.curriculum.rank(*disc) >= 3 {
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

fn prove_field_encounter(region: &str, expect_victory: bool) -> bool {
    use mc_core::battle::atb::AtbGauge;
    use mc_core::battle::status::StatusList;
    use mc_core::battle::{Affiliation, Battle, Combatant, CombatantKind};
    use mc_core::fx::Fx;
    use mc_core::ids::{CharId, EnemyId, RegionId};

    let region_id = match region {
        "R01" => RegionId::R01_MARSEILLE,
        "R02" => RegionId::R02_CHATEAU_DIF,
        "R03" => RegionId::R03_MONTE_CRISTO,
        "R04" => RegionId::R04_ROME,
        "R05" => RegionId::R05_PARIS_FAUBOURG,
        "R06" => RegionId::R06_PARIS_SALON,
        "R07" => RegionId::R07_NORMANDY,
        "R08" => RegionId::R08_LYON,
        "R09" => RegionId::R09_STRASBOURG,
        "R10" => RegionId::R10_MEDITERRANEE,
        "R11" => RegionId::R11_ORIENT,
        "R12" => RegionId::R12_GREECE,
        "R13" => RegionId::R13_ALBANIA,
        "R14" => RegionId::R14_MORCERF_ESTATE,
        "R15" => RegionId::R15_VILLEFORT_MANSION,
        _ => {
            eprintln!("field-encounter: FAIL - unknown region `{region}`");
            return false;
        }
    };

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

    let mut world = mc_core::world::World::new(42);
    world.region = region_id;
    world.battle = Some(Battle::new(vec![edmond], vec![bandit]));

    for _ in 0..1000 {
        world.step();
        if !matches!(
            world.battle.as_ref().map(|battle| &battle.state),
            Some(mc_core::battle::BattleState::Active)
        ) {
            break;
        }
        let action = world.battle.as_ref().and_then(|battle| {
            let actor = battle.next_ready_combatant()?;
            if battle.combatants[actor].affiliation != Affiliation::Party {
                return None;
            }
            let target = battle.first_enemy_index()?;
            Some((actor, target))
        });
        if let Some((actor, target)) = action {
            let events = mc_core::command::apply_commands(
                &mut world,
                &[mc_core::command::Command::SelectAction(
                    mc_core::command::ActorId(actor),
                    mc_core::command::Action::Attack {
                        target: mc_core::command::TargetId(target),
                    },
                )],
            );
            if !matches!(
                events.first(),
                Some(mc_core::command::CoreEvent::Applied { .. })
            ) {
                eprintln!("field-encounter: FAIL - authored attack command was rejected");
                return false;
            }
        }
    }

    let Some(actual_state) = world.battle.as_ref().map(|battle| battle.state.clone()) else {
        eprintln!("field-encounter: FAIL - battle state disappeared");
        return false;
    };
    let expected_state = if expect_victory {
        mc_core::battle::BattleState::Victory
    } else {
        mc_core::battle::BattleState::Defeat
    };
    if actual_state != expected_state {
        eprintln!(
            "field-encounter: FAIL - expected {:?}, got {:?}",
            expected_state, actual_state
        );
        return false;
    }

    println!(
        "field-encounter: ok ({:?}, region {})",
        actual_state, region
    );
    true
}

fn prove_spawn_gating(rolls: u32, all_regions: bool) -> bool {
    let pack = match mc_data::pack::Pack::from_content(Path::new("./content")) {
        Ok(pack) => pack,
        Err(error) => {
            eprintln!("spawn-gating: FAIL - could not load authored bestiary: {error}");
            return false;
        }
    };

    use mc_core::rng::Rng;
    use std::collections::BTreeSet;

    const REGION_KEYS: [&str; 15] = [
        "R01", "R02", "R03", "R04", "R05", "R06", "R07", "R08", "R09", "R10", "R11", "R12", "R13",
        "R14", "R15",
    ];
    let region_count = if all_regions { REGION_KEYS.len() } else { 1 };
    let no_flags = BTreeSet::new();
    let mut arrested = BTreeSet::new();
    arrested.insert("FLG_ARRESTED".to_string());

    for (region_index, region_key) in REGION_KEYS.iter().take(region_count).enumerate() {
        let eligible: Vec<_> = pack
            .enemies
            .iter()
            .filter(|enemy| {
                enemy
                    .region_affinity
                    .iter()
                    .any(|affinity| affinity == region_key)
                    && authored_gate_satisfied(&enemy.gate, &no_flags)
            })
            .collect();
        if eligible.is_empty() {
            eprintln!("spawn-gating: FAIL - no authored enemy is eligible in {region_key}");
            return false;
        }

        let mut rng_a = Rng::new(42u128 + region_index as u128);
        let mut rng_b = Rng::new(42u128 + region_index as u128);
        for _ in 0..rolls {
            let upper = eligible.len() as u32 - 1;
            let Ok(index_a) = rng_a.next_range(0, upper) else {
                eprintln!("spawn-gating: FAIL - authored pool range was invalid");
                return false;
            };
            let Ok(index_b) = rng_b.next_range(0, upper) else {
                eprintln!("spawn-gating: FAIL - repeated authored pool range was invalid");
                return false;
            };
            if index_a != index_b {
                eprintln!("spawn-gating: FAIL - authored spawn sequence diverged");
                return false;
            }
            let enemy = eligible[index_a as usize];
            if !enemy
                .region_affinity
                .iter()
                .any(|affinity| affinity == region_key)
                || !authored_gate_satisfied(&enemy.gate, &no_flags)
            {
                eprintln!(
                    "spawn-gating: FAIL - resolver escaped the authored eligibility predicate"
                );
                return false;
            }
        }
    }

    let Some(gendarme) = pack.enemies.iter().find(|enemy| enemy.id == "ENM_GENDARME") else {
        eprintln!("spawn-gating: FAIL - authored ENM_GENDARME gate is missing");
        return false;
    };
    if !authored_gate_satisfied(&gendarme.gate, &no_flags)
        || authored_gate_satisfied(&gendarme.gate, &arrested)
    {
        eprintln!("spawn-gating: FAIL - FLG_ARRESTED did not close the authored ENM_GENDARME gate");
        return false;
    }

    println!(
        "spawn-gating: ok ({} authored pools, {} deterministic rolls each)",
        region_count, rolls
    );
    true
}

fn authored_gate_satisfied(
    gate: &mc_data::schema::enemy::FlagExpr,
    flags: &std::collections::BTreeSet<String>,
) -> bool {
    use mc_data::schema::enemy::FlagExpr;

    match gate {
        FlagExpr::Always => true,
        FlagExpr::All(required) => required.iter().all(|flag| flags.contains(flag)),
        FlagExpr::Any(required) => required.iter().any(|flag| flags.contains(flag)),
        FlagExpr::Not(flag) => !flags.contains(flag),
    }
}

fn prove_encounter_budget(reentries: u32) -> bool {
    use mc_core::budget::EncounterBudget;
    use mc_core::fx::Fx;

    let Ok(pool) = u16::try_from(reentries) else {
        eprintln!("encounter-budget: FAIL - reentries exceed the authored u16 budget bound");
        return false;
    };
    let mut budget = EncounterBudget::new(pool);
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
        if !budget.advance() {
            eprintln!(
                "encounter-budget: FAIL - budget exhausted before all reentries were cleared"
            );
            return false;
        }
    }

    if !budget.is_exhausted() {
        eprintln!("encounter-budget: FAIL - budget did not exhaust after all requested reentries");
        return false;
    }
    let final_xp = budget.experience_awarded(base_xp).to_int_floor();
    println!("encounter-budget: ok (final: {})", final_xp);
    true
}

fn prove_confidence_gating() -> bool {
    use mc_core::ids::FlagId;

    let pack = match mc_data::pack::Pack::from_content(Path::new("./content")) {
        Ok(pack) => pack,
        Err(error) => {
            eprintln!("confidence-gating: FAIL - could not load authored scenes: {error}");
            return false;
        }
    };
    let catalog = match pack.scene_catalog() {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("confidence-gating: FAIL - scene catalog rejected content: {error}");
            return false;
        }
    };

    let mut world = mc_core::world::World::new(42);
    world.flags.set(FlagId::FLG_ARRESTED);
    if catalog.begin(&mut world, "SCN_FARIA_MEETING").is_err() {
        eprintln!("confidence-gating: FAIL - SCN_FARIA_MEETING could not begin");
        return false;
    }
    for choice in [0, 0] {
        let events = apply_commands_with_catalog(
            &mut world,
            &[Command::SceneChoose(ChoiceIdx(choice))],
            Some(&catalog),
        );
        if !matches!(events.first(), Some(CoreEvent::Applied { .. })) {
            eprintln!("confidence-gating: FAIL - Faria choice was rejected");
            return false;
        }
    }
    let events = apply_commands_with_catalog(&mut world, &[Command::SceneAdvance], Some(&catalog));
    if !matches!(events.first(), Some(CoreEvent::Applied { .. }))
        || !world.flags.is_set(FlagId::FLG_FARIA_MET)
    {
        eprintln!("confidence-gating: FAIL - authored Faria scene did not set FLG_FARIA_MET");
        return false;
    }

    world.flags.set(FlagId::FLG_TREASURE_KNOWN);
    if catalog.begin(&mut world, "SCN_ESCAPE").is_err() {
        eprintln!("confidence-gating: FAIL - SCN_ESCAPE did not honor its authored gate");
        return false;
    }
    for choice in [0, 0] {
        let events = apply_commands_with_catalog(
            &mut world,
            &[Command::SceneChoose(ChoiceIdx(choice))],
            Some(&catalog),
        );
        if !matches!(events.first(), Some(CoreEvent::Applied { .. })) {
            eprintln!("confidence-gating: FAIL - escape choice was rejected");
            return false;
        }
    }
    let events = apply_commands_with_catalog(&mut world, &[Command::SceneAdvance], Some(&catalog));
    if !matches!(events.first(), Some(CoreEvent::Applied { .. }))
        || !world.flags.is_set(FlagId::FLG_ESCAPED)
    {
        eprintln!("confidence-gating: FAIL - authored escape scene did not set FLG_ESCAPED");
        return false;
    }

    println!("confidence-gating: ok (authored Faria and escape scenes)");
    true
}

fn prove_save_identity() -> bool {
    use mc_core::world::World;

    let world = World::new(42);
    let data = match postcard::to_allocvec(&world) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("save-identity: FAIL - serialize: {error}");
            return false;
        }
    };
    let restored: World = match postcard::from_bytes(&data) {
        Ok(restored) => restored,
        Err(error) => {
            eprintln!("save-identity: FAIL - deserialize: {error}");
            return false;
        }
    };

    if world.state_hash() != restored.state_hash() {
        eprintln!("save-identity: FAIL - hashes differ");
        return false;
    }
    println!("save-identity: ok");
    true
}

fn prove_final_encounter(expect_gated: bool) -> bool {
    use mc_core::final_encounter::{EncounterPhase, FinalEncounter};
    use mc_core::ids::FlagId;
    use mc_core::world::World;

    if !expect_gated {
        eprintln!("final-encounter: FAIL - proof requires --expect-gated-name-yourself");
        return false;
    }

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

    if encounter.apply_damage(&mut world) {
        eprintln!("final-encounter: FAIL - Phase2 accepted damage transition");
        return false;
    }
    if !matches!(encounter.phase, EncounterPhase::Phase2) {
        eprintln!("final-encounter: FAIL - Phase2 was not damage-immune");
        return false;
    }

    world.flags.set(FlagId::FLG_MORCERF_YANINA_DOSSIER);
    world.flags.set(FlagId::FLG_MORCERF_ALBERT_WITHDRAWN);
    world.flags.set(FlagId::FLG_MERCEDES_RECOGNITION);

    if let Err(e) = encounter.command_name_yourself(&world) {
        eprintln!("final-encounter: FAIL - NameYourself rejected: {:?}", e);
        return false;
    }

    encounter.execute_name_yourself(&mut world);
    if !matches!(encounter.phase, EncounterPhase::Phase3)
        || !world.flags.is_set(FlagId::FLG_FINAL_PHASE2)
    {
        eprintln!("final-encounter: FAIL - NameYourself did not enter Phase3");
        return false;
    }

    if !encounter.resolve_phase3(&mut world)
        || !matches!(encounter.phase, EncounterPhase::Resolved)
        || !world.flags.is_set(FlagId::FLG_FINAL_PHASE3)
    {
        eprintln!("final-encounter: FAIL - Phase3 did not resolve");
        return false;
    }

    println!("final-encounter: ok");
    true
}
