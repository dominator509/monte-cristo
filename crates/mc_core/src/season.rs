//! Paris season clock — 24 fortnights for Act VI.
//!
//! During Act VI (Paris), every campaign action consumes one fortnight.
//! Scheduled events fire at declared fortnights regardless of player
//! attention — notably the Villefort poisonings, which progress on their own
//! timetable.
//!
//! SPEC-001 section 9 is authoritative.

use serde::{Deserialize, Serialize};

/// A named event scheduled for a specific fortnight.
///
/// The `kind` field is a string key identifying the event (e.g.
/// `"villefort_poisoning_1"`, `"heloise_visit"`) that downstream systems
/// (scene, flag reactions) use to dispatch the authored content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeasonEvent {
    /// The fortnight (0-based) when this event fires.
    pub fortnight: u32,
    /// A string key identifying the event kind.
    pub kind: String,
}

impl SeasonEvent {
    /// Create a new season event.
    pub fn new(fortnight: u32, kind: impl Into<String>) -> Self {
        SeasonEvent {
            fortnight,
            kind: kind.into(),
        }
    }
}

/// The Paris season clock.
///
/// Runs for exactly 24 fortnights. Each call to `advance` progresses one
/// fortnight and returns any events scheduled for that fortnight.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeasonClock {
    /// Current fortnight (0-based, 0..24).
    pub fortnight: u32,
    /// Event kinds that have already fired (deduplication).
    pub fired_events: Vec<String>,
    /// The authored schedule of events.
    pub schedule: Vec<SeasonEvent>,
}

impl SeasonClock {
    /// Total number of fortnights in a Paris season.
    pub const TOTAL_FORTNIGHTS: u32 = 24;

    /// Create a new season clock with the given schedule of events.
    pub fn new(schedule: Vec<SeasonEvent>) -> Self {
        SeasonClock {
            fortnight: 0,
            fired_events: Vec::new(),
            schedule,
        }
    }

    /// Advance one fortnight.
    ///
    /// Returns a list of events that fire this fortnight (may be empty).
    /// Each event fires at most once; subsequent calls at the same fortnight
    /// (after the clock has advanced past it) will not re-fire it.
    ///
    /// If the clock is already complete (all 24 fortnights elapsed), this is
    /// a no-op returning an empty vector.
    pub fn advance(&mut self) -> Vec<&SeasonEvent> {
        let mut events = Vec::new();
        if self.fortnight >= Self::TOTAL_FORTNIGHTS {
            return events;
        }

        // Collect all scheduled events for this fortnight that haven't fired yet.
        for event in &self.schedule {
            if event.fortnight == self.fortnight && !self.fired_events.contains(&event.kind) {
                events.push(event);
                self.fired_events.push(event.kind.clone());
            }
        }

        self.fortnight += 1;
        events
    }

    /// Check whether the season clock is complete.
    pub fn is_complete(&self) -> bool {
        self.fortnight >= Self::TOTAL_FORTNIGHTS
    }

    /// Reset the clock to its initial state, keeping the schedule.
    pub fn reset(&mut self) {
        self.fortnight = 0;
        self.fired_events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_clock_at_zero() {
        let clock = SeasonClock::new(vec![]);
        assert_eq!(clock.fortnight, 0);
        assert!(!clock.is_complete());
    }

    #[test]
    fn advance_increments_fortnight() {
        let mut clock = SeasonClock::new(vec![]);
        let count = clock.advance().len();
        assert_eq!(clock.fortnight, 1);
        assert_eq!(count, 0);
    }

    #[test]
    fn event_fires_at_scheduled_fortnight() {
        let schedule = vec![
            SeasonEvent::new(0, "start"),
            SeasonEvent::new(5, "mid_point"),
            SeasonEvent::new(23, "finale"),
        ];
        let mut clock = SeasonClock::new(schedule);

        // Fortnight 0: "start" fires
        let events = clock.advance();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "start");

        // Advance to fortnight 5
        for _ in 1..5 {
            clock.advance();
        }
        assert_eq!(clock.fortnight, 5);
        let events = clock.advance();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "mid_point");
    }

    #[test]
    fn event_only_fires_once() {
        let schedule = vec![SeasonEvent::new(3, "unique")];
        let mut clock = SeasonClock::new(schedule);

        // Fast-forward to fortnight 3
        for _ in 0..3 {
            clock.advance();
        }
        // Fortnight 3 fires
        let events = clock.advance();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "unique");

        // Advance past — no repeat
        let events = clock.advance();
        assert!(events.is_empty());
    }

    #[test]
    fn clock_completes_at_24() {
        let mut clock = SeasonClock::new(vec![]);
        for _ in 0..23 {
            clock.advance();
        }
        assert_eq!(clock.fortnight, 23);
        assert!(!clock.is_complete());

        // 24th advance → complete
        clock.advance();
        assert!(clock.is_complete());
    }

    #[test]
    fn noop_after_complete() {
        let mut clock = SeasonClock::new(vec![]);
        // Fast-forward past completion
        for _ in 0..30 {
            clock.advance();
        }
        assert!(clock.is_complete());
        assert_eq!(clock.fortnight, 24); // clamped at 24
    }

    #[test]
    fn multiple_events_same_fortnight() {
        let schedule = vec![
            SeasonEvent::new(2, "event_a"),
            SeasonEvent::new(2, "event_b"),
        ];
        let mut clock = SeasonClock::new(schedule);

        for _ in 0..2 {
            clock.advance();
        }
        let events = clock.advance();
        assert_eq!(events.len(), 2);
        let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();
        assert!(kinds.contains(&"event_a"));
        assert!(kinds.contains(&"event_b"));
    }

    #[test]
    fn reset_keeps_schedule() {
        let schedule = vec![SeasonEvent::new(0, "test")];
        let mut clock = SeasonClock::new(schedule.clone());
        clock.advance();
        assert_eq!(clock.fortnight, 1);
        assert_eq!(clock.fired_events.len(), 1);

        clock.reset();
        assert_eq!(clock.fortnight, 0);
        assert!(clock.fired_events.is_empty());
        // Schedule should still be there
        assert_eq!(clock.schedule.len(), 1);
    }

    #[test]
    fn total_fortnights_constant() {
        assert_eq!(SeasonClock::TOTAL_FORTNIGHTS, 24);
    }
}
