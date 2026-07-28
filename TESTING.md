# TESTING -- MONTE CRISTO

L5 VERIFICATION. Gates never weaken mid-run. You may fix code to satisfy a gate; you may
never edit a gate to satisfy code.

## 1. The pyramid, and why it is shaped this way

Because mc_core is pure and deterministic (INV-01), the expensive end of the pyramid is
cheap here. A full-campaign end-to-end test is a headless tape replay that finishes in under
15 minutes with no browser, no driver, no container, and no flake surface. That inverts the
usual economics, and this project exploits it deliberately.

| Level | Subject | Where | Runner |
|---|---|---|---|
| Unit | pure functions in mc_core and mc_data | `crates/*/src/**` `#[cfg(test)]` | `scripts/test-unit.sh` |
| Property | combat, poison, fixed point, spawn eligibility | `crates/mc_core/tests/prop_*.rs` | `scripts/test-unit.sh` |
| Integration | real files: bake, load, save, migrate, confinement | `crates/*/tests/*.rs` | `scripts/test-integration.sh` |
| End to end | tape replay through the real command bus | `crates/mc_tape/tests/e2e_*.rs` | `scripts/test-e2e.sh` |
| Live fire | the twelve core outcomes | `scripts/live-fire.sh` | `scripts/live-fire.sh` |
| Fuzz | the two untrusted parsers | `fuzz/fuzz_targets/*.rs` | `scripts/security-check.sh` |
| Bench | frame and step budgets | `crates/mc_core/benches/*.rs` | `scripts/smoke-test.sh` |

## 2. TEST DOUBLE ZONE

Mocks, fakes, and fixtures are legal **only** in these paths:

    crates/*/tests/**
    crates/*/benches/**
    crates/*/src/**  inside #[cfg(test)] modules only
    fuzz/**
    tests/fixtures/**

Everywhere else is a production path and the reality gate enforces it.

Even inside the zone: integration tests use real files on a real filesystem; e2e tests drive
the real command bus with the real content pack; live-fire uses the real binary. There is no
in-memory impostor of the save layer, because the save layer is where the bugs live.

## 3. Mocking rules

- Never mock the thing under test. A test that asserts on a mock of the combat resolver
  tests the mock.
- mc_core needs no mocks at all. It is pure; give it a state and a command and assert on the
  next state. If you find yourself wanting a mock in a core test, the code has an I/O
  dependency it should not have (INV-01).
- The only legitimate doubles in this project are: a temporary directory standing in for
  MC_DATA_DIR, a small hand-authored content pack standing in for the full one, and a
  deliberately corrupted byte buffer standing in for a damaged save. All three are real data,
  not simulated behaviour.

## 4. Required tests per feature

Every feature merges with all of:
1. a unit test of the pure logic,
2. an entry in the determinism property test if it touches `step::ORDER`,
3. an integration test if it touches a file,
4. a tape-level assertion if it changes observable game state,
5. a content validator rule if it introduces a content type,
6. a live-fire assertion if it touches one of the twelve core outcomes.

## 5. Forced-failure tests (real error paths, not simulated ones)

These prove handling rather than describing it. All are required by EP-007.

| Forced failure | How it is really forced | Expected |
|---|---|---|
| Truncated save | write a real save, truncate the file to 60 percent | typed `SaveError::Truncated`, no panic |
| Corrupted save digest | flip one byte in the payload | typed `SaveError::DigestMismatch`, no panic |
| Future schema version | write a save with version + 1 | typed `SaveError::UnsupportedVersion`, refusal, no guessing |
| Content pack digest mismatch | flip one byte in content.pack | refuse to load, exit nonzero with a clear message |
| Dangling content reference | add a scene referencing a missing enemy id | bake fails with `CONTENT_DANGLING_REF` naming the file and line |
| Supernatural family | add a bestiary entry with family `DEMON` | bake fails with the closed-set error |
| Path traversal | request `../../etc/passwd` through the save API | `fsroot::confine` rejects before any open |
| Disk full on save | write into a size-limited loopback or a full tmpfs | typed error, previous save left intact |
| Read-only data dir | chmod the directory | typed error, clear message, no partial write |
| Missing content pack | delete it | clear startup failure, exit nonzero |

## 6. Determinism testing (the project's defining suite)

1. **Property:** for 10,000 random command sequences, `step` applied twice from a cloned
   state yields identical state hashes.
2. **Cross-run:** the same tape replayed in two separate processes yields identical hashes.
3. **Cross-profile:** debug and release builds yield identical hashes.
4. **Cross-platform:** each artifact produced in EP-009 replays the golden tape and yields
   the hash recorded on Linux. This is a ship gate.
5. **Regression:** every committed tape carries its expected terminal hash in
   `tapes/HASHES.txt`; `scripts/test-e2e.sh` asserts every one.

A determinism failure is never worked around, never quarantined, and never marked flaky. Its
signature is `DETERMINISM_HASH_MISMATCH` and the cause is always one of the four listed in
.agent/LOOPS.md.

## 7. Fixtures and test data

Fixtures live in `tests/fixtures/`. The small content pack used by integration tests is
`tests/fixtures/mini-content/` and is authored, committed, and validated by the same bake as
the real tree. Every test that writes creates its own temporary directory under the system
temp root and removes it in a guard, including on panic. No test writes into MC_DATA_DIR.
No test leaves a file behind; `scripts/test-integration.sh` asserts a clean tree afterwards.

## 8. Coverage floors

Measured by `cargo llvm-cov`. These are floors, not targets, and EP-007 raises the code to
meet them rather than lowering them.

| Crate | Line floor | Branch floor |
|---|---|---|
| mc_core | 90 percent | 85 percent |
| mc_data | 85 percent | 80 percent |
| mc_tape | 85 percent | 80 percent |
| mc_shell | 60 percent | 50 percent |
| mc_tools | 70 percent | 60 percent |

mc_shell's floor is lower because rendering correctness is proven by the e2e entry-point
tests and by human review of the visual result, not by line coverage. That is a stated
trade-off, not an oversight.

## 9. Flaky-test policy

A flaky test is a bug. Fix it or delete it with an ADR naming what coverage was lost and
what replaces it. Never retry until green, never add a sleep, never mark it ignored. In this
project a flaky test in mc_core is almost certainly an INV-01 violation, and is therefore a
severity-one finding rather than a nuisance.

## 10. Validation matrix

Every specified behaviour maps to a test path. The full matrix lives in each spec's
Validation section; this table maps the twelve core outcomes to their proofs.

| Outcome | Live-fire proof | Backing test |
|---|---|---|
| LF-01 | `lf01_new_game_to_arrest` | `crates/mc_tape/tests/e2e_act1.rs` |
| LF-02 | `lf02_if_calendar_and_curriculum` | `crates/mc_core/tests/curriculum.rs` |
| LF-03 | `lf03_field_encounter_resolves` | `crates/mc_core/tests/battle_resolve.rs` |
| LF-04 | `lf04_terrain_gated_spawns` | `crates/mc_core/tests/prop_spawn_eligibility.rs` |
| LF-05 | `lf05_encounter_budget_no_grind` | `crates/mc_core/tests/encounter_budget.rs` |
| LF-06 | `lf06_confidence_scene_gates_story` | `crates/mc_core/tests/confidence_flags.rs` |
| LF-07 | `lf07_save_load_state_identity` | `crates/mc_data/tests/save_roundtrip.rs` |
| LF-08 | `lf08_golden_tape_full_run` | `crates/mc_tape/tests/e2e_golden.rs` |
| LF-09 | `lf09_determinism_cross_run` | `crates/mc_tape/tests/e2e_determinism.rs` |
| LF-10 | `lf10_content_integrity` | `crates/mc_data/tests/content_integrity.rs` |
| LF-11 | `lf11_frame_budget` | `crates/mc_core/benches/battle_step.rs` |
| LF-12 | `lf12_final_boss_two_phase` | `crates/mc_core/tests/final_encounter.rs` |

## 11. Definition of test-done

All gate sentinels observed in one fresh `scripts/verify.sh` run in this session; coverage
floors met; every forced-failure test present and passing; every committed tape matching its
recorded hash; zero ignored tests; zero `#[should_panic]` used to paper over an error path
that should return a typed error instead.
