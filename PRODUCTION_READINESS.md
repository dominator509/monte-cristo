# PRODUCTION READINESS — Monte Cristo

Every item below must be checked before the ship gate passes (EP-010 condition 5).

## Code Quality
- [x] All source files build with zero warnings (`cargo build -D warnings`)
- [x] All tests pass (`cargo test --workspace`)
- [x] 0 ignored tests
- [x] Clippy clean (`cargo clippy -- -D warnings`)
- [x] Format check clean (`cargo fmt --check`)

## Security (SECURITY.md section 13)
- [x] `#![forbid(unsafe_code)]` in mc_core and mc_data
- [x] No f32/f64, HashMap, HashSet, SystemTime, thread_rng in mc_core
- [x] No networking crates in dependency tree
- [x] Single point of file access via fsroot::confine (INV-07)
- [x] No committed secrets
- [x] Dependency policy passes (`cargo deny check`)
- [x] Fuzz targets exist with non-empty corpora
- [x] Security tests exist (no_socket, log_redaction, fsroot_confine)
- [x] mc_core has no logging dependency

## Determinism (SPEC-000)
- [x] All game state is pure function of (seed, content pack, input tape)
- [x] Tape replay is deterministic (hash matches across runs)
- [x] No I/O in mc_core
- [x] Golden-full.tape and golden-smoke.tape recorded and verified

## Content
- [x] All content files parse
- [x] Content pack bakes deterministically
- [x] All cross-file references resolve
- [x] Content invariants pass (6/6 tests)
- [x] Forced-failure tests pass (7/7 tests)

## Observability (SPEC-007)
- [x] JSON structured logging (monte-cristo-<date>.jsonl)
- [x] Log rotation (10 MiB)
- [x] Metrics file on clean exit (metrics-<date>.json)
- [x] Crash reports (crash/<timestamp>.json)
- [x] Profiler module (behind `profiling` feature)
- [x] Debug overlay module (behind `debug-overlay` feature)

## CI/CD
- [x] GitHub Actions workflow (`.github/workflows/ci.yml`)
- [x] Dockerfile (multi-stage build)
- [x] Release script (cross-compile + SHA256SUMS)

## Live Fire
- [x] LF-01: act1 tape replay (hash match + FLG_ARRESTED)
- [x] LF-02: 168 months calendar, 4 disciplines at rank 3
- [x] LF-03: Field encounter resolves
- [x] LF-04: Spawn gating (500 rolls, all regions)
- [x] LF-05: Encounter budget decay
- [x] LF-06: Confidence scene gating
- [x] LF-07: Save-load identity
- [x] LF-08: Golden full tape replay (hash match)
- [x] LF-09: Determinism across runs (hash consistent)
- [ ] LF-10: Content integrity
- [x] LF-11: Frame budget benchmark (p99 0.000400 ms from the EP-010 M4 benchmark and live-fire run)
- [x] LF-12: Final encounter two-phase
