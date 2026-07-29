//! Generate golden-full.tape and golden-smoke.tape using the real RecordTape API.
//! Run with: cargo run --release --bin gen-golden-tapes

use mc_core::calendar::CalendarAction;
use mc_core::command::{CampaignAction, CampaignId, ChoiceIdx, Command, Dir, PersonaId, SaveSlot};
use mc_core::curriculum::Discipline;
use mc_core::ids::RegionId;
use mc_core::world::World;
use mc_tape::format::Tape;
use mc_tape::record::RecordTape;
use std::path::Path;

fn hex_fmt(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn write_tape(tape: &Tape, path: &Path) {
    let bytes = tape.to_bytes().expect("serialization");
    std::fs::write(path, &bytes).expect("write tape");
    eprintln!(
        "wrote {} ({} bytes, {} entries)",
        path.display(),
        bytes.len(),
        tape.len()
    );
}

fn build_golden_smoke(seed: u128) -> Tape {
    let world = World::new(seed);
    let mut r = RecordTape::new(world);
    // Act I: Marseille docks — intro scenes
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::Move(Dir::North)).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::OpenMenu).unwrap();
    r.record_command(Command::CloseMenu).unwrap();
    r.record_command(Command::Move(Dir::East)).unwrap();
    r.record_command(Command::Move(Dir::East)).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Move(Dir::South)).unwrap();
    r.record_command(Command::Move(Dir::West)).unwrap();
    r.record_command(Command::Interact).unwrap();
    // Act II: Château d'If — calendar actions
    r.record_command(Command::CalendarAct(CalendarAction::Dig))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Study(
        Discipline::Fencing,
    )))
    .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Endure))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Observe))
        .unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.finalize().expect("finalize smoke tape")
}

fn build_golden_full(seed: u128) -> Tape {
    let world = World::new(seed);
    let mut r = RecordTape::new(world);

    // ── Act I: Marseille ────────────────────────────────────────
    // Edmond at the docks, farewell to Mercedes, arrest scene
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::Move(Dir::North)).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::OpenMenu).unwrap();
    r.record_command(Command::CloseMenu).unwrap();
    r.record_command(Command::Move(Dir::East)).unwrap();
    r.record_command(Command::Move(Dir::East)).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Move(Dir::South)).unwrap();
    r.record_command(Command::Move(Dir::West)).unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SceneChoose(ChoiceIdx(0)))
        .unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::Move(Dir::North)).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();

    // ── Act II: Château d'If ────────────────────────────────────
    // Edmond meets Faria, learns, digs, studies
    r.record_command(Command::CalendarAct(CalendarAction::Dig))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Study(
        Discipline::Fencing,
    )))
    .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Endure))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Observe))
        .unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Dig))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Study(
        Discipline::Chemistry,
    )))
    .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Endure))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Observe))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Dig))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Study(
        Discipline::Languages,
    )))
    .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Dig))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Study(
        Discipline::Mathematics,
    )))
    .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Endure))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Dig))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Study(
        Discipline::HistoryPolitics,
    )))
    .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Dig))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Study(
        Discipline::NaturalPhilosophy,
    )))
    .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Dig))
        .unwrap();
    r.record_command(Command::CalendarAct(CalendarAction::Dig))
        .unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();

    // ── Act III: Monte Cristo ───────────────────────────────────
    // Treasure found, persona swap, fast travel
    r.record_command(Command::SwapPersona(PersonaId::MonteCristo))
        .unwrap();
    r.record_command(Command::FastTravel(RegionId::R01_MARSEILLE))
        .unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SwapPersona(PersonaId::Sinbad))
        .unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SwapPersona(PersonaId::Busoni))
        .unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SwapPersona(PersonaId::MonteCristo))
        .unwrap();
    r.record_command(Command::FastTravel(RegionId::R03_MONTE_CRISTO))
        .unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();

    // ── Act IV: Rome ────────────────────────────────────────────
    r.record_command(Command::FastTravel(RegionId::R04_ROME))
        .unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SceneChoose(ChoiceIdx(1)))
        .unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::OpenMenu).unwrap();
    r.record_command(Command::CloseMenu).unwrap();
    r.record_command(Command::Interact).unwrap();

    // ── Act V: Paris (prelude) ──────────────────────────────────
    r.record_command(Command::FastTravel(RegionId::R05_PARIS_FAUBOURG))
        .unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SwapPersona(PersonaId::MonteCristo))
        .unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneChoose(ChoiceIdx(0)))
        .unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Interact).unwrap();

    // ── Act VI: Paris Season ────────────────────────────────────
    // Three campaigns: Morcerf, Danglars, Villefort
    r.record_command(Command::SeasonAct(
        CampaignId("morcerf".into()),
        CampaignAction::Investigate,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("morcerf".into()),
        CampaignAction::Advance,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("danglars".into()),
        CampaignAction::Investigate,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("danglars".into()),
        CampaignAction::Advance,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("villefort".into()),
        CampaignAction::Investigate,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("villefort".into()),
        CampaignAction::Advance,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("morcerf".into()),
        CampaignAction::Confront,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("danglars".into()),
        CampaignAction::Confront,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("villefort".into()),
        CampaignAction::Confront,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("morcerf".into()),
        CampaignAction::Rest,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("danglars".into()),
        CampaignAction::Rest,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("villefort".into()),
        CampaignAction::Rest,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("morcerf".into()),
        CampaignAction::Advance,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("danglars".into()),
        CampaignAction::Advance,
    ))
    .unwrap();
    r.record_command(Command::SeasonAct(
        CampaignId("villefort".into()),
        CampaignAction::Advance,
    ))
    .unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();

    // ── Act VII: Final ──────────────────────────────────────────
    // Confrontation, name yourself, final encounter
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneChoose(ChoiceIdx(1)))
        .unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::Save(SaveSlot(0))).unwrap();
    r.record_command(Command::Interact).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();
    r.record_command(Command::SceneAdvance).unwrap();

    r.finalize().expect("finalize golden-full tape")
}

/// Generate and verify both golden tapes and their hash manifest in `tapes_dir`.
pub fn generate_to(tapes_dir: &Path) {
    let seed: u128 = 42;

    std::fs::create_dir_all(tapes_dir).expect("create tapes dir");

    // Golden full
    let full_tape = build_golden_full(seed);
    let full_path = tapes_dir.join("golden-full.tape");
    write_tape(&full_tape, &full_path);

    // Golden smoke
    let smoke_tape = build_golden_smoke(seed);
    let smoke_path = tapes_dir.join("golden-smoke.tape");
    write_tape(&smoke_tape, &smoke_path);

    // Verify determinism: generate the same tape twice
    let full_v2 = build_golden_full(seed);
    assert_eq!(
        full_tape.final_hash, full_v2.final_hash,
        "golden-full determinism check"
    );
    let smoke_v2 = build_golden_smoke(seed);
    assert_eq!(
        smoke_tape.final_hash, smoke_v2.final_hash,
        "golden-smoke determinism check"
    );

    // Write HASHES.txt
    let mut hashes = String::new();
    hashes.push_str(&format!(
        "golden-full.tape {}\n",
        hex_fmt(&full_tape.final_hash)
    ));
    hashes.push_str(&format!(
        "golden-smoke.tape {}\n",
        hex_fmt(&smoke_tape.final_hash)
    ));
    let hashes_path = tapes_dir.join("HASHES.txt");
    std::fs::write(&hashes_path, &hashes).expect("write HASHES.txt");
    eprintln!("wrote {}", hashes_path.display());

    // Verify replay determinism via the replay module
    let result = mc_tape::replay::replay(&full_tape).expect("replay golden-full");
    assert!(
        result.first_divergence.is_none(),
        "golden-full replay divergence"
    );
    assert_eq!(
        result.final_hash, full_tape.final_hash,
        "golden-full hash match"
    );

    let result = mc_tape::replay::replay(&smoke_tape).expect("replay golden-smoke");
    assert!(
        result.first_divergence.is_none(),
        "golden-smoke replay divergence"
    );
    assert_eq!(
        result.final_hash, smoke_tape.final_hash,
        "golden-smoke hash match"
    );

    eprintln!("Golden tapes generated and verified. All checks pass.");
}
