# GRAPH -- MONTE CRISTO build arc

L1 (rules half, immutable during a run) and L3 (the table, fixed at generation).

## Narrative of the build arc

The build proves the engine before it proves the game. EP-000 pins the toolchain and
establishes that every command in COMMANDS.md reaches its sentinel against an empty
skeleton, so nothing later can be blamed on the environment. EP-001 raises the five-crate
workspace with its lockfile, formatting, linting, layering enforcement, and one real passing
test, then proves verify.sh green end to end on that skeleton. EP-002 builds the heart:
mc_core, the pure deterministic simulation -- fixed-point arithmetic, the seeded generator,
the ATB resolver, the bestiary and region model, the terrain-gated spawn function, the
encounter budget, the Curriculum, the poison model, and the story-flag graph -- with no I/O
anywhere in it. EP-003 gives that heart a memory: the RON content schema, the bake pipeline,
the content-addressed pack, and versioned save files with real round-trip and migration
tests against real files. EP-004 exposes the core as a command bus and builds the tape
harness, which is the mechanism by which every later claim becomes a test. EP-005 draws it:
macroquad, the 256x224 render target, tilemaps, sprites, the battle interface, menus, the
Confidence scene presentation, and the accessibility surface. EP-006 hardens the two
untrusted parsers, confines the filesystem, and proves no socket is ever opened. EP-007
raises coverage, adds property and fuzz corpora, and records the golden tapes -- including
the one that plays the whole campaign to the Fernand encounter. EP-008 adds local structured
logging, the frame-time histogram, and the runbooks. EP-009 produces the three platform
artifacts and drills the rollback. EP-010 runs the ship gate and stops, because
Auto-Deploy Authorization is `no`.

The branch at EP-005/EP-006 exists because the presentation shell and the security baseline
are genuinely independent once the command bus of EP-004 is stable; a second agent may take
one while the first takes the other. Everything else is a chain, deliberately.

## GRAPH-TABLE

GRAPH-TABLE-BEGIN
NODE EP-000 DEPS -
NODE EP-001 DEPS EP-000
NODE EP-002 DEPS EP-001
NODE EP-003 DEPS EP-002
NODE EP-004 DEPS EP-003
NODE EP-005 DEPS EP-004
NODE EP-006 DEPS EP-004
NODE EP-007 DEPS EP-005,EP-006
NODE EP-008 DEPS EP-007
NODE EP-009 DEPS EP-008
NODE EP-010 DEPS EP-009
GRAPH-TABLE-END

## Node law

- One node = one ExecPlan = one bounded unit of work with entry evidence, exit evidence, and
  a green tag.
- SINGLE WRITER: at most one node is IN_PROGRESS repo-wide, ever, recorded by a LEASE event.
- A node is DONE only when all five conditions hold: every milestone passed with evidence;
  the node VERIFY command printed its VERIFY_SENTINEL in this session; the Expected Changed
  Files audit passed; NODE_DONE was appended to the ledger; and `green/<ID>` was tagged.
- Cycles between nodes are forbidden. The only cycles anywhere are the bounded intra-
  milestone loops in .agent/LOOPS.md.

## Dispatch table

`sh scripts/graph-next.sh` prints exactly one line:

| Output | Action |
|---|---|
| `NEXT <id>` | Append LEASE. Execute that ExecPlan from milestone 1. |
| `RESUME <id>` | A lease is open. Yours: continue at the first unchecked milestone after re-verifying the previous one's sentinel. Another agent's, last event older than 90 minutes: append LEASE_TAKEOVER and continue. Otherwise: do nothing. |
| `BLOCKED <id>` | Terminally halted. Read the report in that plan's Progress. Human intervention required. Do not restart, do not work around. |
| `STALL <id>` | Graph defect. Append NODE_BLOCKED with detail GRAPH_STALL. Treat as BLOCKED. |
| `ALL_DONE` | Run the ship gate in AGENTS.md section 15, then append RUN_COMPLETE. |

## Ledger grammar

    <ISO8601-UTC> | <AGENT_ID> | <NODE|-> | <EVENT> | <detail>

EVENTS: RUN_INIT, PREFLIGHT_OK, LEASE, HEARTBEAT, MILESTONE_PASS, ATTEMPT_FAIL, SIG,
FALLBACK_TAKEN, ROLLBACK, NODE_DONE, NODE_BLOCKED, LEASE_RELEASE, LEASE_TAKEOVER,
RUN_COMPLETE.

Never edit or delete a line. Details never contain the sequence " | ". Node status is
DERIVED from the ledger by `sh scripts/ledger.sh status <NODE>`; no separate status file
exists, so nothing can fall out of sync.

## Checkpoint and rollback

Commit after every milestone as `[EP-XXX][M<k>] <summary>`. Tag `green/EP-XXX` at NODE_DONE.
Rollback only from ladder rung 4: reset hard to the last green tag or previous milestone
commit, append ROLLBACK naming the ref, re-enter on the declared FALLBACK. Rollback never
crosses a completed node's green tag.
