//! Real CLI coverage for the developer and ship-gate entry point.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const REPO_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "monte-cristo-mc-tools-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("cli")
        ));
        if path.exists() {
            std::fs::remove_dir_all(&path).expect("remove stale CLI test directory");
        }
        std::fs::create_dir_all(&path).expect("create CLI test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mc_tools"))
        .current_dir(REPO_ROOT)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("run mc_tools {args:?}: {error}"))
}

fn assert_success(args: &[&str], expected: &str) -> Output {
    let output = run(args);
    assert!(
        output.status.success(),
        "mc_tools {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(expected),
        "mc_tools {args:?} omitted `{expected}`\nstdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    output
}

fn assert_failure(args: &[&str], expected: &str) {
    let output = run(args);
    assert!(
        !output.status.success(),
        "mc_tools {args:?} unexpectedly succeeded"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains(expected),
        "mc_tools {args:?} omitted `{expected}`\noutput:\n{combined}"
    );
}

#[test]
fn cli_success_paths_cover_real_content_tapes_and_proofs() {
    let temp = TempDir::new();
    let pack = temp.path().join("content.pack");
    let commands = temp.path().join("commands.txt");
    let recorded = temp.path().join("recorded.tape");
    std::fs::write(
        &commands,
        "# all command variants\nInteract\nMove North\nMove South\nMove East\nMove West\n\
         OpenMenu\nCloseMenu\nSceneAdvance\nCancelSelection\nNameYourself\n",
    )
    .expect("write command fixture");

    assert_success(&["validate", "--input", "./content"], "content: ok");
    assert_success(
        &[
            "bake",
            "--input",
            "./content",
            "--output",
            pack.to_str().expect("pack path is UTF-8"),
        ],
        "bake: ok",
    );
    assert!(pack.is_file(), "bake command must write the pack");

    assert_success(
        &[
            "record",
            "--out",
            recorded.to_str().expect("tape path is UTF-8"),
            "--commands",
            commands.to_str().expect("commands path is UTF-8"),
            "--seed",
            "42",
        ],
        "tape written",
    );
    assert_success(
        &[
            "replay",
            "--tape",
            recorded.to_str().expect("tape path is UTF-8"),
            "--assert-hash",
        ],
        "hash: match",
    );
    let printed_hash = assert_success(
        &["replay", "--tape", "tapes/golden-full.tape", "--print-hash"],
        "",
    );
    assert_eq!(
        String::from_utf8_lossy(&printed_hash.stdout).trim().len(),
        64,
        "printed state hash is 32-byte lowercase hex"
    );

    assert_success(&["report", "bestiary"], "report bestiary: ok");
    let fixture_dir = Path::new(REPO_ROOT).join("tests/fixtures/saves-v1");
    let fixture = fixture_dir.join("save.v1");
    let fixture_before = std::fs::read(&fixture).expect("read migration fixture before dry-run");
    assert_success(
        &[
            "save-migrate",
            "--dir",
            fixture_dir.to_str().expect("fixture directory is UTF-8"),
            "--dry-run",
        ],
        "save-migrate: ok (1 fixture(s), dry-run)",
    );
    assert_eq!(
        std::fs::read(&fixture).expect("read migration fixture after dry-run"),
        fixture_before,
        "save-migrate --dry-run must not modify source fixtures"
    );
    assert_success(&["prove", "act1-arrest"], "act1-arrest: ok");
    assert_success(&["prove", "epilogue"], "epilogue: ok");
    assert_success(&["prove", "if-calendar"], "if-calendar: ok");
    assert_success(&["prove", "field-encounter"], "field-encounter: ok");
    assert_success(
        &["prove", "spawn-gating", "--all-regions"],
        "spawn-gating: ok",
    );
    assert_success(
        &["prove", "encounter-budget", "--reentries", "40"],
        "encounter-budget: ok",
    );
    assert_success(&["prove", "confidence-gating"], "confidence-gating: ok");
    assert_success(&["prove", "save-identity"], "save-identity: ok");
    assert_success(
        &["prove", "final-encounter", "--expect-gated-name-yourself"],
        "final-encounter: ok",
    );
}

#[test]
fn cli_failure_paths_report_real_errors() {
    let temp = TempDir::new();
    let missing = temp.path().join("missing");
    let bad_commands = temp.path().join("bad-commands.txt");
    let bad_tape = temp.path().join("bad.tape");
    std::fs::write(&bad_commands, "Move Up\n").expect("write invalid command fixture");
    std::fs::write(&bad_tape, b"not a tape").expect("write invalid tape fixture");

    assert_failure(
        &[
            "save-migrate",
            "--dir",
            missing.to_str().expect("missing path is UTF-8"),
            "--dry-run",
        ],
        "cannot read save fixture directory",
    );
    assert_failure(
        &[
            "validate",
            "--input",
            missing.to_str().expect("missing path is UTF-8"),
        ],
        "content:",
    );
    assert_failure(
        &[
            "report",
            "--input",
            missing.to_str().expect("missing path is UTF-8"),
            "bestiary",
        ],
        "content directory not found",
    );
    assert_failure(
        &[
            "replay",
            "--tape",
            bad_tape.to_str().expect("bad tape path is UTF-8"),
            "--assert-hash",
        ],
        "failed to deserialize tape",
    );
    assert_failure(
        &[
            "record",
            "--out",
            temp.path()
                .join("should-not-exist.tape")
                .to_str()
                .expect("output path is UTF-8"),
            "--commands",
            bad_commands.to_str().expect("commands path is UTF-8"),
        ],
        "invalid direction",
    );

    let smoke_path = Path::new(REPO_ROOT).join("tapes/golden-smoke.tape");
    let mut mismatched = mc_tape::format::Tape::from_bytes(&std::fs::read(smoke_path).unwrap())
        .expect("golden smoke tape parses");
    mismatched.final_hash = [0xFF; 32];
    let mismatched_path = temp.path().join("mismatched.tape");
    std::fs::write(
        &mismatched_path,
        mismatched.to_bytes().expect("mismatched tape serializes"),
    )
    .expect("write mismatched tape");
    assert_failure(
        &[
            "replay",
            "--tape",
            mismatched_path.to_str().expect("mismatched path is UTF-8"),
            "--assert-hash",
        ],
        "hash mismatch",
    );

    for flag in [
        "ARRESTED",
        "FLG_FARIA_MET",
        "FLG_TREASURE_KNOWN",
        "FLG_ESCAPED",
        "FLG_COMTE_IDENTITY",
        "FLG_SINDBAD_VISITED",
        "FLG_MORCERF_DOSSIER",
        "FLG_MORCERF_YANINA_DOSSIER",
        "FLG_MORCERF_ALBERT_WITHDRAWN",
        "FLG_DANGLARS_LETTER",
        "FLG_VILLEFORT_DOSSIER",
        "FLG_HELOISE_POISONING",
        "FLG_VALENTINE_SAFE",
        "FLG_MERCEDES_RECOGNITION",
        "FLG_EDOUARD_TRUTH",
        "FLG_FERNAND_CONFRONTED",
        "FLG_DANGLARS_CONFRONTED",
        "FLG_VILLEFORT_CONFRONTED",
        "FLG_MERCEDES_FORGIVEN",
        "FLG_FINAL_PHASE1",
        "FLG_FINAL_PHASE2",
        "FLG_FINAL_PHASE3",
    ] {
        let output = run(&[
            "replay",
            "--tape",
            "tapes/golden-full.tape",
            "--require-flag",
            flag,
        ]);
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !combined.contains("unknown flag name"),
            "locked flag `{flag}` must parse"
        );
    }
    assert_failure(
        &[
            "replay",
            "--tape",
            "tapes/golden-full.tape",
            "--require-flag",
            "FLG_NOT_REAL",
        ],
        "unknown flag name",
    );
}

#[test]
fn golden_generator_writes_replayable_artifacts_outside_the_repository() {
    let temp = TempDir::new();
    mc_tools::golden_tapes::generate_to(temp.path());

    let hashes = std::fs::read_to_string(temp.path().join("HASHES.txt"))
        .expect("read generated hash manifest");
    for name in ["golden-full.tape", "golden-smoke.tape"] {
        let data = std::fs::read(temp.path().join(name)).expect("read generated tape");
        let tape = mc_tape::format::Tape::from_bytes(&data).expect("parse generated tape");
        let replay = mc_tape::replay::replay(&tape).expect("replay generated tape");
        assert_eq!(replay.final_hash, tape.final_hash, "{name} hash");
        assert!(
            hashes.contains(name),
            "hash manifest must contain generated {name}"
        );
    }
}
