//! Monte Cristo presentation shell entry point.
//!
//! SPEC-004 sections 1, 10, 11. MC_HEADLESS=1 suppresses window + audio.
//! The binary is named "monte-cristo".

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use std::{fs, io::Write};

use mc_core::item::AuthoredItemCatalog;
use mc_core::scene::AuthoredSceneCatalog;
use mc_shell::app::App;
use mc_shell::config::ValidatedConfig;
use mc_shell::fsroot::{self, Root};
use mc_shell::persistence::SlotStore;

/// Get the data directory for settings and saves.
fn data_dir() -> Result<PathBuf, String> {
    let base = std::env::var("MC_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            PathBuf::from(home).join(".local/share/monte-cristo")
        });
    // The confinement helper requires the root itself to exist before it can
    // resolve a child path. Create the user-owned root first and fail startup
    // if that operation is not possible; silently continuing would make saves
    // and settings appear to work while all writes are discarded.
    fs::create_dir_all(&base)
        .map_err(|error| format!("cannot create MC_DATA_DIR root {}: {error}", base.display()))?;
    // Use fsroot for confined directory creation — set MC_DATA_DIR for it.
    std::env::set_var(Root::Data.env_var(), &base);
    fsroot::create_dir_all(Root::Data, Path::new(""))
        .map_err(|error| format!("cannot confine MC_DATA_DIR root: {error}"))?;
    Ok(base)
}

/// Run the headless application (no window, no audio).
fn run_headless(
    seed: u128,
    config: ValidatedConfig,
    scene_catalog: AuthoredSceneCatalog,
    item_catalog: AuthoredItemCatalog,
    slot_store: SlotStore,
) -> Result<(), String> {
    tracing::info!("starting headless mode");
    let startup_started = Instant::now();
    let mut app = App::new_with_catalog_and_items_and_store(
        seed,
        config,
        true,
        scene_catalog,
        item_catalog,
        Some(slot_store),
    )?;
    mc_shell::obs::record_startup_to_title(startup_started.elapsed());
    for _i in 0..60 {
        app.headless_update();
    }
    tracing::info!(
        "headless run complete: tick={}, state_hash={:?}",
        app.world.tick,
        app.world.state_hash()
    );
    if let Err(error) = mc_shell::obs::write_metrics() {
        eprintln!("Metrics write failed: {error}");
    }
    Ok(())
}

/// Initialise local-only observability after the data root has been resolved.
///
/// A logging failure is reported to stderr and falls back to a console
/// subscriber so the game remains playable when the data directory is
/// read-only. Crash reports still use the same confined root when available.
fn init_observability() {
    if let Err(error) = mc_shell::obs::init_logging() {
        eprintln!("Logging initialization failed: {error}");
        let _ = tracing_subscriber::fmt().with_target(false).try_init();
    }
    mc_shell::obs::install_crash_hook();
}

/// Load the authored catalog for the real game loop.
///
/// Release artifacts contain a verified `content.pack`; debug source checkouts
/// may still run directly from the RON tree before a bake has occurred. A
/// release binary refuses that loose-source fallback.
fn load_runtime_catalog() -> Result<(AuthoredSceneCatalog, AuthoredItemCatalog, [u8; 32]), String> {
    let load_started = Instant::now();
    let mut candidates = Vec::new();
    let manifest_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest_root.join("../.."));
    if let Ok(value) = std::env::var(Root::Content.env_var()) {
        if !value.trim().is_empty() {
            candidates.push(PathBuf::from(value));
        }
    }
    if let Ok(current) = std::env::current_dir() {
        candidates.push(current.clone());
        candidates.push(current.join("content"));
    }
    if let Ok(executable) = std::env::current_exe() {
        if let Some(parent) = executable.parent() {
            candidates.push(parent.to_path_buf());
        }
    }

    for candidate in candidates {
        let pack_path = candidate.join("content.pack");
        let source_path = candidate.join("scenes");
        if !pack_path.is_file() && !source_path.is_dir() {
            continue;
        }
        std::env::set_var(Root::Content.env_var(), &candidate);
        let confined = fsroot::confine(Root::Content, Path::new(""))
            .map_err(|error| format!("content root is not available: {error}"))?;
        let pack = if pack_path.is_file() {
            mc_data::pack::Pack::load_from_dir(&confined)
                .map_err(|error| format!("content pack failed integrity verification: {error}"))?
        } else {
            if !cfg!(debug_assertions) {
                continue;
            }
            mc_data::pack::Pack::from_content(&confined)
                .map_err(|error| format!("content source failed validation: {error}"))?
        };
        let content_digest = pack
            .digest()
            .map_err(|error| format!("content digest failed: {error}"))?;
        let scene_catalog = pack
            .scene_catalog()
            .map_err(|error| format!("authored scene catalog failed: {error}"))?;
        let item_catalog = pack
            .item_catalog()
            .map_err(|error| format!("authored item catalog failed: {error}"))?;
        mc_shell::obs::record_content_load(load_started.elapsed());
        return Ok((scene_catalog, item_catalog, *content_digest.as_bytes()));
    }

    Err(
        "No content.pack or authored content tree is available; restore the game content and retry"
            .into(),
    )
}

/// Real main — checks MC_HEADLESS before any window creation.
fn verify_content() -> ExitCode {
    let content_dir = match fsroot::confine(Root::Content, Path::new("")) {
        Ok(path) => path,
        Err(error) => {
            eprintln!(
                "Content verification failed for content.pack: {} is not a valid confined root: {error}",
                Root::Content.env_var()
            );
            return ExitCode::FAILURE;
        }
    };

    match mc_data::pack::Pack::load_from_dir(&content_dir) {
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
    let mut output = String::with_capacity(64);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

fn save_info(path: &Path) -> ExitCode {
    let confined_path = match fsroot::confine(Root::Data, path) {
        Ok(path) => path,
        Err(error) => {
            eprintln!(
                "Save inspection failed for {} within {}: {error}",
                path.display(),
                Root::Data.env_var()
            );
            return ExitCode::FAILURE;
        }
    };

    match mc_data::save::Save::from_file(&confined_path) {
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
    let result = fsroot::read(Root::Data, path)
        .map_err(|error| {
            format!(
                "failed to read tape {} within {}: {error}",
                path.display(),
                Root::Data.env_var()
            )
        })
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

    let (scene_catalog, item_catalog, content_digest) = match load_runtime_catalog() {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("Game startup failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    let seed: u128 = 42;
    let dd = match data_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Game startup failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    init_observability();
    let config = ValidatedConfig::load_or_default(dd.clone());
    let slot_store = SlotStore::new(dd, content_digest);

    // Headless mode: no window, no audio, pure simulation
    if std::env::var("MC_HEADLESS").is_ok() {
        return match run_headless(seed, config, scene_catalog, item_catalog, slot_store) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Headless startup failed: {error}");
                ExitCode::FAILURE
            }
        };
    }

    // Windowed mode: use macroquad
    match run_windowed(seed, config, scene_catalog, item_catalog, slot_store) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Windowed startup failed: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Run the windowed application via macroquad.
fn run_windowed(
    seed: u128,
    config: ValidatedConfig,
    scene_catalog: AuthoredSceneCatalog,
    item_catalog: AuthoredItemCatalog,
    slot_store: SlotStore,
) -> Result<(), String> {
    let mut app = App::new_with_catalog_and_items_and_store(
        seed,
        config,
        false,
        scene_catalog,
        item_catalog,
        Some(slot_store),
    )?;
    macroquad::Window::new("Monte Cristo", async move {
        tracing::info!("starting windowed mode: 256x224 internal resolution");
        let startup_started = Instant::now();
        mc_shell::obs::record_startup_to_title(startup_started.elapsed());
        let render_target = mc_shell::render::target::ShellRenderTarget::new();
        app.render_target = Some(render_target);

        loop {
            app.windowed_frame();
            macroquad::prelude::next_frame().await;
        }
    });
    Ok(())
}
