//! Monte Cristo developer and CI CLI.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Validate { input } => {
            let errors = mc_data::bake::bake(&input);
            match errors {
                Ok(()) => {
                    println!("content: ok");
                }
                Err(errs) => {
                    for err in &errs {
                        eprintln!("{}", err);
                    }
                    eprintln!("content: {} error(s) found", errs.len());
                    std::process::exit(1);
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
                std::process::exit(1);
            }
            let pack = mc_data::pack::Pack::from_content(&input)
                .expect("bake validation passed but Pack::from_content failed");
            if let Err(e) = pack.save(&output) {
                eprintln!("bake: failed to write pack: {}", e);
                std::process::exit(1);
            }
            println!("bake: ok -> {}", output.display());
        }
        Command::Replay(args) => {
            if let Err(e) = cmd_replay::execute(&args) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Command::Record(args) => {
            if let Err(e) = cmd_record::execute(&args) {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Command::Prove(args) => {
            let exit_code = cmd_prove::execute(&args);
            std::process::exit(match exit_code {
                std::process::ExitCode::SUCCESS => 0,
                _ => 1,
            });
        }
        Command::Report(args) => {
            if !cmd_report::execute(&args) {
                std::process::exit(1);
            }
        }
    }
}
