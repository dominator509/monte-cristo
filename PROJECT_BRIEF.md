# PROJECT BRIEF -- MONTE CRISTO

## Project name
MONTE CRISTO

## Problem statement

Adaptations of The Count of Monte Cristo soften it. They cut Edouard, they invent a happy
reunion with Mercedes, they turn the four-target vengeance into a swordfight. Meanwhile the
16-bit RPG form -- wronged hero, unjust imprisonment, mentor in the dark, buried treasure,
world tour, a run of end-game antagonists, a moral collapse, and grace -- is already the
novel's shape. Nobody has built it.

The engineering problem underneath is different and harder: a 40-hour narrative RPG is
normally impossible to test. Its claims ("the campaign is completable", "this encounter is
survivable", "the poison arc resolves") are usually verified by a human playing it. This
project's architecture exists to make those claims into assertions.

## Target users

Players of 16-bit-era Japanese RPGs who want a historically grounded, narratively serious
campaign rather than a fantasy pastiche. Readers of Dumas who want an adaptation that does
not flinch. Speedrunners and tool-assisted-run authors, for whom bit-exact determinism and a
first-class input-tape format are designed features rather than accidents. And the operator:
one developer running this pack hands-off through a terminal agent on a self-hosted Linux
workstation, with no cloud service, no account, and no runtime network dependency.

## Primary user outcomes

These are the ship criteria. Each has a named live-fire proof in scripts/live-fire.sh and
each is stated in full, verbatim, in SPEC-000.

LF-01 new-game-to-arrest
LF-02 if-calendar-and-curriculum
LF-03 field-encounter-resolves
LF-04 terrain-gated-spawns
LF-05 encounter-budget-no-grind
LF-06 confidence-scene-gates-story
LF-07 save-load-state-identity
LF-08 golden-tape-full-run
LF-09 determinism-cross-run
LF-10 content-integrity
LF-11 frame-budget
LF-12 final-boss-two-phase

## Business goals

Ship a complete, content-complete, single-purchase offline game with no recurring cost, no
vendor lock-in, and no service dependency. Every dependency permissively licensed and that
licensing enforced by a scripted gate. The source novel is public domain worldwide; no
licensed intellectual property enters the repository.

## Technical goals

1. The entire game is a pure headless deterministic state machine. The shell only draws it.
2. Every gameplay claim is provable by replaying an input tape and hashing the state.
3. Bit-exact determinism across platforms and across debug and release builds.
4. 60 frames per second at 256x224 on 2017-class integrated graphics.
5. The full campaign replays headlessly in under 15 minutes, so it can be a per-commit gate.
6. Rust throughout; no Node, npm, or web toolchain anywhere in any build chain.

## Out of scope (binding non-goals)

No multiplayer, networking, leaderboard, or cloud save. No procedural generation of any
kind. No microtransactions, downloadable-content hooks, or analytics. No 3D or lighting
engine, and no shader that breaks the act-locked 15-bit palette discipline. No modding API
in 1.0. No mobile or console port. No localisation beyond the shipped English script,
although the string table is externalised so a later one is possible.

Content non-goals, which are testable invariants and not preferences: no alternate endings,
no romance route for Mercedes, no path that spares Villefort, and no path that saves
Edouard. SPEC-000 states the assertion that proves each.

## Success metrics

Every live-fire proof passes in a single fresh verify run. Golden-tape replay under 15
minutes. p99 core step under 4.0 ms, p99 frame under 16.6 ms. Cold start under 2.5 seconds.
Zero dangling content references. Zero unsafe code in mc_core and mc_data. Zero committed
secrets. Zero sockets opened during a full replay.

## Production readiness

Defined mechanically in PRODUCTION_READINESS.md, gated by
`sh scripts/production-readiness-check.sh`, and summarised as the ship gate in AGENTS.md
section 15. Auto-Deploy Authorization is `no`: the run ends at a proven, tagged artifact set
and prints one MANUAL publish command.
