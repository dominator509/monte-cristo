PRIME-BLOCK-BEGIN
This repository is governed by a 6LAYER blueprint pack. AGENTS.md is the authoritative control plane; if anything here conflicts with AGENTS.md, AGENTS.md wins.
On every session start, execute THE BOOT SEQUENCE:
1. Read AGENTS.md fully. 2. Read COMMANDS.md. 3. Read .agent/GRAPH.md and .agent/LOOPS.md. 4. Run: sh scripts/ledger.sh tail 30. 5. Run: sh scripts/preflight.sh -- it MUST print "preflight: ok"; if it fails, report the exact missing items from PREFLIGHT.md and stop (this is the only legitimate pre-run stop). 6. Run: sh scripts/graph-next.sh and dispatch on its one-line output exactly as .agent/GRAPH.md specifies. 7. Repeat step 6 after every completed node until ALL_DONE, then run the ship gate in AGENTS.md.
Hard rules: do not ask the user questions; choose the smallest reversible option, record it, continue. Use only commands from COMMANDS.md. Never invent an API, route, table, flag, or env var -- verify in-repo or transcribe from the pack. One node at a time; milestones in order; commit after every milestone; append ledger events as .agent/LOOPS.md requires. Bounded retries per .agent/LOOPS.md -- never repeat a failed fix. No mocks, stubs, demo modes, or placeholder code in production paths; scripts/reality-gate.sh and scripts/live-fire.sh must genuinely pass. Never weaken a gate, skip a test, or claim an unrun result. Stop only at NODE_BLOCKED (with the full evidence report) or ALL_DONE.
PRIME-BLOCK-END

---

One command is failing. Operationalise the ladder. Do not improvise.

**Step 0 -- capture.** Run the failing command exactly as written in COMMANDS.md. Capture the
full output, not the last line. Record the exit code.

**Step 1 -- signature.** Take the first error line. Normalise it: strip timestamps, variable
path segments, memory addresses, and counts. That string is the ERROR SIGNATURE. Append
`SIG <signature>` to the ledger. Count how many times this signature has already appeared for
this milestone -- that count is your current rung.

**Rung 1.** Read the whole error. Form exactly ONE hypothesis and write it in Surprises and
Discoveries before touching anything. Make the smallest fix consistent with it. Rerun the
NARROWEST command that reproduces the failure, not the whole suite.

**Rung 2.** Stop patching. Isolate. Write a narrower diagnostic: one test
(`cargo test --locked -p <crate> --test <file> <exact_test_name>`), one module, one added
assertion. Confirm or kill the hypothesis with evidence before touching code again.

**Rung 3.** The approach is wrong. Write every failed hypothesis into Surprises and
Discoveries. Switch to the milestone's declared FALLBACK. A fallback is a simpler real
implementation; it is never a mock and never a weakened gate.

**Rung 4.** Rollback to the last checkpoint per AGENTS.md 4.3, append ROLLBACK naming the
ref, and attempt the fallback once from clean state.

**Rung 5.** Append NODE_BLOCKED with the full report from LOOPS.md 5.7. Terminal.

**Never:** apply the same fix twice; comment out the failing test; add `#[ignore]`; add a
sleep; weaken a gate; edit a threshold; re-record a tape hash. If you are tempted by any of
those, you are on the wrong rung.

**Project-specific first questions.** If the signature is `DETERMINISM_HASH_MISMATCH`, do not
form a general hypothesis. Check these four in order, because it is always one of them:
a float in a state-affecting path; a `HashMap` or `HashSet` iteration whose order reaches
state; a system clock call; a thread. If the signature is `LAYER_VIOLATION`, run
`cargo tree -p <crate> --depth 1` and remove the upward import. If it is `CONTENT_DANGLING_REF`,
fix the content, never the validator.
