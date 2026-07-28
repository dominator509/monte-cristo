//! Schema types for enemy definitions loaded from content files.
//!
//! INV-11: Enemy eligibility is a pure function of `(region, flags)` with no
//! other inputs — no randomness, no special cases, no exceptions.
//! INV-06: Enemy definitions (region_affinity, gate) live in content, not code.
//! This module defines the *types and mechanism* only.

use mc_core::battle::status::StatusKind;
use mc_core::bestiary::Family;
use serde::{Deserialize, Serialize};

/// A boolean expression over flag names (strings), used in content files.
///
/// Unlike `mc_core::flags::FlagExpr` which uses typed `FlagId`s, this schema
/// version works with human-readable string flag names for serialization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlagExpr {
    /// Always true — no flag restriction.
    Always,
    /// All of the named flags must be set.
    All(Vec<String>),
    /// At least one of the named flags must be set.
    Any(Vec<String>),
    /// The named flag must NOT be set.
    Not(String),
}

/// Core combat statistics for an enemy.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    /// Hit points.
    pub hp: u32,
    /// Attack power.
    pub atk: u32,
    /// Defense rating.
    pub def: u32,
    /// Speed (determines turn order).
    pub spd: u32,
}

/// A loot entry: item ID and drop weight / quantity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootEntry(pub String, pub u32);

/// An enemy definition loaded from content files.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Enemy {
    /// Unique identifier for this enemy (e.g. "rat", "guard_tower_1").
    pub id: String,
    /// Localisation key for the enemy's display name.
    pub name_key: String,
    /// The enemy's family (determines behaviour, terror applicability, etc.).
    pub family: Family,
    /// Regions where this enemy can appear (region IDs).
    pub region_affinity: Vec<String>,
    /// Flag expression controlling when this enemy spawns.
    pub gate: FlagExpr,
    /// Base combat stats.
    pub stats: Stats,
    /// Status effects this enemy resists.
    pub resist: Vec<StatusKind>,
    /// Ability IDs this enemy can use.
    pub abilities: Vec<String>,
    /// Items this enemy can drop.
    pub loot: Vec<LootEntry>,
    /// Experience points awarded on defeat.
    pub xp: u32,
    /// Difficulty tier.
    pub tier: u32,
    /// Sprite/asset key for rendering.
    pub sprite: String,
}
