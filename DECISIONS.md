# DECISIONS

## Decision table

| ID | Decision | Status | Node |
|---|---|---|---|
| ADR-001 | Rust workspace of five crates with an enforced layer DAG | Accepted | EP-001 |
| ADR-002 | mc_core is pure, headless, and deterministic; the shell only draws | Accepted | EP-002 |
| ADR-003 | Fixed-point Q16.16 for all state arithmetic; no floating point in core | Accepted | EP-002 |
| ADR-004 | macroquad 0.4.13 as the presentation shell | Accepted | EP-005 |
| ADR-005 | No database; versioned postcard save files with blake3 integrity | Accepted | EP-003 |
| ADR-006 | Content authored as RON, baked to a content-addressed pack | Accepted | EP-003 |
| ADR-007 | Input tapes as the universal test mechanism | Accepted | EP-004 |
| ADR-008 | No dialogue combat system; Confidences carry Trust and Mask only | Accepted | EP-002 |
| ADR-009 | Fernand de Morcerf as final boss, with a damage-immune phase 2 | Accepted | EP-002 |
| ADR-010 | Anti-grind encounter budget lives in core, not in content | Accepted | EP-002 |
| ADR-011 | No supernatural content; enforced by schema field and bake lint | Accepted | EP-003 |
| ADR-012 | Auto-deploy withheld; the run ends at a proven artifact | Accepted | EP-010 |
| ADR-013 | Self-hosted CI that only invokes scripts/verify.sh | Accepted | EP-001 |
| ADR-014 | cargo-deny licence allowlist as a hard gate | Accepted | EP-006 |

---

## ADR-001 Five-crate workspace with an enforced layer DAG

**Context.** A 40-hour RPG accumulates coupling. The single property this project cannot
lose is determinism, and the most common way to lose it is for a rendering, timing, or I/O
concern to leak into simulation.

**Decision.** Five crates -- mc_core, mc_data, mc_tape, mc_shell, mc_tools -- with the
dependency table in ARCHITECTURE.md section 4, enforced by `cargo tree` in scripts/lint.sh.

**Consequences.** Some plumbing is more verbose. In exchange, the leak that would destroy
INV-01 becomes a compile error rather than a subtle bug found in month nine.

**Alternatives rejected.** A single crate with module discipline (the discipline always
erodes). A workspace without an enforced check (documentation is not enforcement).

## ADR-002 mc_core is pure, headless, and deterministic

**Context.** Narrative RPG claims are normally verified by humans playing.

**Decision.** The entire game is a pure state machine with no I/O, no clock, and no threads.
The shell is a renderer and an input source, nothing else.

**Consequences.** Every gameplay claim becomes an assertion over a replayed tape. The ship
gate becomes a script. The cost is that anything convenient but impure -- reading a file
mid-simulation, asking the OS for the time, spawning a worker -- is simply unavailable.

## ADR-003 Fixed-point Q16.16

**Context.** Cross-platform bit-exact determinism is a ship criterion (LF-09). IEEE-754 is
deterministic in principle but not in practice across compilers, targets, and optimisation
levels once transcendentals, fused multiply-add, and x87 excess precision are involved.

**Decision.** A single `Fx` type, i32-backed Q16.16, for all state arithmetic. No f32 or f64
in mc_core at all -- absence rather than prohibition.

**Consequences.** Damage curves, interpolation, and physics are written in integers.
Precision is 1/65536, which is far finer than any quantity in this game. Rendering
interpolation may use floats because it does not feed back into state (INV-04).

## ADR-004 macroquad 0.4.13

**Context.** The shell needs a window, GPU-backed 2D, input, and audio, with no Node or npm
in the build chain (technical constraint) and minimal dependency surface.

**Decision.** macroquad 0.4.13.

**Consequences.** Small dependency tree, pure Rust, cross-platform, no web toolchain. Less
control than raw wgpu. EP-005 declares a FALLBACK to a direct wgpu blit path if the frame
budget cannot be met, which is possible because the shell owns nothing authoritative.

## ADR-005 No database

**Context.** Single-player, offline, file-based. A database would add an operational
dependency, a preflight credential, and a failure mode, in exchange for nothing.

**Decision.** Versioned save files serialized with postcard, integrity-checked with blake3,
under MC_DATA_DIR. Migrations are expand-migrate-contract.

**Consequences.** PREFLIGHT.md has no credentials at all. EP-003 tests real files on a real
filesystem rather than a database container. Save corruption becomes a parser-hardening
problem, which is why saves are a fuzz target (INV-08).

## ADR-006 RON content baked to a content-addressed pack

**Context.** 118 maps, 102 bestiary entries, 45 scenes, 210,000 words. None of it belongs in
Rust source (INV-06).

**Decision.** Author RON, bake to a single content-addressed pack with a blake3 digest.
Release builds refuse loose files.

**Consequences.** Content changes require no recompilation. The bake is the place to enforce
schema, references, orphans, region affinity, and the no-supernatural lint. Content cannot
introduce behaviour, only configure mechanisms that already exist.

## ADR-007 Input tapes as the universal test mechanism

**Context.** Given ADR-002, a tape plus a seed reproduces any game state exactly.

**Decision.** A first-class tape format: a list of (tick, Command) pairs plus a seed and a
content digest. mc_tape records and replays; mc_tools exposes both. Golden tapes are
committed.

**Consequences.** Live-fire proofs are tape replays with assertions. Regressions are caught
as hash mismatches. Speedrun tooling is a side effect rather than a feature request.

## ADR-008 No dialogue combat system

**Context.** An earlier design draft modelled conversation as turn-based combat with
Composure, Poise, and Leverage meters. It was bureaucratic: it turned character work into
resource management and made every scene feel like an encounter.

**Decision.** Dialogue is an authored branching scene called a Confidence, carrying exactly
two hidden values: a per-character Trust integer and one global Mask integer. No hit points,
no turn order, no resource meters, no failure state. Documents are key items that unlock
scenes and regions, never ammunition.

**Consequences.** Character development happens in writing rather than in systems, which is
where this particular novel lives. Combat carries the mechanical load instead, which is why
the bestiary grew to 102 entries. Any implementation that gives dialogue a battle interface
fails review.

## ADR-009 Fernand de Morcerf as final boss with a damage-immune phase 2

**Context.** The novel's Morcerf thread ends with Fernand arriving at the Count's house
armed, demanding to know who is destroying him, and receiving the answer "I am Edmond
Dantes." He is the only one of the four antagonists who is a soldier, and that scene is the
last face-to-face confrontation in the book.

**Decision.** Fernand is the final boss. Phase 1 is a genuine high-tier ATB fight. Phase 2
begins at zero hit points, cannot be resolved by damage, and requires
`Command::NameYourself`, which the bus rejects unless MORCERF_YANINA_DOSSIER,
MORCERF_ALBERT_WITHDRAWN, and MERCEDES_RECOGNITION are all set. Phase 3 is a scripted
pursuit, not a fight.

**Consequences.** The Morcerf campaign must close last, which requires gating its dossier
behind the Danglars bank inquiry to Yanina (canonical) and the collapse of the Villefort
household (procedural access to the Chamber of Peers). That compresses the novel's
chronology by a few weeks, which is recorded as deviation 1 in docs/GAME_DESIGN.md section 9.
The gain is that the Count enters the final encounter already broken by Edouard's death,
which is the novel's actual emotional order.

## ADR-010 Anti-grind budget in core

**Context.** Design law L4 forbids grinding. A content convention would be authored around
within a month.

**Decision.** Each region-chapter carries a finite spawn budget; repeat experience decays 30
percent compounding and floors at zero. Implemented in mc_core (INV-12), proven by LF-05.

**Consequences.** Progression must come from the Curriculum and story milestones, which is
also the thematically correct answer for a game about a man who was educated in a cell.

## ADR-011 No supernatural content

**Context.** Design law L1. The novel is realist; monsters would break it.

**Decision.** Every bestiary entry declares a family from a closed set, and the bake rejects
any entry outside it. Every enemy is a man, an animal, or the environment.

**Consequences.** The content lint enforces a creative rule mechanically, which is the only
way a rule survives a long project.

## ADR-012 Auto-deploy withheld

**Context.** Auto-Deploy Authorization is `no`.

**Decision.** The run completes the entire ship gate and stops, printing one MANUAL publish
command.

**Consequences.** Nothing is published without a human. The gate is still fully automated,
so the human decision is a decision, not a verification.

## ADR-013 Self-hosted CI that only invokes scripts/verify.sh

**Context.** The ship gate must not depend on a hosted service account.

**Decision.** CI configuration contains no gate logic; it runs `sh scripts/verify.sh`.

**Consequences.** The gate is identical locally and in CI, by construction. EP-001 M8
asserts the CI file contains no additional test invocation.

## ADR-014 cargo-deny licence allowlist as a hard gate

**Context.** Business constraint: permissive licences only.

**Decision.** `deny.toml` allowlists MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, Zlib, and
Unicode-3.0. Anything else fails `scripts/dependency-audit.sh`, which is STOP condition (c).

**Consequences.** A licence problem halts the run rather than shipping.

---

## Rules for adding decisions

Any pre-decided fork, any assumption made under uncertainty, any spec change, and any
reality-allow addition requires an entry here. Use .agent/templates/adr-template.md. Append
a Decision Log line in the active ExecPlan at the moment the decision is made, and a ledger
event. Never retro-fit a decision after the fact.
