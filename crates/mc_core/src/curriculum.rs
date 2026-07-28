//! Curriculum — seven disciplines with rank progression.
//!
//! Each discipline has ranks 0–5. Thresholds are cumulative months studied:
//! rank 1 at 1 month, rank 2 at 3, rank 3 at 7, rank 4 at 13, rank 5 at 21.
//!
//! SPEC-001 section 7 is authoritative.

use serde::{Deserialize, Serialize};

/// The seven disciplines a character may study.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Discipline {
    Fencing,
    Chemistry,
    NaturalPhilosophy,
    Mathematics,
    Languages,
    HistoryPolitics,
    Economics,
}

impl Discipline {
    /// All seven disciplines in a fixed order.
    pub const ALL: [Discipline; 7] = [
        Discipline::Fencing,
        Discipline::Chemistry,
        Discipline::NaturalPhilosophy,
        Discipline::Mathematics,
        Discipline::Languages,
        Discipline::HistoryPolitics,
        Discipline::Economics,
    ];

    /// Human-readable name for the discipline.
    pub fn name(self) -> &'static str {
        match self {
            Discipline::Fencing => "Fencing",
            Discipline::Chemistry => "Chemistry",
            Discipline::NaturalPhilosophy => "Natural Philosophy",
            Discipline::Mathematics => "Mathematics",
            Discipline::Languages => "Languages",
            Discipline::HistoryPolitics => "History & Politics",
            Discipline::Economics => "Economics",
        }
    }

    /// The number of disciplines.
    pub const COUNT: usize = 7;
}

/// Cumulative months needed to reach each rank level (1-indexed by index).
///
/// Index 0 → rank 0 (0 months), index 1 → rank 1 (1 month),
/// index 2 → rank 2 (3 months), … index 5 → rank 5 (21 months).
pub const RANK_THRESHOLDS: [u32; 6] = [0, 1, 3, 7, 13, 21];

/// Tracks cumulative study months per discipline and derives rank.
///
/// Backed by a fixed array of 7 `u32` values. No HashMap.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Curriculum {
    /// Cumulative months spent studying each discipline, indexed by
    /// `Discipline` discriminant (0 = Fencing, …, 6 = Economics).
    pub months: [u32; 7],
}

impl Curriculum {
    /// Create a fresh curriculum with zero progress in all disciplines.
    pub fn new() -> Self {
        Curriculum { months: [0; 7] }
    }

    /// Add `months` of study to the given discipline (saturating).
    pub fn add_months(&mut self, discipline: Discipline, months: u32) {
        let idx = discipline as usize;
        self.months[idx] = self.months[idx].saturating_add(months);
    }

    /// Return the current rank (0–5) for a discipline.
    pub fn rank(&self, discipline: Discipline) -> u32 {
        let m = self.months[discipline as usize];
        for (i, &threshold) in RANK_THRESHOLDS.iter().enumerate().rev() {
            if m >= threshold {
                return i as u32;
            }
        }
        0
    }

    /// Return the cumulative months studied for a discipline.
    pub fn months_for(&self, discipline: Discipline) -> u32 {
        self.months[discipline as usize]
    }

    /// Return the cumulative months needed for the next rank (or `None` at max rank).
    pub fn months_to_next_rank(&self, discipline: Discipline) -> Option<u32> {
        let current = self.rank(discipline) as usize;
        if current >= 5 {
            return None; // already max rank
        }
        let next_threshold = RANK_THRESHOLDS[current + 1];
        let studied = self.months[discipline as usize];
        Some(next_threshold.saturating_sub(studied))
    }
}

impl Default for Curriculum {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_curriculum_all_zero() {
        let c = Curriculum::new();
        for d in Discipline::ALL {
            assert_eq!(c.rank(d), 0);
            assert_eq!(c.months_for(d), 0);
        }
    }

    #[test]
    fn rank_0_at_zero_months() {
        let c = Curriculum::new();
        assert_eq!(c.rank(Discipline::Fencing), 0);
    }

    #[test]
    fn rank_1_at_one_month() {
        let mut c = Curriculum::new();
        c.add_months(Discipline::Fencing, 1);
        assert_eq!(c.rank(Discipline::Fencing), 1);
    }

    #[test]
    fn rank_2_at_three_months() {
        let mut c = Curriculum::new();
        c.add_months(Discipline::Fencing, 3);
        assert_eq!(c.rank(Discipline::Fencing), 2);
    }

    #[test]
    fn rank_3_at_seven_months() {
        let mut c = Curriculum::new();
        c.add_months(Discipline::Chemistry, 7);
        assert_eq!(c.rank(Discipline::Chemistry), 3);
    }

    #[test]
    fn rank_4_at_thirteen_months() {
        let mut c = Curriculum::new();
        c.add_months(Discipline::Mathematics, 13);
        assert_eq!(c.rank(Discipline::Mathematics), 4);
    }

    #[test]
    fn rank_5_at_twentyone_months() {
        let mut c = Curriculum::new();
        c.add_months(Discipline::Languages, 21);
        assert_eq!(c.rank(Discipline::Languages), 5);
    }

    #[test]
    fn rank_5_stays_at_five() {
        let mut c = Curriculum::new();
        c.add_months(Discipline::Economics, 30);
        assert_eq!(c.rank(Discipline::Economics), 5);
    }

    #[test]
    fn months_to_next_rank() {
        let mut c = Curriculum::new();
        // Rank 0: need 1 to reach rank 1
        assert_eq!(c.months_to_next_rank(Discipline::Fencing), Some(1));
        c.add_months(Discipline::Fencing, 1);
        // Rank 1: need 2 more to reach rank 2 (threshold 3 - 1 = 2)
        assert_eq!(c.months_to_next_rank(Discipline::Fencing), Some(2));
        c.add_months(Discipline::Fencing, 2);
        // Rank 2: need 4 more to reach rank 3 (threshold 7 - 3 = 4)
        assert_eq!(c.months_to_next_rank(Discipline::Fencing), Some(4));
        c.add_months(Discipline::Fencing, 18);
        // Rank 5: no next rank
        assert_eq!(c.months_to_next_rank(Discipline::Fencing), None);
    }

    #[test]
    fn disciplines_are_separate() {
        let mut c = Curriculum::new();
        c.add_months(Discipline::Fencing, 5);
        assert_eq!(c.rank(Discipline::Fencing), 2);
        assert_eq!(c.rank(Discipline::Chemistry), 0);
    }

    #[test]
    fn default_equals_new() {
        assert_eq!(Curriculum::default(), Curriculum::new());
    }

    #[test]
    fn all_disciplines_count() {
        assert_eq!(Discipline::COUNT, 7);
        assert_eq!(Discipline::ALL.len(), 7);
    }

    #[test]
    fn rank_thresholds_len() {
        assert_eq!(RANK_THRESHOLDS.len(), 6); // ranks 0..=5
    }
}
