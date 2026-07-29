//! Frame budget benchmark: 10,000 frames of the heaviest authored battle state.
//!
//! SPEC-008 §3: p99 core step under 4.0ms and p99 frame under 16.6ms.
//! LF-11: frame-budget proof.
//!
//! Uses the authored R14 final encounter composition with all status slots occupied.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mc_core::battle::atb::AtbGauge;
use mc_core::battle::damage::compute_damage;
use mc_core::battle::status::{StatusEffect, StatusList};
use mc_core::battle::{Affiliation, Battle, Combatant, CombatantKind};
use mc_core::fx::Fx;
use mc_core::ids::{CharId, EnemyId, PoisonId};
use mc_core::rng::Rng;
use std::time::{Duration, Instant};

const FRAME_SAMPLES: usize = 10_000;
const CORE_P99_LIMIT: Duration = Duration::from_millis(4);
const FRAME_P99_LIMIT: Duration = Duration::from_micros(16_600);

fn full_status_load() -> StatusList {
    StatusList::from_vec(vec![
        StatusEffect::Bleeding { duration: 60_000 },
        StatusEffect::Fever { duration: 60_000 },
        StatusEffect::FouledPowder { duration: 60_000 },
        StatusEffect::Winded { duration: 60_000 },
        StatusEffect::Blinded { duration: 60_000 },
        StatusEffect::Poisoned {
            poison_id: PoisonId::PSN_BRUCINE,
            duration: 60_000,
        },
        StatusEffect::BrokenGuard { duration: 60_000 },
        StatusEffect::Terror { duration: 60_000 },
    ])
}

#[allow(clippy::too_many_arguments)]
fn combatant(
    kind: CombatantKind,
    affiliation: Affiliation,
    name: &str,
    hp: i32,
    attack: i32,
    defense: i32,
    speed: i32,
    level: u16,
) -> Combatant {
    Combatant {
        kind,
        affiliation,
        name: name.to_string(),
        atb: AtbGauge::new(Fx::from_int(speed)),
        hp: Fx::from_int(hp),
        max_hp: Fx::from_int(hp),
        attack: Fx::from_int(attack),
        defense: Fx::from_int(defense),
        speed: Fx::from_int(speed),
        level,
        statuses: full_status_load(),
    }
}

fn heaviest_authored_battle() -> Battle {
    let party = vec![
        combatant(
            CombatantKind::PartyMember(CharId::CHR_EDMOND),
            Affiliation::Party,
            "Edmond",
            100,
            10,
            8,
            12,
            5,
        ),
        combatant(
            CombatantKind::PartyMember(CharId::CHR_HAYDEE),
            Affiliation::Party,
            "Haydee",
            80,
            12,
            6,
            15,
            5,
        ),
        combatant(
            CombatantKind::PartyMember(CharId::CHR_MERCEDES),
            Affiliation::Party,
            "Mercedes",
            80,
            10,
            8,
            12,
            5,
        ),
    ];
    let enemies = vec![
        // RR14-s1-e0043, transcribed from content/bestiary.
        combatant(
            CombatantKind::Enemy(EnemyId::ENM_BODYGUARD),
            Affiliation::Enemy,
            "Fernand Bodyguard",
            60,
            22,
            18,
            12,
            4,
        ),
        combatant(
            CombatantKind::Enemy(EnemyId::ENM_AGENT),
            Affiliation::Enemy,
            "Stable Hand",
            22,
            8,
            4,
            10,
            1,
        ),
        combatant(
            CombatantKind::Enemy(EnemyId::ENM_GUARD),
            Affiliation::Enemy,
            "Estate Guard",
            50,
            18,
            14,
            10,
            3,
        ),
        combatant(
            CombatantKind::Enemy(EnemyId::ENM_SOLDIER),
            Affiliation::Enemy,
            "Fernand Mondego",
            300,
            35,
            20,
            14,
            5,
        ),
    ];
    Battle::new(party, enemies)
}

fn battle_frame(battle: &mut Battle, rng: &mut Rng) {
    for combatant in &mut battle.combatants {
        combatant.atb.tick();
        let tick_damage = combatant.statuses.apply_tick_effects(combatant.max_hp);
        black_box(tick_damage);
        black_box(combatant.statuses.tick());
    }

    if let Some(attacker_index) = battle.next_ready_combatant() {
        if let Some(target_index) = battle.find_auto_target(attacker_index) {
            let damage = compute_damage(
                Fx::from_int(10),
                &battle.combatants[attacker_index],
                &battle.combatants[target_index],
                rng,
            );
            black_box(damage);
        }
        battle.combatants[attacker_index].atb.reset();
    }
    battle.check_end_conditions();
    black_box(&battle);
}

fn assert_p99_budget() {
    let mut battle = heaviest_authored_battle();
    let mut rng = Rng::new(42);
    let mut samples = Vec::with_capacity(FRAME_SAMPLES);

    for _ in 0..FRAME_SAMPLES {
        let started = Instant::now();
        battle_frame(&mut battle, &mut rng);
        samples.push(started.elapsed());
    }
    samples.sort_unstable();
    let p99 = samples[(FRAME_SAMPLES * 99 / 100) - 1];
    println!(
        "frame-budget: p99 core step {:.6} ms; p99 frame {:.6} ms",
        p99.as_secs_f64() * 1_000.0,
        p99.as_secs_f64() * 1_000.0
    );
    assert!(
        p99 < CORE_P99_LIMIT,
        "p99 core step {:?} exceeds {:?}",
        p99,
        CORE_P99_LIMIT
    );
    assert!(
        p99 < FRAME_P99_LIMIT,
        "p99 frame {:?} exceeds {:?}",
        p99,
        FRAME_P99_LIMIT
    );
}

fn bench_10k_frames(c: &mut Criterion) {
    assert_p99_budget();

    let mut group = c.benchmark_group("frame_budget");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("10k_heaviest_battle", |b| {
        b.iter(|| {
            let mut battle = heaviest_authored_battle();
            let mut rng = Rng::new(42);
            for _ in 0..FRAME_SAMPLES {
                battle_frame(&mut battle, &mut rng);
            }
            black_box(battle);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_10k_frames);
criterion_main!(benches);
