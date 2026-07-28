# PRODUCTION READINESS -- MONTE CRISTO

Every line has a verifying command or an artifact path. `sh scripts/production-readiness-check.sh`
automates every line marked AUTO and prints `production readiness: ok` only when all of them
pass. Lines marked DOC are verified by opening the named file during EP-010's review
milestone and are checked off with the evidence recorded in the ExecPlan.

## Functional

- [ ] AUTO -- LF-01 through LF-12 all pass: `sh scripts/live-fire.sh` prints `live-fire: ok`
- [ ] AUTO -- every spec-required behaviour implemented: `sh scripts/test-integration.sh` and `sh scripts/test-e2e.sh`
- [ ] AUTO -- the golden tape reaches the epilogue: `cargo run --locked -p mc_tools -- replay --tape tapes/golden-full.tape --assert-hash`
- [ ] AUTO -- final encounter phase 2 is damage-immune and flag-gated: `cargo test --locked -p mc_core --test final_encounter`
- [ ] AUTO -- content non-goals hold (no alternate ending, no Mercedes route, no spared Villefort, no saved Edouard): `cargo test --locked -p mc_data --test content_invariants`
- [ ] DOC -- no known critical bugs, or each accepted by an ADR: DECISIONS.md

## Testing

- [ ] AUTO -- one fresh run of `sh scripts/verify.sh` shows every sentinel through `verify: ok`
- [ ] AUTO -- coverage floors met: `cargo llvm-cov --locked --workspace --fail-under-lines 85`
- [ ] AUTO -- every committed tape matches its recorded hash: `sh scripts/test-e2e.sh`
- [ ] AUTO -- zero ignored tests: `cargo test --locked --workspace -- --list | grep -c ignored` returns 0
- [ ] AUTO -- forced-failure suite present and passing: `cargo test --locked --workspace --test forced_failures`

## Reality

- [ ] AUTO -- `sh scripts/reality-gate.sh` prints `reality gate: ok`
- [ ] AUTO -- `sh scripts/live-fire.sh` prints `live-fire: ok`
- [ ] AUTO -- zero test-double leakage into production paths: covered by the reality gate over `crates/*/src` and `content`
- [ ] AUTO -- zero demo modes and zero behaviour-altering cfg in mc_core: `sh scripts/security-check.sh`

## Security

- [ ] AUTO -- no committed secrets: `sh scripts/security-check.sh`
- [ ] AUTO -- logs redacted: `cargo test --locked -p mc_shell --test log_redaction`
- [ ] AUTO -- dependency audit within threshold: `sh scripts/dependency-audit.sh`
- [ ] AUTO -- licences inside the allowlist: `cargo deny check licenses`
- [ ] AUTO -- no unsafe in mc_core or mc_data: `sh scripts/security-check.sh`
- [ ] AUTO -- no socket opened during a full replay: `cargo test --locked -p mc_shell --test no_socket`
- [ ] AUTO -- path confinement holds against traversal: `cargo test --locked -p mc_shell --test fsroot_confine`
- [ ] AUTO -- both parsers fuzzed with committed corpora: `sh scripts/security-check.sh`
- [ ] DOC -- production-data rules documented: SECURITY.md section 10

## Performance

- [ ] AUTO -- p99 core step under 4.0 ms: `cargo bench --locked -p mc_core -- battle_step`
- [ ] AUTO -- p99 frame under 16.6 ms on the reference machine: `sh scripts/smoke-test.sh`
- [ ] AUTO -- cold start to title under 2.5 s: `sh scripts/smoke-test.sh`
- [ ] AUTO -- golden-tape replay under 15 minutes: timed by `sh scripts/live-fire.sh`
- [ ] AUTO -- no unbounded memory growth over a long replay: `cargo test --locked -p mc_tape --test memory_ceiling`
- [ ] DOC -- MC_REFERENCE_MACHINE recorded in the metrics artifact: `$MC_DATA_DIR/logs/metrics-*.json`

## Accessibility

- [ ] AUTO -- input fully remappable and the remap survives restart: `cargo test --locked -p mc_shell --test input_remap`
- [ ] AUTO -- no information carried by colour alone: `cargo test --locked -p mc_data --test glyph_parity`
- [ ] AUTO -- shake and flash zero setting produces zero camera offset and zero luminance delta: `cargo test --locked -p mc_shell --test motion_zero`
- [ ] AUTO -- ATB Wait mode halts the gauge whenever a menu is open: `cargo test --locked -p mc_core --test atb_wait_mode`
- [ ] DOC -- dyslexia-friendly font present at identical metrics: `crates/mc_shell/assets/fonts/`
- [ ] DOC -- captions exist for every information-bearing audio cue: SPEC-004 caption table

## Privacy

- [ ] AUTO -- no personal data collected: `sh scripts/security-check.sh` (no telemetry crates, no socket)
- [ ] DOC -- exact list of files written, and uninstall instructions: DEPLOYMENT.md section 3, OPERATIONS.md section 7
- [ ] AUTO -- content advisory screen present before the title screen and remembered: `cargo test --locked -p mc_shell --test advisory_screen`

## Operations

- [ ] AUTO -- backup and restore verified for real, including corruption rejection: `cargo test --locked -p mc_data --test backup_restore`
- [ ] AUTO -- previous-version saves migrate cleanly: `cargo run --locked -p mc_tools -- save-migrate --dir tests/fixtures/saves-v1 --dry-run`
- [ ] AUTO -- rollback drill performed: evidence in EP-009 Progress, M5
- [ ] DOC -- runbooks complete: OPERATIONS.md sections 5 and 6

## Release

- [ ] AUTO -- artifacts exist for all three targets with SHA256SUMS: `sh scripts/build.sh` then `ls "$MC_ARTIFACT_DIR"`
- [ ] AUTO -- each artifact replays the golden tape to the Linux hash: EP-009 M4
- [ ] AUTO -- THIRD-PARTY-LICENSES.txt generated and inside the allowlist: `cargo deny check licenses`
- [ ] DOC -- version tag created and CHANGELOG updated: RELEASE.md
- [ ] DOC -- Auto-Deploy is `no`, so the MANUAL publish command is printed and not executed: EP-010 final milestone
