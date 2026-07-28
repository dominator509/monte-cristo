# SPEC-007 -- Observability

Full detail in OBSERVABILITY.md; this spec is the behavioural contract EP-008 implements
against.

## 1. Required behaviours

1. mc_core emits no logs and has no logging dependency. It returns `CoreEvent`s; the shell
   decides what to record. Verified by `cargo tree -p mc_core` containing no `tracing`.
2. Logs are newline-delimited JSON at `$MC_DATA_DIR/logs/monte-cristo-<date>.jsonl`, rotated
   daily, seven files retained, older deleted.
3. Every record carries `ts`, `level`, `target`, `msg`, `session`, `build`, and `tick` when a
   tick is in scope. `session` is random per process and is never persisted.
4. Domain records add `region`, `scene`, `encounter`, `enemy`, `phase`, `act`, `state_hash`
   where applicable.
5. Redaction: no absolute path outside a declared root, no home directory, no username, no
   environment variable value, no save contents. Paths are logged relative to their root.
6. Metrics are written to `$MC_DATA_DIR/logs/metrics-<date>.json` on clean exit, containing
   every metric in OBSERVABILITY.md section 4 plus `MC_REFERENCE_MACHINE`.
7. The debug overlay is behind the `debug-overlay` feature, reads `StateView` only, and
   cannot alter a replay hash.
8. Crash reports go to `$MC_DATA_DIR/crash/` with the panic message, backtrace, tick, and
   state hash. Nothing is transmitted.

## 2. Explicit non-behaviours

No telemetry, no analytics, no crash upload, no update check, no remote configuration, no
identifier of any kind. These are not defaults to be changed; the code to do them does not
exist and the dependencies that would enable them are absent (INV-09).

## 3. Acceptance criteria (EP-008)

| Criterion | Verification |
|---|---|
| JSON log with the required fields | `crates/mc_shell/tests/log_schema.rs` |
| rotation and seven-file retention | `crates/mc_shell/tests/log_rotation.rs` (fabricates eight days of files) |
| redaction over a full replay | `crates/mc_shell/tests/log_redaction.rs` |
| metrics file complete on clean exit | `crates/mc_shell/tests/metrics_file.rs` |
| reference machine recorded | same test |
| overlay does not affect the hash | `crates/mc_shell/tests/overlay_hash_stability.rs` |
| crash report contains tick and hash | `crates/mc_shell/tests/crash_report.rs` |
| mc_core has no logging dependency | `scripts/security-check.sh` |

## 4. Debugging contract

Because the game is deterministic, the reproduction unit is `(save, tape)`. A player report
containing both reproduces the exact situation on the developer's machine. The tape format is
therefore a first-class shipped artifact, documented in the artifact README, and not a test
detail.
