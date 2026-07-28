//! Tests for ATB wait mode — gauge freezing when menu is open,
//! enemy gauge advancement during wait, active vs wait mode differences.

use mc_core::battle::atb::{advance_gauges, advance_gauges_party, AtbGauge};
use mc_core::fx::Fx;

#[test]
fn active_mode_advances_all_gauges() {
    let mut gauges = vec![
        AtbGauge::new(Fx::from_int(12)), // speed 12, fills in 5 ticks
        AtbGauge::new(Fx::from_int(60)), // speed 60, fills in 1 tick
        AtbGauge::new(Fx::from_int(30)), // speed 30, fills in 2 ticks
    ];

    advance_gauges(&mut gauges, false); // active mode

    // All should have advanced
    assert!(gauges[0].progress() > Fx::ZERO);
    assert!(gauges[1].is_full()); // filled immediately
    assert!(gauges[2].progress() > Fx::ZERO);
}

#[test]
fn wait_mode_freezes_non_full_gauges() {
    let mut gauges = vec![
        AtbGauge::new(Fx::from_int(12)),
        AtbGauge::new(Fx::from_int(60)),
    ];

    advance_gauges(&mut gauges, true); // wait mode

    // Neither was full, so both remain at 0
    assert_eq!(gauges[0].progress(), Fx::ZERO);
    assert_eq!(gauges[1].progress(), Fx::ZERO);
}

#[test]
fn wait_mode_allows_full_gauges_to_stay_full() {
    let mut gauges = vec![AtbGauge::new(Fx::from_int(12))];
    gauges[0].force_full();

    advance_gauges(&mut gauges, true); // wait mode
    assert!(gauges[0].is_full()); // stays full
}

#[test]
fn party_gauges_freeze_in_wait_mode_with_menu_open() {
    let mut party = vec![
        AtbGauge::new(Fx::from_int(60)), // would fill in 1 tick
        AtbGauge::new(Fx::from_int(30)), // would fill in 2 ticks
    ];
    let mut enemies = vec![
        AtbGauge::new(Fx::from_int(60)), // would fill in 1 tick
    ];

    advance_gauges_party(&mut party, &mut enemies, true); // menu open = wait

    // Party should NOT have advanced
    assert_eq!(party[0].progress(), Fx::ZERO);
    assert_eq!(party[1].progress(), Fx::ZERO);

    // Enemies should have advanced (they always advance)
    assert!(enemies[0].is_full());
}

#[test]
fn all_gauges_advance_when_menu_closed() {
    let mut party = vec![
        AtbGauge::new(Fx::from_int(60)), // fills in 1 tick
    ];
    let mut enemies = vec![
        AtbGauge::new(Fx::from_int(30)), // fills in 2 ticks
    ];

    advance_gauges_party(&mut party, &mut enemies, false); // menu closed

    // Everyone advances
    assert!(party[0].is_full());
    assert!(enemies[0].progress() > Fx::ZERO);
}

#[test]
fn atb_fill_rate_is_speed_divided_by_60() {
    // Speed 60 -> fill rate = 1.0 per tick
    let mut fast = AtbGauge::new(Fx::from_int(60));
    assert!(fast.tick());
    assert!(fast.is_full());

    // Speed 30 -> fill rate = 0.5 per tick
    let mut medium = AtbGauge::new(Fx::from_int(30));
    assert!(!medium.tick()); // 0.5
    assert!(medium.tick()); // 1.0
    assert!(medium.is_full());

    // Speed 20 -> fill rate = 0.333 per tick (truncated in Q16.16)
    let mut slow = AtbGauge::new(Fx::from_int(20));
    assert!(!slow.tick()); // 0.333
    assert!(!slow.tick()); // 0.666
    assert!(!slow.tick()); // 0.999 (rounded down)
    assert!(slow.tick()); // 1.0
    assert!(slow.is_full());
}

#[test]
fn atb_resets_after_acting() {
    let mut gauge = AtbGauge::new(Fx::from_int(60));
    gauge.tick();
    assert!(gauge.is_full());

    gauge.reset();
    assert!(!gauge.is_full());
    assert_eq!(gauge.progress(), Fx::ZERO);
}

#[test]
fn atb_new_with_start_provides_initial_progress() {
    let gauge = AtbGauge::new_with_start(Fx::from_int(12), Fx::HALF);
    assert_eq!(gauge.progress(), Fx::HALF);
    assert!(!gauge.is_full());
}

#[test]
fn wait_mode_enemy_advantage() {
    // In wait mode with menu open, enemies still build gauge while party doesn't
    let mut party = vec![
        AtbGauge::new(Fx::from_int(60)), // fills in 1 tick
    ];
    let mut enemies = vec![
        AtbGauge::new(Fx::from_int(12)), // fills in 5 ticks
    ];

    // Simulate 5 ticks with menu open (speed 12 needs 6 ticks)
    for _ in 0..5 {
        advance_gauges_party(&mut party, &mut enemies, true);
    }

    // Party should still be at 0
    assert_eq!(party[0].progress(), Fx::ZERO);

    // Enemy should be at ~5/6 progress
    assert!(enemies[0].progress() > Fx::ZERO);
    assert!(!enemies[0].is_full()); // not full yet after 5 ticks

    // 1 more tick should fill the enemy
    advance_gauges_party(&mut party, &mut enemies, true);
    assert!(enemies[0].is_full());
}

#[test]
fn active_mode_vs_wait_mode_difference() {
    let mut active = AtbGauge::new(Fx::from_int(30));
    let mut wait = AtbGauge::new(Fx::from_int(30));

    // Advance active gauge in active mode
    active.tick();
    assert!(active.progress() > Fx::ZERO);

    // Advance wait gauge in wait mode (no-op since not full)
    wait.tick_wait();
    assert_eq!(wait.progress(), Fx::ZERO);
}

#[test]
fn tick_party_menu_closed_advances_normally() {
    let mut gauge = AtbGauge::new(Fx::from_int(30));
    assert!(!gauge.tick_party(false)); // 0.5
    assert!(gauge.tick_party(false));  // 1.0
    assert!(gauge.is_full());
}

#[test]
fn tick_party_menu_open_freezes() {
    let mut gauge = AtbGauge::new(Fx::from_int(30));
    assert!(!gauge.tick_party(true)); // frozen at 0
    assert!(!gauge.tick_party(true)); // still frozen at 0
    assert_eq!(gauge.progress(), Fx::ZERO);
}

#[test]
fn fill_rate_never_exceeds_one() {
    let mut gauge = AtbGauge::new(Fx::from_int(120)); // speed 120 = 2.0 fill rate
    gauge.tick();
    // Should saturate at ATB_MAX (1.0)
    assert!(gauge.is_full());
    assert_eq!(gauge.progress(), Fx::ONE);
}

#[test]
fn atb_does_not_overflow() {
    let mut gauge = AtbGauge::new(Fx::from_int(60));
    gauge.force_full();
    // Multiple ticks on a full gauge should keep it at max
    gauge.tick();
    assert_eq!(gauge.progress(), Fx::ONE);
    gauge.tick_wait();
    assert_eq!(gauge.progress(), Fx::ONE);
    gauge.tick_party(false);
    assert_eq!(gauge.progress(), Fx::ONE);
}

#[test]
fn multiple_party_members_freeze_independently() {
    let mut party = vec![
        AtbGauge::new(Fx::from_int(30)), // 0.5 per tick
        AtbGauge::new(Fx::from_int(60)), // 1.0 per tick
        AtbGauge::new(Fx::from_int(12)), // 0.2 per tick
    ];
    let mut enemies = vec![AtbGauge::new(Fx::from_int(30))];

    // Menu open: all party members frozen
    advance_gauges_party(&mut party, &mut enemies, true);
    for p in &party {
        assert_eq!(p.progress(), Fx::ZERO);
    }
    assert!(enemies[0].progress() > Fx::ZERO);
}

#[test]
fn transition_from_wait_to_active() {
    let mut party = vec![AtbGauge::new(Fx::from_int(30))];
    let mut enemies = vec![AtbGauge::new(Fx::from_int(60))];

    // Menu open for 2 ticks (wait)
    advance_gauges_party(&mut party, &mut enemies, true);
    advance_gauges_party(&mut party, &mut enemies, true);

    assert_eq!(party[0].progress(), Fx::ZERO);
    assert!(enemies[0].is_full());

    // Close menu: both advance
    enemies[0].reset();
    advance_gauges_party(&mut party, &mut enemies, false);

    assert!(party[0].progress() > Fx::ZERO);
    assert!(enemies[0].progress() > Fx::ZERO);
}
