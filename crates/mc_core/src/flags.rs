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
    /// Create an empty flags.
    pub fn new() -> Self {
        FlagSet { bits: [0u64; 1] }
    }

    /// Set a flag.
    pub fn set(&mut self, flag: FlagId) {
        let idx = flag.raw() as usize;
        self.bits[idx / 64] |= 1u64 << (idx % 64);
    }

    /// Clear a flag.
    pub fn clear(&mut self, flag: FlagId) {
        let idx = flag.raw() as usize;
        self.bits[idx / 64] &= !(1u64 << (idx % 64));
    }

    /// Check if a flag is set.
    pub fn is_set(&self, flag: FlagId) -> bool {
        let idx = flag.raw() as usize;
        (self.bits[idx / 64] & (1u64 << (idx % 64))) != 0
    }

    /// Check if this expression is satisfied by the current flags.
    pub fn satisfies(&self, expr: &FlagExpr) -> bool {
        match expr {
            FlagExpr::Always => true,
            FlagExpr::Never => false,
            FlagExpr::Set(flag) => self.is_set(*flag),
            FlagExpr::NotSet(flag) => !self.is_set(*flag),
            FlagExpr::All(exprs) => exprs.iter().all(|e| self.satisfies(e)),
            FlagExpr::Any(exprs) => exprs.iter().any(|e| self.satisfies(e)),
            FlagExpr::Not(expr) => !self.satisfies(expr),
        }
    }

    /// Get the raw bits (for hashing).
    pub fn raw_bits(&self) -> u64 {
        self.bits[0]
    }
}

/// A boolean expression over flags.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlagExpr {
    /// Always true.
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
}
