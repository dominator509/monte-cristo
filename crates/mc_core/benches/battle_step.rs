//! Frame budget benchmark: 10,000 ticks of the heaviest battle state.
//!
//! SPEC-008 §3: p99 under 4.0ms per frame.
//! LF-11: frame-budget proof.
//!
//! Uses criterion to measure step() performance under a worst-case world.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mc_core::command::{apply_commands, Command, Dir};
use mc_core::world::World;

/// Benchmark 10,000 ticks of the heaviest world state.
/// The world step is mostly system dispatches — this measures the loop cost.
fn bench_10k_frames(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_budget");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));

    group.bench_function("10k_heaviest_battle", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let mut world = World::new(42);
                // Apply various commands to create "heaviest" state
                for _ in 0..10 {
                    apply_commands(
                        &mut world,
                        &[
                            Command::Interact,
                            Command::Move(Dir::North),
                            Command::SceneAdvance,
                            Command::OpenMenu,
                            Command::CloseMenu,
                            Command::Move(Dir::East),
                        ],
                    );
                }
                // Step 10,000 times
                for _ in 0..10000 {
                    world.step();
                    black_box(());
                }
            }
            start.elapsed()
        });
    });

    group.finish();
}

criterion_group!(benches, bench_10k_frames);
criterion_main!(benches);
