# GAPS LOG — Monte Cristo Spec-to-Code Reconciliation

Generated: 2026-07-28

Rules: Every item must be coded in full, then re-reconciled against its spec(s) and
execplan milestone(s). All tests must pass before the item is crossed off.

---

## TIER 1: Fix Existing Code Regressions

These are bugs in code that *should* work but doesn't compile or test correctly.

### G-001: prop_combat.rs unused import `Family`
- **Spec:** SPEC-001 §17 (Validation), SPEC-009 §2
- **ExecPlan:** EP-007 M2
- **File:** `crates/mc_core/tests/prop_combat.rs:11`
- **Issue:** Unused import `use mc_core::bestiary::Family;` causes compile failure under `-D warnings`
- **Fix:** Remove unused import
- **Verification:** `cargo test -p mc_core --test prop_combat` passes
- **Status:** [] PENDING

### G-002: migrate.rs needless question mark
- **Spec:** SPEC-002 (data model)
- **ExecPlan:** EP-003
- **File:** `crates/mc_data/src/migrate.rs:46`
- **Issue:** `Ok(save.to_bytes()?)` — question mark is useless when immediately wrapped in Ok
- **Fix:** Change to `save.to_bytes()`
- **Verification:** `cargo build -p mc_data` passes without clippy errors
- **Status:** [] PENDING

### G-003: advisory_screen.rs test failures (2 tests)
- **Spec:** SPEC-004 §8 (Accessibility), SPEC-004 §12
- **ExecPlan:** EP-005 M8
- **File:** `crates/mc_shell/tests/advisory_screen.rs`
- **Issue:** Two tests fail: `advisory_not_shown_after_acknowledge` and `advisory_acknowledged_persists`
- **Fix:** Debug and fix advisory persistence logic
- **Verification:** `cargo test -p mc_shell --test advisory_screen` passes all tests
- **Status:** [] PENDING

### G-004: mc_tools bake unused `output` variable
- **Spec:** SPEC-003 (API contracts)
- **ExecPlan:** EP-004 M5
- **File:** `crates/mc_tools/src/main.rs:58`
- **Issue:** `Command::Bake { input, output }` — `output` is unused (warning, not error)
- **Fix:** Handle output parameter properly or prefix with underscore
- **Verification:** `cargo build -p mc_tools` emits zero warnings
- **Status:** [] PENDING

---

## TIER 2: Spec Compliance Fixes

Code exists but doesn't match spec requirements.

### G-005: content_invariants.rs wrong reserved identifiers
- **Spec:** SPEC-000 §4 (table), SPEC-009 §9, SPEC-010 §10
- **Files:** `crates/mc_data/tests/content_invariants.rs`
- **Issue:** Reserved identifiers test checks `MERCEDES_ROUTE`, `FERNAND_FORGIVEN`, `POWER_OF_FRIENDSHIP`, `DEUS_EX_MACHINA` instead of spec's `MERCEDES_ROMANCE`, `VILLEFORT_SPARED`, `EDOUARD_SAVED`, `ENDING_ALT`
- **Fix:** Replace with correct reserved identifiers
- **Also covers:** SPEC-000 §4 Mercedes romance invariant, Villefort invariant
- **Verification:** `cargo test -p mc_data --test content_invariants` passes
- **Status:** [] PENDING

### G-006: content_invariants.rs missing Villefort/supernatural/procedural tests
- **Spec:** SPEC-000 §4 (5 invariants: one ending, no Mercedes, Villefort, Edouard, no procedural, no supernatural)
- **Files:** `crates/mc_data/tests/content_invariants.rs`
- **Issue:** Missing tests:
  - Villefort path always reaches `VILLEFORT_MADNESS`
  - Supernatural family check (every bestiary family in closed set)
  - Procedural content check (no generated files; digest comparison)
- **Fix:** Add 3 missing invariant tests
- **Verification:** `cargo test -p mc_data --test content_invariants` passes, all 5 invariants tested
- **Status:** [] PENDING

### G-007: mask starts at 50, spec says 100
- **Spec:** SPEC-001 §13 (Trust and Mask), SPEC-010 §4
- **File:** `crates/mc_core/src/world.rs:148`
- **Issue:** `mask: 50` in World::new() — spec says mask starts at 100
- **Fix:** Change to `mask: 100`
- **Verification:** All tests that depend on mask value updated and pass
- **Status:** [] PENDING

---

## TIER 3: LF-01 Proof Completeness

### G-008: `mc_tools replay` needs `--require-flag <FLAG>` option
- **Spec:** SPEC-000 LF-01, SPEC-003 §4-5
- **File:** `crates/mc_tools/src/cmd_replay.rs`
- **Issue:** Can't assert a story flag was set at end of replay — needed for LF-01 (ACT1_ARREST)
- **Fix:** Add `--require-flag <FLAG>` that checks flag at end of replay, exits nonzero if not set
- **Verification:** `cargo run -p mc_tools -- replay --tape tapes/act1.tape --require-flag ACT1_ARREST` exits 0
- **Status:** [] PENDING

---

## TIER 4: EP-007 M3-M7 (Testing Hardening)

### G-009: Record golden-full.tape and golden-smoke.tape
- **Spec:** SPEC-000 LF-08, SPEC-003 §4-5
- **ExecPlan:** EP-007 M3
- **Files to create:** `tapes/golden-full.tape`, `tapes/golden-smoke.tape`, `tapes/HASHES.txt` (append), `crates/mc_tape/tests/e2e_golden.rs`
- **Issue:** Golden tape for full campaign doesn't exist
- **Fix:** Record tape that plays the full campaign to EPILOGUE_SAIL. Record smoke tape. Add e2e_golden test.
- **Verification:** `cargo test -p mc_tape --test e2e_golden` passes; `hash: match` for golden-full
- **Status:** [] PENDING

### G-010: Build real live-fire.sh (12 proofs)
- **Spec:** SPEC-000 §2 (all 12 LFs), SPEC-008 §1
- **ExecPlan:** EP-007 M4
- **Files:** `scripts/live-fire.sh` (complete rewrite)
- **Issue:** Current live-fire.sh is fabricated — calls `mc prove`, `mc bench`, `--require-flag` which don't exist
- **Fix:** Build real scripts/commands for each proof. Each LF must use real binary, real content, real assertions.
- **Details:**
  - LF-01: replay act1.tape --assert-hash + --require-flag ACT1_ARREST
  - LF-02: replay if_calendar tape, assert 168 months, Faria at 72, 4 disciplines at rank 3
  - LF-03: replay R03 field encounter tape, assert victory + loot + wounds persist
  - LF-04: run spawn eligibility checker for all 15 regions, 500 rolls each
  - LF-05: run encounter budget decay checker
  - LF-06: replay confidence scene tape, assert flag variant
  - LF-07: create save mid-battle, reload, assert hash match
  - LF-08: replay golden-full.tape --assert-hash + --require-flag EPILOGUE_SAIL
  - LF-09: replay golden-full.tape twice, assert same hash
  - LF-10: mc_tools validate --strict-orphans
  - LF-11: frame budget bench
  - LF-12: final encounter phase gating test
- **Verification:** `sh scripts/live-fire.sh` prints 12 ok lines then `live-fire: ok`
- **Status:** [] PENDING

### G-011: Build mc_tools prove subcommand
- **Spec:** SPEC-000 §2 (LF-02 through LF-07, LF-12), SPEC-001 §10-11
- **ExecPlan:** EP-007 M4
- **Files:** `crates/mc_tools/src/cmd_prove.rs` + `main.rs` integration
- **Issue:** Need a `mc_tools prove` command for 6 proofs that can't be done with replay alone
- **Fix:** Implement prove subcommand with sub-verbs: if-calendar, field-encounter, spawn-gating, encounter-budget, confidence-gating, save-identity, final-encounter
- **Verification:** Each sub-command returns exit 0 with correct assertions
- **Status:** [] PENDING

### G-012: Frame budget benchmark + memory ceiling test
- **Spec:** SPEC-000 LF-11, SPEC-008 §3
- **ExecPlan:** EP-007 M5
- **Files:** `crates/mc_core/benches/battle_step.rs`, `crates/mc_tape/tests/memory_ceiling.rs`
- **Issue:** No benchmarks or memory tests exist
- **Fix:** Create criterion bench for 10,000 frames of heaviest battle. Create memory ceiling test for long replay.
- **Verification:** `cargo bench -p mc_core` reports p99 under 4.0ms; memory test passes
- **Status:** [] PENDING

### G-013: Coverage to 85% + zero ignored tests
- **Spec:** SPEC-008 §3, TESTING.md §8
- **ExecPlan:** EP-007 M6
- **Issue:** `cargo llvm-cov` never run. Coverage unknown.
- **Fix:** Run coverage, add tests for uncovered branches to reach 85%. Find and fix/delete ignored tests.
- **Verification:** `cargo llvm-cov --workspace --fail-under-lines 85` exits 0; zero ignored tests
- **Status:** [] PENDING

### G-014: verify.sh passes
- **Spec:** SPEC-008 §1
- **ExecPlan:** EP-007 M7
- **Issue:** verify.sh fails due to G-001, G-002, G-003 and missing live-fire
- **Fix:** All prior items must be green first
- **Verification:** `sh scripts/verify.sh` prints `verify: ok`
- **Status:** [] PENDING

---

## TIER 5: Missing Content Directories

### G-015: Create missing content directories and files
- **Spec:** SPEC-002 §1, SPEC-009, SPEC-010
- **Files to create:** `content/maps/*.ron`, `content/encounters/*.ron`, `content/abilities/*.ron`, `content/techs/*.ron`, `content/strings/en/*.ron`, `content/party.ron`, `content/curriculum.ron`, `content/poisons.ron`
- **Issue:** 8 required directories/files missing from content tree
- **Fix:** Create minimal valid files for each. Party roster (11 members). Curriculum (7 disciplines, rank thresholds). Poisons (5 compounds). Strings (key references). Abilities/techs referenced by bestiary entries. Maps/encounters basic stubs.
- **Verification:** `mc_tools validate --input ./content` prints `content: ok`
- **Status:** [] PENDING

---

## TIER 6: EP-008 (Observability)

### G-016: JSON structured logging
- **Spec:** SPEC-007 §1-2
- **ExecPlan:** EP-008 M1
- **Issue:** No JSON logging infrastructure exists
- **Fix:** Add structured JSON logging to mc_shell, output to `$MC_DATA_DIR/logs/monte-cristo-<date>.jsonl`, rotation, required fields (ts, level, target, msg, session, build, tick)
- **Verification:** `crates/mc_shell/tests/log_schema.rs` passes
- **Status:** [] PENDING

### G-017: Metrics file on clean exit
- **Spec:** SPEC-007 §1.6
- **ExecPlan:** EP-008 M2
- **Issue:** No metrics collection
- **Fix:** Write metrics to `$MC_DATA_DIR/logs/metrics-<date>.json` on clean exit
- **Verification:** `crates/mc_shell/tests/metrics_file.rs` passes
- **Status:** [] PENDING

### G-018: Crash reports
- **Spec:** SPEC-007 §1.8, SPEC-006 §5
- **ExecPlan:** EP-008 M3
- **Issue:** No crash report infrastructure
- **Fix:** Write crash reports to `$MC_DATA_DIR/crash/` with panic message, backtrace, tick, state hash
- **Verification:** `crates/mc_shell/tests/crash_report.rs` passes
- **Status:** [] PENDING

### G-019: Profiler and debug overlay
- **Spec:** SPEC-007 §1.6-1.7
- **ExecPlan:** EP-008 M4
- **Issue:** No profiling instrumentation, no debug overlay
- **Fix:** Add profiling behind `profiling` feature. Debug overlay behind `debug-overlay` feature (reads StateView only, cannot alter hash).
- **Verification:** Overlay test asserts hash stability
- **Status:** [] PENDING

---

## TIER 7: EP-009 (CI/CD)

### G-020: GitHub Actions CI
- **Spec:** SPEC-008 §1
- **ExecPlan:** EP-009 M1
- **Issue:** No CI pipeline
- **Fix:** Create `.github/workflows/ci.yml` — build, test, lint, audit, coverage on push/PR
- **Verification:** Workflow file exists, valid YAML, covers all check types
- **Status:** [] PENDING

### G-021: Dockerfile
- **Spec:** SPEC-008 (deployment)
- **ExecPlan:** EP-009 M2
- **Issue:** No container build
- **Fix:** Multi-stage Dockerfile for release build
- **Verification:** `docker build .` succeeds
- **Status:** [] PENDING

### G-022: Release automation
- **Spec:** SPEC-008 (release)
- **ExecPlan:** EP-009 M3
- **Issue:** No release script, no cross-compilation
- **Fix:** Create `scripts/release.sh` — cross-compile for Linux x86_64, aarch64, Windows, macOS. Create artifact manifest with SHA256SUMS. Document CHANGELOG.
- **Verification:** Script runs and produces artifacts (may require toolchain install)
- **Status:** [] PENDING

---

## TIER 8: EP-010 (Production Readiness)

### G-023: Full ship gate execution
- **Spec:** SPEC-008 §1 (10 conditions)
- **ExecPlan:** EP-010 M1-M6
- **Issue:** Ship gate never run
- **Fix:** Execute all 10 conditions of SPEC-008 §1 in one session:
  1. verify.sh prints `verify: ok`
  2. live-fire.sh prints `live-fire: ok` with all 12 LFs
  3. reality-gate.sh prints `reality gate: ok`
  4. production-readiness-check.sh prints `production readiness: ok`
  5. Every PRODUCTION_READINESS.md item checked
  6. Every ExecPlan acceptance criterion met
  7. Artifacts exist for 3 targets with SHA256SUMS
  8. Version tag + CHANGELOG Determinism subsection
  9. MANUAL publish command printed
  10. RUN_COMPLETE appended to ledger
- **Verification:** All 10 conditions met
- **Status:** [] PENDING

---

## SUMMARY

| Tier | Items | Priority |
|------|-------|----------|
| 1: Fix regressions | G-001 through G-004 | CRITICAL — blocking verify.sh |
| 2: Spec compliance | G-005 through G-007 | HIGH |
| 3: LF-01 proof | G-008 | HIGH |
| 4: EP-007 rest | G-009 through G-014 | HIGH |
| 5: Content dirs | G-015 | HIGH |
| 6: EP-008 | G-016 through G-019 | MEDIUM |
| 7: EP-009 | G-020 through G-022 | MEDIUM |
| 8: EP-010 | G-023 | MEDIUM |

Total: 23 gaps identified across 11 specs and 11 execplans.
