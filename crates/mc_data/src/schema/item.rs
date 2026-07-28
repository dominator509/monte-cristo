//! Content schema types for item definitions.
//!
//! Items are objects the player can collect, use, or equip. This module
//! defines the static, authored item types used by the content pipeline.

use serde::{Deserialize, Serialize};

/// The general category of an item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    /// A consumable item that is used up (e.g. potions, food).
    Consumable,
    /// A key item required to open doors or progress.
    Key,
    /// A quest item used in story objectives.
    Quest,
    /// Equippable gear (weapons, armour, accessories).
    Equipment,
}

/// An item definition loaded from content files.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    /// Unique identifier for this item (e.g. "potion_hp_small", "old_key").
    pub id: String,
    /// Localisation key for the item's display name.
    pub name_key: String,
    /// Localisation key for the item's description.
    pub description_key: String,
    /// The category this item belongs to.
    pub item_type: ItemType,
    /// Amount of HP restored when consumed (if applicable).
    #[serde(default)]
    pub heal_hp: Option<u32>,
    /// Name of a special effect triggered on use (if applicable).
    #[serde(default)]
    pub effect: Option<String>,
    /// List of contexts (scene/fight IDs) where this item can be used.
    /// Empty means usable anywhere.
    #[serde(default)]
    pub usable_in: Vec<String>,
    pub value: u32,
}
