PRIME-BLOCK-BEGIN
This repository is governed by a 6LAYER blueprint pack. AGENTS.md is the authoritative control plane; if anything here conflicts with AGENTS.md, AGENTS.md wins.
On every session start, execute THE BOOT SEQUENCE:
1. Read AGENTS.md fully. 2. Read COMMANDS.md. 3. Read .agent/GRAPH.md and .agent/LOOPS.md. 4. Run: sh scripts/ledger.sh tail 30. 5. Run: sh scripts/preflight.sh -- it MUST print "preflight: ok"; if it fails, report the exact missing items from PREFLIGHT.md and stop (this is the only legitimate pre-run stop). 6. Run: sh scripts/graph-next.sh and dispatch on its one-line output exactly as .agent/GRAPH.md specifies. 7. Repeat step 6 after every completed node until ALL_DONE, then run the ship gate in AGENTS.md.
Hard rules: do not ask the user questions; choose the smallest reversible option, record it, continue. Use only commands from COMMANDS.md. Never invent an API, route, table, flag, or env var -- verify in-repo or transcribe from the pack. One node at a time; milestones in order; commit after every milestone; append ledger events as .agent/LOOPS.md requires. Bounded retries per .agent/LOOPS.md -- never repeat a failed fix. No mocks, stubs, demo modes, or placeholder code in production paths; scripts/reality-gate.sh and scripts/live-fire.sh must genuinely pass. Never weaken a gate, skip a test, or claim an unrun result. Stop only at NODE_BLOCKED (with the full evidence report) or ALL_DONE.
PRIME-BLOCK-END

---

Run the boot sequence now and continue dispatching until ALL_DONE or NODE_BLOCKED.

Your session ends only at RUN_COMPLETE or a blocked report. Do not stop to summarise progress,
do not ask whether to continue, and do not ask which node to take next -- `sh scripts/graph-next.sh`
answers that mechanically and it is the only authority.

This repository builds MONTE CRISTO, a deterministic offline RPG in Rust. Its defining
property is that mc_core is a pure, headless, deterministic state machine. Anything that
threatens that property -- a float in a state path, a HashMap iteration that affects order, a
clock, a thread -- is the highest-severity defect class in the project, regardless of what
feature it enables. If a determinism test fails, find the cause. Never re-record a tape to
make a hash test pass.

When you finish a node, immediately run `sh scripts/graph-next.sh` again and take the next
one. When it prints ALL_DONE, run the ship gate in AGENTS.md section 15, append RUN_COMPLETE,
and print the MANUAL publish command without executing it, because Auto-Deploy Authorization
for this project is `no`.
