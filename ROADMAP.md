# ROADMAP -- MONTE CRISTO

**Do not implement from this file. Implementation happens only through the graph: run
`sh scripts/graph-next.sh`.**

This file is strategic narrative for a human reader. It mirrors the GRAPH-TABLE one to one
and carries no instruction that is not already in an ExecPlan.

## Phase 0 -- EP-000 Discovery and toolchain
Purpose: make the environment unable to be the cause of a later failure. Pin Rust 1.83.0,
install and version-check the cargo subcommands, resolve the registry mode (online or
vendored), initialise the repository, and prove that every command in COMMANDS.md reaches
its sentinel against an empty skeleton.
Dependencies: none. Exit: `sh scripts/verify.sh` reaches `verify: ok` on a skeleton whose
gates are real but whose subject is empty.
Specs: SPEC-000. Plan: EP-000.

## Phase 1 -- EP-001 Foundation
Purpose: raise the five-crate workspace with its committed lockfile, formatting, clippy at
deny level, the crate-layer check, the test harness proven by one real passing test, the
self-hosted CI entry point, and the environment validation.
Dependencies: EP-000. Exit: verify green end to end on the skeleton; `green/EP-001`.
Specs: SPEC-000, SPEC-008. Plan: EP-001.

## Phase 2 -- EP-002 Core domain
Purpose: mc_core. Fixed-point arithmetic, the seeded generator, the world tree, the ATB
resolver with techs and statuses, the bestiary and region model, the terrain-gated spawn
function, the encounter budget, the Curriculum, the poison and tolerance model, the
story-flag graph, the Confidence scene model, and the final-encounter phase machine. Pure,
no I/O, unit-tested against real logic.
Dependencies: EP-001. Exit: determinism property tests green; `green/EP-002`.
Specs: SPEC-001, SPEC-006, SPEC-009, SPEC-010. Plan: EP-002.

## Phase 3 -- EP-003 Data and persistence
Purpose: the RON content schema, the bake pipeline with its validators, the content-addressed
pack, the loader, and versioned save files with real round-trip and migration tests against
real files on a real filesystem.
Dependencies: EP-002. Exit: content integrity proof and save round-trip proof; `green/EP-003`.
Specs: SPEC-002. Plan: EP-003.

## Phase 4 -- EP-004 Command bus and tape harness
Purpose: the public simulation API (`Command`, `StateView`, `apply_commands`, `step`) and
mc_tape: the tape format, recorder, replayer, and state hashing. This is the node that turns
every later claim into a test.
Dependencies: EP-003. Exit: a short tape records and replays to an identical hash twice;
`green/EP-004`.
Specs: SPEC-003. Plan: EP-004.

## Phase 5a -- EP-005 Presentation shell
Purpose: mc_shell. macroquad, the 256x224 render target and integer scaling, tilemap and
sprite rendering, the act-locked palette system, the battle interface, menus, the Confidence
scene presentation, audio, input remapping, and the full accessibility surface including the
content advisory screen.
Dependencies: EP-004. Runs in parallel with EP-006. Exit: e2e through the real entry point;
`green/EP-005`. Specs: SPEC-004. Plan: EP-005.

## Phase 5b -- EP-006 Security baseline
Purpose: there is no authentication in this product, so this node implements what actually
applies: parser hardening and fuzzing for both untrusted formats, filesystem confinement,
the no-socket assertion, the no-unsafe assertion, licence policy, and the committed-secret scan.
Dependencies: EP-004. Runs in parallel with EP-005. Exit: fuzz corpora committed and clean;
`green/EP-006`. Specs: SPEC-005. Plan: EP-006.

## Phase 6 -- EP-007 Testing hardening
Purpose: coverage to the TESTING.md floors, property tests over combat and poison, forced-
failure tests, the flaky-test purge, and the golden tapes -- including recording
tapes/golden-full.tape, the full campaign through the Fernand encounter to the epilogue.
Dependencies: EP-005 and EP-006. Exit: all twelve live-fire proofs green; `green/EP-007`.
Specs: SPEC-008. Plan: EP-007.

## Phase 7 -- EP-008 Observability and operations
Purpose: structured local logging with redaction, the frame-time and step-time histograms,
the debug overlay behind a feature, local crash reports, and the runbooks.
Dependencies: EP-007. Exit: operational smoke green; `green/EP-008`.
Specs: SPEC-007. Plan: EP-008.

## Phase 8 -- EP-009 Deployment and release
Purpose: reproducible per-target builds, the tarball and SHA256SUMS artifact set, the
cross-platform determinism check that replays the golden tape against each produced binary,
the release checklist, and a rollback drill that is actually performed.
Dependencies: EP-008. Exit: artifacts staged in MC_ARTIFACT_DIR; `green/EP-009`.
Specs: SPEC-008. Plan: EP-009.

## Phase 9 -- EP-010 Production readiness and ship
Purpose: full verify from scratch, reality gate, all twelve live-fire proofs, the security,
performance, accessibility, and privacy reviews against their specs, the backup and restore
verification, the deployment dry run, the rollback drill, the documentation review, and the
ship gate. Because Auto-Deploy Authorization is `no`, the node ends by printing the single
MANUAL publish command.
Dependencies: EP-009. Exit: `production readiness: ok` and RUN_COMPLETE; `green/EP-010`.
Specs: SPEC-008. Plan: EP-010.
