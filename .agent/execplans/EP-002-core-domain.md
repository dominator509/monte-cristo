NODE-META-BEGIN
ID: EP-002
DEPS: EP-001
MAX_ATTEMPTS_PER_MILESTONE: 6
VERIFY: sh scripts/test-unit.sh
VERIFY_SENTINEL: unit tests: ok
GREEN_TAG: green/EP-002
NODE-META-END

# EP-002 -- Core domain (mc_core)

## 1. Purpose / Big Picture

Build the game. Everything that decides what happens lives in this node: fixed-point
arithmetic, the seeded generator, the world tree, the tick order, the ATB resolver, statuses,
the bestiary and region model, terrain-gated spawn eligibility, the anti-grind encounter
budget, the Curriculum, the Chateau d'If calendar, the Paris season clock, the poison and
tolerance model, the Confidence scene model, and the three-phase final encounter. All of it
pure, with no I/O, no clock, no threads, and no floating point.

This is the node where INV-01 is either established or lost. Everything after it depends on
the property being true.

## 2. Scope

`crates/mc_core` in full, plus its unit and property tests. Content is not authored here;
core defines the *types* content will fill and the *mechanisms* content will configure.

## 3. Non-goals

No file I/O of any kind -- not even reading a content file. No RON parsing (EP-003). No save
writing (EP-003). No tape format (EP-004). No rendering (EP-005). No logging. Do not add
`tracing` to mc_core's dependencies for any reason; if you want to observe something, return
a `CoreEvent`.

## 4. Context and Orientation

SPEC-001 is the contract and is authoritative for every type and rule in this node. SPEC-009
holds the locked content vocabulary (regions, families, curriculum grants, poison table,
campaign gating). SPEC-010 holds the Confidence rules. Read all three before starting.

The single most common way to fail this node is to reach for a convenient standard-library
type that breaks determinism. `HashMap` is the usual culprit. It is not merely discouraged:
mc_core must not import it at all.

## 5. Files to Read First

- .agent/specs/SPEC-001-core-domain.md
- .agent/specs/SPEC-009-content-bestiary-and-regions.md
- .agent/specs/SPEC-010-narrative-and-confidences.md
- ARCHITECTURE.md sections 5, 7
- docs/GAME_DESIGN.md sections 1, 3, 5, 6

## 6. Expected Changed Files

- crates/mc_core/src/lib.rs
- crates/mc_core/src/fx.rs
- crates/mc_core/src/rng.rs
- crates/mc_core/src/ids.rs
- crates/mc_core/src/world.rs
- crates/mc_core/src/flags.rs
- crates/mc_core/src/step.rs
- crates/mc_core/src/battle/mod.rs
- crates/mc_core/src/battle/atb.rs
- crates/mc_core/src/battle/damage.rs
- crates/mc_core/src/battle/status.rs
- crates/mc_core/src/bestiary.rs
- crates/mc_core/src/spawn.rs
- crates/mc_core/src/budget.rs
- crates/mc_core/src/curriculum.rs
- crates/mc_core/src/calendar.rs
- crates/mc_core/src/season.rs
- crates/mc_core/src/poison.rs
- crates/mc_core/src/scene.rs
- crates/mc_core/src/final_encounter.rs
- crates/mc_core/src/hash.rs
- crates/mc_core/tests/prop_fixed_point.rs
- crates/mc_core/tests/prop_determinism.rs
- crates/mc_core/tests/prop_spawn_eligibility.rs
- crates/mc_core/tests/battle_resolve.rs
- crates/mc_core/tests/encounter_budget.rs
- crates/mc_core/tests/status_rules.rs
- crates/mc_core/tests/curriculum.rs
- crates/mc_core/tests/poison_tolerance.rs
- crates/mc_core/tests/confidence_flags.rs
- crates/mc_core/tests/final_encounter.rs
- crates/mc_core/tests/atb_wait_mode.rs
- crates/mc_core/Cargo.toml

## 7. Interfaces and Contracts

Every type name, field name, and rule comes from SPEC-001. Every identifier
(`ENM_*`, `R01`..`R15`, `CHR_*`, `PSN_*`, `ABL_*`, flag names) comes from SPEC-009 and
SPEC-010 and is used exactly as written. Do not invent an identifier; if one appears missing,
that is a spec defect to be fixed by the spec-update rule, not improvised in code.

## 8. Milestones

### M1: Fixed point
GOAL: `Fx` exists, saturates rather than wrapping, and never panics.
READ: SPEC-001 section 1, ARCHITECTURE.md INV-02
CHANGE: crates/mc_core/src/fx.rs, crates/mc_core/src/lib.rs, crates/mc_core/tests/prop_fixed_point.rs
CONTENT: `Fx(i32)` Q16.16 with `from_int`, `to_int_floor`, `add`, `sub`, `mul` (i64
  intermediate then `>> 16`), `div` (i64 numerator `<< 16`), all saturating; `Display`;
  `serde` derive; `Ord`. No `f32`, no `f64`, no `From<f32>`, not even in tests. The property
  test asserts over 1,000,000 generated pairs that no operation panics and that saturation
  clamps at `i32::MAX`/`i32::MIN`.
RUN:
  cargo test --locked -p mc_core --test prop_fixed_point
  grep -c 'f32\|f64' crates/mc_core/src/fx.rs
EXPECT: test passes; grep prints `0`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-002 MILESTONE_PASS "M1 fx property tests pass, zero floats"
FALLBACK: if Q16.16 proves too coarse for a damage curve, move to Q24.8 with a wider i64
  backing and update SPEC-001 section 1 by the spec-update rule. Never introduce a float.
COMMIT: git add -A && git commit -m "[EP-002][M1] add saturating Q16.16 fixed point"

### M2: Deterministic RNG and identifiers
GOAL: A seeded PCG64 with unbiased range selection, and the locked identifier types.
READ: SPEC-001 section 2, SPEC-009 sections 1, 2, 5
CHANGE: crates/mc_core/src/rng.rs, crates/mc_core/src/ids.rs
CONTENT: `Rng` with a 128-bit state, `next_u32`, `next_range(lo, hi)` by rejection sampling
  (no modulo bias), and `weighted_pick(&BTreeMap<Id, u32>)`. Identifier newtypes
  `RegionId`, `EnemyId`, `CharId`, `ItemId`, `AbilityId`, `TechId`, `SceneId`, `FlagId`,
  `PoisonId`, each a compact interned index with a `&'static str` table for the built-in
  vocabulary from SPEC-009.
RUN:
  cargo test --locked -p mc_core --lib rng
  grep -c 'HashMap\|HashSet' crates/mc_core/src/rng.rs crates/mc_core/src/ids.rs
EXPECT: tests pass; grep prints `0` for both files
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-002 MILESTONE_PASS "M2 rng and ids, zero hash maps"
FALLBACK: none needed -- PCG64 is fully specified and small.
COMMIT: git add -A && git commit -m "[EP-002][M2] add seeded PCG64 and locked identifier types"

### M3: World, flags, and tick order
GOAL: The `World` tree exists and `step::ORDER` is declared exactly as SPEC-001 section 4.
READ: SPEC-001 sections 3, 4, ARCHITECTURE.md section 7
CHANGE: crates/mc_core/src/world.rs, crates/mc_core/src/flags.rs, crates/mc_core/src/step.rs
CONTENT: `World` with the fields listed in SPEC-001 section 3, in that order. `FlagSet` as a
  sorted bitset with `set`, `clear`, `is_set`, and `satisfies(&FlagExpr)` supporting
  `Always`, `All`, `Any`, `Not`. `step::ORDER` as a const array of the thirteen system names
  in the exact order given, and `step()` dispatching over it. No dynamic registration.
RUN:
  cargo test --locked -p mc_core --lib step
  grep -c 'HashMap\|HashSet\|SystemTime\|std::thread' crates/mc_core/src/world.rs
EXPECT: tests pass; grep prints `0`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-002 MILESTONE_PASS "M3 world and tick order"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-002][M3] add World, FlagSet, and the declared tick order"

### M4: Determinism property test
GOAL: The project's defining property is asserted before more logic is added to threaten it.
READ: SPEC-001 section 16, TESTING.md section 6
CHANGE: crates/mc_core/src/hash.rs, crates/mc_core/tests/prop_determinism.rs
CONTENT: `World::state_hash()` via blake3 over a canonical postcard encoding with sorted map
  iteration. The property test generates 10,000 random command sequences, applies each to two
  clones of the same world, and asserts the hashes are equal at every step. It also asserts
  that a world round-tripped through postcard hashes identically.
RUN:
  cargo test --locked -p mc_core --test prop_determinism
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-002 MILESTONE_PASS "M4 determinism property holds"
FALLBACK: none needed. If this fails, the cause is a defect in M1 through M3 and is fixed
  there; do not weaken the property.
COMMIT: git add -A && git commit -m "[EP-002][M4] add state hashing and the determinism property test"

### M5: Bestiary, spawn eligibility, and the encounter budget
GOAL: `eligible(region, flags)` is pure, and repeat experience decays to zero.
READ: SPEC-001 sections 10, 11, SPEC-009 sections 2, 3, ARCHITECTURE.md INV-11, INV-12
CHANGE: crates/mc_core/src/bestiary.rs, crates/mc_core/src/spawn.rs, crates/mc_core/src/budget.rs,
  crates/mc_core/tests/prop_spawn_eligibility.rs, crates/mc_core/tests/encounter_budget.rs
CONTENT: `Family` as a closed enum with exactly the ten variants in SPEC-009 section 2.
  `Enemy` with `region_affinity: Vec<RegionId>` and `gate: FlagExpr`. `eligible` taking only
  `(RegionId, &FlagSet)` and returning a `Vec<EnemyId>` sorted by id -- no other inputs, no
  randomness, no special cases. `EncounterBudget` with `pool`, `spent`, and the 7/10
  compounding decay flooring at zero. The property test runs 500 rolls per region across all
  fifteen and asserts every result declares that region in its affinity and satisfies its
  gate. The budget test re-enters a region 40 times and asserts strict decay to zero.
RUN:
  cargo test --locked -p mc_core --test prop_spawn_eligibility
  cargo test --locked -p mc_core --test encounter_budget
EXPECT: both tests pass
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-002 MILESTONE_PASS "M5 spawn eligibility pure, budget decays"
FALLBACK: if the 7/10 decay reaches zero too slowly for the 40-entry assertion, adjust the
  ratio in SPEC-001 section 11 by the spec-update rule and update both the code and the test
  together. Never special-case the test.
COMMIT: git add -A && git commit -m "[EP-002][M5] add bestiary model, terrain-gated spawns, anti-grind budget"

### M6: ATB battle, damage, statuses, wait mode
GOAL: A battle resolves deterministically, wounds persist, and Wait mode halts gauges.
READ: SPEC-001 sections 5, 6, docs/GAME_DESIGN.md section 3
CHANGE: crates/mc_core/src/battle/mod.rs, crates/mc_core/src/battle/atb.rs,
  crates/mc_core/src/battle/damage.rs, crates/mc_core/src/battle/status.rs,
  crates/mc_core/tests/battle_resolve.rs, crates/mc_core/tests/status_rules.rs,
  crates/mc_core/tests/atb_wait_mode.rs
CONTENT: ATB gauges in `Fx` filling at `speed / 60` per tick. Actions `Attack`, `Tech`,
  `Item`, `Guard`, `Flee`. Damage `base * atk / (atk + def)` in `Fx`, then status multipliers,
  then a critical roll from `Rng`. The eight statuses of SPEC-001 section 6 with their
  stacking rules, including that `Terror` has no effect on `Family::BEAST` or
  `Family::VERMIN`. Battle end writes remaining HP back to the party (wounds persist, design
  law L6). `SetWaitMode` halts all gauges whenever a menu is open.
RUN:
  cargo test --locked -p mc_core --test battle_resolve
  cargo test --locked -p mc_core --test status_rules
  cargo test --locked -p mc_core --test atb_wait_mode
EXPECT: all three pass
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-002 MILESTONE_PASS "M6 battle, statuses, wait mode"
FALLBACK: if positional area techs prove too complex to resolve deterministically at this
  stage, ship single-target and line techs only, record an ADR, and add area techs in EP-007.
  A reduced-but-real tech set is a legitimate fallback; a stubbed one is not.
COMMIT: git add -A && git commit -m "[EP-002][M6] add ATB battle, damage model, statuses, wait mode"

### M7: Curriculum, calendar, season clock
GOAL: 168 months of Act II progress correctly and grant the SPEC-009 abilities.
READ: SPEC-001 sections 7, 8, 9, SPEC-009 section 6
CHANGE: crates/mc_core/src/curriculum.rs, crates/mc_core/src/calendar.rs,
  crates/mc_core/src/season.rs, crates/mc_core/tests/curriculum.rs
CONTENT: seven disciplines, ranks 0-5, thresholds at 1/3/7/13/21 cumulative months, grants
  exactly as SPEC-009 section 6 and idempotent on re-application. `IfCalendar` of 168 months
  with the four actions, Faria joining unconditionally at month 72. `SeasonClock` of 24
  fortnights with scheduled events firing regardless of player attention.
RUN:
  cargo test --locked -p mc_core --test curriculum
EXPECT: test passes, including an assertion that Faria joins at month 72 and that four
  disciplines can reach rank 3 within 168 months
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-002 MILESTONE_PASS "M7 curriculum and calendars"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-002][M7] add curriculum, If calendar, Paris season clock"

### M8: Poison and tolerance
GOAL: Valentine survives by simulated brucine tolerance, not by a script.
READ: SPEC-001 section 14, SPEC-009 section 7
CHANGE: crates/mc_core/src/poison.rs, crates/mc_core/tests/poison_tolerance.rs
CONTENT: the five compounds with the exact onset, potency, tolerance step, and lethal dose
  values in SPEC-009 section 7. Tolerance accrues per character per compound on surviving a
  sub-lethal dose and decays slowly. The test administers PSN_BRUCINE at 0.5 across 18
  simulated days and asserts the resulting tolerance exceeds 1.6, so that Heloise's dose is
  survived -- and asserts that skipping the regimen kills.
RUN:
  cargo test --locked -p mc_core --test poison_tolerance
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-002 MILESTONE_PASS "M8 poison tolerance simulated"
FALLBACK: if the tolerance curve cannot be tuned to produce the required outcome with the
  SPEC-009 numbers, adjust the numbers in SPEC-009 by the spec-update rule and record an ADR.
  Never hardcode Valentine's survival.
COMMIT: git add -A && git commit -m "[EP-002][M8] add poison model with cumulative tolerance"

### M9: Confidences and the final encounter
GOAL: Dialogue carries only Trust and Mask; phase 2 is damage-immune and flag-gated.
READ: SPEC-001 sections 12, 13, 15, SPEC-010, ARCHITECTURE.md INV-14
CHANGE: crates/mc_core/src/scene.rs, crates/mc_core/src/final_encounter.rs,
  crates/mc_core/tests/confidence_flags.rs, crates/mc_core/tests/final_encounter.rs
CONTENT: `SceneState` as a position in a branching tree with `SceneAdvance` and
  `SceneChoose`. Its effects are limited to flags, trust, mask, item grant or consume, and
  next node. It has no hit points, no turn order, and no meters, and the type must make that
  impossible rather than merely unused. `FinalEncounter` with `Phase1`, `Phase2`, `Phase3`;
  no damage path transitions out of `Phase2`; `Command::NameYourself` is rejected unless
  `MORCERF_YANINA_DOSSIER`, `MORCERF_ALBERT_WITHDRAWN`, and `MERCEDES_RECOGNITION` are all
  set. The test asserts that 10,000 ticks of maximum damage in Phase2 does not end it, and
  that `NameYourself` is rejected with each of the three flags individually absent.
RUN:
  cargo test --locked -p mc_core --test confidence_flags
  cargo test --locked -p mc_core --test final_encounter
EXPECT: both pass
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-002 MILESTONE_PASS "M9 confidences and final encounter gated"
FALLBACK: none needed -- both are small state machines and fully specified.
COMMIT: git add -A && git commit -m "[EP-002][M9] add Confidence scenes and the three-phase final encounter"

### M10: Node verification
GOAL: The whole core suite is green and the invariant greps are clean.
READ: ARCHITECTURE.md section 14
CHANGE: (none)
CONTENT: none.
RUN:
  sh scripts/test-unit.sh
  grep -rc 'f32\|f64\|HashMap\|HashSet\|SystemTime\|std::thread' crates/mc_core/src/ | grep -v ':0' || echo "clean"
  cargo tree -p mc_core --depth 1
EXPECT: `unit tests: ok`; the grep pipeline prints `clean`; the tree shows no project crate
  and no `tracing`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-002 MILESTONE_PASS "M10 unit tests: ok, core clean"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-002][M10] verify core domain suite and invariants"

## 9. Validation and Acceptance

| Criterion | Command | Expected |
|---|---|---|
| No floats in core | `grep -rc 'f32\|f64' crates/mc_core/src/` | every file `0` |
| No hash iteration in core | `grep -rc 'HashMap\|HashSet' crates/mc_core/src/` | every file `0` |
| No clock or threads in core | `grep -rc 'SystemTime\|std::thread' crates/mc_core/src/` | every file `0` |
| Determinism holds | `cargo test --locked -p mc_core --test prop_determinism` | pass |
| Spawns are terrain gated | `cargo test --locked -p mc_core --test prop_spawn_eligibility` | pass |
| Budget floors at zero | `cargo test --locked -p mc_core --test encounter_budget` | pass |
| Wounds persist | `cargo test --locked -p mc_core --test battle_resolve` | pass |
| Terror spares beasts | `cargo test --locked -p mc_core --test status_rules` | pass |
| Valentine survives by tolerance | `cargo test --locked -p mc_core --test poison_tolerance` | pass |
| Phase 2 damage-immune and gated | `cargo test --locked -p mc_core --test final_encounter` | pass |
| Core has no logging dependency | `cargo tree -p mc_core --depth 1` | no `tracing` |
| Node gate | `sh scripts/test-unit.sh` | `unit tests: ok` |

## 10. Idempotence and Recovery

Each milestone adds one module and its tests. To re-enter cold: read Progress, find the first
unchecked milestone, re-run the previous milestone's RUN to confirm its EXPECT, and continue.
If the tree is dirty, reset to the last `[EP-002][M<k>]` commit; every milestone is a complete
unit and redoing one from the top is cheap. Never resume mid-module by guessing what was left
half-written -- reset and redo.

## 11. Progress

- [ ] M1 fixed point
- [ ] M2 deterministic RNG and identifiers
- [ ] M3 world, flags, and tick order
- [ ] M4 determinism property test
- [ ] M5 bestiary, spawn eligibility, encounter budget
- [ ] M6 ATB battle, damage, statuses, wait mode
- [ ] M7 curriculum, calendar, season clock
- [ ] M8 poison and tolerance
- [ ] M9 confidences and the final encounter
- [ ] M10 node verification

## 12. Surprises and Discoveries

<empty>

## 13. Decision Log

<empty>

## 14. Outcomes and Retrospective

- All twelve acceptance rows are met. Historical M1-M10 ledger evidence covers the
  no-float/no-hash/no-clock invariants and each named domain test; the 2026-08-01 clean
  workspace verify re-ran the core suite successfully.
- Changed-files audit: 38 paths changed at the original green-tag boundary. Declared core
  modules and tests were present except `prop_fixed_point.rs`, which was added and verified
  in the recorded reconciliation commit after the first tag. Extras were this ExecPlan,
  the L6 ledger, Cargo.lock, and the generated spawn proptest regression file; the Decision
  Log already records those deviations.
- Retrospective: current tests prove the reconciled deterministic domain behavior.

## 13. Decision Log

| Date | Event | Detail |
|------|-------|--------|
| 2026-07-28 | DRIFT_FIX | `crates/mc_core/tests/prop_fixed_point.rs` was missing despite being in M1's CHANGE list. Created in reconciliation pass with 10 property tests covering all operations (no-panic, saturation bounds, round-trip). Also tolerates: `.agent/state/LEDGER.md` (L6 state), `Cargo.lock` (dependency management), and `*.proptest-regressions` (auto-generated) as natural extra files outside Expected Changed Files. |
