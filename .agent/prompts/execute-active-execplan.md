PRIME-BLOCK-BEGIN
This repository is governed by a 6LAYER blueprint pack. AGENTS.md is the authoritative control plane; if anything here conflicts with AGENTS.md, AGENTS.md wins.
On every session start, execute THE BOOT SEQUENCE:
1. Read AGENTS.md fully. 2. Read COMMANDS.md. 3. Read .agent/GRAPH.md and .agent/LOOPS.md. 4. Run: sh scripts/ledger.sh tail 30. 5. Run: sh scripts/preflight.sh -- it MUST print "preflight: ok"; if it fails, report the exact missing items from PREFLIGHT.md and stop (this is the only legitimate pre-run stop). 6. Run: sh scripts/graph-next.sh and dispatch on its one-line output exactly as .agent/GRAPH.md specifies. 7. Repeat step 6 after every completed node until ALL_DONE, then run the ship gate in AGENTS.md.
Hard rules: do not ask the user questions; choose the smallest reversible option, record it, continue. Use only commands from COMMANDS.md. Never invent an API, route, table, flag, or env var -- verify in-repo or transcribe from the pack. One node at a time; milestones in order; commit after every milestone; append ledger events as .agent/LOOPS.md requires. Bounded retries per .agent/LOOPS.md -- never repeat a failed fix. No mocks, stubs, demo modes, or placeholder code in production paths; scripts/reality-gate.sh and scripts/live-fire.sh must genuinely pass. Never weaken a gate, skip a test, or claim an unrun result. Stop only at NODE_BLOCKED (with the full evidence report) or ALL_DONE.
PRIME-BLOCK-END

---

Execute exactly one node: [EXECPLAN_PATH]

Optional additional context from the operator: [OPTIONAL_USER_REQUEST]

Procedure:
1. Run the boot sequence above.
2. Confirm `sh scripts/graph-next.sh` names this node as NEXT or RESUME. If it names a
   different node, stop and report the discrepancy -- do not work a node the scheduler did
   not dispatch.
3. Append LEASE.
4. Read the plan in full, including its Non-goals, before acting.
5. Execute milestones strictly in order under the laws in .agent/LOOPS.md.
6. At the end: run the node VERIFY command, run the Expected Changed Files audit, append
   NODE_DONE, tag `green/<ID>`, release the lease.
7. Report per AGENTS.md section 16.

Do not continue to the next node in this mode. This prompt is for surgical, single-node work.
