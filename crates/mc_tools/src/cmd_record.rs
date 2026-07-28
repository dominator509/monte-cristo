//! `mc_tools record` — record commands into a tape.

use std::path::PathBuf;

/// Arguments for the record subcommand.
#[derive(clap::Args, Debug)]
pub struct RecordArgs {
    /// Path where the recorded tape will be written.
    #[arg(long)]
    pub out: PathBuf,

    /// Random seed for the world (default: 42).
    #[arg(long, default_value_t = 42)]
    pub seed: u128,

    /// Path to a file containing commands (one per line).
    /// If omitted, reads from stdin.
    #[arg(long)]
    pub commands: Option<PathBuf>,
}

/// Execute the record subcommand.
pub fn execute(args: &RecordArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Read command lines from the specified file or stdin.
    let input: Box<dyn std::io::Read> = if let Some(ref path) = args.commands {
        Box::new(
            std::fs::File::open(path)
                .map_err(|e| format!("failed to open commands file {:?}: {}", path, e))?,
        )
    } else {
        Box::new(std::io::stdin())
    };

    use std::io::BufRead;
    let reader = std::io::BufReader::new(input);

    let world = mc_core::world::World::new(args.seed);
    let mut recorder = mc_tape::record::RecordTape::new(world);

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let cmd = parse_command(trimmed)?;
        recorder
            .record_command(cmd)
            .map_err(|e| format!("failed to record command '{}': {}", trimmed, e))?;
    }

    let tape = recorder
        .finalize()
        .map_err(|e| format!("failed to finalize tape: {}", e))?;
    let bytes = tape
        .to_bytes()
        .map_err(|e| format!("failed to serialize tape: {}", e))?;

    std::fs::write(&args.out, &bytes)
        .map_err(|e| format!("failed to write tape to {:?}: {}", args.out, e))?;

    println!("tape written to {:?} ({} entries, {} bytes)", args.out, tape.len(), bytes.len());
    Ok(())
}

/// Parse a single command from a string like "Interact" or "Move North".
fn parse_command(s: &str) -> Result<mc_core::command::Command, String> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return Err("empty command".into());
    }
    match parts[0] {
        "Interact" => Ok(mc_core::command::Command::Interact),
        "OpenMenu" => Ok(mc_core::command::Command::OpenMenu),
        "CloseMenu" => Ok(mc_core::command::Command::CloseMenu),
        "SceneAdvance" => Ok(mc_core::command::Command::SceneAdvance),
        "CancelSelection" => Ok(mc_core::command::Command::CancelSelection),
        "NameYourself" => Ok(mc_core::command::Command::NameYourself),
        "Move" => {
            let dir = match parts.get(1).copied() {
                Some("North") => mc_core::command::Dir::North,
                Some("South") => mc_core::command::Dir::South,
                Some("East") => mc_core::command::Dir::East,
                Some("West") => mc_core::command::Dir::West,
                other => return Err(format!("invalid direction: {:?}", other)),
            };
            Ok(mc_core::command::Command::Move(dir))
        }
        _ => Err(format!("unknown command: {}", parts[0])),
    }
}
