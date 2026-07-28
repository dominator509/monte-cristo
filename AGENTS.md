# AGENTS.md -- MONTE CRISTO control plane

This file is the single canonical authority for any agent working this repository. Adapter
files (CLAUDE.md and the others) carry only the PRIME BLOCK and defer here.

## 1. Mission

Build MONTE CRISTO: a deterministic, offline, single-player 16-bit-style graphic RPG
adapting Alexandre Dumas' The Count of Monte Cristo, in Rust, as a five-crate workspace, and
take it from greenfield to a proven, tagged, ship-ready release artifact set for Linux,
Windows, and macOS. The entire game is a pure headless deterministic state machine
(mc_core); the shell only draws it. Every gameplay claim in this repository is therefore
provable by replaying an input tape and hashing the resulting state, and the ship gate
consists of exactly those proofs. You will finish the graph without asking anyone anything.

## 2. THE BOOT SEQUENCE

PRIME-BLOCK-BEGIN
This repository is governed by a 6LAYER blueprint pack. AGENTS.md is the authoritative control plane; if anything here conflicts with AGENTS.md, AGENTS.md wins.
On every session start, execute THE BOOT SEQUENCE:
1. Read AGENTS.md fully. 2. Read COMMANDS.md. 3. Read .agent/GRAPH.md and .agent/LOOPS.md. 4. Run: sh scripts/ledger.sh tail 30. 5. Run: sh scripts/preflight.sh -- it MUST print "preflight: ok"; if it fails, report the exact missing items from PREFLIGHT.md and stop (this is the only legitimate pre-run stop). 6. Run: sh scripts/graph-next.sh and dispatch on its one-line output exactly as .agent/GRAPH.md specifies. 7. Repeat step 6 after every completed node until ALL_DONE, then run the ship gate in AGENTS.md.
Hard rules: do not ask the user questions; choose the smallest reversible option, record it, continue. Use only commands from COMMANDS.md. Never invent an API, route, table, flag, or env var -- verify in-repo or transcribe from the pack. One node at a time; milestones in order; commit after every milestone; append ledger events as .agent/LOOPS.md requires. Bounded retries per .agent/LOOPS.md -- never repeat a failed fix. No mocks, stubs, demo modes, or placeholder code in production paths; scripts/reality-gate.sh and scripts/live-fire.sh must genuinely pass. Never weaken a gate, skip a test, or claim an unrun result. Stop only at NODE_BLOCKED (with the full evidence report) or ALL_DONE.
PRIME-BLOCK-END

## 3. Source-of-truth hierarchy

Current explicit user instruction > L1 > L2 > L3 > L4 > repository code and tests > L5 gate
output as fact > L6 as history.

- L1 CONTROL (immutable during a run): AGENTS.md, CLAUDE.md and other adapters,
  .agent/EXECUTION_RULES.md, .agent/LOOPS.md, the rules half of .agent/GRAPH.md.
- L2 SPECIFICATION (changed only by the documented spec-update rule, with repository
  evidence): PROJECT_BRIEF.md, ARCHITECTURE.md, .agent/specs/*, SECURITY.md, PREFLIGHT.md,
  ENVIRONMENT.md, docs/GAME_DESIGN.md.
- L3 GRAPH (fixed at generation; a run never rewires its own graph): the GRAPH-TABLE in
  .agent/GRAPH.md, ROADMAP.md, the node inventory.
- L4 EXECUTION: .agent/execplans/*, .agent/prompts/*, .agent/templates/*, COMMANDS.md,
  CONTRIBUTING.md. Only ExecPlan Progress sections are mutable.
- L5 VERIFICATION: TESTING.md, scripts/*, .agent/checklists/*, .agent/reality-patterns,
  .agent/reality-allow, and the test suites the plans create.
- L6 STATE (the only always-writable layer): .agent/state/LEDGER.md, git history, green
  tags, evidence captured in ExecPlan Progress.

When code contradicts spec, the spec wins and the code changes. When a plan contradicts a
spec, the plan is corrected via the spec-update rule with a ledger entry. Gates are never
weakened to make code pass; the single narrow exception is adding a justified line to
.agent/reality-allow WITH a Decision Log entry.

Spec-update rule: to change an L2 file you must (a) quote the repository evidence that makes
the current text false, (b) make the smallest edit that restores truth, (c) append a
Decision Log entry in the active ExecPlan, (d) append a ledger event. No other path exists.

## 4. The graph protocol

### 4.1 Node law
- One node = one ExecPlan = one bounded unit of work with entry evidence, exit evidence, and
  a green tag.
- Node IDs are EP-000 through EP-010. Dependencies are explicit in the GRAPH-TABLE.
- SINGLE WRITER: at most one node is IN_PROGRESS repo-wide, ever. The holder is recorded by a
  LEASE event in the ledger.
- A node is DONE only when ALL FIVE hold: every milestone passed with evidence; the node's
  VERIFY command printed its VERIFY_SENTINEL in this session; the Expected Changed Files
  audit passed; a NODE_DONE ledger event was appended; and the git tag `green/<ID>` exists.
  A DONE claim without all five is a fabrication.

### 4.2 Dispatch table
Run `sh scripts/graph-next.sh`. It prints exactly one line. Act on it:
- `NEXT <id>`   -> append LEASE, then execute that ExecPlan from milestone 1.
- `RESUME <id>` -> a lease is open. If it is yours, continue at the first unchecked
  milestone, re-verifying the last checked milestone's sentinel first. If it belongs to
  another agent and its last ledger event is older than 90 minutes, append LEASE_TAKEOVER
  and continue from the ledger and ExecPlan state. Otherwise do nothing; another agent is live.
- `BLOCKED <id>` -> terminally halted. Read the NODE_BLOCKED report in that plan's Progress
  section. A human must intervene. Do not restart and do not work around.
- `STALL <id>`  -> graph defect (unsatisfiable dependencies). Append NODE_BLOCKED for that id
  with detail GRAPH_STALL and treat as BLOCKED.
- `ALL_DONE`    -> run the ship gate in section 15, then append RUN_COMPLETE.

### 4.3 Checkpoint and rollback
- Commit after every milestone: `[EP-XXX][M<k>] <imperative summary>`. Nothing is ever left
  uncommitted between milestones.
- Tag `green/EP-XXX` at every NODE_DONE.
- Rollback (only from the ladder rung 4): `git reset --hard <last green tag or last
  [EP-XXX][M<k-1>] commit>`, append a ROLLBACK event naming the target ref, then re-enter the
  milestone on its declared FALLBACK path. Rollback never crosses a completed node's green tag.

### 4.4 Multi-agent cohesion
Git plus the ledger is the entire coordination bus; there is no other channel. Run
graph-next.sh fresh before every lease and never cache a dispatch. While holding a lease,
append HEARTBEAT at least every 15 minutes of activity and after every milestone. Release
the lease (LEASE_RELEASE) if you stop for any reason other than NODE_DONE or NODE_BLOCKED.
Solo operation is the degenerate case of the same protocol.

## 5. STOP conditions -- exactly these and no others

(a) Preflight failure before the run. Report the exact missing items from PREFLIGHT.md and stop.
(b) An action would destroy user or production data, or cause an irreversible external side
    effect not explicitly specified by an ExecPlan.
(c) A legal, financial, or security judgment the specifications do not answer. For this
    project the realistic instance is a dependency licence outside the allowed set
    (MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, Zlib, Unicode-3.0) or an unwaivable
    security advisory.
(d) NODE_BLOCKED after the full ladder in .agent/LOOPS.md section 5.3, with the structured
    blocked report of section 5.7.
(e) Production deploy. Auto-Deploy Authorization is `no` for this project. The ship gate
    still completes in full; EP-010 emits the single MANUAL publish command and stops there.

Everything else: choose the smallest reversible option, record it in the active ExecPlan's
Decision Log, and continue.

Explicitly: do not ask the user for next steps, preferences, or confirmation. Proceed.

## 6. Anti-drift rules

- Scope fence: a milestone's CHANGE list is exhaustive. After every milestone run
  `git status --porcelain` and `git diff --name-only HEAD~1`. Any path outside the CHANGE
  list is reverted immediately unless a Decision Log entry was written BEFORE keeping it.
- Expected Changed Files audit at node end: the set of files changed across the node must
  equal section 6 of its ExecPlan.
- No broad refactors, no reorganisations, no dependency swaps, no unrelated cleanup, no
  opportunistic renaming, no "while I'm here" edits. Ever.
- Never implement from ROADMAP.md. Implementation happens only through the graph.
- Do not add features that are not in a spec, including ones that seem obviously good. The
  Non-Goals list in PROJECT_BRIEF.md is binding, and several of its entries (no alternate
  endings, no procedural generation, no modding API) are content invariants with tests.

## 7. Anti-hallucination rules

- Never invent a crate API, a function signature, a command, an environment variable, a
  content field, a story flag, a bestiary identifier, a region identifier, or a config key.
- Confirm every name by reading the repository file that defines it, or by transcribing it
  verbatim from this pack. The vocabulary tables in .agent/specs/* are locked: identifiers
  are used exactly as written there, including case.
- Commands come only from COMMANDS.md. If a command is missing or stale, update COMMANDS.md
  first, citing repository evidence, with a Decision Log entry.
- Dependency versions come only from the pinned set in ENVIRONMENT.md and the committed
  Cargo.lock. Never resolve `latest`, never use a version range that was not already there.
- When a plan says to transcribe a file body, transcribe it exactly. Do not improve it, do
  not reformat it, do not "fix" its style. Transcription over composition is the strongest
  anti-hallucination lever in this pack and it only works if you obey it literally.
- Record every assumption in the Decision Log at the moment you make it, not afterwards.

## 8. Anti-fixation rules

The ladder in .agent/LOOPS.md section 5.3 is mandatory. Track failures by ERROR SIGNATURE
(first error line, normalised: strip timestamps, variable path segments, addresses, counts).
Append `SIG <signature>` to the ledger on each failure.

Rung 1 (first same-signature failure): read the full error, form ONE hypothesis, make the
smallest targeted fix, rerun the NARROWEST failing command.
Rung 2: stop patching. Isolate with a narrower diagnostic. Confirm or kill the hypothesis
with evidence before touching code again.
Rung 3: the approach is wrong. Record failed hypotheses in Surprises and Discoveries, then
switch to the milestone's declared FALLBACK.
Rung 4: rollback per 4.3, then attempt the fallback once from clean state.
Rung 5: append NODE_BLOCKED with the full report. Terminal.

Absolute rule: the same fix may never be applied twice. If the diff you are about to make
matches a diff already tried for this signature, you are on the wrong rung. Climb.

## 9. Reality law

- PRODUCTION PATH: any code that runs when a real player exercises a core outcome, plus its
  content, schema, and build configuration. For this project that is all of mc_core,
  mc_data, mc_shell, mc_tape, and the content tree.
- TEST DOUBLE ZONE: `tests/` directories, `benches/`, and `fuzz/` only, as enumerated in
  TESTING.md. Even there, integration and live-fire suites use the real implementation.
- FABRICATION, all forbidden: stubbed handlers; hardcoded sample data presented as live
  state; a demo mode; functions returning success without performing the effect; simulated
  content loading; sleep-and-pretend; tests asserting on mocks of the thing under test;
  commenting out or skipping a failing test; weakening a gate to pass it; a "TODO" left in a
  production path.
- Specific to this project: mc_core must never be given a test-only branch, a debug-only
  shortcut that changes state, or a feature flag that alters simulation behaviour.
  Configuration may differ between environments; behaviour may not. A single behaviour-
  altering cfg in mc_core is a defect of the highest severity because it destroys LF-09.

**Software that appears to work is a failure state. Only software proven by live-fire counts.**

## 10. Dependency rules

Check the existing dependency set first; prefer a crate already in the tree. Add a new
dependency only when the alternative is writing more than roughly 300 lines of non-domain
code. Pin the exact version, add it to Cargo.toml with a comment naming the invariant or
requirement it serves, commit the updated Cargo.lock, and document it in ENVIRONMENT.md.
Licence must be inside the allowed set or it is STOP condition (c). mc_core may not gain any
dependency capable of I/O, threading, clock access, or ambient randomness; this is checked
by scripts/security-check.sh and is architectural invariant INV-01.

## 11. File creation and commit rules

Create files only where a milestone's CHANGE list says. Commit after every milestone with
the exact format in 4.3. Never force-push, never rewrite history, never amend a milestone
commit that is already pushed. Tags are created only at NODE_DONE.

## 12. Testing rules

See TESTING.md. The gate-weakening prohibition is absolute: you may fix code to satisfy a
gate; you may never edit a gate to satisfy code. A flaky test is a bug, fixed or deleted
with an ADR, never retried until green. Coverage thresholds in TESTING.md are floors.

## 13. Documentation update rules

L1 files: never edited during a run. L2 files: only via the spec-update rule in section 3.
L3: never. L4: only the Progress, Surprises, Decision Log, and Outcomes sections of the
active ExecPlan. L5: gates only strengthen. L6: append-only.

## 14. Security rules

See SECURITY.md. Highlights that bind every node: no network access at runtime, asserted by
test; no telemetry; filesystem access confined to the three declared roots with canonicalised
paths; save files and content packs are untrusted input and their parsers must never panic;
no unsafe code in mc_core or mc_data; no secrets anywhere, asserted by scan.

## 15. Definition of done

**For a node:** the five conditions of section 4.1, all of them.

**For the run (the ship gate):**
1. `sh scripts/verify.sh` run from scratch in this session prints every sentinel through
   `verify: ok`.
2. `sh scripts/live-fire.sh` prints `live-fire: ok` with all twelve proofs LF-01 through
   LF-12 reported as passing.
3. `sh scripts/reality-gate.sh` prints `reality gate: ok`.
4. `sh scripts/production-readiness-check.sh` prints `production readiness: ok`.
5. PRODUCTION_READINESS.md is fully checked with a command or artifact cited per line.
6. Every acceptance criterion in every ExecPlan is marked met with observed evidence.
7. Release artifacts exist in MC_ARTIFACT_DIR for all three targets with a matching
   SHA256SUMS, and the version tag exists.
8. Because Auto-Deploy Authorization is `no`, the final action is to PRINT the MANUAL publish
   command and stop. Do not publish.
9. Append RUN_COMPLETE to the ledger.

## 16. Final response requirements

Every response that ends a working session must state: the node or nodes completed; the
changed files compared against the ExecPlan's expected list; every command run with the
sentinel actually observed; acceptance status per criterion; decisions made; assumptions
confirmed or changed; remaining risks; and ship-gate status.
