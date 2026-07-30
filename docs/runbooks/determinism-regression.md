# Runbook: determinism regression

**Applies to:** build host | published artifact
**Severity:** blocking

## Symptom
A committed tape reports a state-hash mismatch, or two identical replays produce different
hashes.

## Diagnosis
| Step | Command | Interpretation |
|---|---|---|
| 1 | `cargo run --locked -p mc_tools -- replay --tape <tape> --print-hash` | Captures the first observed hash. |
| 2 | `cargo run --locked -p mc_tools -- replay --tape <tape> --print-hash` | A different result proves cross-run nondeterminism. |
| 3 | `cargo run --locked -p mc_tools -- replay --tape <tape> --assert-hash` | Reports the first divergent checkpoint against the tape contract. |

## Action
Stop release work. Inspect the first divergent transition for floating-point state, unstable
iteration order, clock access, threading, ambient randomness, or platform-dependent input.
Fix the cause in the active ExecPlan; never rewrite the tape merely to accept the new hash.

## Verification
Run `sh scripts/live-fire.sh`; it must report LF-09 and `live-fire: ok`.

## If this does not work
Append the normalized error signature and follow the failure ladder in `.agent/LOOPS.md`
through its declared fallback or structured `NODE_BLOCKED` report.

## Prevention
`scripts/live-fire.sh` proves cross-run determinism, and `scripts/smoke-test.sh` asserts the
golden smoke tape before release.
