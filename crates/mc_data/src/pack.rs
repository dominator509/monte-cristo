//! Content-addressed pack system.
//!
//! A `Pack` bundles all content data into a single canonical binary
//! addressed by its BLAKE3 digest.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::error::{ContentError, SaveError};
use crate::schema::enemy::Enemy;
use crate::schema::item::Item;
use crate::schema::region::Region;
use crate::schema::scene::Scene;
use crate::schema::spawn_table::SpawnTable;
use serde::{Deserialize, Serialize};

/// A content-addressed pack containing all authored game data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pack {
    pub enemies: Vec<Enemy>,
    pub regions: Vec<Region>,
    pub scenes: Vec<Scene>,
    pub spawn_tables: Vec<SpawnTable>,
    pub items: Vec<Item>,
    pub flags: Vec<String>,
    /// Localised string key-value pairs.
    pub strings: Vec<(String, String)>,
}

impl Pack {
    /// Build a Pack from content directory.
    pub fn from_content(root: &Path) -> Result<Pack, ContentError> {
        let bestiary_dir = root.join("bestiary");
        let regions_dir = root.join("regions");
        let scenes_dir = root.join("scenes").join("act1");
        let spawn_dir = root.join("spawn_tables");
        let items_dir = root.join("items");
        let flags_file = root.join("flags.ron");
        let strings_dir = root.join("strings").join("en");

        let enemies = load_ron_dir::<Enemy>(&bestiary_dir)?;
        let regions = load_ron_dir::<Region>(&regions_dir)?;
        let scenes = load_ron_dir::<Scene>(&scenes_dir)?;
        let spawn_tables = load_ron_dir::<SpawnTable>(&spawn_dir)?;
        let items = load_ron_dir::<Item>(&items_dir)?;

        let flags = if flags_file.exists() {
            let data = fs::read_to_string(&flags_file)
                .map_err(|e| ContentError::new(format!("cannot read {flags_file:?}: {e}")))?;
            ron::from_str(&data)
                .map_err(|e| ContentError::new(format!("cannot parse {flags_file:?}: {e}")))?
        } else {
            Vec::new()
        };

        let strings = if strings_dir.exists() {
            load_ron_dir::<(String, String)>(&strings_dir)?
        } else {
            Vec::new()
        };

        Ok(Pack {
            enemies,
            regions,
            scenes,
            spawn_tables,
            items,
            flags,
            strings,
        })
    }

    /// Canonical binary encoding.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SaveError> {
        postcard::to_stdvec(self)
            .map_err(|e| SaveError::Deserialize(format!("serialization failed: {e}")))
    }

    /// BLAKE3 digest of the canonical encoding.
    pub fn digest(&self) -> Result<blake3::Hash, SaveError> {
        Ok(blake3::hash(&self.to_bytes()?))
    }

    /// Save pack and digest to disk.
    pub fn save(&self, output: &Path) -> Result<(), SaveError> {
        let bytes = self.to_bytes()?;
        let hash = blake3::hash(&bytes);

        fs::write(output, &bytes)?;

        let digest_path = output.with_extension("pack.blake3");
        fs::write(&digest_path, format!("{}\n", hash.to_hex()))?;

        Ok(())
    }

    /// Load pack from disk, verifying digest.
    pub fn load(pack_path: &Path, digest_path: &Path) -> Result<Pack, SaveError> {
        let expected_hex = fs::read_to_string(digest_path).map_err(SaveError::Io)?;
        let expected_hex = expected_hex.trim().to_string();

        let bytes = fs::read(pack_path).map_err(SaveError::Io)?;

        let actual_hash = blake3::hash(&bytes);
        let actual_hex = actual_hash.to_hex().to_string();
        if actual_hex != expected_hex {
            return Err(SaveError::DigestMismatch {
                expected: expected_hex,
                actual: actual_hex,
            });
        }

        postcard::from_bytes(&bytes).map_err(|e| SaveError::Deserialize(e.to_string()))
    }

    /// Load from directory containing content.pack and content.pack.blake3.
    pub fn load_from_dir(dir: &Path) -> Result<Pack, SaveError> {
        let pack_path = dir.join("content.pack");
        let digest_path = dir.join("content.pack.blake3");
        Pack::load(&pack_path, &digest_path)
    }

    /// Load from raw bytes (fuzz target). No digest verification.
    pub fn load_from_bytes(data: &[u8]) -> Result<Pack, SaveError> {
        postcard::from_bytes(data).map_err(|e| SaveError::Deserialize(e.to_string()))
    }

    /// Get the number of entries in each collection.
    pub fn counts(&self) -> PackCounts {
        PackCounts {
            enemies: self.enemies.len(),
            regions: self.regions.len(),
            scenes: self.scenes.len(),
            spawn_tables: self.spawn_tables.len(),
            items: self.items.len(),
            flags: self.flags.len(),
        }
    }
}

/// Human-readable summary of pack contents.
#[derive(Debug, Clone, Copy)]
pub struct PackCounts {
    pub enemies: usize,
    pub regions: usize,
    pub scenes: usize,
    pub spawn_tables: usize,
    pub items: usize,
    pub flags: usize,
}

/// Load all .ron files from a directory into a Vec<T>.
fn load_ron_dir<T: serde::de::DeserializeOwned>(dir: &Path) -> Result<Vec<T>, ContentError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| ContentError::new(format!("cannot read {dir:?}: {e}")))?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "ron") {
            let data = fs::read_to_string(&path)
                .map_err(|e| ContentError::new(format!("cannot read {}: {e}", path.display())))?;
            let item: T = ron::from_str(&data)
                .map_err(|e| ContentError::new(format!("cannot parse {}: {e}", path.display())))?;
            items.push(item);
        }
    }
    Ok(items)
}

/// Verify all cross-file references within a pack.
pub fn verify_references(pack: &Pack) -> Vec<String> {
    let mut errors = Vec::new();

    let enemy_ids: HashSet<&str> = pack.enemies.iter().map(|e| e.id.as_str()).collect();
    let region_ids: HashSet<&str> = pack.regions.iter().map(|r| r.id.as_str()).collect();
    let _item_ids: HashSet<&str> = pack.items.iter().map(|i| i.id.as_str()).collect();

    // Spawn table references
    for table in &pack.spawn_tables {
        for entry in &table.entries {
            if !enemy_ids.contains(entry.enemy.as_str()) {
                errors.push(format!(
                    "spawn table {} references enemy {} not in bestiary",
                    table.region, entry.enemy
                ));
            }
        }
    }

    // Region connections
    for region in &pack.regions {
        for conn in &region.connections {
            if !region_ids.contains(conn.as_str()) {
                errors.push(format!(
                    "region {} connects to {} not in regions",
                    region.id, conn
                ));
            }
        }
    }

    errors
}
