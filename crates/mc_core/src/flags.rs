//! FlagSet — a sorted bitset over the locked flag vocabulary.
//!
//! Supports `set`, `clear`, `is_set`, and `satisfies(&FlagExpr)`.
//! Backed by a fixed-size bit array. No HashMap, no HashSet.

use crate::ids::FlagId;
use serde::{Deserialize, Serialize};

const _FLAG_COUNT: usize = FlagId::COUNT; // 22

/// A bitset over the locked flag vocabulary.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlagSet {
    bits: [u64; 1], // 22 flags fits in one u64
}

impl FlagSet {
    /// Return the bounded bit location for a locked flag identifier.
    ///
    /// `FlagId` can be constructed from untrusted serialized data, so every
    /// bitset access must validate the raw value before indexing the word.
    fn bit_location(flag: FlagId) -> Option<(usize, u32)> {
        let index = flag.raw() as usize;
        (index < FlagId::COUNT).then_some((index / 64, (index % 64) as u32))
    }

    /// Create an empty flags.
    pub fn new() -> Self {
        FlagSet { bits: [0u64; 1] }
    }

    /// Set a flag.
    pub fn set(&mut self, flag: FlagId) {
        if let Some((word, bit)) = Self::bit_location(flag) {
            self.bits[word] |= 1u64 << bit;
        }
    }

    /// Clear a flag.
    pub fn clear(&mut self, flag: FlagId) {
        if let Some((word, bit)) = Self::bit_location(flag) {
            self.bits[word] &= !(1u64 << bit);
        }
    }

    /// Check if a flag is set.
    pub fn is_set(&self, flag: FlagId) -> bool {
        Self::bit_location(flag)
            .map(|(word, bit)| (self.bits[word] & (1u64 << bit)) != 0)
            .unwrap_or(false)
    }

    /// Check if this expression is satisfied by the current flags.
    pub fn satisfies(&self, expr: &FlagExpr) -> bool {
        if !Self::expression_is_valid(expr) {
            return false;
        }
        match expr {
            FlagExpr::Always => true,
            FlagExpr::Never => false,
            // Invalid raw identifiers must fail closed in both positive and
            // negative expressions; treating an unknown flag as "not set"
            // could unlock authored content from malformed input.
            FlagExpr::Set(flag) => Self::bit_location(*flag).is_some() && self.is_set(*flag),
            FlagExpr::NotSet(flag) => Self::bit_location(*flag).is_some() && !self.is_set(*flag),
            FlagExpr::All(exprs) => exprs.iter().all(|e| self.satisfies(e)),
            FlagExpr::Any(exprs) => exprs.iter().any(|e| self.satisfies(e)),
            FlagExpr::Not(expr) => !self.satisfies(expr),
        }
    }

    /// Validate every identifier in an authored expression before evaluating
    /// it. This keeps `Not` and `Any` from turning one malformed child into a
    /// gate that accidentally passes.
    fn expression_is_valid(expr: &FlagExpr) -> bool {
        match expr {
            FlagExpr::Always | FlagExpr::Never => true,
            FlagExpr::Set(flag) | FlagExpr::NotSet(flag) => Self::bit_location(*flag).is_some(),
            FlagExpr::All(exprs) | FlagExpr::Any(exprs) => {
                exprs.iter().all(Self::expression_is_valid)
            }
            FlagExpr::Not(expr) => Self::expression_is_valid(expr),
        }
    }

    /// Get the raw bits (for hashing).
    pub fn raw_bits(&self) -> u64 {
        self.bits[0]
    }
}

/// A boolean expression over flags.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FlagExpr {
    /// Always true.
    #[default]
    Always,
    /// Always false.
    Never,
    /// A specific flag is set.
    Set(FlagId),
    /// A specific flag is NOT set.
    NotSet(FlagId),
    /// All sub-expressions must be true.
    All(Vec<FlagExpr>),
    /// At least one sub-expression must be true.
    Any(Vec<FlagExpr>),
    /// The sub-expression must be false.
    Not(Box<FlagExpr>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::FlagId;

    #[test]
    fn set_and_check() {
        let mut fs = FlagSet::new();
        assert!(!fs.is_set(FlagId::FLG_ARRESTED));
        fs.set(FlagId::FLG_ARRESTED);
        assert!(fs.is_set(FlagId::FLG_ARRESTED));
    }

    #[test]
    fn clear_flag() {
        let mut fs = FlagSet::new();
        fs.set(FlagId::FLG_ARRESTED);
        fs.clear(FlagId::FLG_ARRESTED);
        assert!(!fs.is_set(FlagId::FLG_ARRESTED));
    }

    #[test]
    fn multiple_flags() {
        let mut fs = FlagSet::new();
        fs.set(FlagId::FLG_ARRESTED);
        fs.set(FlagId::FLG_FARIA_MET);
        assert!(fs.is_set(FlagId::FLG_ARRESTED));
        assert!(fs.is_set(FlagId::FLG_FARIA_MET));
        assert!(!fs.is_set(FlagId::FLG_ESCAPED));
    }

    #[test]
    fn expr_always() {
        let fs = FlagSet::new();
        assert!(fs.satisfies(&FlagExpr::Always));
    }

    #[test]
    fn expr_never() {
        let fs = FlagSet::new();
        assert!(!fs.satisfies(&FlagExpr::Never));
    }

    #[test]
    fn expr_set() {
        let mut fs = FlagSet::new();
        fs.set(FlagId::FLG_ARRESTED);
        assert!(fs.satisfies(&FlagExpr::Set(FlagId::FLG_ARRESTED)));
        assert!(!fs.satisfies(&FlagExpr::Set(FlagId::FLG_ESCAPED)));
    }

    #[test]
    fn expr_all() {
        let mut fs = FlagSet::new();
        fs.set(FlagId::FLG_ARRESTED);
        fs.set(FlagId::FLG_FARIA_MET);
        assert!(fs.satisfies(&FlagExpr::All(vec![
            FlagExpr::Set(FlagId::FLG_ARRESTED),
            FlagExpr::Set(FlagId::FLG_FARIA_MET),
        ])));
        assert!(!fs.satisfies(&FlagExpr::All(vec![
            FlagExpr::Set(FlagId::FLG_ARRESTED),
            FlagExpr::Set(FlagId::FLG_ESCAPED),
        ])));
    }

    #[test]
    fn expr_any() {
        let mut fs = FlagSet::new();
        fs.set(FlagId::FLG_ARRESTED);
        assert!(fs.satisfies(&FlagExpr::Any(vec![
            FlagExpr::Set(FlagId::FLG_ARRESTED),
            FlagExpr::Set(FlagId::FLG_ESCAPED),
        ])));
        assert!(!fs.satisfies(&FlagExpr::Any(vec![
            FlagExpr::Set(FlagId::FLG_ESCAPED),
            FlagExpr::Set(FlagId::FLG_FARIA_MET),
        ])));
    }

    #[test]
    fn expr_not() {
        let fs = FlagSet::new();
        assert!(fs.satisfies(&FlagExpr::Not(Box::new(FlagExpr::Set(
            FlagId::FLG_ARRESTED
        )))));
    }

    #[test]
    fn invalid_raw_flag_is_ignored_without_panicking() {
        let invalid = FlagId::from_raw(FlagId::COUNT as u16);
        let mut flags = FlagSet::new();

        flags.set(invalid);
        assert!(!flags.is_set(invalid));
        flags.clear(invalid);
        assert!(!flags.is_set(invalid));
        assert!(!flags.satisfies(&FlagExpr::Set(invalid)));
        assert!(!flags.satisfies(&FlagExpr::NotSet(invalid)));
        assert!(!flags.satisfies(&FlagExpr::Not(Box::new(FlagExpr::NotSet(invalid,)))));
        assert!(!flags.satisfies(&FlagExpr::Any(vec![
            FlagExpr::Set(invalid),
            FlagExpr::Always,
        ])));
    }
}
