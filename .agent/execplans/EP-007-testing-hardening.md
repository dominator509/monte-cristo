NODE-META-BEGIN
ID: EP-007
DEPS: EP-005,EP-006
MAX_ATTEMPTS_PER_MILESTONE: 6
VERIFY: sh scripts/live-fire.sh
VERIFY_SENTINEL: live-fire: ok
GREEN_TAG: green/EP-007
NODE-META-END

# EP-007 -- Testing hardening and the golden tape

## 1. Purpose / Big Picture

Raise the suite to the point where the ship gate means something. Coverage to the TESTING.md
floors, property tests over combat and poison, forced-failure coverage across every crate, a
flaky-test purge, and -- the centrepiece -- recording `tapes/golden-full.tape`, which plays
the entire campaign from a new game through the Fernand de Morcerf encounter to the epilogue
and locks it behind a hash.

After this node, all twelve live-fire proofs exist and pass.

## 2. Scope

Remaining content authoring for Acts III through VII sufficient to complete the campaign;
`scripts/live-fire.sh` in full; the golden tapes; coverage work; the determinism suite
extended to cross-profile.

## 3. Non-goals

No observability work (EP-008). No release engineering (EP-009). No new gameplay mechanisms:
if a live-fire proof cannot be satisfied by existing mechanisms plus content, that is a
specification gap to be recorded, not a licence to invent a system here.

## 4. Context and Orientation

This node is where the twelve outcomes of SPEC-000 section 2 stop being intentions. Each gets
one scripted, non-interactive proof driving the real entry point against the real content
pack.

The golden tape is the single most valuable regression asset in the repository. Re-recording
it to make a test pass is the most serious process violation available (CONTRIBUTING.md
section 4). If its hash changes, that is a finding, not a chore.

## 5. Files to Read First

- .agent/specs/SPEC-000-product-scope.md section 2
- TESTING.md in full
- .agent/specs/SPEC-009-content-bestiary-and-regions.md
- docs/GAME_DESIGN.md sections 2, 6

## 6. Expected Changed Files

- content/regions/R02.ron through content/regions/R15.ron
- content/bestiary/*.ron  (the remaining entries to 102)
- content/spawn_tables/*.ron  (to 45)
- content/encounters/*.ron  (to 180)
- content/scenes/**/*.ron  (to 45 Confidences plus scripted scenes)
- content/strings/en/*.ron
- content/curriculum.ron
- content/poisons.ron
- content/party.ron
- scripts/live-fire.sh
- tapes/golden-full.tape
- tapes/golden-smoke.tape
- tapes/HASHES.txt
- crates/mc_core/tests/prop_combat.rs
- crates/mc_core/tests/prop_poison.rs
- crates/mc_tape/tests/e2e_golden.rs
- crates/mc_tape/tests/memory_ceiling.rs
- crates/mc_core/benches/battle_step.rs

## 7. Interfaces and Contracts

The twelve proofs are named exactly `lf01_new_game_to_arrest` through
`lf12_final_boss_two_phase` and are invoked in that order by `scripts/live-fire.sh`, which
prints one line per proof and then `live-fire: ok`.

## 8. Milestones

### M1: Complete the content tree
GOAL: All 15 regions, 102 bestiary entries, 45 spawn tables, 180 encounters, and 45
  Confidences exist and validate.
READ: SPEC-009 sections 1, 3, 4, SPEC-010 section 2
CHANGE: content/**
CONTENT: author the remaining content to the counts in SPEC-009 section 4, using the exact
  identifiers of SPEC-009 section 3. Every enemy declares its `region_affinity` and its
  `gate`. R14's four entries are tier 1 by design. The Morcerf gating expression of SPEC-009
  section 8 is authored exactly as written.
RUN:
  cargo run --locked -p mc_tools -- validate --input ./content
  cargo run --locked -p mc_tools -- report bestiary
EXPECT: `content: ok`; the report prints `regions: 15`, `enemies: 102`, `spawn_tables: 45`,
  `encounters: 180`, `bosses: 21`, `confidences: 45`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-007 MILESTONE_PASS "M1 content complete and validating"
FALLBACK: if the full content tree exceeds the node budget, author to the minimum that
  completes the campaign path -- every region, every boss, every gating flag, and the 45
  Confidences -- and defer non-gating field encounters to a follow-up maintenance task
  recorded in the Decision Log. The campaign must be completable; the field density may lag.
COMMIT: git add -A && git commit -m "[EP-007][M1] complete the content tree to spec counts"

### M2: Property tests over combat and poison
GOAL: Combat and poison hold their invariants across generated input.
READ: TESTING.md section 1, SPEC-001 sections 5, 14
CHANGE: crates/mc_core/tests/prop_combat.rs, crates/mc_core/tests/prop_poison.rs
CONTENT: combat properties -- damage is never negative, never exceeds remaining HP plus
  overkill margin, a battle always terminates within a bounded tick count, and `Terror` never
  applies to `BEAST` or `VERMIN`. Poison properties -- tolerance is monotone non-decreasing
  under sub-lethal dosing, a lethal dose above tolerance always kills, and onset never fires
  before its declared tick.
RUN: cargo test --locked -p mc_core --test prop_combat --test prop_poison
EXPECT: both pass
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-007 MILESTONE_PASS "M2 combat and poison properties hold"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-007][M2] add combat and poison property tests"

### M3: Record the golden tape
GOAL: A tape plays the full campaign to the epilogue and its hash is recorded.
READ: SPEC-000 section 2 (LF-08), SPEC-003 sections 4, 5
CHANGE: tapes/golden-full.tape, tapes/golden-smoke.tape, tapes/HASHES.txt,
  crates/mc_tape/tests/e2e_golden.rs
CONTENT: record a tape that completes the campaign: Act I to the arrest, the 168-month If
  calendar with a Curriculum build, the escape, the sea and the island, the four Act IV
  chapters, Rome, the four Paris campaigns in an order that satisfies the SPEC-009 section 8
  gating, the Fernand encounter through all three phases, and the epilogue to `EPILOGUE_SAIL`.
  Record a short `golden-smoke.tape` for post-deploy verification. Append both hashes to
  `tapes/HASHES.txt`.
RUN:
  cargo run --locked -p mc_tools -- replay --tape tapes/golden-full.tape --assert-hash
  cargo test --locked -p mc_tape --test e2e_golden
EXPECT: `hash: match`; the test passes and asserts `EPILOGUE_SAIL` is set at the end
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-007 MILESTONE_PASS "M3 golden tape recorded, hash: match"
FALLBACK: if a single tape cannot cover the whole campaign within the 15-minute replay
  budget, split it into seven act tapes chained by save handoff, each with its own recorded
  hash, and assert the chain. Record an ADR. Never shorten the campaign to fit the tape.
COMMIT: git add -A && git commit -m "[EP-007][M3] record the golden full-campaign tape"

### M4: The twelve live-fire proofs
GOAL: `live-fire.sh` runs all twelve proofs against the real entry point and real content.
READ: SPEC-000 section 2, TESTING.md section 10
CHANGE: scripts/live-fire.sh
CONTENT: one proof per outcome, in order, each booting the real system headlessly, executing
  the outcome end to end, asserting on a real observable effect, and tearing down. The script
  prints one line per proof naming it, then `live-fire: ok`.
RUN: sh scripts/live-fire.sh
EXPECT: twelve lines `LF-01 ... ok` through `LF-12 ... ok`, then `live-fire: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-007 MILESTONE_PASS "M4 live-fire: ok"
FALLBACK: none needed -- each proof is a replay plus an assertion, and the mechanisms all
  exist by this point.
COMMIT: git add -A && git commit -m "[EP-007][M4] wire all twelve live-fire proofs"

### M5: Frame budget and memory ceiling
GOAL: The performance claims are measured, not asserted.
READ: SPEC-008 section 3, OBSERVABILITY.md section 4
CHANGE: crates/mc_core/benches/battle_step.rs, crates/mc_tape/tests/memory_ceiling.rs
CONTENT: a criterion bench of 10,000 frames of the heaviest authored battle (the Fernand
  encounter phase 1 with full status load) reporting p99 core step time; a long-replay test
  asserting peak resident memory stays under 512 MB with no unbounded growth.
RUN:
  cargo bench --locked -p mc_core -- battle_step
  cargo test --locked -p mc_tape --test memory_ceiling
EXPECT: bench reports p99 under 4.0 ms; the memory test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-007 MILESTONE_PASS "M5 p99 step under budget"
FALLBACK: if p99 exceeds 4.0 ms, profile and optimise the hottest system in `step::ORDER`
  before touching the budget. Changing the budget requires a spec update and an ADR, and is
  the last resort, not the first.
COMMIT: git add -A && git commit -m "[EP-007][M5] add frame budget bench and memory ceiling test"

### M6: Coverage and flaky-test purge
GOAL: Coverage meets the TESTING.md floors and no test is ignored or flaky.
READ: TESTING.md sections 8, 9
CHANGE: (test files across the workspace as needed)
CONTENT: raise coverage by adding real tests for uncovered branches. Do not lower a floor.
  Delete or fix every ignored test; a deletion requires an ADR naming what coverage was lost.
RUN:
  cargo llvm-cov --locked --workspace --fail-under-lines 85
  cargo test --locked --workspace -- --list | grep -c ignored
EXPECT: coverage command exits 0; the ignored count is `0`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-007 MILESTONE_PASS "M6 coverage floors met, zero ignored"
FALLBACK: if a crate cannot reach its floor because a code path is genuinely unreachable,
  remove the unreachable code rather than testing it, and record an ADR.
COMMIT: git add -A && git commit -m "[EP-007][M6] raise coverage to floors and purge ignored tests"

### M7: Node verification
GOAL: The live-fire gate is green from a clean state.
READ: COMMANDS.md section 3
CHANGE: (none)
CONTENT: none.
RUN:
  sh scripts/verify.sh
EXPECT: every sentinel through `verify: ok`, including `live-fire: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-007 MILESTONE_PASS "M7 verify: ok with live-fire"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-007][M7] verify testing hardening node"

## 9. Validation and Acceptance

| Criterion | Command | Expected |
|---|---|---|
| Content complete | `cargo run --locked -p mc_tools -- report bestiary` | the six counts of SPEC-009 section 4 |
| Content validates | `cargo run --locked -p mc_tools -- validate --input ./content` | `content: ok` |
| Combat properties | `cargo test --locked -p mc_core --test prop_combat` | pass |
| Poison properties | `cargo test --locked -p mc_core --test prop_poison` | pass |
| Golden tape reaches the epilogue | `cargo test --locked -p mc_tape --test e2e_golden` | pass |
| Golden hash locked | `cargo run --locked -p mc_tools -- replay --tape tapes/golden-full.tape --assert-hash` | `hash: match` |
| All twelve proofs | `sh scripts/live-fire.sh` | twelve ok lines then `live-fire: ok` |
| p99 step under budget | `cargo bench --locked -p mc_core -- battle_step` | p99 under 4.0 ms |
| Memory ceiling | `cargo test --locked -p mc_tape --test memory_ceiling` | pass |
| Coverage floors | `cargo llvm-cov --locked --workspace --fail-under-lines 85` | exit 0 |
| Zero ignored tests | `cargo test --locked --workspace -- --list \| grep -c ignored` | `0` |
| Node gate | `sh scripts/live-fire.sh` | `live-fire: ok` |

## 10. Idempotence and Recovery

Content authoring is additive; the bake is a pure transform. Tape recording is the one
non-idempotent act in this node: `tapes/golden-full.tape` is written once in M3 and is
thereafter read-only. To re-enter cold: read Progress, find the first unchecked milestone,
re-run the previous milestone's RUN, continue. **If the golden tape exists and its hash does
not match `tapes/HASHES.txt`, do not re-record it.** Enter the ladder with signature
`DETERMINISM_HASH_MISMATCH` and find the cause.

## 11. Progress

- [x] M1 complete the content tree
- [x] M2 property tests over combat and poison
- [x] M3 record the golden tape
- [ ] M4 the twelve live-fire proofs
- [ ] M5 frame budget and memory ceiling
- [ ] M6 coverage and flaky-test purge
- [ ] M7 node verification

## 12. Surprises and Discoveries

<empty>

## 13. Decision Log

- 2026-07-29, M3 resume: Keep the narrow out-of-list change to
  `crates/mc_tools/src/cmd_replay.rs`. The exact M3 replay command exited successfully but
  printed `hash: ok`, while this ExecPlan and the prior ledger evidence require the
  observable sentinel `hash: match`. Changing only that success string restores the
  specified CLI contract without changing replay or hashing behaviour.

## 14. Outcomes and Retrospective

<empty>
