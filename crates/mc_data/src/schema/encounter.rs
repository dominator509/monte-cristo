//! Schema types for hand-placed encounter definitions.
//!
//! An encounter specifies one or more enemy groups that appear at a
//! specific location in the overworld. Unlike procedural spawn tables,
//! hand-placed encounters have fixed compositions and appear at
//! deterministic locations.

use mc_core::flags::FlagExpr;
use serde::{Deserialize, Serialize};

/// A hand-placed encounter definition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encounter {
    /// Unique encounter identifier (e.g. "R01-s1-e001").
    pub id: String,
    /// Region where this encounter occurs.
    pub region: String,
    /// Chapter stage when this encounter is active.
    pub chapter_stage: u32,
    /// Pool index for disambiguation within the same region/stage.
    pub pool: u32,
    /// Enemy IDs that participate in this encounter.
    pub enemies: Vec<String>,
    /// Minimum number of enemies that appear (1-3).
    pub min_count: u32,
    /// Maximum number of enemies that appear (1-6).
    pub max_count: u32,
    /// Relative weight for determining how likely this encounter is selected.
    pub weight: u32,
    /// Flag condition required for this encounter to be available.
    #[serde(default)]
    pub gate: FlagExpr,
    /// Optional narrative trigger scene ID.
    #[serde(default)]
    pub trigger_scene: Option<String>,
}
