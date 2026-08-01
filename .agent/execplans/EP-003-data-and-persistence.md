NODE-META-BEGIN
ID: EP-003
DEPS: EP-002
MAX_ATTEMPTS_PER_MILESTONE: 6
VERIFY: sh scripts/test-integration.sh
VERIFY_SENTINEL: integration tests: ok
GREEN_TAG: green/EP-003
NODE-META-END

# EP-003 -- Data and persistence (mc_data)

## 1. Purpose / Big Picture

Give the core a memory and a body of content. This node builds the RON schema, the bake
pipeline with all seven validators, the content-addressed pack, the loader, and versioned
save files with real round-trip and migration tests against real files on a real filesystem.
It also authors enough real content -- all of Act I and all of region R03 -- to prove the
pipeline end to end rather than on a toy.

## 2. Scope

`crates/mc_data` in full; the `content/` tree for Act I and R03; save format and migration;
integration tests that touch a real filesystem; the forced-failure suite for both parsers.

## 3. Non-goals

No rendering (EP-005). No tape format (EP-004). No full content authoring -- Acts III through
VII are authored incrementally alongside later nodes; this node proves the pipeline and ships
enough content for LF-01 and LF-03. Do not implement `fsroot::confine` here; that is EP-006,
and until then mc_data takes already-validated paths from its caller.

## 4. Context and Orientation

SPEC-002 is authoritative. The bake is where design law L1 (no supernatural) and every
referential rule are enforced mechanically. A validator that can be worked around is not a
validator, so each one fails loudly with the file, the line, and the offending identifier.

## 5. Files to Read First

- .agent/specs/SPEC-002-data-model.md
- .agent/specs/SPEC-009-content-bestiary-and-regions.md
- .agent/specs/SPEC-006-error-handling.md
- SECURITY.md sections 1, 4, 11
- TESTING.md section 5

## 6. Expected Changed Files

- crates/mc_data/src/lib.rs
- crates/mc_data/src/schema/mod.rs
- crates/mc_data/src/schema/enemy.rs
- crates/mc_data/src/schema/region.rs
- crates/mc_data/src/schema/scene.rs
- crates/mc_data/src/schema/spawn_table.rs
- crates/mc_data/src/schema/item.rs
- crates/mc_data/src/bake.rs
- crates/mc_data/src/validate.rs
- crates/mc_data/src/pack.rs
- crates/mc_data/src/save.rs
- crates/mc_data/src/migrate.rs
- crates/mc_data/src/error.rs
- crates/mc_data/tests/content_integrity.rs
- crates/mc_data/tests/content_invariants.rs
- crates/mc_data/tests/scene_schema.rs
- crates/mc_data/tests/save_roundtrip.rs
- crates/mc_data/tests/save_migrate.rs
- crates/mc_data/tests/backup_restore.rs
- crates/mc_data/tests/forced_failures.rs
- content/flags.ron
- content/regions/R01.ron
- content/regions/R03.ron
- content/bestiary/*.ron  (the 11 R01 entries and the 17 R03 entries plus their two bosses)
- content/spawn_tables/R01-s1.ron
- content/spawn_tables/R03-s1.ron
- content/scenes/act1/*.ron
- content/strings/en/act1.ron
- tests/fixtures/mini-content/
- tests/fixtures/saves-v1/

## 7. Interfaces and Contracts

Schemas exactly as SPEC-002 sections 3, 4, 5, 6. Error variants exactly as SPEC-006 section
2. Content identifiers exactly as SPEC-009. Reserved identifiers from SPEC-009 section 9 must
fail the bake.

## 8. Milestones

### M1: Schema types
GOAL: Every content type deserializes from RON into a typed struct.
READ: SPEC-002 sections 3, 4, 5
CHANGE: crates/mc_data/src/schema/*.rs, crates/mc_data/src/lib.rs
CONTENT: one module per content type, `serde` derive, field names exactly as the spec.
  `Family` re-exported from mc_core so there is one closed set, not two.
RUN: cargo test --locked -p mc_data --lib schema
EXPECT: tests pass
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-003 MILESTONE_PASS "M1 schema types deserialize"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-003][M1] add content schema types"

### M2: Author Act I and R03 content
GOAL: Real content exists for two regions, enough to prove the pipeline and support LF-01 and LF-03.
READ: SPEC-009 sections 1, 3, docs/GAME_DESIGN.md sections 2, 4
CHANGE: content/flags.ron, content/regions/R01.ron, content/regions/R03.ron,
  content/bestiary/*.ron, content/spawn_tables/R01-s1.ron, content/spawn_tables/R03-s1.ron,
  content/scenes/act1/*.ron, content/strings/en/act1.ron
CONTENT: the 11 R01 bestiary entries and the 17 R03 entries plus ANTOINE_THE_STRANGLER and
  LE_RONGEUR, each with the exact identifier from SPEC-009 section 3, a family from the closed
  set, a `region_affinity` containing that region, a `gate`, stats, loot, and xp. The Act I
  scenes ending in `ACT1_ARREST`. The flag vocabulary from SPEC-009 section 9 including the
  four reserved-and-forbidden names.
RUN: cargo run --locked -p mc_tools -- validate --input ./content || true
EXPECT: the command runs; failures at this point are expected because the validator does not
  exist yet, so record its output and proceed
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-003 MILESTONE_PASS "M2 act1 and R03 content authored"
FALLBACK: if authoring 30 bestiary entries exceeds the milestone budget, author R03 only
  (17 entries plus two bosses) and defer R01 to M3 of this node; R03 is the one LF-03 needs.
COMMIT: git add -A && git commit -m "[EP-003][M2] author Act I and Chateau d'If content"

### M3: Bake and validators
GOAL: All seven validators run and each fails loudly with file, line, and identifier.
READ: SPEC-002 sections 2, 9, ARCHITECTURE.md section 9
CHANGE: crates/mc_data/src/bake.rs, crates/mc_data/src/validate.rs, crates/mc_data/src/error.rs
CONTENT: the pipeline in SPEC-002 section 2, in that order. Validators: schema, vocabulary,
  reference resolution, orphan detection, supernatural lint (family closed set), region
  affinity agreement between spawn tables and enemies, and reserved-identifier rejection.
RUN:
  cargo run --locked -p mc_tools -- validate --input ./content
EXPECT: `content: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-003 MILESTONE_PASS "M3 content: ok"
FALLBACK: none needed -- each validator is a small pure function over already-parsed data.
COMMIT: git add -A && git commit -m "[EP-003][M3] add bake pipeline and seven content validators"

### M4: Content pack and loader
GOAL: The bake is a pure transform and the pack is content-addressed.
READ: SPEC-002 sections 2, 6
CHANGE: crates/mc_data/src/pack.rs, crates/mc_data/tests/content_integrity.rs
CONTENT: canonical encoding, blake3 digest written to `content.pack.blake3`, loader that
  verifies the digest before interpreting structure. Release builds compile out loose-RON
  loading entirely (not flag-disabled -- compiled out, per INV-10). The integrity test bakes
  twice and asserts identical digests, then asserts every reference in the pack resolves.
RUN:
  cargo run --locked -p mc_tools -- bake --input ./content --output content.pack
  cargo test --locked -p mc_data --test content_integrity
EXPECT: `bake: ok` then the test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-003 MILESTONE_PASS "M4 bake: ok, integrity holds"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-003][M4] add content-addressed pack and verifying loader"

### M5: Content invariants
GOAL: The content non-goals of SPEC-000 section 4 are enforced by test.
READ: SPEC-000 section 4, SPEC-010 sections 7, 9
CHANGE: crates/mc_data/tests/content_invariants.rs, crates/mc_data/tests/scene_schema.rs
CONTENT: assertions that exactly one scene is `terminal`; that no reserved flag is used; that
  `ITM_ANTIDOTE_EDOUARD` exists with `usable_in: []`; and that the scene schema has no field
  capable of expressing hit points, turn order, or a resource meter.
RUN: cargo test --locked -p mc_data --test content_invariants --test scene_schema
EXPECT: both pass
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-003 MILESTONE_PASS "M5 content invariants enforced"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-003][M5] enforce content non-goals as tests"

### M6: Save format and round-trip
GOAL: A save round-trips byte-identically and its digest is verified before interpretation.
READ: SPEC-002 section 6, SECURITY.md section 4
CHANGE: crates/mc_data/src/save.rs, crates/mc_data/tests/save_roundtrip.rs
CONTENT: the `Save` struct of SPEC-002 section 6; strict load order (header, schema version,
  digest, content digest, then decode); every length-prefixed field bounded before allocation;
  typed `SaveError` variants exactly as SPEC-006. The round-trip test writes a real file to a
  temporary directory, reloads it, and asserts an identical `World::state_hash()`.
RUN: cargo test --locked -p mc_data --test save_roundtrip
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-003 MILESTONE_PASS "M6 save round-trip identical"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-003][M6] add versioned save format with digest verification"

### M7: Migration and backup
GOAL: v1 fixtures migrate, and a corrupted backup is rejected while an intact one loads.
READ: SPEC-002 section 7, SECURITY.md section 11, OPERATIONS.md section 7
CHANGE: crates/mc_data/src/migrate.rs, crates/mc_data/tests/save_migrate.rs,
  crates/mc_data/tests/backup_restore.rs, tests/fixtures/saves-v1/
CONTENT: expand-migrate-contract migration; the original is renamed with a `.bak` suffix and
  retained, never deleted. Committed v1 fixture saves. The backup test copies a save
  directory, flips one byte in the copy, and asserts the corrupt file is rejected with
  `SaveError::DigestMismatch` while the intact file still loads.
RUN:
  cargo test --locked -p mc_data --test save_migrate --test backup_restore
EXPECT: both pass
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-003 MILESTONE_PASS "M7 migration and backup verified"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-003][M7] add save migration and verified backup path"

### M8: Forced failures
GOAL: Every failure mode in TESTING.md section 5 is really forced and really handled.
READ: TESTING.md section 5, SPEC-006 section 5
CHANGE: crates/mc_data/tests/forced_failures.rs
CONTENT: real truncation of a real file; a real flipped byte; a real future schema version; a
  real dangling reference in a real RON file; a real supernatural family value; a real
  read-only directory. No simulated conditions.
RUN: cargo test --locked -p mc_data --test forced_failures
EXPECT: test passes with every case producing its exact typed error and no panic
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-003 MILESTONE_PASS "M8 forced failures handled"
FALLBACK: if a case cannot be forced on this platform (for example a full filesystem), force
  the nearest real condition, document the substitution in Surprises and Discoveries, and
  record an ADR. Never simulate the error by injecting it.
COMMIT: git add -A && git commit -m "[EP-003][M8] add forced-failure suite for both parsers"

### M9: Node verification
GOAL: The integration gate is green and the tree is clean afterwards.
READ: COMMANDS.md section 3
CHANGE: (none)
CONTENT: none.
RUN:
  sh scripts/test-integration.sh
  git status --porcelain
EXPECT: `integration tests: ok`; `git status --porcelain` prints nothing (no test left a file)
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-003 MILESTONE_PASS "M9 integration tests: ok"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-003][M9] verify data and persistence node"

## 9. Validation and Acceptance

| Criterion | Command | Expected |
|---|---|---|
| Content validates | `cargo run --locked -p mc_tools -- validate --input ./content` | `content: ok` |
| Bake is a pure transform | `cargo test --locked -p mc_data --test content_integrity` | pass |
| Supernatural family rejected | `cargo test --locked -p mc_data --test forced_failures` | pass |
| Dangling reference rejected | same | pass |
| Save round-trips identically | `cargo test --locked -p mc_data --test save_roundtrip` | pass |
| Newer schema refused | `cargo test --locked -p mc_data --test forced_failures` | pass |
| v1 fixtures migrate | `cargo test --locked -p mc_data --test save_migrate` | pass |
| Backup corruption detected | `cargo test --locked -p mc_data --test backup_restore` | pass |
| Content invariants hold | `cargo test --locked -p mc_data --test content_invariants` | pass |
| Scene schema has no combat fields | `cargo test --locked -p mc_data --test scene_schema` | pass |
| Node gate | `sh scripts/test-integration.sh` | `integration tests: ok` |
| No test residue | `git status --porcelain` | empty |

## 10. Idempotence and Recovery

The bake is a pure transform, so re-running it is always safe. Tests create and remove their
own temporary directories. To re-enter cold: read Progress, find the first unchecked
milestone, re-run the previous milestone's RUN, and continue. If `content.pack` is stale,
re-bake; it is a build artifact and is gitignored.

## 11. Progress

- [ ] M1 schema types
- [ ] M2 author Act I and R03 content
- [ ] M3 bake and validators
- [ ] M4 content pack and loader
- [ ] M5 content invariants
- [ ] M6 save format and round-trip
- [ ] M7 migration and backup
- [ ] M8 forced failures
- [ ] M9 node verification

## 12. Surprises and Discoveries

<empty>

## 13. Decision Log

<empty>

## 14. Outcomes and Retrospective

- All twelve acceptance rows are met. The node ledger records content-pack determinism,
  invariants, save identity, migration, backup, forced-failure handling, and `integration
  tests: ok`; the clean EP-010 verify re-ran the current data and persistence suites.
- Changed-files audit: 84 paths changed from `green/EP-002` to `green/EP-003`. The declared
  schema, pack, save, migration, Act I content, and fixture surface is represented, but the
  boundary also contains user-approved Hermes implementation paths: L6 ledger state,
  `mc_core` vocabulary/status support, `mc_tools` wiring, root helper scripts/tests, R02,
  and item content. The aggregate `content/strings/en/act1.ron` and mini-content directory
  do not appear as literal boundary paths; current per-string content and real fixtures
  satisfy the behavior and every acceptance command passes.
- Retrospective: file layout drifted, while the locked persistence contracts stayed green.
