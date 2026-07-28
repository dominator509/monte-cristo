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
    /// Amount of HP restored when using this item, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heal_hp: Option<u32>,
    /// Arbitrary effect string (e.g. "cure_poison", "reveal_secret"), if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<String>,
    /// Scene / context IDs where this item can be used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub usable_in: Vec<String>,
    /// Monetary value of this item (in the game's currency).
    pub value: u32,
}
