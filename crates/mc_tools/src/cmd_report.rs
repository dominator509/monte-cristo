//! `mc_tools report` — content statistics and diagnostics.
//!
//! Each sub-verb loads the content pack and prints summary statistics.

use std::path::Path;

#[derive(clap::Subcommand, Debug)]
pub enum ReportCommand {
    /// Print bestiary/content statistics: regions, enemies, items, strings.
    Bestiary,
}

#[derive(clap::Args, Debug)]
pub struct ReportArgs {
    #[command(subcommand)]
    pub command: ReportCommand,
    /// Content directory (default: ./content)
    #[arg(long, default_value = "./content")]
    pub input: String,
}

pub fn execute(args: &ReportArgs) -> bool {
    match &args.command {
        ReportCommand::Bestiary => report_bestiary(&args.input),
    }
}

fn report_bestiary(input: &str) -> bool {
    let content_dir = Path::new(input);
    if !content_dir.exists() {
        eprintln!("report bestiary: FAIL - content directory not found: {input}");
        return false;
    }

    let pack = match mc_data::pack::Pack::from_content(content_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("report bestiary: FAIL - could not load content pack: {e}");
            return false;
        }
    };

    // Count enemies by family
    use std::collections::HashMap;
    let mut family_counts: HashMap<String, usize> = HashMap::new();
    for enemy in &pack.enemies {
        let family = format!("{:?}", enemy.family);
        *family_counts.entry(family).or_insert(0) += 1;
    }

    // Count enemies by tier
    let mut tier_counts: HashMap<u32, usize> = HashMap::new();
    for enemy in &pack.enemies {
        *tier_counts.entry(enemy.tier).or_insert(0) += 1;
    }

    // Count items by category (from item id prefix)
    let mut item_categories: HashMap<String, usize> = HashMap::new();
    for item in &pack.items {
        let prefix = item.id.split('_').next().unwrap_or("unknown").to_string();
        *item_categories.entry(prefix).or_insert(0) += 1;
    }

    println!("=== Bestiary Report ===");
    println!("Regions:      {}", pack.regions.len());
    println!("Enemies:      {}", pack.enemies.len());
    println!("Encounters:   {}", pack.encounters.len());
    println!("Spawn Tables: {}", pack.spawn_tables.len());
    println!("Items:        {}", pack.items.len());
    println!("Flags:        {}", pack.flags.len());
    println!("Strings:      {}", pack.strings.len());

    // SPEC-002 names these authoring domains explicitly.  Pack currently
    // carries only the schemas it can load, so report their presence here
    // instead of allowing a missing directory to look like an empty, valid
    // release surface.
    let required_domains = [
        "maps",
        "bestiary",
        "encounters",
        "spawn_tables",
        "items",
        "abilities",
        "techs",
        "scenes",
        "strings/en",
        "party.ron",
        "curriculum.ron",
        "poisons.ron",
    ];
    let missing_domains: Vec<_> = required_domains
        .iter()
        .copied()
        .filter(|domain| !content_dir.join(domain).exists())
        .collect();
    println!();
    println!("--- Required Content Domains (SPEC-002) ---");
    if missing_domains.is_empty() {
        println!("  all required domains present");
    } else {
        for domain in &missing_domains {
            println!("  MISSING {domain}");
        }
        println!(
            "  report bestiary: warning - {} required domain(s) missing; this is not ship-ready content",
            missing_domains.len()
        );
    }

    let expected_counts = [
        ("regions", 15usize, pack.regions.len()),
        ("enemies", 102usize, pack.enemies.len()),
        ("encounters", 180usize, pack.encounters.len()),
        ("spawn_tables", 45usize, pack.spawn_tables.len()),
    ];
    let counts_match = expected_counts
        .iter()
        .all(|(_, expected, actual)| expected == actual);
    println!();
    println!("--- Locked Count Checks (SPEC-002 / SPEC-009) ---");
    for (label, expected, actual) in expected_counts {
        if expected == actual {
            println!("  {label}: {actual} (expected)");
        } else {
            println!("  MISMATCH {label}: {actual} (expected {expected})");
        }
    }
    println!();
    println!("--- Enemies by Family ---");
    let mut fams: Vec<_> = family_counts.into_iter().collect();
    fams.sort_by(|a, b| b.1.cmp(&a.1));
    for (family, count) in &fams {
        println!("  {family:30} {count:3}");
    }
    println!();
    println!("--- Enemies by Tier ---");
    let mut tiers: Vec<_> = tier_counts.into_iter().collect();
    tiers.sort();
    for (tier, count) in &tiers {
        println!("  Tier {tier}: {count}");
    }
    println!();
    println!("--- Items by Category ---");
    let mut cats: Vec<_> = item_categories.into_iter().collect();
    cats.sort();
    for (cat, count) in &cats {
        println!("  {cat:20} {count:3}");
    }

    println!();
    if missing_domains.is_empty() && counts_match {
        println!("report bestiary: ok");
        true
    } else {
        println!("report bestiary: FAIL - locked content requirements are not met");
        false
    }
}
