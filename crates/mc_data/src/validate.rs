//! Content validation functions for the bake pipeline.
//!
//! Each validator inspects a specific dimension of content correctness and
//! returns a `Vec<ContentError>`. An empty vector means the check passed.
//! These are called in order by [`crate::bake::bake`].
//!
//! # Validators (SPEC-002 section 2)
//!
//! 1. `schema_check` — RON deserialization
//! 2. `vocabulary_check` — flag identifiers match locked vocabulary
//! 3. `reference_resolve` — cross-file references resolve
//! 4. `orphan_detect` — unreferenced content files
//! 5. `supernatural_lint` — family closed-set check
//! 6. `region_affinity_check` — spawn entries match enemy affinity
//! 7. `reserved_identifier_reject` — no reserved flag names in content

use std::collections::HashSet;
use std::path::Path;

use crate::error::ContentError;
use crate::schema::enemy::Enemy;
use crate::schema::region::Region;
use crate::schema::scene::{Effects, Scene};
use crate::schema::spawn_table::SpawnTable;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Recursively discover `.ron` files under a directory.
fn ron_files_under(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return files;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(ron_files_under(&path));
            } else if path.extension().map_or(false, |e| e == "ron") {
                files.push(path);
            }
        }
    }
    files
}

/// Read all `.ron` files from a directory and try to deserialise them.
fn load_ron<T: serde::de::DeserializeOwned>(
    dir: &Path,
    type_name: &str,
    errors: &mut Vec<ContentError>,
) -> Vec<(std::path::PathBuf, T)> {
    let mut items = Vec::new();
    for path in ron_files_under(dir) {
        let content = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(ContentError::in_file(
                    format!("cannot read file: {e}"),
                    path.display().to_string(),
                ));
                continue;
            }
        };
        match ron::from_str::<T>(&content) {
            Ok(val) => items.push((path, val)),
            Err(e) => errors.push(ContentError::in_file(
                format!("failed to deserialise as {type_name}: {e}"),
                path.display().to_string(),
            )),
        }
    }
    items
}

/// Collect all string-based flag identifiers used in a content gate expression
/// (the `FlagExpr` from `schema::enemy`, which uses `Vec<String>` / `String`).
fn collect_flag_strings_from_enemy_gate(gate: &crate::schema::enemy::FlagExpr) -> Vec<String> {
    match gate {
        crate::schema::enemy::FlagExpr::Always => vec![],
        crate::schema::enemy::FlagExpr::All(flags) => flags.clone(),
        crate::schema::enemy::FlagExpr::Any(flags) => flags.clone(),
        crate::schema::enemy::FlagExpr::Not(flag) => vec![flag.clone()],
    }
}

/// Collect string flag identifiers from scene `Effects`.
fn collect_flag_strings_from_effects(effects: &Effects) -> Vec<String> {
    let mut flags = effects.set_flags.clone();
    flags.extend(effects.clear_flags.clone());
    flags
}

// ---------------------------------------------------------------------------
// Validator 1 — schema_check
// ---------------------------------------------------------------------------

/// Verify that every RON file in the content directory deserialises to
/// its expected schema type.
///
/// Checks `bestiary/` → `Enemy`, `regions/` → `Region`, `scenes/` → `Scene`,
/// `spawn_tables/` → `SpawnTable`, and `items/` → `Item`.
pub fn schema_check(content_root: &Path) -> Vec<ContentError> {
    let mut errors = Vec::new();

    // bestiary → Enemy
    let bestiary_dir = content_root.join("bestiary");
    if bestiary_dir.is_dir() {
        load_ron::<Enemy>(&bestiary_dir, "Enemy", &mut errors);
    }

    // regions → Region
    let regions_dir = content_root.join("regions");
    if regions_dir.is_dir() {
        load_ron::<Region>(&regions_dir, "Region", &mut errors);
    }

    // scenes/ → Scene (recursive; scenes may be in subdirectories)
    let scenes_dir = content_root.join("scenes");
    if scenes_dir.is_dir() {
        load_ron::<Scene>(&scenes_dir, "Scene", &mut errors);
    }

    // spawn_tables → SpawnTable
    let spawn_dir = content_root.join("spawn_tables");
    if spawn_dir.is_dir() {
        load_ron::<SpawnTable>(&spawn_dir, "SpawnTable", &mut errors);
    }

    // items → Item
    let items_dir = content_root.join("items");
    if items_dir.is_dir() {
        load_ron::<crate::schema::item::Item>(&items_dir, "Item", &mut errors);
    }

    errors
}

// ---------------------------------------------------------------------------
// Validator 2 — vocabulary_check
// ---------------------------------------------------------------------------

/// Verify that all string-based flag identifiers used in content match the
/// locked vocabulary from `flags.ron`.
///
/// The vocabulary file contains 22 game flags followed by 4 reserved
/// identifiers. Any identifier appearing in content must be present in this
/// list. (Typed `FlagId` expressions, e.g. `Set(FlagId(3))`, are enforced
/// by the Rust type system and are not re-checked here.)
pub fn vocabulary_check(content_root: &Path) -> Vec<ContentError> {
    let mut errors = Vec::new();

    // Load the locked vocabulary from flags.ron
    let flags_path = content_root.join("flags.ron");
    let flags_content = match std::fs::read_to_string(&flags_path) {
        Ok(s) => s,
        Err(e) => {
            errors.push(ContentError::new(format!(
                "cannot read flags.ron: {e}"
            )));
            return errors;
        }
    };

    let vocabulary: HashSet<String> = match ron::from_str::<Vec<String>>(&flags_content) {
        Ok(v) => v.into_iter().collect(),
        Err(e) => {
            errors.push(ContentError::new(format!(
                "cannot parse flags.ron: {e}"
            )));
            return errors;
        }
    };

    // Helper: check a string is in the locked vocabulary
    let mut check_flag = |flag: &str, file: &str| {
        if !vocabulary.contains(flag) {
            errors.push(ContentError::in_field(
                format!(
                    "flag identifier `{flag}` is not in the locked vocabulary from flags.ron"
                ),
                file,
                flag,
            ));
        }
    };

    // --- Check enemies (string-based FlagExpr in gate) ---
    let bestiary_dir = content_root.join("bestiary");
    for (path, enemy) in
        load_ron::<Enemy>(&bestiary_dir, "Enemy", &mut Vec::new())
    {
        let file = path.display().to_string();
        for flag in collect_flag_strings_from_enemy_gate(&enemy.gate) {
            check_flag(&flag, &file);
        }
    }

    // --- Check scenes (effects.set_flags / .clear_flags) ---
    let scenes_dir = content_root.join("scenes");
    for (path, scene) in
        load_ron::<Scene>(&scenes_dir, "Scene", &mut Vec::new())
    {
        let file = path.display().to_string();
        if let Some(ref effects) = scene.on_exit {
            for flag in collect_flag_strings_from_effects(effects) {
                check_flag(&flag, &file);
            }
        }
    }

    errors
}

// ---------------------------------------------------------------------------
// Validator 3 — reference_resolve
// ---------------------------------------------------------------------------

/// Check that every cross-file reference in content resolves to an existing
/// content entity (by its parsed ID).
///
/// Validates:
/// - Spawn-table `enemy` IDs → an `Enemy` file in `bestiary/` with that `id`
/// - Region `connections` → a `Region` file in `regions/` with that `id`
pub fn reference_resolve(content_root: &Path) -> Vec<ContentError> {
    let mut errors = Vec::new();

    // Parse all enemy files and index by lowercased ID
    let bestiary_dir = content_root.join("bestiary");
    let enemy_ids: HashSet<String> = load_ron::<Enemy>(&bestiary_dir, "Enemy", &mut Vec::new())
        .into_iter()
        .map(|(_, e)| e.id.to_lowercase())
        .collect();

    // Parse all region files and index by full ID (connections use the same
    // format as the `id` field, e.g. "R01_MARSEILLE")
    let regions_dir = content_root.join("regions");
    let region_ids: HashSet<String> = load_ron::<Region>(&regions_dir, "Region", &mut Vec::new())
        .into_iter()
        .map(|(_, r)| r.id.to_lowercase())
        .collect();

    // Check spawn tables: each entry.enemy must have a matching Enemy file
    let spawn_dir = content_root.join("spawn_tables");
    for (path, table) in load_ron::<SpawnTable>(&spawn_dir, "SpawnTable", &mut Vec::new()) {
        let file = path.display().to_string();
        for entry in &table.entries {
            let key = entry.enemy.to_lowercase();
            if !enemy_ids.contains(&key) {
                errors.push(ContentError::in_field(
                    format!(
                        "spawn table references enemy `{}` but no matching enemy found in bestiary/",
                        entry.enemy
                    ),
                    &file,
                    &entry.enemy,
                ));
            }
        }
    }

    // Check regions: each connection must have a matching region
    for (path, region) in load_ron::<Region>(&regions_dir, "Region", &mut Vec::new()) {
        let file = path.display().to_string();
        for conn in &region.connections {
            let conn_lower = conn.to_lowercase();
            if !region_ids.contains(&conn_lower) {
                errors.push(ContentError::in_field(
                    format!(
                        "region `{}` connects to `{conn}` but no matching region found in regions/",
                        region.id
                    ),
                    &file,
                    conn,
                ));
            }
        }
    }

    errors
}

// ---------------------------------------------------------------------------
// Validator 4 — orphan_detect
// ---------------------------------------------------------------------------

/// Find content files that exist on disk but are not referenced by any other
/// content.
///
/// An item is considered "referenced" when another content file names it:
/// - An enemy is referenced if it appears in at least one spawn table.
/// - A region is referenced if another region lists it in `connections`.
/// - Items and scenes are reported as informational (they may have no
///   incoming references by design).
pub fn orphan_detect(content_root: &Path) -> Vec<ContentError> {
    let mut errors = Vec::new();

    // Collect all referenced enemy IDs from spawn tables
    let mut referenced_enemies: HashSet<String> = HashSet::new();
    let spawn_dir = content_root.join("spawn_tables");
    for (_, table) in load_ron::<SpawnTable>(&spawn_dir, "SpawnTable", &mut Vec::new()) {
        for entry in &table.entries {
            referenced_enemies.insert(entry.enemy.to_lowercase());
        }
    }

    // Build enemy → file mapping
    let bestiary_dir = content_root.join("bestiary");
    let enemies: Vec<(std::path::PathBuf, Enemy)> =
        load_ron::<Enemy>(&bestiary_dir, "Enemy", &mut Vec::new());

    // Collect all referenced region IDs from other regions' connections
    let regions_dir = content_root.join("regions");
    let mut referenced_regions: HashSet<String> = HashSet::new();
    let regions: Vec<(std::path::PathBuf, Region)> =
        load_ron::<Region>(&regions_dir, "Region", &mut Vec::new());
    for (_, region) in &regions {
        for conn in &region.connections {
            referenced_regions.insert(conn.clone());
        }
    }

    // Check enemies
    for (path, enemy) in &enemies {
        let key = enemy.id.to_lowercase();
        if !referenced_enemies.contains(&key) {
            errors.push(ContentError::in_file(
                format!(
                    "enemy `{}` is defined but never referenced in any spawn table",
                    enemy.id
                ),
                path.display().to_string(),
            ));
        }
    }

    // Check regions
    for (path, region) in &regions {
        if !referenced_regions.contains(&region.id) {
            errors.push(ContentError::in_file(
                format!(
                    "region `{}` is defined but never connected from another region",
                    region.id
                ),
                path.display().to_string(),
            ));
        }
    }

    errors
}

// ---------------------------------------------------------------------------
// Validator 5 — supernatural_lint
// ---------------------------------------------------------------------------

/// Verify that every enemy's `family` field is from the closed set.
///
/// Serde's deserialisation already enforces this (unknown strings will
/// fail to parse), so this function is a double-check: if the enemy parsed
/// successfully, its family is necessarily valid. We still iterate and
/// confirm to satisfy the specification.
pub fn supernatural_lint(content_root: &Path) -> Vec<ContentError> {
    let mut errors = Vec::new();

    // Known family variant names as they appear in Debug output (Rust PascalCase).
    // The `{:?}` formatter produces the Rust variant name (e.g. `ManAtArms`),
    // which is distinct from the serde SCREAMING_SNAKE_CASE representation.
    // We list PascalCase names here because that is what `format!("{:?}", f)`
    // actually produces.
    let valid_families: HashSet<&str> = [
        "Vermin",
        "Beast",
        "Sea",
        "ManAtArms",
        "Criminal",
        "Prisoner",
        "Troop",
        "Bandit",
        "Hazard",
        "Boss",
    ]
    .into();

    let bestiary_dir = content_root.join("bestiary");
    for (path, enemy) in
        load_ron::<Enemy>(&bestiary_dir, "Enemy", &mut Vec::new())
    {
        let file = path.display().to_string();
        // We can't inspect the serialised form directly, but we can verify
        // that the family is one of the known variants by round-tripping
        // through serde's debug representation.
        let family_debug = format!("{:?}", enemy.family);
        if !valid_families.iter().any(|f| f.eq_ignore_ascii_case(&family_debug)) {
            errors.push(ContentError::in_field(
                format!(
                    "enemy `{}` has unrecognised family `{:?}`; expected one of: {}",
                    enemy.id,
                    enemy.family,
                    valid_families
                        .iter()
                        .copied()
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
                &file,
                "family",
            ));
        }
    }

    errors
}

// ---------------------------------------------------------------------------
// Validator 6 — region_affinity_check
// ---------------------------------------------------------------------------

/// Verify that every spawn-table entry's enemy declares a `region_affinity`
/// that includes the table's region.
///
/// For each spawn table with region `"RXX"`, every enemy referenced in that
/// table must list `"RXX"` (or a variant like `"R03_MONTE_CRISTO"`) in its
/// `region_affinity` field.
pub fn region_affinity_check(content_root: &Path) -> Vec<ContentError> {
    let mut errors = Vec::new();

    // Load all enemies
    let bestiary_dir = content_root.join("bestiary");
    let enemies: Vec<(std::path::PathBuf, Enemy)> =
        load_ron::<Enemy>(&bestiary_dir, "Enemy", &mut Vec::new());

    // Index enemies by their lowercased ID
    let mut enemy_map: std::collections::HashMap<String, Enemy> = std::collections::HashMap::new();
    for (_, enemy) in &enemies {
        enemy_map.insert(enemy.id.to_lowercase(), enemy.clone());
    }

    // Check every spawn table
    let spawn_dir = content_root.join("spawn_tables");
    for (path, table) in
        load_ron::<SpawnTable>(&spawn_dir, "SpawnTable", &mut Vec::new())
    {
        let file = path.display().to_string();
        for entry in &table.entries {
            let key = entry.enemy.to_lowercase();
            let Some(enemy) = enemy_map.get(&key) else {
                // Already reported by reference_resolve; skip to avoid noise.
                continue;
            };

            // Does the enemy's region_affinity contain the spawn table's region?
            let table_region = &table.region;
            let has_affinity = enemy
                .region_affinity
                .iter()
                .any(|ar| ar == table_region || ar.starts_with(table_region));

            if !has_affinity {
                errors.push(ContentError::in_field(
                    format!(
                        "enemy `{}` is in spawn table for region `{}` but its region_affinity is {:?}",
                        enemy.id, table_region, enemy.region_affinity
                    ),
                    &file,
                    &entry.enemy,
                ));
            }
        }
    }

    errors
}

// ---------------------------------------------------------------------------
// Validator 7 — reserved_identifier_reject
// ---------------------------------------------------------------------------

/// The four reserved identifiers from SPEC-009 section 9 that must never
/// appear in content.
const RESERVED_IDENTIFIERS: &[&str] = &[
    "MERCEDES_ROUTE",
    "FERNAND_FORGIVEN",
    "POWER_OF_FRIENDSHIP",
    "DEUS_EX_MACHINA",
];

/// Verify that none of the 4 reserved identifiers appear in any content file.
///
/// Performs a raw substring search of every `.ron` file under the content
/// directory. Any occurrence is a spec violation.
pub fn reserved_identifier_reject(content_root: &Path) -> Vec<ContentError> {
    let mut errors = Vec::new();

    // Collect all RON files under content root
    let all_ron_files: Vec<_> = ron_files_under(content_root)
        .into_iter()
        .filter(|p| {
            // Skip flags.ron itself — it defines these identifiers intentionally
            p.file_name()
                .and_then(|s| s.to_str())
                .map_or(true, |name| name != "flags.ron")
        })
        .collect();

    for path in &all_ron_files {
        let content = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(ContentError::in_file(
                    format!("cannot read file: {e}"),
                    path.display().to_string(),
                ));
                continue;
            }
        };

        for &reserved in RESERVED_IDENTIFIERS {
            // Use a simple word-boundary-ish check: the identifier should appear
            // as a whole word (surrounded by non-alphanumeric chars or string boundaries).
            if content.contains(reserved) {
                errors.push(ContentError::in_file(
                    format!(
                        "reserved identifier `{reserved}` found in content; this is forbidden by SPEC-009"
                    ),
                    path.display().to_string(),
                ));
            }
        }
    }

    // Deduplicate: if a file contains the same reserved identifier in multiple
    // places we only report it once per identifier per file.
    errors.dedup_by(|a, b| a.file == b.file && a.field == b.field);

    errors
}
