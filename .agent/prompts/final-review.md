PRIME-BLOCK-BEGIN
This repository is governed by a 6LAYER blueprint pack. AGENTS.md is the authoritative control plane; if anything here conflicts with AGENTS.md, AGENTS.md wins.
On every session start, execute THE BOOT SEQUENCE:
1. Read AGENTS.md fully. 2. Read COMMANDS.md. 3. Read .agent/GRAPH.md and .agent/LOOPS.md. 4. Run: sh scripts/ledger.sh tail 30. 5. Run: sh scripts/preflight.sh -- it MUST print "preflight: ok"; if it fails, report the exact missing items from PREFLIGHT.md and stop (this is the only legitimate pre-run stop). 6. Run: sh scripts/graph-next.sh and dispatch on its one-line output exactly as .agent/GRAPH.md specifies. 7. Repeat step 6 after every completed node until ALL_DONE, then run the ship gate in AGENTS.md.
Hard rules: do not ask the user questions; choose the smallest reversible option, record it, continue. Use only commands from COMMANDS.md. Never invent an API, route, table, flag, or env var -- verify in-repo or transcribe from the pack. One node at a time; milestones in order; commit after every milestone; append ledger events as .agent/LOOPS.md requires. Bounded retries per .agent/LOOPS.md -- never repeat a failed fix. No mocks, stubs, demo modes, or placeholder code in production paths; scripts/reality-gate.sh and scripts/live-fire.sh must genuinely pass. Never weaken a gate, skip a test, or claim an unrun result. Stop only at NODE_BLOCKED (with the full evidence report) or ALL_DONE.
PRIME-BLOCK-END

---

Final review. Assume nothing. Cached green is not green.

1. **Verify from scratch.** `cargo clean`, then `sh scripts/verify.sh`. Observe every
   sentinel in real output, in order, through `verify: ok`. Record each one.
2. **Reality gate.** `sh scripts/reality-gate.sh` -> `reality gate: ok`.
3. **Live fire.** `sh scripts/live-fire.sh` -> `live-fire: ok`, with LF-01 through LF-12 each
   reported individually.
4. **Expected-files audit.** For every node EP-000 through EP-010, compare the files actually
   changed (`git diff --name-only green/EP-<prev>..green/EP-<this>`) against section 6 of that
   node's ExecPlan. Report any difference.
5. **Acceptance walk.** Open every ExecPlan's Validation and Acceptance section and confirm
   each criterion is marked met with the observed evidence recorded, not merely checked.
6. **Production readiness.** `sh scripts/production-readiness-check.sh` ->
   `production readiness: ok`. Then walk PRODUCTION_READINESS.md line by line and confirm
   every DOC line by opening the named file.
7. **Documentation truth.** Confirm: ARCHITECTURE.md invariants match the code;
   ENVIRONMENT.md dependency table matches Cargo.lock; PREFLIGHT.md matches
   scripts/preflight.sh; COMMANDS.md matches the scripts directory; every spec's Validation
   table names tests that exist.
8. **Outcomes and Retrospective.** Write the section in every completed ExecPlan.
9. **Final report** per AGENTS.md section 16, plus the ship-gate status and, because
   Auto-Deploy Authorization is `no`, the MANUAL publish command printed and not executed.

If any step fails, do not paper over it. Enter the ladder at the failing gate.
