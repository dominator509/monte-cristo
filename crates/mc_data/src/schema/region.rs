use mc_core::flags::FlagExpr;
use serde::{Deserialize, Serialize};

/// A region (map area) in the game world.
///
/// Regions are zones in the overworld or dungeon maps. Each region has a
/// connect graph of neighbouring regions, a tier indicating relative
/// difficulty/importance, and an optional gate expression that must be
/// satisfied before the region becomes accessible.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    /// Unique identifier for this region.
    pub id: String,
    /// i18n key for the region name.
    pub name_key: String,
    /// i18n key for the region description.
    pub description_key: String,
    /// Tier / difficulty level of this region (higher = harder).
    pub tier: u32,
    /// IDs of neighbouring regions this one connects to.
    pub connections: Vec<String>,
    /// Whether this region is initially locked.
    pub locked: bool,
    /// Flag expression that must be satisfied to enter this region.
    pub gate: FlagExpr,
}
