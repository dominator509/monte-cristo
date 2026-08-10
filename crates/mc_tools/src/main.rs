//! Monte Cristo developer and CI CLI.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

mod cmd_prove;
mod cmd_record;
mod cmd_replay;
mod cmd_report;

#[derive(Parser)]
#[command(name = "mc_tools", about = "Monte Cristo developer and CI tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate content files
    Validate {
        /// Content directory (default: ./content)
        #[arg(long, default_value = "./content")]
        input: PathBuf,
    },
    /// Bake content into a content-addressed pack
    Bake {
        /// Content directory (default: ./content)
        #[arg(long, default_value = "./content")]
        input: PathBuf,
        /// Output pack file (default: content.pack)
        #[arg(long, default_value = "content.pack")]
        output: PathBuf,
    },
    /// Replay a tape and verify its hash
    Replay(cmd_replay::ReplayArgs),
    /// Record a tape from commands
    Record(cmd_record::RecordArgs),
    /// Run live-fire proofs for the ship gate
    Prove(cmd_prove::ProveArgs),
    /// Print content and diagnostic reports
    Report(cmd_report::ReportArgs),
    /// Validate or perform save migration for every fixture in a directory
    SaveMigrate {
        /// Directory containing historical save fixtures
        #[arg(long)]
        dir: PathBuf,
        /// Validate every migration in memory without modifying any fixture
        #[arg(long)]
        dry_run: bool,
    },
}

fn save_migrate(dir: &std::path::Path, dry_run: bool) -> Result<usize, String> {
    let entries = fs::read_dir(dir).map_err(|error| {
        format!(
            "cannot read save fixture directory {}: {error}",
            dir.display()
        )
    })?;
    let mut paths = entries
        .map(|entry| entry.map(|item| item.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot enumerate save fixtures: {error}"))?;
    paths.sort();
    paths.retain(|path| path.is_file());
    if paths.is_empty() {
        return Err(format!(
            "save fixture directory {} contains no files",
            dir.display()
        ));
    }

    for path in &paths {
        if dry_run {
            let data = fs::read(path)
                .map_err(|error| format!("cannot read save fixture {}: {error}", path.display()))?;
            let migrated = mc_data::migrate::migrate_save(&data)
                .map_err(|error| format!("cannot migrate {}: {error}", path.display()))?;
            mc_data::save::Save::load(&migrated)
                .map_err(|error| format!("migrated save {} is invalid: {error}", path.display()))?;
            println!("save-migrate: {}: ok (dry-run)", path.display());
        } else {
            mc_data::migrate::migrate_save_file(path)
                .map_err(|error| format!("cannot migrate {}: {error}", path.display()))?;
            println!("save-migrate: {}: ok", path.display());
        }
    }

    Ok(paths.len())
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Validate { input } => {
            let errors = mc_data::bake::bake(&input);
            match errors {
                Ok(()) => {
                    // The schema validators cover the directory tree, while
                    // Pack::from_content covers root-level locked tables such
                    // as party.ron, curriculum.ron, and poisons.ron. Run both
                    // before reporting the authoritative content sentinel.
                    match mc_data::pack::Pack::from_content(&input) {
                        Ok(_) => {
                            println!("content: ok");
                            ExitCode::SUCCESS
                        }
                        Err(error) => {
                            eprintln!("content: pack load failed: {error}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(errs) => {
                    for err in &errs {
                        eprintln!("{}", err);
                    }
                    eprintln!("content: {} error(s) found", errs.len());
                    ExitCode::FAILURE
                }
            }
        }
        Command::Bake { input, output } => {
            let errors = mc_data::bake::bake(&input);
            if let Err(errs) = errors {
                for err in &errs {
                    eprintln!("{}", err);
                }
                eprintln!("bake failed: {} error(s)", errs.len());
                return ExitCode::FAILURE;
            }
            let pack = match mc_data::pack::Pack::from_content(&input) {
                Ok(pack) => pack,
                Err(error) => {
                    eprintln!("bake: failed to load validated content: {error}");
                    return ExitCode::FAILURE;
                }
            };
            if let Err(e) = pack.save(&output) {
                eprintln!("bake: failed to write pack: {}", e);
                return ExitCode::FAILURE;
            }
            println!("bake: ok -> {}", output.display());
            ExitCode::SUCCESS
        }
        Command::Replay(args) => match cmd_replay::execute(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {}", e);
                ExitCode::FAILURE
            }
        },
        Command::Record(args) => match cmd_record::execute(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {}", e);
                ExitCode::FAILURE
            }
        },
        Command::Prove(args) => cmd_prove::execute(&args),
        Command::Report(args) => {
            if cmd_report::execute(&args) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Command::SaveMigrate { dir, dry_run } => match save_migrate(&dir, dry_run) {
            Ok(count) => {
                let mode = if dry_run { "dry-run" } else { "migrated" };
                println!("save-migrate: ok ({count} fixture(s), {mode})");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("save-migrate: FAIL - {error}");
                ExitCode::FAILURE
            }
        },
    }
}
