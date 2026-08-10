# Changelog

All notable changes to Monte Cristo are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
against observable game behaviour (see RELEASE.md).

## [0.1.1] — 2026-08-09

### Changed

- Release metadata and native artifact packaging now identify the immutable `v0.1.1` patch release.
- The presentation shell's field and menu surfaces receive the 16-bit palette and HUD facelift described by the existing game design.

### Fixed

- Final readiness evidence now records the measured frame-budget result instead of leaving LF-11 skipped.

### Determinism

- **Golden tape hash unchanged.** The patch changes release metadata and presentation-only rendering; the deterministic core and all recorded tape hashes remain unchanged.

## [0.1.0] — 2026-07-29

### Added

- **Core determinism engine:** blake3-based state hashing, PCG64 seeded RNG, Q16.16 fixed-point arithmetic (`Fx`), deterministic `World` step loop with 13 ordered systems.
- **Content pipeline:** 7-validator bake pipeline (schema, vocabulary, references, orphans, supernatural lint, region affinities, reserved identifiers), deterministic content-addressed `Pack` system with pure-transform bake guarantee.
- **Content tree:** Act I scenes (arrival, arrest, Faria meeting, treasure reveal, escape, Sindbad), 15 regions with 138 enemies across 45 spawn tables, 6 encounter budgets, items, flags, bestiary (12 families), curriculum (7 disciplines), poison table (5 compounds), season clock.
- **If calendar and curriculum:** 168-month Château d'If simulation, Faria join at month 72, 7-discipline study system with ranked thresholds (ranks 0–5).
- **Scene system:** Branching narrative tree with `SceneAdvance`, `SceneChoice`, `SceneEffect` (set/clear flags, consume/grant items, trust adjustments, mask meter), `FlagExpr` gating.
- **Final encounter:** Three-phase boss against Fernand Mondego, Phase 2 damage immunity, `NameYourself` gating on three flags.
- **Encounter budget system:** Terrain-gated spawn eligibility, budget pool with decay factor, experience scaling.
- **Combat engine:** ATB gauge with wait/active mode, damage formula with critical hits, 8 status effects (poison with 5 compounds, bleeding, fever, terror), guard action.
- **Save system:** Content-addressed format with digest verification, version migration (v1 → current), backup/restore with integrity check.
- **Tape system:** Binary tape format (`MCTAPE01` magic), record/replay with checkpoint hashing, divergence detection with exact-tick reporting.
- **Command system:** 12 command variants (Move, Interact, SceneAdvance, SceneChoose, Save, Load, Calendar, Season, OpenMenu, CloseMenu, SetWaitMode, NameYourself, FastTravel, SwapPersona, Battle) with validation against `World.state_view()`.
- **User interface (mc_shell):** 256×224 render target with 16-colour palette, scene rendering, confidence scene dialogue, battle interface with ATB display, text rendering with glyph atlas, input remapping (keyboard + gamepad), advisory screen, caption overlay, screen shake.
- **File confinement (fsroot):** Restricted I/O rooted at `$MC_ROOT`, path traversal prevention, structured `FsError` reporting.
- **Observability:** JSON structured logging via `tracing-subscriber`, rotating file writer (7-generation retention), session-scoped crash reports with `$MC_TRIAGE`, metrics file on clean exit, profiler with CBOR output (`mc_profiler.cbor`).
- **Tooling (mc_tools):** `bake`, `replay`, `record`, `validate`, `prove`, `divergence`, `report bestiary` subcommands with 12 live-fire proofs.
- **Live-fire proofs:** LF-01 (new game to arrest with flag assertion), LF-02 (calendar and curriculum), LF-03 (field encounter resolve), LF-04 (terrain-gated spawns), LF-05 (encounter budget decay), LF-06 (confidence gating), LF-07 (save/load identity), LF-08 (golden tape + epilogue content), LF-09 (cross-run determinism), LF-10 (content integrity), LF-12 (final boss two-phase).
- **Testing:** 191 unit tests, 150+ integration tests, property-based tests (determinism, fixed-point, combat, spawn eligibility, poison, ATB), forced-failure tests (corrupted saves, truncated tapes, supernatural content).
- **CI/CD:** GitHub Actions workflow (`ci.yml`) with 8 jobs (build, test, lint, format, deny, coverage, fuzz, release), multi-stage Dockerfile.
- **Security:** `cargo-deny` advisories/bans/licenses/sources, no-networking enforcement, filesystem confinement, log redaction.
- **Documentation:** PRODUCTION_READINESS.md (14/14 conditions), RELEASE.md, DEPLOYMENT.md, GAME_DESIGN.md, ARCHITECTURE.md, CONTRIBUTING.md, ENVIRONMENT.md, ASSUMPTIONS.md, PREFLIGHT.md.

### Changed

- None (initial release).

### Fixed

- None (initial release).

### Determinism

- **Golden tape hash unchanged.** All recorded tapes (`golden-full.tape`, `golden-smoke.tape`, `act1.tape`) produce their original hashes after all implementation work:
  - `golden-full.tape`: `7eb44fdbf8f29f7671be60cc150bc5a240b189f1a79ac5c516a3362166444398`
  - `golden-smoke.tape`: `680ca47aba7adf5334a4fb58460789391a9206403406391b4db7b530235bafec`
  - `act1.tape`: `9b0b9f86d81dffb7b131c52b5e4e64ccd2caa9d4a04a3d08ac01d02d6b6a7e7c`
- Determinism invariant maintained: identical input tapes + identical seed produce identical final hash across all 10 execution plans. 200+ tests assert hash stability.
- Hash stability verified by: crate-level determinism tests, cross-run determinism proof (LF-09), fixed-point arithmetic invariants, debug-overlay hash stability test, audio mute/unmute hash stability test.

[0.1.1]: https://github.com/dominator509/monte-cristo/releases/tag/v0.1.1
[0.1.0]: https://github.com/dominator509/monte-cristo/releases/tag/v0.1.0
