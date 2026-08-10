//! Content-addressed pack system.
//!
//! A `Pack` bundles all content data into a single canonical binary
//! addressed by its BLAKE3 digest.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::error::{ContentError, SaveError};
use crate::schema::encounter::Encounter;
use crate::schema::enemy::Enemy;
use crate::schema::item::Item;
use crate::schema::region::Region;
use crate::schema::scene::Scene;
use crate::schema::spawn_table::SpawnTable;
use mc_core::ids::{CharId, FlagId, ItemId};
use mc_core::item::{AuthoredItemCatalog, AuthoredItemDefinition, ItemKind};
use mc_core::scene::{
    AuthoredChoiceDefinition, AuthoredNodeDefinition, AuthoredSceneCatalog,
    AuthoredSceneDefinition, SceneEffect,
};
use serde::{Deserialize, Serialize};

/// A content-addressed pack containing all authored game data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pack {
    pub enemies: Vec<Enemy>,
    pub encounters: Vec<Encounter>,
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
        let scenes_dir = root.join("scenes");
        let spawn_dir = root.join("spawn_tables");
        let items_dir = root.join("items");
        let encounter_dir = root.join("encounters");
        let flags_file = root.join("flags.ron");
        let strings_dir = root.join("strings").join("en");

        let enemies = load_ron_dir::<Enemy>(&bestiary_dir)?;
        let regions = load_ron_dir::<Region>(&regions_dir)?;
        let scenes = load_ron_tree::<Scene>(&scenes_dir)?;
        let spawn_tables = load_ron_dir::<SpawnTable>(&spawn_dir)?;
        let items = load_ron_dir::<Item>(&items_dir)?;
        let encounters = if encounter_dir.exists() {
            load_ron_dir::<Encounter>(&encounter_dir)?
        } else {
            Vec::new()
        };

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
            encounters,
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

    /// Convert authored RON scenes into the deterministic core runtime catalog.
    pub fn scene_catalog(&self) -> Result<AuthoredSceneCatalog, ContentError> {
        let mut definitions = Vec::with_capacity(self.scenes.len());
        for scene in &self.scenes {
            let mut nodes = Vec::with_capacity(scene.nodes.len());
            for node in &scene.nodes {
                let mut choices = Vec::with_capacity(node.choices.len());
                for choice in &node.choices {
                    let effects = choice
                        .trust
                        .as_ref()
                        .map(|trust| {
                            trust
                                .iter()
                                .map(|effect| trust_effect(&scene.id, effect))
                                .collect::<Result<Vec<_>, _>>()
                        })
                        .transpose()?
                        .unwrap_or_default();
                    choices.push(AuthoredChoiceDefinition {
                        label: localize(&self.strings, &choice.text_key),
                        to: choice.to.clone(),
                        condition: choice.requires.clone().unwrap_or_default(),
                        effects,
                    });
                }
                nodes.push(AuthoredNodeDefinition {
                    id: node.id.clone(),
                    text_key: localize(&self.strings, &node.text_key),
                    choices,
                });
            }
            definitions.push(AuthoredSceneDefinition {
                id: scene.id.clone(),
                requires: scene.requires.clone(),
                nodes,
                on_exit: scene_effects(&scene.id, scene.on_exit.as_ref())?,
                terminal: scene.terminal,
            });
        }

        AuthoredSceneCatalog::from_definitions(definitions)
            .map_err(|error| ContentError::new(format!("scene catalog: {error}")))
    }

    /// Convert authored item definitions into the deterministic core catalog.
    pub fn item_catalog(&self) -> Result<AuthoredItemCatalog, ContentError> {
        let definitions = self
            .items
            .iter()
            .map(|item| {
                let id = parse_item("item catalog", &item.id)?;
                let kind = match item.item_type {
                    crate::schema::item::ItemType::Consumable => ItemKind::Consumable,
                    crate::schema::item::ItemType::Key => ItemKind::Key,
                    crate::schema::item::ItemType::Quest => ItemKind::Quest,
                    crate::schema::item::ItemType::Equipment => ItemKind::Equipment,
                };
                Ok(AuthoredItemDefinition {
                    id,
                    kind,
                    heal_hp: item.heal_hp,
                })
            })
            .collect::<Result<Vec<_>, ContentError>>()?;
        AuthoredItemCatalog::from_definitions(definitions)
            .map_err(|error| ContentError::new(format!("item catalog: {error}")))
    }
}

fn localize(strings: &[(String, String)], key: &str) -> String {
    strings
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map_or_else(|| key.to_string(), |(_, value)| value.clone())
}

fn scene_effects(
    scene_id: &str,
    effects: Option<&crate::schema::scene::Effects>,
) -> Result<Vec<SceneEffect>, ContentError> {
    let Some(effects) = effects else {
        return Ok(Vec::new());
    };
    let mut resolved = Vec::new();
    for id in &effects.set_flags {
        resolved.push(SceneEffect::SetFlag(parse_flag(scene_id, id)?));
    }
    for id in &effects.clear_flags {
        resolved.push(SceneEffect::ClearFlag(parse_flag(scene_id, id)?));
    }
    for id in &effects.consume {
        resolved.push(SceneEffect::ConsumeItem(parse_item(scene_id, id)?, 1));
    }
    for id in &effects.grant {
        resolved.push(SceneEffect::GrantItem(parse_item(scene_id, id)?, 1));
    }
    if let Some(trust) = &effects.trust {
        for effect in trust {
            resolved.push(trust_effect(scene_id, effect)?);
        }
    }
    if let Some(mask) = effects.mask {
        let value = i16::try_from(mask).map_err(|_| {
            ContentError::in_field(
                format!("mask adjustment {mask} does not fit i16"),
                scene_id,
                "on_exit.mask",
            )
        })?;
        if value >= 0 {
            resolved.push(SceneEffect::AddMask(value));
        } else {
            resolved.push(SceneEffect::SubMask(value.saturating_abs()));
        }
    }
    Ok(resolved)
}

fn trust_effect(
    scene_id: &str,
    effect: &crate::schema::scene::TrustEffect,
) -> Result<SceneEffect, ContentError> {
    let character = parse_char(scene_id, &effect.0)?;
    let value = i16::try_from(effect.1).map_err(|_| {
        ContentError::in_field(
            format!("trust adjustment {} does not fit i16", effect.1),
            scene_id,
            "trust",
        )
    })?;
    if value >= 0 {
        Ok(SceneEffect::AddTrust(character, value))
    } else {
        Ok(SceneEffect::SubTrust(character, value.saturating_abs()))
    }
}

fn parse_flag(scene_id: &str, id: &str) -> Result<FlagId, ContentError> {
    let flag = match id {
        "FLG_ARRESTED" => FlagId::FLG_ARRESTED,
        "FLG_FARIA_MET" => FlagId::FLG_FARIA_MET,
        "FLG_TREASURE_KNOWN" => FlagId::FLG_TREASURE_KNOWN,
        "FLG_ESCAPED" => FlagId::FLG_ESCAPED,
        "FLG_COMTE_IDENTITY" => FlagId::FLG_COMTE_IDENTITY,
        "FLG_SINDBAD_VISITED" => FlagId::FLG_SINDBAD_VISITED,
        "FLG_MORCERF_DOSSIER" => FlagId::FLG_MORCERF_DOSSIER,
        "FLG_MORCERF_YANINA_DOSSIER" => FlagId::FLG_MORCERF_YANINA_DOSSIER,
        "FLG_MORCERF_ALBERT_WITHDRAWN" => FlagId::FLG_MORCERF_ALBERT_WITHDRAWN,
        "FLG_DANGLARS_LETTER" => FlagId::FLG_DANGLARS_LETTER,
        "FLG_VILLEFORT_DOSSIER" => FlagId::FLG_VILLEFORT_DOSSIER,
        "FLG_HELOISE_POISONING" => FlagId::FLG_HELOISE_POISONING,
        "FLG_VALENTINE_SAFE" => FlagId::FLG_VALENTINE_SAFE,
        "FLG_MERCEDES_RECOGNITION" => FlagId::FLG_MERCEDES_RECOGNITION,
        "FLG_EDOUARD_TRUTH" => FlagId::FLG_EDOUARD_TRUTH,
        "FLG_FERNAND_CONFRONTED" => FlagId::FLG_FERNAND_CONFRONTED,
        "FLG_DANGLARS_CONFRONTED" => FlagId::FLG_DANGLARS_CONFRONTED,
        "FLG_VILLEFORT_CONFRONTED" => FlagId::FLG_VILLEFORT_CONFRONTED,
        "FLG_MERCEDES_FORGIVEN" => FlagId::FLG_MERCEDES_FORGIVEN,
        "FLG_FINAL_PHASE1" => FlagId::FLG_FINAL_PHASE1,
        "FLG_FINAL_PHASE2" => FlagId::FLG_FINAL_PHASE2,
        "FLG_FINAL_PHASE3" => FlagId::FLG_FINAL_PHASE3,
        _ => {
            return Err(ContentError::in_field(
                format!("unknown flag {id}"),
                scene_id,
                "flags",
            ))
        }
    };
    Ok(flag)
}

fn parse_item(scene_id: &str, id: &str) -> Result<ItemId, ContentError> {
    let item = match id {
        "ITM_POTION" => ItemId::ITM_POTION,
        "ITM_HI_POTION" => ItemId::ITM_HI_POTION,
        "ITM_ANTIDOTE" => ItemId::ITM_ANTIDOTE,
        "ITM_PANACEA" => ItemId::ITM_PANACEA,
        "ITM_SMOKE_BOMB" => ItemId::ITM_SMOKE_BOMB,
        "ITM_PHIAL_BRUCINE" => ItemId::ITM_PHIAL_BRUCINE,
        "ITM_TREASURE_MAP" => ItemId::ITM_TREASURE_MAP,
        "ITM_EDOUARD_LOCKET" => ItemId::ITM_EDOUARD_LOCKET,
        _ => {
            return Err(ContentError::in_field(
                format!("unknown item {id}"),
                scene_id,
                "items",
            ))
        }
    };
    Ok(item)
}

fn parse_char(scene_id: &str, id: &str) -> Result<CharId, ContentError> {
    let character = match id {
        "CHR_EDMOND" => CharId::CHR_EDMOND,
        "CHR_ABBE_FARIA" => CharId::CHR_ABBE_FARIA,
        "CHR_HAYDEE" => CharId::CHR_HAYDEE,
        "CHR_MERCEDES" => CharId::CHR_MERCEDES,
        "CHR_ALBERT" => CharId::CHR_ALBERT,
        "CHR_FERNAND" => CharId::CHR_FERNAND,
        "CHR_DANGLARS" => CharId::CHR_DANGLARS,
        "CHR_VILLEFORT" => CharId::CHR_VILLEFORT,
        "CHR_VALENTINE" => CharId::CHR_VALENTINE,
        "CHR_NOIRTIER" => CharId::CHR_NOIRTIER,
        "CHR_BERTUCCIO" => CharId::CHR_BERTUCCIO,
        "CHR_HELOISE" => CharId::CHR_HELOISE,
        _ => {
            return Err(ContentError::in_field(
                format!("unknown character {id}"),
                scene_id,
                "trust",
            ))
        }
    };
    Ok(character)
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

/// Load all RON files below a directory in canonical path order.
///
/// Scene content is partitioned by act subdirectories, so a flat directory
/// loader would silently omit authored scenes after Act I.
fn load_ron_tree<T: serde::de::DeserializeOwned>(dir: &Path) -> Result<Vec<T>, ContentError> {
    let mut paths = Vec::new();
    collect_ron_paths(dir, &mut paths)?;
    paths.sort();

    paths
        .into_iter()
        .map(|path| {
            let data = fs::read_to_string(&path)
                .map_err(|e| ContentError::new(format!("cannot read {}: {e}", path.display())))?;
            ron::from_str(&data)
                .map_err(|e| ContentError::new(format!("cannot parse {}: {e}", path.display())))
        })
        .collect()
}

fn collect_ron_paths(dir: &Path, paths: &mut Vec<std::path::PathBuf>) -> Result<(), ContentError> {
    if !dir.exists() {
        return Ok(());
    }
    let entries =
        fs::read_dir(dir).map_err(|e| ContentError::new(format!("cannot read {dir:?}: {e}")))?;
    for entry in entries {
        let path = entry
            .map_err(|e| ContentError::new(format!("cannot read entry in {dir:?}: {e}")))?
            .path();
        if path.is_dir() {
            collect_ron_paths(&path, paths)?;
        } else if path.extension().map_or(false, |e| e == "ron") {
            paths.push(path);
        }
    }
    Ok(())
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
