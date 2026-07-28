//! Schema types for spawn table definitions loaded from content files.
//!
//! A spawn table groups enemies by region and chapter stage into a weighted pool,
//! with flag-based gates controlling individual enemy eligibility.

use mc_core::flags::FlagExpr;
use serde::{Deserialize, Serialize};

/// A single entry in a spawn table pool.
///
/// Each entry has an enemy ID, a relative weight (controlling spawn probability),
/// and an optional gate expression that must be satisfied for this entry to be
/// eligible.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnEntry {
    /// Enemy ID (matches `mc_data::schema::enemy::Enemy::id`).
    pub enemy: String,
    /// Relative spawn weight. Higher values make this entry more likely.
    pub weight: u32,
    /// Flag condition that must be satisfied for this entry to spawn.
    #[serde(default)]
    pub gate: FlagExpr,
}

/// A spawn table mapping a region and chapter stage to a weighted enemy pool.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnTable {
    /// Region identifier (e.g. "chateau_dif", "paris_streets").
    pub region: String,
    /// Which chapter/stage of the story this table applies to.
    pub chapter_stage: u32,
    /// Pool identifier — multiple pools can exist per region/stage.
    pub pool: u32,
    /// Weighted enemy entries in this pool.
    pub entries: Vec<SpawnEntry>,
}
