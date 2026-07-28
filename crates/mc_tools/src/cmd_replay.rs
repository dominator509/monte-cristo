//! `mc_tools replay` — replay a tape and verify its hash.

use std::path::PathBuf;

/// Arguments for the replay subcommand.
#[derive(clap::Args, Debug)]
pub struct ReplayArgs {
    /// Path to the tape file to replay.
    #[arg(long)]
    pub tape: PathBuf,

    /// Print the final state hash as hex.
    #[arg(long)]
    pub print_hash: bool,

    /// Assert that the replayed hash matches the tape's recorded final_hash.
    /// Exits non-zero on divergence.
    #[arg(long)]
    pub assert_hash: bool,
}

/// Execute the replay subcommand.
pub fn execute(args: &ReplayArgs) -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read(&args.tape)
        .map_err(|e| format!("failed to read tape file {:?}: {}", args.tape, e))?;
    let tape = mc_tape::format::Tape::from_bytes(&data)
        .map_err(|e| format!("failed to deserialize tape: {}", e))?;

    let result = mc_tape::replay::replay(&tape)
        .map_err(|e| format!("replay failed: {}", e))?;

    if args.print_hash {
        println!("{}", hex_fmt(&result.final_hash));
    }

    if args.assert_hash {
        if result.final_hash != tape.final_hash {
            eprintln!(
                "hash mismatch: expected {}, got {}",
                hex_fmt(&tape.final_hash),
                hex_fmt(&result.final_hash)
            );
            if let Some((tick, expected, actual)) = result.first_divergence {
                eprintln!(
                    "first divergence at tick {}: expected {}, got {}",
                    tick,
                    hex_fmt(&expected),
                    hex_fmt(&actual)
                );
            }
            std::process::exit(1);
        }
        println!("hash: ok");
    }

    Ok(())
}

fn hex_fmt(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}
