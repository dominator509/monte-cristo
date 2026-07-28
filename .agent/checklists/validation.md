# Checklist: validation (every gate, in order, with its sentinel)

Run from the repository root. Observe each sentinel in real output; do not infer.

- [ ] `sh scripts/preflight.sh` -> `preflight: ok`
- [ ] `sh scripts/lint.sh` -> `lint: ok`
- [ ] `sh scripts/format-check.sh` -> `format check: ok`
- [ ] `sh scripts/typecheck.sh` -> `typecheck: ok`
- [ ] `sh scripts/test-unit.sh` -> `unit tests: ok`
- [ ] `sh scripts/test-integration.sh` -> `integration tests: ok`
- [ ] `sh scripts/test-e2e.sh` -> `e2e tests: ok`
- [ ] `sh scripts/build.sh` -> `build: ok`
- [ ] `sh scripts/security-check.sh` -> `security check: ok`
- [ ] `sh scripts/dependency-audit.sh` -> `dependency audit: ok`
- [ ] `sh scripts/reality-gate.sh` -> `reality gate: ok`
- [ ] `sh scripts/smoke-test.sh` -> `smoke test: ok`
- [ ] `sh scripts/live-fire.sh` -> `live-fire: ok`
- [ ] `sh scripts/verify.sh` -> `verify: ok`

**Project-specific spot checks:**
- [ ] `cargo run --locked -p mc_tools -- replay --tape tapes/golden-full.tape --assert-hash`
      -> `hash: match`
- [ ] `cargo run --locked -p mc_tools -- validate --input "$MC_CONTENT_DIR"` -> `content: ok`
- [ ] `for f in AGENTS.md CLAUDE.md .hermes/6layer.md .openclaw/6layer.md; do awk '/PRIME-BLOCK-BEGIN/,/PRIME-BLOCK-END/' "$f" | cksum; done`
      -> all four lines identical
- [ ] No ignored tests: `cargo test --locked --workspace -- --list | grep -c ignored` -> 0
