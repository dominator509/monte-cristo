NODE-META-BEGIN
ID: EP-008
DEPS: EP-007
MAX_ATTEMPTS_PER_MILESTONE: 6
VERIFY: sh scripts/smoke-test.sh
VERIFY_SENTINEL: smoke test: ok
GREEN_TAG: green/EP-008
NODE-META-END

# EP-008 -- Observability and operations

## 1. Purpose / Big Picture

Make the software explicable after the fact, entirely locally. Structured JSON logs with
redaction and rotation, the frame-time and step-time histograms, the debug overlay behind a
feature, local crash reports, and the runbooks that turn a player report into a reproduction.

Nothing is transmitted anywhere. The reproduction unit for this project is `(save, tape)`,
which is strictly more useful than telemetry and requires no privacy trade-off at all.

## 2. Scope

Logging, metrics, the overlay, crash reports, and the operational runbooks.

## 3. Non-goals

No telemetry, analytics, crash upload, update check, or remote configuration -- the code for
these must not exist (INV-09). No release engineering (EP-009). No logging in mc_core, ever.

## 4. Context and Orientation

SPEC-007 is authoritative. The constraint that shapes this node is that mc_core has no
logging dependency and must not gain one, because a logging crate brings a clock and a clock
breaks INV-01. Core returns `CoreEvent`s; the shell decides what to record.

## 5. Files to Read First

- .agent/specs/SPEC-007-observability.md
- OBSERVABILITY.md in full
- OPERATIONS.md sections 4, 5, 6
- SECURITY.md section 8

## 6. Expected Changed Files

- crates/mc_shell/src/log.rs
- crates/mc_shell/src/metrics.rs
- crates/mc_shell/src/overlay.rs
- crates/mc_shell/src/crash.rs
- crates/mc_shell/src/app.rs
- crates/mc_shell/Cargo.toml
- crates/mc_shell/tests/log_schema.rs
- crates/mc_shell/tests/log_rotation.rs
- crates/mc_shell/tests/metrics_file.rs
- crates/mc_shell/tests/overlay_hash_stability.rs
- crates/mc_shell/tests/crash_report.rs
- scripts/smoke-test.sh
- docs/runbooks/content-pack-failure.md
- docs/runbooks/save-failure.md
- docs/runbooks/determinism-regression.md

## 7. Interfaces and Contracts

Log record fields exactly as SPEC-007 section 1 item 3. Metric names and budgets exactly as
OBSERVABILITY.md section 4. The overlay is behind the `debug-overlay` cargo feature and is
absent from a release build's default feature set.

## 8. Milestones

### M1: Structured logging
GOAL: Newline-delimited JSON logs with the required fields, under the data root.
READ: SPEC-007 section 1, OBSERVABILITY.md sections 1, 2
CHANGE: crates/mc_shell/src/log.rs, crates/mc_shell/src/app.rs, crates/mc_shell/Cargo.toml,
  crates/mc_shell/tests/log_schema.rs
CONTENT: `tracing` with a JSON file appender writing to
  `$MC_DATA_DIR/logs/monte-cristo-<date>.jsonl` through `fsroot::confine`. Every record
  carries `ts`, `level`, `target`, `msg`, `session`, `build`, and `tick` when in scope.
  `session` is random per process and never persisted. Domain records add `region`, `scene`,
  `encounter`, `enemy`, `phase`, `act`, `state_hash`.
RUN:
  cargo test --locked -p mc_shell --test log_schema
  cargo tree -p mc_core --depth 1
EXPECT: test passes; `cargo tree` for mc_core shows no `tracing`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-008 MILESTONE_PASS "M1 log schema, core still clean"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-008][M1] add structured JSON logging"

### M2: Rotation and retention
GOAL: Daily rotation with seven files retained and older files deleted.
READ: SPEC-007 section 1 item 2
CHANGE: crates/mc_shell/src/log.rs, crates/mc_shell/tests/log_rotation.rs
CONTENT: daily rotation by date-stamped filename; on startup, delete log files older than the
  seventh most recent. The test fabricates eight days of files and asserts exactly seven
  survive and that the oldest is the one removed.
RUN: cargo test --locked -p mc_shell --test log_rotation
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-008 MILESTONE_PASS "M2 rotation and retention"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-008][M2] add log rotation with seven-file retention"

### M3: Metrics
GOAL: Every metric in OBSERVABILITY.md section 4 is recorded and written on clean exit.
READ: OBSERVABILITY.md section 4, SPEC-007 section 1 item 6
CHANGE: crates/mc_shell/src/metrics.rs, crates/mc_shell/tests/metrics_file.rs
CONTENT: histograms for core step, total frame, content load, startup to title, save write,
  save load, peak resident memory, and encounter resolve ticks, written to
  `$MC_DATA_DIR/logs/metrics-<date>.json` on clean exit, with `MC_REFERENCE_MACHINE` recorded
  alongside so a budget failure can be attributed to a machine.
RUN: cargo test --locked -p mc_shell --test metrics_file
EXPECT: test passes with every metric present and the reference machine recorded
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-008 MILESTONE_PASS "M3 metrics file complete"
FALLBACK: if peak resident memory cannot be sampled portably, record it on Linux and macOS
  and omit it on Windows with a documented `null`, rather than reporting a fabricated number.
COMMIT: git add -A && git commit -m "[EP-008][M3] add local metrics histograms"

### M4: Debug overlay
GOAL: The overlay exists behind a feature and cannot change a replay hash.
READ: OBSERVABILITY.md section 7
CHANGE: crates/mc_shell/src/overlay.rs, crates/mc_shell/tests/overlay_hash_stability.rs
CONTENT: behind the `debug-overlay` feature, off in release. Shows tick, state hash, p99 step
  and frame times, active region and encounter, spawn budget remaining, and ATB gauges. It
  reads `StateView` only. The test replays the same tape with the feature on and off and
  asserts identical hashes.
RUN: cargo test --locked -p mc_shell --features debug-overlay --test overlay_hash_stability
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-008 MILESTONE_PASS "M4 overlay hash stable"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-008][M4] add debug overlay with proven hash stability"

### M5: Crash reports
GOAL: A panic writes a local report containing the tick and state hash, and transmits nothing.
READ: OBSERVABILITY.md section 10
CHANGE: crates/mc_shell/src/crash.rs, crates/mc_shell/tests/crash_report.rs
CONTENT: a panic hook writing to `$MC_DATA_DIR/crash/` with the panic message, backtrace,
  current tick, and state hash, redacted per SECURITY.md section 8. The test forces a real
  panic in a child process and asserts the report exists with both fields and that no socket
  was opened.
RUN: cargo test --locked -p mc_shell --test crash_report
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-008 MILESTONE_PASS "M5 crash report local only"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-008][M5] add local crash reports"

### M6: Runbooks and operational smoke
GOAL: The three runbooks exist and `smoke-test.sh` proves the operational surface.
READ: OPERATIONS.md sections 4, 5, .agent/templates/runbook-template.md
CHANGE: docs/runbooks/*.md, scripts/smoke-test.sh
CONTENT: three runbooks following the template: content pack failure, save failure, and
  determinism regression. `smoke-test.sh` exercises the four health checks of OPERATIONS.md
  section 4 plus the cold-start and frame budgets, and prints `smoke test: ok`.
RUN: sh scripts/smoke-test.sh
EXPECT: `smoke test: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-008 MILESTONE_PASS "M6 smoke test: ok"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-008][M6] add runbooks and operational smoke test"

## 9. Validation and Acceptance

| Criterion | Command | Expected |
|---|---|---|
| Log schema correct | `cargo test --locked -p mc_shell --test log_schema` | pass |
| Rotation and retention | `cargo test --locked -p mc_shell --test log_rotation` | pass |
| Redaction still holds | `cargo test --locked -p mc_shell --test log_redaction` | pass |
| Metrics complete | `cargo test --locked -p mc_shell --test metrics_file` | pass |
| Overlay cannot change a hash | `cargo test --locked -p mc_shell --features debug-overlay --test overlay_hash_stability` | pass |
| Crash report local | `cargo test --locked -p mc_shell --test crash_report` | pass |
| Core has no logging dependency | `cargo tree -p mc_core --depth 1` | no `tracing` |
| Node gate | `sh scripts/smoke-test.sh` | `smoke test: ok` |

## 10. Idempotence and Recovery

All milestones are additive. Log and metrics files are regenerable runtime artifacts under
the data root and are safe to delete. To re-enter cold: read Progress, find the first
unchecked milestone, re-run the previous milestone's RUN, continue.

## 11. Progress

- [ ] M1 structured logging
- [ ] M2 rotation and retention
- [ ] M3 metrics
- [ ] M4 debug overlay
- [ ] M5 crash reports
- [ ] M6 runbooks and operational smoke

## 12. Surprises and Discoveries

<empty>

## 13. Decision Log

<empty>

## 14. Outcomes and Retrospective

<empty>
