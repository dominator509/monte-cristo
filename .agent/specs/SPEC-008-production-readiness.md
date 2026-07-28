# SPEC-008 -- Production readiness

The mechanical definition of shippable. PRODUCTION_READINESS.md is the checklist instance;
this spec is the contract the checklist implements.

## 1. Ship gate

All of the following, observed in one session, not recalled from a previous run:

1. `sh scripts/verify.sh` from scratch prints every sentinel through `verify: ok`.
2. `sh scripts/live-fire.sh` prints `live-fire: ok`, with LF-01 through LF-12 each reported
   individually as passing.
3. `sh scripts/reality-gate.sh` prints `reality gate: ok`.
4. `sh scripts/production-readiness-check.sh` prints `production readiness: ok`.
5. Every line of PRODUCTION_READINESS.md is checked with its command or artifact cited.
6. Every acceptance criterion in every ExecPlan is marked met with observed evidence.
7. Artifacts exist for all three targets with a verifying SHA256SUMS, and each replays the
   golden tape to the hash recorded on Linux.
8. The version tag exists and CHANGELOG.md has a Determinism subsection for it.
9. Auto-Deploy Authorization is `no`, so the final action is to print the MANUAL publish
   command and stop.
10. `RUN_COMPLETE` appended to the ledger.

## 2. Evidence rules

A gate passes only if it was run in this session and its sentinel appeared in real output.
Claiming a pass from memory, from a previous run, or from reading the script is fabrication
(AGENTS.md section 9). Every `MILESTONE_PASS` ledger event carries the observed sentinel in
its detail. Final review re-runs `verify.sh` from scratch; cached green is not green.

## 3. Quality floors

| Dimension | Floor |
|---|---|
| Coverage | TESTING.md section 8 |
| p99 core step | 4.0 ms |
| p99 frame | 16.6 ms |
| Cold start to title | 2.5 s |
| Golden-tape replay | 15 minutes |
| Peak resident memory | 512 MB |
| Advisories | zero high or critical unwaived |
| Licences | inside the allowlist, no waiver path |
| unsafe in mc_core and mc_data | zero |
| Ignored tests | zero |
| Dangling content references | zero |

## 4. Regression protection

Every core outcome has a live-fire proof; every committed tape has a recorded hash; every
historical fuzz crash has a permanent corpus entry and a unit test; every rollback has a
postmortem ADR. The golden tape is the single most valuable regression asset in the
repository and re-recording it to make a test pass is the most serious process violation
available (CONTRIBUTING.md section 4).

## 5. Documentation completeness

Checked in EP-010's review milestone: ARCHITECTURE.md invariants match the code;
ENVIRONMENT.md dependency table matches Cargo.lock; PREFLIGHT.md matches
scripts/preflight.sh; COMMANDS.md matches the scripts directory; every spec's Validation
table names tests that exist.

## 6. What is deliberately not gated

Art quality, music quality, writing quality, and balance. No script can judge them. They are
reviewed by a human before publication, which is the reason Auto-Deploy Authorization is
`no` -- the human step exists precisely to hold the things a gate cannot.
