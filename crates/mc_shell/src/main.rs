//! Monte Cristo presentation shell entry point.
//!
//! SPEC-004 sections 1, 10, 11. MC_HEADLESS=1 suppresses window + audio.
//! The binary is named "monte-cristo".

use std::path::PathBuf;

use mc_shell::app::App;
use mc_shell::config::ValidatedConfig;
use mc_shell::render::target::ShellRenderTarget;

/// Get the data directory for settings and saves.
fn data_dir() -> PathBuf {
    let base = std::env::var("MC_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            PathBuf::from(home).join(".local/share/monte-cristo")
        });
    let _ = std::fs::create_dir_all(&base);
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
fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let print_version = args.iter().any(|a| a == "--version" || a == "-V");
    if print_version {
        println!("Monte Cristo v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let seed: u128 = 42;
    let dd = data_dir();
    let config = ValidatedConfig::load_or_default(dd);

    // Headless mode: no window, no audio, pure simulation
    if std::env::var("MC_HEADLESS").is_ok() {
        run_headless(seed, config);
        return;
    }

    // Windowed mode: use macroquad
    run_windowed(seed, config);
}

/// Run the windowed application via macroquad.
fn run_windowed(seed: u128, config: ValidatedConfig) {
    macroquad::Window::new(
        "Monte Cristo",
        async move {
            tracing::info!("starting windowed mode: 256x224 internal resolution");
            let mut app = App::new(seed, config, false);
            let render_target = mc_shell::render::target::ShellRenderTarget::new();
            app.render_target = Some(render_target);

            loop {
                app.windowed_frame();
                macroquad::prelude::next_frame().await;
            }
        },
    );
}
