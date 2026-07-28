# LOOPS -- every loop is declared, bounded, and terminates

L1, immutable during a run. Every loop below exits in bounded iterations into exactly one of
{pass, fallback, rollback, NODE_BLOCKED}. A fake pass is forbidden by the evidence rules in
section 6.

## 5.1 The run loop (outermost)

    while true:
        run sh scripts/graph-next.sh
        dispatch per the table in .agent/GRAPH.md
        on ALL_DONE: run the ship gate in AGENTS.md section 15, append RUN_COMPLETE, exit

Bounded because the node count is finite (eleven), every node terminates by 5.2, and BLOCKED
exits the loop.

## 5.2 The node loop

For the leased node: execute milestones strictly in order, each via 5.3. After the last
milestone: run the node VERIFY command, run the Expected Changed Files audit, append
NODE_DONE, tag green, release the lease. Bounded because milestones are finite and 5.3
terminates.

## 5.3 The milestone loop (verify-fix ladder)

Each milestone ends with RUN commands and EXPECT sentinels. On mismatch, climb this ladder.

Track failures by ERROR SIGNATURE: the first error line of output, normalised -- strip
timestamps, variable path segments, memory addresses, and counts. Append `SIG <signature>`
to the ledger on each failure. The ladder counts SAME-signature failures. A NEW signature
resets to rung 1, but total attempts for the milestone are capped at MAX_ATTEMPTS (default
6, set per node in NODE-META).

- **Rung 1** (first same-signature failure): read the FULL error, not the last line. Form ONE
  hypothesis. Make the smallest targeted fix. Rerun the NARROWEST failing command -- a single
  test, not the whole suite.
- **Rung 2** (second): stop patching and isolate. Write or run a narrower diagnostic: one
  test, one module, one added assertion, `cargo test -p mc_core --lib <exact_test_name>`.
  Confirm or kill the hypothesis with evidence before touching code again.
- **Rung 3** (third): the approach is wrong. Record every failed hypothesis in the ExecPlan's
  Surprises and Discoveries section, then switch to the milestone's pre-declared FALLBACK --
  a simpler design, an already-pinned alternative, or a reduced-but-real implementation that
  still satisfies the spec. A fallback is never a mock.
- **Rung 4** (fallback exhausts its own three attempts, or MAX_ATTEMPTS is reached): ROLLBACK
  per GRAPH.md, then attempt the fallback once from clean state.
- **Rung 5**: append NODE_BLOCKED with the structured report of 5.7. Terminal. Never loop
  back, never fake a pass, never comment out the failing test, never mark the milestone done.

**Absolute rule:** the same fix may never be applied twice. If the diff you are about to
make matches a diff already tried for this signature, you are on the wrong rung. Climb.

### Project-specific signatures worth recognising early

- `DETERMINISM_HASH_MISMATCH` -- two replays of one tape produced different hashes. Do not
  patch the test. The cause is always one of: floating point in a state path, HashMap
  iteration order, system time, or thread scheduling. Find which. This is invariant INV-01
  and it is never worked around.
- `CONTENT_DANGLING_REF` -- the bake found a reference to a missing identifier. Fix the
  content, never the validator.
- `LAYER_VIOLATION` -- a crate imported upward. Fix the import, never the check.
- `BUDGET_EXCEEDED` -- see 5.5.
- `READINESS_TIMEOUT_replay` -- see 5.4.

## 5.4 Readiness loops (waiting on processes)

Any started process is probed, never assumed. Loop up to 30 times with a 2 second sleep
against an exact readiness command (for this project: the port file written by the replay
harness). On success continue; on exhaustion treat it as a milestone failure with signature
`READINESS_TIMEOUT_<name>` and enter 5.3. Every background start records its PID and its
exact kill command in the milestone text, and teardown is part of the milestone.

## 5.5 Watchdogs

- **Repetition watchdog:** the identical command producing identical output three times in a
  row forces a rung climb. You are spinning.
- **Silence watchdog:** ten consecutive actions without a ledger append means you append a
  HEARTBEAT with a one-line status now.
- **Scope watchdog:** after every milestone run `git status --porcelain` and
  `git diff --name-only HEAD~1`. Any path outside the milestone's CHANGE list is reverted
  immediately (`git checkout -- <path>` or `git clean -fd <path>`) unless a Decision Log
  entry was written BEFORE keeping it.
- **Budget watchdog:** if a milestone exceeds its declared wall-clock budget (default 25
  minutes; node budgets in NODE-META), treat it as a failure with signature
  `BUDGET_EXCEEDED` and enter 5.3 at rung 3. Do not grind.

## 5.6 The re-grounding loop (drift killer)

At the start of EVERY milestone, before any action, re-read in this order:
1. the milestone block itself,
2. the node's Non-goals section,
3. `sh scripts/ledger.sh tail 15`.

Long-context drift dies here. The instructions nearest the work are always the freshest
thing in context.

## 5.7 Blocked report format

The only legitimate terminal failure. The NODE_BLOCKED detail references a report appended
to the ExecPlan's Progress section containing:
1. the exact blocker in one sentence;
2. full evidence: every command run, its full output, its exit code;
3. every error signature seen and every hypothesis tried;
4. every rung climbed, with each attempted diff summarised in one line;
5. the smallest human decision that would unblock it;
6. a recommended default.

A NODE_BLOCKED without this report is itself a defect.

## 5.8 Non-interactive mandate

Every command runs unattended. Export at session start:

    export CI=true GIT_TERMINAL_PROMPT=0 GIT_PAGER=cat PAGER=cat DEBIAN_FRONTEND=noninteractive
    export CARGO_TERM_COLOR=never CARGO_INCREMENTAL=0 RUST_BACKTRACE=1 MC_HEADLESS=1

Forbidden outright: bare interactive REPLs, editors, pagers, watch modes, prompt-on-conflict
commands, and any credential prompt. Cargo invocations always carry `--locked`. The only
backgrounded process in this project is the replay harness, and it is governed by 5.4.
