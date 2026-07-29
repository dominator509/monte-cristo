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

    /// Assert that a named story flag is set at the end of replay.
    /// Exits non-zero if the flag is not found. Flag names are the string
    /// identifiers from flags.ron (e.g., "FLG_ARRESTED").
    #[arg(long)]
    pub require_flag: Option<String>,
}

/// Execute the replay subcommand.
pub fn execute(args: &ReplayArgs) -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read(&args.tape)
        .map_err(|e| format!("failed to read tape file {:?}: {}", args.tape, e))?;
    let tape = mc_tape::format::Tape::from_bytes(&data)
        .map_err(|e| format!("failed to deserialize tape: {}", e))?;

    let result = mc_tape::replay::replay(&tape).map_err(|e| format!("replay failed: {}", e))?;

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

    if let Some(ref flag_name) = args.require_flag {
        let flag_id = parse_flag_id(flag_name)
            .ok_or_else(|| format!("unknown flag name: `{}`", flag_name))?;
        if !result.final_world.flags.is_set(flag_id) {
            eprintln!("flag `{}` is not set at end of replay", flag_name);
            std::process::exit(1);
        }
        println!("flag `{}`: ok", flag_name);
    }

    Ok(())
}

/// Parse a flag name string (e.g. "FLG_ARRESTED") into a FlagId.
/// Supports the "FLG_" prefixed form and the bare name form.
fn parse_flag_id(name: &str) -> Option<mc_core::ids::FlagId> {
    // Strip optional "FLG_" prefix for matching
    let stem = if let Some(s) = name.strip_prefix("FLG_") {
        s
    } else {
        name
    };

    // Map stem to FlagId
    match stem {
        "ARRESTED" => Some(mc_core::ids::FlagId::FLG_ARRESTED),
        "FARIA_MET" => Some(mc_core::ids::FlagId::FLG_FARIA_MET),
        "TREASURE_KNOWN" => Some(mc_core::ids::FlagId::FLG_TREASURE_KNOWN),
        "ESCAPED" => Some(mc_core::ids::FlagId::FLG_ESCAPED),
        "COMTE_IDENTITY" => Some(mc_core::ids::FlagId::FLG_COMTE_IDENTITY),
        "SINDBAD_VISITED" => Some(mc_core::ids::FlagId::FLG_SINDBAD_VISITED),
        "MORCERF_DOSSIER" => Some(mc_core::ids::FlagId::FLG_MORCERF_DOSSIER),
        "MORCERF_YANINA_DOSSIER" => Some(mc_core::ids::FlagId::FLG_MORCERF_YANINA_DOSSIER),
        "MORCERF_ALBERT_WITHDRAWN" => Some(mc_core::ids::FlagId::FLG_MORCERF_ALBERT_WITHDRAWN),
        "DANGLARS_LETTER" => Some(mc_core::ids::FlagId::FLG_DANGLARS_LETTER),
        "VILLEFORT_DOSSIER" => Some(mc_core::ids::FlagId::FLG_VILLEFORT_DOSSIER),
        "HELOISE_POISONING" => Some(mc_core::ids::FlagId::FLG_HELOISE_POISONING),
        "VALENTINE_SAFE" => Some(mc_core::ids::FlagId::FLG_VALENTINE_SAFE),
        "MERCEDES_RECOGNITION" => Some(mc_core::ids::FlagId::FLG_MERCEDES_RECOGNITION),
        "EDOUARD_TRUTH" => Some(mc_core::ids::FlagId::FLG_EDOUARD_TRUTH),
        "FERNAND_CONFRONTED" => Some(mc_core::ids::FlagId::FLG_FERNAND_CONFRONTED),
        "DANGLARS_CONFRONTED" => Some(mc_core::ids::FlagId::FLG_DANGLARS_CONFRONTED),
        "VILLEFORT_CONFRONTED" => Some(mc_core::ids::FlagId::FLG_VILLEFORT_CONFRONTED),
        "MERCEDES_FORGIVEN" => Some(mc_core::ids::FlagId::FLG_MERCEDES_FORGIVEN),
        "FINAL_PHASE1" => Some(mc_core::ids::FlagId::FLG_FINAL_PHASE1),
        "FINAL_PHASE2" => Some(mc_core::ids::FlagId::FLG_FINAL_PHASE2),
        "FINAL_PHASE3" => Some(mc_core::ids::FlagId::FLG_FINAL_PHASE3),
        _ => None,
    }
}

fn hex_fmt(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}
