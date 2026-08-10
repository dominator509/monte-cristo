//! Authored item definitions used by deterministic command resolution.
//!
//! The content crate converts its RON item schema into this small domain-only
//! catalog. `mc_core` therefore remains free of I/O and content-parser
//! dependencies while item mechanics stay data-driven.

use crate::ids::ItemId;
use std::collections::BTreeMap;

/// The authored category of an item.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Consumable,
    Key,
    Quest,
    Equipment,
}

/// One item definition needed at the command boundary.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AuthoredItemDefinition {
    pub id: ItemId,
    pub kind: ItemKind,
    pub heal_hp: Option<u32>,
}

/// Deterministic lookup table for the authored item pack.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuthoredItemCatalog {
    definitions: BTreeMap<ItemId, AuthoredItemDefinition>,
}

impl AuthoredItemCatalog {
    /// Build a catalog, rejecting duplicate identifiers before runtime use.
    pub fn from_definitions(definitions: Vec<AuthoredItemDefinition>) -> Result<Self, String> {
        let mut catalog = Self::default();
        for definition in definitions {
            if catalog
                .definitions
                .insert(definition.id, definition)
                .is_some()
            {
                return Err(format!(
                    "duplicate authored item identifier: {:?}",
                    definition.id
                ));
            }
        }
        Ok(catalog)
    }

    /// Return the authored definition for an item identifier.
    pub fn get(&self, id: ItemId) -> Option<&AuthoredItemDefinition> {
        self.definitions.get(&id)
    }

    /// Number of authored item definitions.
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Whether the catalog contains no authored definitions.
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_resolves_authored_consumable() {
        let catalog = AuthoredItemCatalog::from_definitions(vec![AuthoredItemDefinition {
            id: ItemId::ITM_POTION,
            kind: ItemKind::Consumable,
            heal_hp: Some(25),
        }])
        .expect("unique item definitions should build");
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog.get(ItemId::ITM_POTION).unwrap().heal_hp, Some(25));
    }

    #[test]
    fn catalog_rejects_duplicate_identifiers() {
        let definition = AuthoredItemDefinition {
            id: ItemId::ITM_POTION,
            kind: ItemKind::Consumable,
            heal_hp: Some(25),
        };
        let error = AuthoredItemCatalog::from_definitions(vec![definition, definition])
            .expect_err("duplicate item identifiers must be rejected");
        assert!(error.contains("duplicate authored item identifier"));
    }
}
