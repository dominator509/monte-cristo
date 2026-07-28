# SPEC-002 -- Data model, content pipeline, and persistence

## 1. Content source layout

    content/
      regions/*.ron          15 files, one per region
      maps/*.ron             118 files
      bestiary/*.ron         102 files, one per enemy
      encounters/*.ron       180 hand-placed encounters
      spawn_tables/*.ron     15 regions x 3 chapter stages
      items/*.ron
      abilities/*.ron
      techs/*.ron
      scenes/*.ron           45 Confidences plus scripted scenes
      strings/en/*.ron       externalised string table
      flags.ron              the locked flag vocabulary
      party.ron
      curriculum.ron
      poisons.ron

## 2. Bake pipeline

    content/**.ron -> parse -> schema check -> vocabulary check -> reference resolve
                   -> orphan detect -> supernatural lint -> region-affinity check
                   -> canonical encode -> blake3 -> content.pack + content.pack.blake3

Each stage fails loudly with the file, the line, and the offending identifier. The bake is a
pure transform of committed sources: running it twice on the same tree yields the same digest,
and `content_integrity.rs` asserts that.

Release builds load only `content.pack`. Loose RON loading exists in debug builds for
authoring and is compiled out of release entirely -- not flag-disabled, compiled out, because
INV-10 forbids behaviour-altering configuration.

## 3. Schema: bestiary entry

    Enemy(
      id: "ENM_CELL_RAT",
      name_key: "enemy.cell_rat.name",
      family: VERMIN,                     // closed set, SPEC-009 section 2
      region_affinity: ["R03", "R04", "R14"],
      gate: FlagExpr::Always,             // or All([...]) / Any([...]) / Not(...)
      stats: Stats(hp: 14, atk: 6, def: 3, spd: 22),
      resist: [Terror],                   // vermin cannot be frightened
      abilities: ["ABL_BITE", "ABL_SWARM_CALL"],
      loot: [(ITM_RAT_PELT, 40), (ITM_BREAD_CRUST, 10)],
      xp: 3,
      tier: 1,
      sprite: "spr/enemy/cell_rat",
    )

`family` outside the closed set fails the bake with the closed-set error. This is how design
law L1 (no supernatural) is enforced mechanically rather than by review (ADR-011).

## 4. Schema: spawn table

    SpawnTable(
      region: "R12",
      chapter_stage: 2,
      pool: 26,                            // finite; feeds EncounterBudget
      entries: [
        (enemy: "ENM_SEWER_RAT_SWARM", weight: 30, gate: Always),
        (enemy: "ENM_BENEDETTO_BRAVO", weight: 18, gate: All(["CADEROUSSE_BURGLARY"])),
        (enemy: "ENM_QUARRY_SCAVENGER", weight: 12, gate: All(["QUARRIES_OPENED"])),
      ],
    )

The bake asserts every entry's enemy declares this region in its `region_affinity`. A table
entry that contradicts the enemy's affinity is a content error, not a silent override.

## 5. Schema: scene (Confidence)

    Scene(
      id: "SCN_CADEROUSSE_DIAMOND",
      act: ACT_IV_TOUR,
      participants: ["CHR_COUNT", "CHR_CADEROUSSE"],
      requires: All(["BUSONI_AVAILABLE"]),
      nodes: [
        Node(
          id: "n0",
          text_key: "scene.caderousse_diamond.n0",
          choices: [
            Choice(text_key: "...", to: "n1", trust: [("CHR_CADEROUSSE", 2)]),
            Choice(text_key: "...", to: "n2", trust: [("CHR_CADEROUSSE", -1)], requires: All(["ITM_DIAMOND"])),
          ],
        ),
      ],
      on_exit: Effects(
        set_flags: ["CADEROUSSE_TESTIMONY"],
        consume: ["ITM_DIAMOND"],
        mask: -2,
      ),
      terminal: false,
    )

No hit points, no turns, no meters. The schema has no field for them and cannot gain one
without a spec change (ADR-008).

## 6. Save model

    Save(
      schema_version: u16,      // independent of the product version
      product_version: String,  // informational only
      content_digest: [u8; 32], // must match the loaded pack or refuse
      world: World,             // canonical postcard encoding
      digest: [u8; 32],         // blake3 over everything above
    )

Load order, strictly: read header, check `schema_version` (a newer version is refused with
`SaveError::UnsupportedVersion`, never guessed), verify `digest`, verify `content_digest`,
then decode `world`. Structural interpretation never happens before digest verification
(INV-08).

Bounds: every length-prefixed field is checked against a declared maximum before allocation.
The parser never panics; every failure is a typed `SaveError`.

## 7. Save migration

Expand-migrate-contract. Version N reads N and N-1. On migration the original is renamed
`<name>.v<N-1>.bak` and retained; nothing is deleted. `mc_tools save-migrate` defaults to
`--dry-run` in every documented invocation.

Every previously shipped schema version keeps a fixture save in `tests/fixtures/saves-v<N>/`,
and EP-009 asserts they all migrate before a release ships.

## 8. Persistence boundaries

mc_core defines and serializes the save model but performs no file I/O. mc_shell and mc_tools
do the writing, through `fsroot::confine` (INV-07). This is why save round-trip logic is
testable without a filesystem, and why the filesystem tests are a separate, smaller suite.

## 9. Data validation rules

| Rule | Enforced at | Failure |
|---|---|---|
| every reference resolves | bake | `CONTENT_DANGLING_REF` with file and identifier |
| no orphaned content | bake | `CONTENT_ORPHAN` listing unreferenced ids |
| family inside closed set | bake | closed-set error |
| spawn entry matches enemy affinity | bake | affinity contradiction error |
| flag identifiers exist in flags.ron | bake | unknown-flag error |
| string keys exist in the string table | bake | missing-string error |
| exactly one terminal scene | bake | ending-count error |
| reserved identifiers unused | bake | reserved-identifier error (this is how the no-Mercedes-route invariant is enforced) |
| save digest matches | load | `SaveError::DigestMismatch` |
| save schema not newer | load | `SaveError::UnsupportedVersion` |
| content digest matches save | load | `SaveError::ContentMismatch` |

## 10. Test-data lifecycle

`tests/fixtures/mini-content/` is a small, real, committed content tree that passes the same
bake as the full one. Every test that writes uses its own temporary directory removed by a
guard, including on panic. No test writes into `MC_DATA_DIR`. `scripts/test-integration.sh`
asserts a clean working tree afterwards.

## 11. Backup and restore

Documented in OPERATIONS.md section 7. Verified for real in EP-010: copy the save directory,
corrupt one byte of the copy, confirm the corrupt file is rejected with a typed error and the
intact file still loads.

## 12. Validation

| Behaviour | Test |
|---|---|
| bake is a pure transform | `crates/mc_data/tests/content_integrity.rs` |
| dangling reference fails | `crates/mc_data/tests/forced_failures.rs` |
| supernatural family fails | `crates/mc_data/tests/forced_failures.rs` |
| save round-trips byte-identically | `crates/mc_data/tests/save_roundtrip.rs` |
| truncated save is a typed error | `crates/mc_data/tests/forced_failures.rs` |
| newer schema is refused | `crates/mc_data/tests/forced_failures.rs` |
| v1 fixtures migrate | `crates/mc_data/tests/save_migrate.rs` |
| backup, corrupt, restore | `crates/mc_data/tests/backup_restore.rs` |
| content invariants (endings, Edouard, reserved ids) | `crates/mc_data/tests/content_invariants.rs` |
