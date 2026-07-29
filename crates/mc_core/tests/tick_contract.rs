//! EP-004 M2: Tick contract tests (INV-05).
//!
//! Commands apply only at tick boundaries. The shell accumulates real elapsed
//! time and calls `step` the correct number of times; it may drop frames but
//! may never drop, subdivide, or double a tick. Accumulated remainder is
//! carried, never discarded.

use mc_core::command::{apply_commands, Command, CoreEvent, Dir};
use mc_core::fx::Fx;
use mc_core::world::World;

/// Commands submitted at tick 0 take effect immediately (the very first tick
/// boundary is at tick 0 for a fresh world).
#[test]
fn command_at_tick_zero_is_valid() {
    let mut world = World::new(42);
    assert_eq!(world.tick, 0);
    let events = apply_commands(&mut world, &[Command::Move(Dir::North)]);
    assert!(matches!(&events[0], CoreEvent::Applied { .. }));
}

/// After applying commands and stepping, the tick advances exactly once per
/// step call — no tick is ever subdivided or doubled.
#[test]
fn one_step_advances_exactly_one_tick() {
    let mut world = World::new(42);
    let t0 = world.tick;
    apply_commands(&mut world, &[Command::Move(Dir::North)]);
    world.step();
    assert_eq!(world.tick, t0 + 1, "one step must advance exactly one tick");
}

/// Stepping 100 times advances the tick by 100, regardless of how many
/// commands were submitted in between.
#[test]
fn hundred_steps_advances_100_ticks_regardless_of_commands() {
    let mut world = World::new(42);
    let t0 = world.tick;
    for _ in 0..100 {
        // Submit varying numbers of commands between steps
        apply_commands(&mut world, &[Command::Move(Dir::North)]);
        if world.tick % 3 == 0 {
            apply_commands(&mut world, &[Command::Interact, Command::SceneAdvance]);
        }
        world.step();
    }
    assert_eq!(world.tick, t0 + 100, "100 steps = 100 ticks exactly");
}

/// No tick is ever dropped: the difference between consecutive tick values
/// is always exactly 1 after a step.
#[test]
fn no_tick_is_ever_dropped() {
    let mut world = World::new(42);
    let mut prev = world.tick;
    for _ in 0..10_000 {
        apply_commands(&mut world, &[Command::Move(Dir::North)]);
        world.step();
        assert_eq!(
            world.tick,
            prev + 1,
            "tick jumped from {prev} to {}",
            world.tick
        );
        prev = world.tick;
    }
}

/// Multiple steps with no commands in between still advance the tick
/// monotonically.
#[test]
fn ticks_advance_even_without_commands() {
    let mut world = World::new(42);
    for expected in 1..=1_000 {
        world.step();
        assert_eq!(world.tick, expected, "tick should be {expected} after step");
    }
}

/// Commands batched before a step all apply at the same tick boundary.
/// After step, tick advances.
#[test]
fn batched_commands_apply_same_tick() {
    let mut world = World::new(42);
    let t0 = world.tick;
    // Batch 5 commands
    let cmds = vec![
        Command::Move(Dir::North),
        Command::Move(Dir::East),
        Command::Interact,
        Command::SceneAdvance,
        Command::OpenMenu,
    ];
    let events = apply_commands(&mut world, &cmds);
    // All commands applied at tick 0
    assert_eq!(world.tick, t0, "tick must not advance before step");
    assert_eq!(events.len(), 5, "all 5 commands should produce events");
    for ev in &events {
        assert!(matches!(ev, CoreEvent::Applied { .. }));
    }
    // Now step — tick advances by exactly 1
    world.step();
    assert_eq!(
        world.tick,
        t0 + 1,
        "after step tick = {expected}",
        expected = t0 + 1
    );
}

/// State hash changes after a step but not after commands alone.
#[test]
fn hash_unchanged_by_commands_changed_by_step() {
    let mut world = World::new(42);
    let h0 = world.state_hash();
    apply_commands(&mut world, &[Command::Move(Dir::South)]);
    let h1 = world.state_hash();
    assert_eq!(
        h0, h1,
        "state hash must not change from commands alone (no step)"
    );
    world.step();
    let h2 = world.state_hash();
    assert_ne!(h1, h2, "state hash must change after a step");
}

/// The accumulator remainder test: simulate irregular frame deltas as the
/// shell would. The shell accumulates real time and steps the correct number
/// of ticks at 1/60s each, carrying remainder.
///
/// We simulate 10,000 irregular "frames" with deltas between 0.5 and 3.0
/// ticks worth of time, and verify that the total accumulated ticks is
/// exactly the sum of floor'd deltas (i.e. never a dropped or doubled tick).
#[test]
fn irregular_frame_deltas_carry_remainder() {
    use mc_core::command::Dir;
    // Simulate the shell's accumulator: Fx accumulates real time in
    // ticks (1 tick = Fx::ONE), and steps the world floor(delta) times.
    let mut world = World::new(42);
    let mut accumulator = Fx::ZERO;
    let _tick_duration = Fx::ONE; // 1 tick = 1.0 in Q16.16

    // Pre-computed irregular frame deltas (in ticks): 10,000 values between
    // 0.5 and 3.0 ticks. Generated on the fly.
    let mut expected_ticks: u64 = 0;

    for i in 0..10_000u64 {
        // Create an irregular delta: oscillate between 0.5, 1.0, 1.5, 2.0, 2.5, 3.0
        let delta_raw = match i % 6 {
            0 => Fx::HALF,                            // 0.5 ticks
            1 => Fx::ONE,                             // 1.0 ticks
            2 => Fx::from_raw(3 * Fx::ONE.raw() / 2), // 1.5 ticks
            3 => Fx::from_int(2),                     // 2.0 ticks
            4 => Fx::from_raw(5 * Fx::ONE.raw() / 2), // 2.5 ticks
            _ => Fx::from_int(3),                     // 3.0 ticks
        };

        accumulator = accumulator + delta_raw;

        // How many whole ticks does the accumulator contain?
        let whole_ticks = accumulator.to_int_floor() as u64;
        if whole_ticks > 0 {
            // Submit a command before each step (player input)
            for _ in 0..whole_ticks {
                apply_commands(
                    &mut world,
                    &[Command::Move(if i % 2 == 0 {
                        Dir::North
                    } else {
                        Dir::South
                    })],
                );
            }
            for _ in 0..whole_ticks {
                world.step();
            }
            expected_ticks += whole_ticks;
            // Carry the remainder (not discarded!)
            let consumed = Fx::from_int(whole_ticks as i32);
            accumulator = accumulator - consumed;
        }
    }

    assert_eq!(
        world.tick,
        expected_ticks,
        "total ticks after {fcount} irregular frames should equal sum of floor'd deltas: \
         expected {expected_ticks}, got {actual}",
        fcount = 10_000,
        actual = world.tick,
    );

    // Verify the accumulator remainder was carried (not zeroed)
    assert!(
        accumulator >= Fx::ZERO,
        "accumulator remainder must never go negative"
    );
    assert!(
        accumulator < Fx::ONE,
        "accumulator remainder must be less than one tick, got {acc:?}",
        acc = accumulator
    );
}
