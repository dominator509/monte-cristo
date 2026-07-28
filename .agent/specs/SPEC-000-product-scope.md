# SPEC-000 -- Product scope and ship criteria

Behaviour first. Vocabulary is locked: identifiers below are used exactly as written,
including case, everywhere in code, content, tests, and plans.

## 1. What this software is

A single-player, offline, deterministic 16-bit-style graphic RPG adapting The Count of Monte
Cristo. Seven acts, 36 to 40 hours, one ending. The player controls Edmond Dantes and, in
four authored chapters, Haydee, Valentine, and Maximilien.

## 2. The twelve core outcomes (ship criteria)

Each is a live-fire proof in `scripts/live-fire.sh`. Each is stated here in the exact words
that the proof asserts.

**LF-01 new-game-to-arrest.** A new game plays from the title screen through Act I and
reaches the La Reserve arrest scene with story flag `ACT1_ARREST` set.

**LF-02 if-calendar-and-curriculum.** The Chateau d'If calendar advances 168 months, Faria
joins at month 72, and at least four Curriculum disciplines reach rank 3 with the correct
ability grants.

**LF-03 field-encounter-resolves.** A visible field enemy in region `R03` is contacted, the
ATB battle resolves deterministically to victory, loot is granted, and wounds persist into
the next map.

**LF-04 terrain-gated-spawns.** For every one of the 15 regions, 500 simulated spawn rolls
yield only bestiary entries whose region affinity includes that region and whose story-flag
gate is satisfied.

**LF-05 encounter-budget-no-grind.** Re-entering a region 40 times yields a strictly decaying
experience award that reaches zero.

**LF-06 confidence-scene-gates-story.** A Confidence scene consumes a key item, sets Trust
and story flags, and a later scene resolves to its alternate variant as a result.

**LF-07 save-load-state-identity.** A save taken mid-battle reloads to a byte-identical core
state hash across an application restart.

**LF-08 golden-tape-full-run.** `tapes/golden-full.tape` replays the full campaign from new
game through the Fernand de Morcerf final boss to the epilogue and matches its recorded
ending state hash.

**LF-09 determinism-cross-run.** The same seed and tape produce identical state hashes on two
independent processes and on both release and debug builds.

**LF-10 content-integrity.** Every map, enemy, item, ability, scene, and flag reference in
the baked content pack resolves, with zero orphans and zero dangling references.

**LF-11 frame-budget.** A 10000-frame headless bench of the heaviest battle scene holds p99
core step time under 4.0 ms and total frame budget under 16.6 ms.

**LF-12 final-boss-two-phase.** The Fernand encounter cannot be ended by damage alone; phase
2 requires `Command::NameYourself`, which is unavailable unless `MORCERF_YANINA_DOSSIER`,
`MORCERF_ALBERT_WITHDRAWN`, and `MERCEDES_RECOGNITION` are all set.

## 3. Act vocabulary (locked)

`ACT_I_MARSEILLE`, `ACT_II_IF`, `ACT_III_SEA`, `ACT_IV_TOUR`, `ACT_V_ROME`,
`ACT_VI_PARIS`, `ACT_VII_EPILOGUE`.

## 4. Content non-goals as testable invariants

These are proven by `crates/mc_data/tests/content_invariants.rs`.

| Invariant | Assertion |
|---|---|
| One ending | exactly one scene carries `terminal: true` |
| No Mercedes romance route | no scene sets a flag matching `MERCEDES_ROMANCE*`; the identifier is reserved and its use fails the bake |
| Villefort is never spared | every path through the Villefort campaign reaches `VILLEFORT_MADNESS` |
| Edouard is never saved | the antidote item `ITM_ANTIDOTE_EDOUARD` exists, is present in inventory at scene `SCN_EDOUARD`, and has `usable_in: []` |
| No procedural content | no content file is generated at build time; the bake is a pure transform of committed sources, asserted by a digest comparison |
| No supernatural | every bestiary entry's `family` is inside the closed set in SPEC-009 |

The Edouard invariant deserves a note, because it looks like a bug: the antidote is a real
inventory item, visible in the menu, and it cannot be used. That is the design (docs/GAME_DESIGN.md
section 9, "Refused"), and the test exists so that a future contributor cannot "fix" it.

## 5. Out of scope

Per PROJECT_BRIEF.md. Restated here for the bake: no networking, no procedural generation,
no modding API, no analytics, no alternate endings, no localisation beyond English in 1.0.

## 6. Definition of shippable

AGENTS.md section 15 plus every line of PRODUCTION_READINESS.md.
