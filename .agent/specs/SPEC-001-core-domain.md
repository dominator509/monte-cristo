# SPEC-001 -- Core domain (mc_core)

Pure, deterministic, no I/O. Behaviour first. Vocabulary locked.

## 1. Fixed-point arithmetic

Type `Fx`: Q16.16, backed by `i32`. Operations: add, sub, mul (i64 intermediate, arithmetic
shift right 16), div (i64 numerator shifted left 16), `from_int`, `to_int_floor`,
`saturating_*`. Overflow saturates and records a `CoreEvent::ArithmeticSaturated`; it never
wraps and never panics.

No `f32` or `f64` appears in mc_core (INV-02).

## 2. Random source

`Rng`: PCG64 with a 128-bit state, seeded from `World.seed`. Advanced only through explicit
calls; there is no ambient generator. Every call site records the call in the tick's event
list under `debug-overlay`, which is how a determinism divergence is localised.

`next_u32`, `next_range(lo, hi)` (rejection sampling, uniform, no modulo bias),
`weighted_pick(&BTreeMap<Id, Weight>)`.

## 3. World

    World {
      seed: u128,
      tick: u64,
      act: Act,
      region: RegionId,
      party: Party,               // up to 3 active, roster of 11
      flags: FlagSet,             // sorted bitset over the locked flag vocabulary
      trust: BTreeMap<CharId, i16>,
      mask: i16,                  // 0..=100
      curriculum: Curriculum,
      inventory: Inventory,
      budgets: BTreeMap<(RegionId, ChapterId), EncounterBudget>,
      battle: Option<Battle>,
      scene: Option<SceneState>,
      calendar: Option<IfCalendar>,   // Some only during ACT_II_IF
      season: Option<SeasonClock>,    // Some only during ACT_VI_PARIS
      rng: Rng,
    }

One owned tree. No interior mutability, no reference cycles, no ECS, no event bus (INV-01).

## 4. Tick order

`step::ORDER` is declared once and is the only place system order is expressed:

1. `scene_advance`
2. `calendar_advance`
3. `season_advance`
4. `field_movement`
5. `spawn_resolution`
6. `encounter_contact`
7. `battle_atb`
8. `battle_action_resolve`
9. `status_tick`
10. `poison_tick`
11. `budget_decay`
12. `flag_reactions`
13. `event_flush`

Adding a system means adding it here and adding it to the determinism property test. There is
no dynamic registration.

## 5. ATB battle

Chrono Trigger model, fought on the field map with no arena transition.

- Each combatant has `atb: Fx` in 0..=1. Fill rate is `speed / 60` per tick, modified by
  statuses. At 1.0 the combatant may act.
- **Wait mode** (accessibility, SPEC-004): when enabled and a menu is open, all gauges halt.
  This is core state, not a shell setting, so it is part of the replay.
- Actions: `Attack`, `Tech(TechId)`, `Item(ItemId)`, `Guard`, `Flee`.
- Positional area techs use the field grid; targets are selected by distance in `Fx`.
- Dual and triple techs require all participants at 1.0 simultaneously and are offered in the
  menu only then.
- Damage: `base * (attack / (attack + defence))` in `Fx`, then status multipliers, then a
  critical roll. Saturating throughout.
- **Wounds persist** (design law L6): battle end writes remaining HP back to the party; only
  a `RestPoint` scene restores fully.
- Pre-emptive strike on contacting an enemy from behind; back attack on being contacted from
  behind. Determined by facing at the contact tick, not by animation.

## 6. Statuses

Closed set: `Bleeding`, `Fever`, `FouledPowder`, `Winded`, `Blinded`, `Poisoned(PoisonId)`,
`BrokenGuard`, `Terror`. Each has a duration in ticks, a per-tick effect, and a stacking rule
(none stack with themselves; `Terror` does not apply to `family: BEAST` or `family: VERMIN`).

## 7. Curriculum

Seven disciplines, ranks 0 through 5: `FENCING`, `CHEMISTRY`, `NATURAL_PHILOSOPHY`,
`MATHEMATICS`, `LANGUAGES`, `HISTORY_POLITICS`, `ECONOMICS`.

During `ACT_II_IF`, a Study action spends one calendar month and grants progress; rank
thresholds are 1, 3, 7, 13, 21 months cumulative per discipline. After Act II, tutors grant
progress at authored scenes only.

Each rank grants specific abilities, listed in the locked table in SPEC-009 section 6.
Grants are idempotent: re-applying a rank never duplicates an ability.

## 8. Chateau d'If calendar

168 months. Actions per month: `Dig`, `Study(Discipline)`, `Endure`, `Observe`. Faria joins
at month 72 unconditionally. `Dig` progress and `Observe` knowledge are integers feeding the
escape sequence's success conditions. `Endure` restores wound damage; nothing else does
during Act II.

## 9. Season clock

`ACT_VI_PARIS` runs 24 fortnights. Every campaign action costs one. Scheduled events fire at
declared fortnights regardless of player attention -- notably the Villefort poisonings, which
progress on their own timetable.

## 10. Spawn eligibility (INV-11)

    eligible(region: RegionId, flags: &FlagSet) -> Vec<EnemyId>

Pure. An enemy is eligible if and only if `region in enemy.region_affinity` and
`flags.satisfies(enemy.gate)`. There are no other inputs, no special cases, no exceptions,
and no randomness in eligibility -- randomness enters only in `weighted_pick` over the
eligible set. This is what LF-04 asserts.

## 11. Encounter budget (INV-12)

Per `(RegionId, ChapterId)`:

    EncounterBudget { pool: u16, spent: u16, decay_num: u16 = 7, decay_den: u16 = 10 }

Experience for the n-th cleared encounter is `base * (7/10)^n` in `Fx`, floored to zero once
below 1. `pool` is finite and authored; once spent, the region stops spawning. Proven by LF-05.

## 12. Confidences (dialogue -- explicitly not combat)

A `SceneState` is a position in an authored branching tree. It has **no** hit points, **no**
turn order, and **no** resource meters. Its only outputs are: set or clear flags, adjust
`trust[char]` by a small integer, adjust `mask`, grant or consume an item, and select the
next node.

Any pull request that introduces a combat-shaped interface for a Confidence fails review
(ADR-008). This is stated in the spec, not only in the ADR, because the spec is what the
executor reads.

## 13. Trust and Mask

`trust[char]` is `i16`, clamped to -50..=50, never displayed as a number. It surfaces as
scene variant selection. Four characters have a decisive threshold: `CHR_MERCEDES` at 20
(recognition), `CHR_ALBERT` at 15 (public withdrawal), `CHR_HAYDEE` at 25 (willingness to
testify), `CHR_MAXIMILIEN` at 20 (confidence at the grotto).

`mask` is a single `i16` in 0..=100, dropping only at scripted moments. Its only mechanical
effects are persona-to-map access and the epilogue's framing-text variant.

## 14. Poison model

Five compounds: `PSN_BRUCINE`, `PSN_ACONITE`, `PSN_BELLADONNA`, `PSN_ARSENIC`,
`PSN_HYDROCYANIC`. Each has onset (ticks), potency (`Fx` per tick), and a tolerance curve.

Tolerance accumulates per character per compound when a sub-lethal dose is survived, and
decays slowly. Valentine's survival is exactly this mechanism: Noirtier administers rising
brucine doses across authored days until her tolerance exceeds Heloise's lethal dose. It is
not scripted; it is simulated, and the numbers are in SPEC-009.

## 15. Final encounter phase machine (INV-14)

`FinalEncounter { phase: Phase1 | Phase2 | Phase3 }`.

- Phase1: a standard ATB battle against `ENM_FERNAND_GENERAL`, solo (no party).
- At 0 HP, transition to Phase2. Fernand does not die.
- Phase2: the command menu is replaced by four entries. Three are inert. The fourth,
  `Command::NameYourself`, is rejected by `apply_commands` unless `MORCERF_YANINA_DOSSIER`,
  `MORCERF_ALBERT_WITHDRAWN`, and `MERCEDES_RECOGNITION` are all set. Damage cannot end
  Phase2 -- there is no damage path that transitions out of it.
- Phase3 is a scripted pursuit scene with no combat.

Proven by LF-12.

## 16. State hashing

`World::state_hash() -> [u8; 32]` via blake3 over a canonical postcard encoding with sorted
map iteration. It is the definition of "the same state" for every determinism test.

## 17. Validation

| Behaviour | Test |
|---|---|
| Fx never wraps | `crates/mc_core/tests/prop_fixed_point.rs` |
| step is deterministic | `crates/mc_core/tests/prop_determinism.rs` |
| spawn eligibility is pure | `crates/mc_core/tests/prop_spawn_eligibility.rs` |
| budget decays to zero | `crates/mc_core/tests/encounter_budget.rs` |
| wounds persist | `crates/mc_core/tests/battle_resolve.rs` |
| Terror does not apply to beasts | `crates/mc_core/tests/status_rules.rs` |
| brucine tolerance saves Valentine | `crates/mc_core/tests/poison_tolerance.rs` |
| phase 2 is damage-immune and gated | `crates/mc_core/tests/final_encounter.rs` |
| Wait mode halts gauges | `crates/mc_core/tests/atb_wait_mode.rs` |
