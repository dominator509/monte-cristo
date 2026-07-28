# OBSERVABILITY -- MONTE CRISTO

Everything here is local. Nothing is transmitted anywhere, ever (INV-09, and the privacy
requirements in PROJECT_BRIEF).

## 1. Logging strategy

`tracing` in mc_shell and mc_tools only. mc_core emits no logs at all -- it returns
structured events in `StateView::events` and the shell decides whether to record them. This
keeps a logging dependency, and therefore a clock, out of the deterministic core (INV-01).

Sink: newline-delimited JSON to `$MC_DATA_DIR/logs/monte-cristo-<date>.jsonl`, rotated
daily, retained for seven files, then deleted. Console output is human-readable and is used
only when a terminal is attached.

Levels: `error` for a failure the player will notice; `warn` for a recovered fault;
`info` for lifecycle (start, content loaded, save written, shutdown); `debug` for scene and
encounter transitions; `trace` for per-tick detail, off by default and never enabled in a
release build's default configuration.

## 2. Structured fields

Every record carries: `ts`, `level`, `target`, `msg`, `session` (a random per-process value
that is not persisted and is not an identifier of the player), `build` (version and git
short hash), and `tick` when a simulation tick is in scope.

Domain records add: `region` (R01 through R15), `scene`, `encounter`, `enemy`, `phase`,
`act`, and `state_hash` at checkpoints.

## 3. Redaction

Never logged: absolute paths outside the three declared roots, the operator's home directory
or username, environment variable values, and save file contents. Paths inside a root are
logged relative to that root. A test replays a golden tape with logging on and asserts the
output contains no `/home/`, no `/Users/`, and no `C:\Users` sequence. That test is part of
EP-008's acceptance and is re-run in the ship gate.

## 4. Metrics

Local histograms, written to `$MC_DATA_DIR/logs/metrics-<date>.json` on clean exit:

| Metric | Unit | Budget |
|---|---|---|
| `core.step.duration` | microseconds | p99 under 4000 |
| `frame.total.duration` | microseconds | p99 under 16600 |
| `content.load.duration` | milliseconds | under 900 |
| `startup.to_title.duration` | milliseconds | under 2500 |
| `save.write.duration` | milliseconds | under 120 |
| `save.load.duration` | milliseconds | under 250 |
| `memory.resident.peak` | bytes | under 536870912 |
| `encounter.resolve.ticks` | ticks | recorded, no budget |

`MC_REFERENCE_MACHINE` is written into the same file so a budget failure can be attributed
to a machine rather than to a regression.

## 5. Traces

Not applicable: there is no distributed system. The equivalent is the tape, which is a
complete, replayable trace of an entire session, and is strictly more useful.

## 6. Health checks

Listed in OPERATIONS.md section 4. In-process, the shell exposes `--verify-content`,
`--check-paths`, and `--save-info`, all of which run without opening a window.

## 7. Debug overlay

Behind the `debug-overlay` cargo feature, off in release. Shows: current tick, state hash,
p99 step and frame times, active region and encounter, spawn budget remaining, and the ATB
gauges. It reads `StateView` only and can never mutate state (INV-04), so enabling it cannot
change a replay's hash -- and there is a test that asserts exactly that.

## 8. Alerts

There is nothing to alert. The build-host equivalent is the gate sentinel: a missing
sentinel is the alert, and it halts the run.

## 9. SLIs and SLOs

Not applicable in the service sense. The equivalent commitments are the performance budgets
in section 4, which are ship criteria (LF-11) rather than aspirational targets.

## 10. Production debugging

A player report is actionable when it includes: the version line, the metrics file, the last
log file, and, ideally, a save. Because the game is deterministic, a save plus a tape
reproduces the exact situation on the developer's machine. That is the whole debugging
strategy, and it is why the tape format is a first-class artifact rather than a test detail.

Crash reports are written to `$MC_DATA_DIR/crash/` as a panic message, a backtrace, the
current tick, and the state hash. They are never transmitted. The README in the artifact
tells the player where they are and that sending one is their choice.

## 11. Observability acceptance criteria (wired into EP-008)

- [ ] JSON log file created under MC_DATA_DIR with the fields in section 2
- [ ] rotation and seven-file retention proven by a test that fabricates eight days of files
- [ ] redaction test passing over a full golden-tape replay
- [ ] metrics file written on clean exit with every metric in section 4 present
- [ ] `MC_REFERENCE_MACHINE` present in the metrics file
- [ ] debug overlay behind its feature, and the overlay-does-not-affect-hash test passing
- [ ] crash report written on a forced panic, containing tick and state hash, and not
      transmitted anywhere
- [ ] mc_core still has no logging dependency in `cargo tree`
