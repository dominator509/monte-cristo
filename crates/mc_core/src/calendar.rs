//! Château d'If calendar — 168 months of decision-making.
//!
//! During Act II the player must allocate each month to one of four actions:
//! `Dig`, `Study(Discipline)`, `Endure`, or `Observe`. Faria joins
//! unconditionally at month 72. Dig progress and Observe knowledge feed the
//! escape sequence's success conditions.
//!
//! SPEC-001 section 8 is authoritative.

use serde::{Deserialize, Serialize};

use crate::curriculum::{Curriculum, Discipline};

/// Actions available during a calendar month in Act II (Château d'If).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalendarAction {
    /// Dig a tunnel toward the sea-galleries.
    Dig,
    /// Study a discipline under Faria's (or one's own) tutelage.
    Study(Discipline),
    /// Endure — restore wound damage. Nothing else restores HP during Act II.
    Endure,
    /// Observe the guards, tides, and routines.
    Observe,
}

/// The Château d'If calendar.
///
/// Tracks the current month, Faria's presence, and the two escape-relevant
/// accumulators (dig progress, observe knowledge).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IfCalendar {
    /// Current month (0-based, 0..168).
    pub month: u32,
    /// Whether Abbé Faria has joined Edmond's cell block.
    pub faria_joined: bool,
    /// Dig progress — an integer feeding the escape sequence's success
    /// conditions. Maximum is the authored threshold in the escape scene.
    pub dig_progress: u32,
    /// Observe knowledge — an integer feeding the escape sequence's success
    /// conditions.
    pub observe_knowledge: u32,
}

impl IfCalendar {
    /// Total number of months in the Château d'If calendar.
    pub const TOTAL_MONTHS: u32 = 168;

    /// The month (0-based) at which Faria unconditionally joins.
    pub const FARIA_JOIN_MONTH: u32 = 72;

    /// Create a fresh calendar at month 0.
    pub fn new() -> Self {
        IfCalendar {
            month: 0,
            faria_joined: false,
            dig_progress: 0,
            observe_knowledge: 0,
        }
    }

    /// Advance one month with the given action, updating the curriculum and
    /// accumulators as appropriate.
    ///
    /// Returns `true` when the calendar is complete (all 168 months elapsed).
    ///
    /// If the calendar is already complete, this is a no-op that returns
    /// `true`.
    pub fn advance(&mut self, action: CalendarAction, curriculum: &mut Curriculum) -> bool {
        if self.month >= Self::TOTAL_MONTHS {
            return true;
        }

        match action {
            CalendarAction::Dig => {
                self.dig_progress = self.dig_progress.saturating_add(1);
            }
            CalendarAction::Study(discipline) => {
                curriculum.add_months(discipline, 1);
            }
            CalendarAction::Endure => {
                // Endure restores wound damage. This is handled by the battle
                // system when it reads the calendar state; the calendar itself
                // only tracks that the action was taken.
            }
            CalendarAction::Observe => {
                self.observe_knowledge = self.observe_knowledge.saturating_add(1);
            }
        }

        self.month += 1;

        // Faria joins unconditionally at month 72.
        if self.month >= Self::FARIA_JOIN_MONTH && !self.faria_joined {
            self.faria_joined = true;
        }

        self.month >= Self::TOTAL_MONTHS
    }

    /// Check whether the calendar is complete (all months elapsed).
    pub fn is_complete(&self) -> bool {
        self.month >= Self::TOTAL_MONTHS
    }

    /// Reset the calendar to its initial state.
    pub fn reset(&mut self) {
        self.month = 0;
        self.faria_joined = false;
        self.dig_progress = 0;
        self.observe_knowledge = 0;
    }
}

impl Default for IfCalendar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_calendar_values() {
        let cal = IfCalendar::new();
        assert_eq!(cal.month, 0);
        assert!(!cal.faria_joined);
        assert_eq!(cal.dig_progress, 0);
        assert_eq!(cal.observe_knowledge, 0);
    }

    #[test]
    fn dig_increments_progress() {
        let mut cal = IfCalendar::new();
        let mut cur = Curriculum::new();
        cal.advance(CalendarAction::Dig, &mut cur);
        assert_eq!(cal.dig_progress, 1);
        assert_eq!(cal.month, 1);
    }

    #[test]
    fn study_increments_curriculum() {
        let mut cal = IfCalendar::new();
        let mut cur = Curriculum::new();
        cal.advance(CalendarAction::Study(Discipline::Chemistry), &mut cur);
        assert_eq!(cur.months_for(Discipline::Chemistry), 1);
        assert_eq!(cal.month, 1);
    }

    #[test]
    fn observe_increments_knowledge() {
        let mut cal = IfCalendar::new();
        let mut cur = Curriculum::new();
        cal.advance(CalendarAction::Observe, &mut cur);
        assert_eq!(cal.observe_knowledge, 1);
        assert_eq!(cal.month, 1);
    }

    #[test]
    fn endure_advances_month() {
        let mut cal = IfCalendar::new();
        let mut cur = Curriculum::new();
        cal.advance(CalendarAction::Endure, &mut cur);
        assert_eq!(cal.month, 1);
    }

    #[test]
    fn faria_joins_at_month_72() {
        let mut cal = IfCalendar::new();
        let mut cur = Curriculum::new();
        // Advance to month 71 (still before Faria)
        for _ in 0..71 {
            cal.advance(CalendarAction::Endure, &mut cur);
        }
        assert_eq!(cal.month, 71);
        assert!(!cal.faria_joined);

        // Month 72 advance: Faria joins
        cal.advance(CalendarAction::Endure, &mut cur);
        assert!(cal.faria_joined);
    }

    #[test]
    fn calendar_completes_at_168() {
        let mut cal = IfCalendar::new();
        let mut cur = Curriculum::new();
        // Advance 167 months — not yet complete
        for _ in 0..167 {
            let done = cal.advance(CalendarAction::Endure, &mut cur);
            if done {
                break;
            }
        }
        assert_eq!(cal.month, 167);
        assert!(!cal.is_complete());

        // One more month → complete
        let done = cal.advance(CalendarAction::Endure, &mut cur);
        assert!(done);
        assert!(cal.is_complete());
    }

    #[test]
    fn noop_after_complete() {
        let mut cal = IfCalendar::new();
        let mut cur = Curriculum::new();
        // Fast-forward past completion
        for _ in 0..200 {
            // ignore return
            let _ = cal.advance(CalendarAction::Dig, &mut cur);
        }
        assert!(cal.is_complete());
        // Dig progress shouldn't increase after completion
        assert_eq!(cal.dig_progress, 168);
    }

    #[test]
    fn reset_restores_initial() {
        let mut cal = IfCalendar::new();
        let mut cur = Curriculum::new();
        for _ in 0..100 {
            cal.advance(CalendarAction::Dig, &mut cur);
        }
        cal.reset();
        assert_eq!(cal.month, 0);
        assert!(!cal.faria_joined);
        assert_eq!(cal.dig_progress, 0);
        assert_eq!(cal.observe_knowledge, 0);
    }

    #[test]
    fn total_months_constant() {
        assert_eq!(IfCalendar::TOTAL_MONTHS, 168);
        assert_eq!(IfCalendar::FARIA_JOIN_MONTH, 72);
    }
}
