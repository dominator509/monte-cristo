use mc_core::world::World;
use mc_tape::divergence::{compare_in_range, compare_replay, compare_tapes};
use mc_tape::format::{Tape, TapeStart};

fn checkpoint_tape(seed: u128) -> Tape {
    let mut world = World::new(seed);
    let mut checkpoints = Vec::new();
    for tick in 1..=5 {
        world.step();
        if tick == 1 || tick == 3 || tick == 5 {
            checkpoints.push((tick, *world.state_hash().as_bytes()));
        }
    }
    let final_hash = *world.state_hash().as_bytes();
    Tape::new(
        seed,
        TapeStart::NewGame,
        Vec::new(),
        checkpoints,
        final_hash,
    )
    .expect("ordered checkpoint tape should be valid")
}

#[test]
fn compare_replay_checks_every_checkpoint_after_the_last_command() {
    let tape = checkpoint_tape(77);
    let report = compare_replay(&tape).expect("valid tape should replay");
    assert_eq!(report.total_checked, 3);
    assert_eq!(report.first_divergent_tick, None);
    assert!(report.divergences.is_empty());

    let mut corrupted = tape.clone();
    corrupted.checkpoints[1].1 = [0xA5; 32];
    let report = compare_replay(&corrupted).expect("corrupt checkpoint is a divergence, not I/O");
    assert_eq!(report.total_checked, 3);
    assert_eq!(report.first_divergent_tick, Some(3));
    assert_eq!(report.divergences.len(), 1);
    assert_eq!(report.divergences[0].0, 3);
}

#[test]
fn tape_and_range_comparisons_use_only_shared_in_range_ticks() {
    let tape = checkpoint_tape(99);
    let mut other = tape.clone();
    other.checkpoints.remove(0);
    other.checkpoints[0].1 = [0x5A; 32];
    other.checkpoints.push((8, [0x11; 32]));

    let shared = compare_tapes(&tape, &other);
    assert_eq!(shared.total_checked, 2);
    assert_eq!(shared.first_divergent_tick, Some(3));
    assert_eq!(shared.divergences.len(), 1);

    let expected = vec![
        (1, tape.checkpoints[0].1),
        (3, [0x33; 32]),
        (5, tape.checkpoints[2].1),
        (8, [0x88; 32]),
    ];
    let ranged = compare_in_range(&tape, &expected, 2, 5);
    assert_eq!(ranged.total_checked, 2);
    assert_eq!(ranged.first_divergent_tick, Some(3));
    assert_eq!(ranged.divergences.len(), 1);
}
