//! Monte Cristo developer and CI CLI.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
            println!("bake: ok");
        }
    }
}
