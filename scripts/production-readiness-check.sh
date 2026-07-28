#!/usr/bin/env sh
# 6LAYER ship gate: verify.sh plus the automatable lines of PRODUCTION_READINESS.md.
set -eu
export CI=true
export GIT_TERMINAL_PROMPT=0
export GIT_PAGER=cat
export PAGER=cat
export DEBIAN_FRONTEND=noninteractive
export CARGO_TERM_COLOR=never
export CARGO_INCREMENTAL=0
export RUST_BACKTRACE=1
export MC_HEADLESS=1
fail() { echo "production readiness: FAIL - $1" >&2; exit 1; }

sh scripts/verify.sh

# Testing block
cargo llvm-cov --locked --workspace --fail-under-lines 85 || fail "coverage below floor"
ig=$(cargo test --locked --workspace -- --list 2>/dev/null | grep -c ignored || true)
[ "$ig" = "0" ] || fail "$ig ignored tests present"

# Functional block: the content invariants that encode the refused designs.
cargo test --locked -p mc_data --test content_invariants || fail "content invariants"
cargo test --locked -p mc_core --test final_encounter || fail "final encounter gating"

# Accessibility and privacy blocks
cargo test --locked -p mc_shell --test motion_zero || fail "motion zero"
cargo test --locked -p mc_shell --test advisory_screen || fail "content advisory"
cargo test --locked -p mc_shell --test log_redaction || fail "log redaction"
cargo test --locked -p mc_data --test glyph_parity || fail "glyph parity"

# Operations block
cargo test --locked -p mc_data --test backup_restore || fail "backup and restore"
cargo run --locked --quiet -p mc_tools -- save-migrate --dir tests/fixtures/saves-v1 --dry-run \
  || fail "previous-version saves do not migrate"

# Release block
OUT="${MC_ARTIFACT_DIR:-target/artifacts}"
[ -f "$OUT/SHA256SUMS" ] || fail "no SHA256SUMS in $OUT"
( cd "$OUT" && if command -v sha256sum >/dev/null 2>&1; then sha256sum -c SHA256SUMS; else shasum -a 256 -c SHA256SUMS; fi ) \
  || fail "artifact checksums do not verify"

# Pack hygiene: no unresolved placeholders in agent-facing docs.
# docs/6Layer-MasterPrompt-* is excluded: it is the generating prompt kept for
# provenance, and its skeleton sections legitimately contain placeholder tokens.
n=$(grep -rn '{{[A-Z_][A-Z_]*}}' --include='*.md' . 2>/dev/null | grep -v '^\./docs/6Layer-MasterPrompt' | wc -l)
[ "$n" -eq 0 ] || fail "$n unresolved placeholder sequences in markdown"

# Adapter parity.
first=""
for f in AGENTS.md CLAUDE.md .hermes/6layer.md .openclaw/6layer.md; do
  c=$(awk '/PRIME-BLOCK-BEGIN/,/PRIME-BLOCK-END/' "$f" | cksum)
  [ -z "$first" ] && first="$c"
  [ "$c" = "$first" ] || fail "adapter parity broken at $f"
done

echo "production readiness: auto-deploy authorization is no; publish step remains MANUAL"
echo "production readiness: ok"
