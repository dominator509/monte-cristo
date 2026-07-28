# Checklist: preflight

- [ ] Working directory is the repository root (AGENTS.md and .agent/ both present)
- [ ] `git status --porcelain` is empty (clean tree before starting a node)
- [ ] `.env` exists and every REQUIRED variable from PREFLIGHT.md section 6 is set
- [ ] The three directories exist and are writable: `$MC_CONTENT_DIR`, `$MC_ARTIFACT_DIR`,
      `$MC_DATA_DIR`
- [ ] `rustc --version` reports exactly `$MC_RUST_VERSION`
- [ ] `cargo deny --version`, `cargo fuzz --version`, `cargo llvm-cov --version` all match
      `$MC_CARGO_TOOLS`
- [ ] `sh scripts/probes/graphics_stack.sh` exits 0
- [ ] `CARGO_REGISTRY_MODE` is `online` or `vendored`, and if `vendored` then `vendor/` is
      populated and `.cargo/config.toml` points at it
- [ ] `MC_HEADLESS` is exactly `1`
- [ ] `sh scripts/preflight.sh` prints `preflight: ok`
- [ ] `sh scripts/ledger.sh tail 30` shows the expected recent history and no open lease you
      do not own
- [ ] No known blocker: `grep -n NODE_BLOCKED .agent/state/LEDGER.md` returns nothing, or the
      blocker has been resolved and recorded
