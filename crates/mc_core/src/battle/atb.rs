//! ATB gauge logic — Chrono Trigger style Active Time Battle.
//!
//! SPEC-001 section 5: ATB fills from 0.0 to 1.0 at rate (speed / 60) per tick.
//! Wait mode halts gauge advancement when the menu is open.

use crate::fx::Fx;
use serde::{Deserialize, Serialize};

/// Maximum ATB value (fully charged).
pub const ATB_MAX: Fx = Fx::ONE;

/// The ATB gauge for a single combatant.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AtbGauge {
    /// Current ATB value in 0..=1 (Q16.16).
    pub value: Fx,
    /// Combatant speed (used to compute fill rate).
    pub speed: Fx,
}

impl AtbGauge {
    /// Create a new ATB gauge starting at 0.
    pub fn new(speed: Fx) -> Self {
        AtbGauge {
            value: Fx::ZERO,
            speed,
        }
    }

    /// Create a new ATB gauge starting at a fraction of max (for battle start).
    pub fn new_with_start(speed: Fx, start_fraction: Fx) -> Self {
        let value = Fx::ONE.saturating_mul(start_fraction);
        AtbGauge { value, speed }
    }

    /// Check if the gauge is full (ready to act).
    pub fn is_full(&self) -> bool {
        self.value >= ATB_MAX
    }

    /// Force the gauge to full (used in tests).
    pub fn force_full(&mut self) {
        self.value = ATB_MAX;
    }

    /// Reset the gauge to 0 after acting.
    pub fn reset(&mut self) {
        self.value = Fx::ZERO;
    }

    /// Advance the gauge by one tick.
    ///
    /// Fill rate = speed / 60. Returns true if the gauge became full.
    pub fn tick(&mut self) -> bool {
        if self.value >= ATB_MAX {
            return true;
        }
        // fill rate = speed / 60
        let fill_rate = self.speed.saturating_div(Fx::from_int(60));
        self.value = self.value.saturating_add(fill_rate);
        if self.value > ATB_MAX {
            self.value = ATB_MAX;
        }
        self.value >= ATB_MAX
    }

    /// Advance the gauge by one tick in wait mode.
    /// In wait mode, the gauge only advances if it is already full (no-op for
    /// non-full gauges), effectively freezing until the menu closes.
    pub fn tick_wait(&mut self) -> bool {
        // Wait mode: do not advance if not full yet
        self.is_full()
    }

    /// Advance the gauge for a party member in wait mode.
    /// When the menu is open, party member gauges freeze.
    /// When the menu is closed, they advance normally.
    pub fn tick_party(&mut self, menu_open: bool) -> bool {
        if menu_open {
            self.tick_wait()
        } else {
            self.tick()
        }
    }

    /// Get the current fill progress as a fraction [0, 1].
    pub fn progress(&self) -> Fx {
        self.value
    }
}

/// Advance all combatant ATB gauges by one tick.
/// In wait mode, non-full gauges do not advance.
pub fn advance_gauges(gauges: &mut [AtbGauge], wait_mode: bool) {
    for gauge in gauges.iter_mut() {
        if wait_mode {
            gauge.tick_wait();
        } else {
            gauge.tick();
        }
    }
}

/// Advance party combatant ATB gauges with menu awareness.
pub fn advance_gauges_party(
    party_gauges: &mut [AtbGauge],
    enemy_gauges: &mut [AtbGauge],
    menu_open: bool,
) {
    // In wait mode with menu open, only enemies advance.
    // In active mode, everyone advances.
    if menu_open {
        // Wait mode: enemies still advance, party freezes
        for gauge in enemy_gauges.iter_mut() {
            gauge.tick();
        }
        // Party gauges do NOT advance while menu is open in wait mode
    } else {
        // Menu closed: everyone advances
        for gauge in party_gauges.iter_mut() {
            gauge.tick();
        }
        for gauge in enemy_gauges.iter_mut() {
            gauge.tick();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_gauge_starts_at_zero() {
        let gauge = AtbGauge::new(Fx::from_int(12));
        assert_eq!(gauge.value, Fx::ZERO);
        assert!(!gauge.is_full());
    }

    #[test]
    fn gauge_fills_over_time() {
        let mut gauge = AtbGauge::new(Fx::from_int(60));
        // speed 60 / 60 = 1.0 per tick
        let result = gauge.tick();
        assert!(result); // should fill instantly
        assert!(gauge.is_full());
    }

    #[test]
    fn gauge_fills_slower_with_lower_speed() {
        let mut gauge = AtbGauge::new(Fx::from_int(12));
        // speed 12 / 60 = 0.2 per tick (in Q16.16: 13107 per tick)
        // Need 5 ticks to reach ~0.999, 6 ticks to reach 1.0 due to truncation
        assert!(!gauge.tick()); // 0.2
        assert!(!gauge.tick()); // 0.4
        assert!(!gauge.tick()); // 0.6
        assert!(!gauge.tick()); // 0.8
        assert!(!gauge.tick()); // 1.0 - rounding may put it just under
        assert!(gauge.tick()); // definitely full now
    }

    #[test]
    fn reset_clears_gauge() {
        let mut gauge = AtbGauge::new(Fx::from_int(60));
        gauge.tick();
        assert!(gauge.is_full());
        gauge.reset();
        assert!(!gauge.is_full());
        assert_eq!(gauge.value, Fx::ZERO);
    }

    #[test]
    fn wait_mode_does_not_advance_non_full() {
        let mut gauge = AtbGauge::new(Fx::from_int(12));
        // Not full, wait mode should not advance
        let result = gauge.tick_wait();
        assert!(!result);
        assert_eq!(gauge.value, Fx::ZERO);
    }

    #[test]
    fn wait_mode_returns_true_when_full() {
        let mut gauge = AtbGauge::new(Fx::from_int(12));
        gauge.force_full();
        let result = gauge.tick_wait();
        assert!(result);
    }

    #[test]
    fn tick_party_menu_open_freeze() {
        let mut gauge = AtbGauge::new(Fx::from_int(60));
        // Menu open should freeze
        let result = gauge.tick_party(true);
        assert!(!result);
        assert_eq!(gauge.value, Fx::ZERO); // did not advance
    }

    #[test]
    fn tick_party_menu_closed_advances() {
        let mut gauge = AtbGauge::new(Fx::from_int(60));
        let result = gauge.tick_party(false);
        assert!(result);
        assert!(gauge.is_full());
    }

    #[test]
    fn advance_gauges_active() {
        let mut gauges = vec![
            AtbGauge::new(Fx::from_int(12)), // speed 12
            AtbGauge::new(Fx::from_int(60)), // speed 60 — fills in 1 tick
        ];
        advance_gauges(&mut gauges, false);
        assert!(!gauges[0].is_full());
        assert!(gauges[1].is_full());
    }

    #[test]
    fn advance_gauges_wait() {
        let mut gauges = vec![
            AtbGauge::new(Fx::from_int(12)),
            AtbGauge::new(Fx::from_int(60)),
        ];
        advance_gauges(&mut gauges, true);
        // In wait mode, neither should advance because neither is full
        assert!(!gauges[0].is_full());
        assert!(!gauges[1].is_full());
        assert_eq!(gauges[0].value, Fx::ZERO);
        assert_eq!(gauges[1].value, Fx::ZERO);
    }

    #[test]
    fn advance_gauges_party_wait_menu_open() {
        let mut party = vec![AtbGauge::new(Fx::from_int(60))]; // would fill in 1 tick
        let mut enemies = vec![AtbGauge::new(Fx::from_int(60))];
        advance_gauges_party(&mut party, &mut enemies, true);
        // Party froze
        assert!(!party[0].is_full());
        assert_eq!(party[0].value, Fx::ZERO);
        // Enemy advanced
        assert!(enemies[0].is_full());
    }

    #[test]
    fn new_with_start() {
        let gauge = AtbGauge::new_with_start(Fx::from_int(12), Fx::HALF);
        assert_eq!(gauge.value, Fx::HALF);
    }

    #[test]
    fn progress() {
        let gauge = AtbGauge::new(Fx::from_int(12));
        assert_eq!(gauge.progress(), Fx::ZERO);
        let mut gauge = AtbGauge::new(Fx::from_int(60));
        gauge.tick();
        assert_eq!(gauge.progress(), Fx::ONE);
    }
}
