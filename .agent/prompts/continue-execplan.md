PRIME-BLOCK-BEGIN
This repository is governed by a 6LAYER blueprint pack. AGENTS.md is the authoritative control plane; if anything here conflicts with AGENTS.md, AGENTS.md wins.
On every session start, execute THE BOOT SEQUENCE:
1. Read AGENTS.md fully. 2. Read COMMANDS.md. 3. Read .agent/GRAPH.md and .agent/LOOPS.md. 4. Run: sh scripts/ledger.sh tail 30. 5. Run: sh scripts/preflight.sh -- it MUST print "preflight: ok"; if it fails, report the exact missing items from PREFLIGHT.md and stop (this is the only legitimate pre-run stop). 6. Run: sh scripts/graph-next.sh and dispatch on its one-line output exactly as .agent/GRAPH.md specifies. 7. Repeat step 6 after every completed node until ALL_DONE, then run the ship gate in AGENTS.md.
Hard rules: do not ask the user questions; choose the smallest reversible option, record it, continue. Use only commands from COMMANDS.md. Never invent an API, route, table, flag, or env var -- verify in-repo or transcribe from the pack. One node at a time; milestones in order; commit after every milestone; append ledger events as .agent/LOOPS.md requires. Bounded retries per .agent/LOOPS.md -- never repeat a failed fix. No mocks, stubs, demo modes, or placeholder code in production paths; scripts/reality-gate.sh and scripts/live-fire.sh must genuinely pass. Never weaken a gate, skip a test, or claim an unrun result. Stop only at NODE_BLOCKED (with the full evidence report) or ALL_DONE.
PRIME-BLOCK-END

---

Resume an interrupted node.

Procedure:
1. Run the boot sequence.
2. `sh scripts/graph-next.sh` -> expect `RESUME <id>`.
3. Read, in this order: the node's ExecPlan Progress section, its Surprises and Discoveries,
   its Decision Log, and `sh scripts/ledger.sh tail 30`.
4. Determine the first unchecked milestone.
5. **Re-verify the last CHECKED milestone's sentinel before proceeding.** A checked box is a
   claim; the sentinel is the evidence. If it no longer passes, uncheck it and resume there
   instead.
6. If the working tree is dirty, compare it against that milestone's CHANGE list. Revert
   anything outside it. If something inside it is half-finished, reset to the last milestone
   commit and redo the milestone from the top -- milestones are idempotent by design.
7. If the lease belongs to another agent and its last ledger event is older than 90 minutes,
   append LEASE_TAKEOVER first. Otherwise, if it is live, do nothing and say so.
8. Continue under the normal node loop.
