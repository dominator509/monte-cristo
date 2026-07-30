//! Monte Cristo presentation shell entry point.
//!
//! SPEC-004 sections 1, 10, 11. MC_HEADLESS=1 suppresses window + audio.
//! The binary is named "monte-cristo".

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{fs, io::Write};

use mc_shell::app::App;
use mc_shell::config::ValidatedConfig;
use mc_shell::fsroot::{self, Root};

/// Get the data directory for settings and saves.
fn data_dir() -> PathBuf {
    let base = std::env::var("MC_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            PathBuf::from(home).join(".local/share/monte-cristo")
        });
    // Use fsroot for confined directory creation — set MC_DATA_DIR for it.
    std::env::set_var(Root::Data.env_var(), &base);
    let _ = fsroot::create_dir_all(Root::Data, Path::new(""));
    base
}

/// Run the headless application (no window, no audio).
fn run_headless(seed: u128, config: ValidatedConfig) {
    tracing::info!("starting headless mode");
    let mut app = App::new(seed, config, true);
    for _i in 0..60 {
        app.headless_update();
    }
    tracing::info!(
        "headless run complete: tick={}, state_hash={:?}",
        app.world.tick,
        app.world.state_hash()
    );
}

/// Real main — checks MC_HEADLESS before any window creation.
fn verify_content() -> ExitCode {
    match mc_data::pack::Pack::load_from_dir(Path::new(".")) {
        Ok(_) => {
            println!("content: ok");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!(
                "Content verification failed for content.pack: {error}. \
                 Restore content.pack and content.pack.blake3, then retry."
            );
            ExitCode::FAILURE
        }
    }
}

fn check_paths() -> ExitCode {
    for root in [Root::Content, Root::Data, Root::Artifact] {
        let Some(path) = root.resolve_env() else {
            eprintln!("Path check failed: {} is unset or empty.", root.env_var());
            return ExitCode::FAILURE;
        };
        let Ok(path) = path.canonicalize() else {
            eprintln!(
                "Path check failed: {} does not resolve to an existing directory.",
                root.env_var()
            );
            return ExitCode::FAILURE;
        };
        if !path.is_dir() {
            eprintln!("Path check failed: {} is not a directory.", root.env_var());
            return ExitCode::FAILURE;
        }

        let probe = path.join(format!(".monte-cristo-write-probe-{}", std::process::id()));
        let write_result = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&probe)
            .and_then(|mut file| file.write_all(b"ok"));
        if let Err(error) = write_result {
            eprintln!(
                "Path check failed: {} is not writable: {error}",
                root.env_var()
            );
            return ExitCode::FAILURE;
        }
        if let Err(error) = fs::remove_file(&probe) {
            eprintln!(
                "Path check failed: could not remove the {} write probe: {error}",
                root.env_var()
            );
            return ExitCode::FAILURE;
        }
    }

    println!("paths: ok");
    ExitCode::SUCCESS
}

fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn save_info(path: &Path) -> ExitCode {
    match mc_data::save::Save::from_file(path) {
        Ok(save) => {
            println!("schema_version: {}", save.schema_version);
            println!("product_version: {}", save.product_version);
            println!("content_digest: {}", hex(&save.content_digest));
            println!("save_digest: {}", hex(&save.digest));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Save inspection failed for {}: {error}", path.display());
            ExitCode::FAILURE
        }
    }
}

fn replay_tape(path: &Path, assert_hash: bool) -> ExitCode {
    let result = fs::read(path)
        .map_err(|error| format!("failed to read tape {}: {error}", path.display()))
        .and_then(|bytes| {
            mc_tape::format::Tape::from_bytes(&bytes)
                .map_err(|error| format!("failed to deserialize tape: {error}"))
        })
        .and_then(|tape| {
            mc_tape::replay::replay(&tape)
                .map_err(|error| format!("replay failed: {error}"))
                .map(|result| (tape, result))
        });

    match result {
        Ok((tape, result)) if assert_hash && result.final_hash != tape.final_hash => {
            eprintln!(
                "hash: mismatch (expected {}, got {})",
                hex(&tape.final_hash),
                hex(&result.final_hash)
            );
            ExitCode::FAILURE
        }
        Ok((_tape, result)) => {
            if assert_hash {
                println!("hash: match");
            } else {
                println!("{}", hex(&result.final_hash));
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    // Initialize tracing
    tracing_subscriber::fmt().with_target(false).init();

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let print_version = args.iter().any(|a| a == "--version" || a == "-V");
    if print_version {
        println!("Monte Cristo v{}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "--verify-content") {
        return verify_content();
    }
    if args.iter().any(|a| a == "--check-paths") {
        return check_paths();
    }
    if let Some(index) = args.iter().position(|arg| arg == "--save-info") {
        let Some(path) = args.get(index + 1) else {
            eprintln!("--save-info requires a save-file path");
            return ExitCode::FAILURE;
        };
        return save_info(Path::new(path));
    }
    if let Some(index) = args.iter().position(|arg| arg == "--replay") {
        let Some(path) = args.get(index + 1) else {
            eprintln!("--replay requires a tape-file path");
            return ExitCode::FAILURE;
        };
        return replay_tape(
            Path::new(path),
            args.iter().any(|arg| arg == "--assert-hash"),
        );
    }

    let seed: u128 = 42;
    let dd = data_dir();
    let config = ValidatedConfig::load_or_default(dd);

    // Headless mode: no window, no audio, pure simulation
    if std::env::var("MC_HEADLESS").is_ok() {
        run_headless(seed, config);
        return ExitCode::SUCCESS;
    }

    // Windowed mode: use macroquad
    run_windowed(seed, config);
    ExitCode::SUCCESS
}

/// Run the windowed application via macroquad.
fn run_windowed(seed: u128, config: ValidatedConfig) {
    macroquad::Window::new("Monte Cristo", async move {
        tracing::info!("starting windowed mode: 256x224 internal resolution");
        let mut app = App::new(seed, config, false);
        let render_target = mc_shell::render::target::ShellRenderTarget::new();
        app.render_target = Some(render_target);

        loop {
            app.windowed_frame();
            macroquad::prelude::next_frame().await;
        }
    });
}
