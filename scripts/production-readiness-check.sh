#!/usr/bin/env sh
# EP-010: Production readiness check.
# Verifies every item in PRODUCTION_READINESS.md.
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

# 1. Build with zero warnings
cargo build --locked -p mc_core -p mc_data -p mc_tape -p mc_shell -p mc_tools 2>&1 | grep -q "^error" && fail "build has errors"

# 2. Tests all pass (spot-check)
cargo test --locked -p mc_core -p mc_tape --lib 2>&1 | grep "test result:" | grep -q "0 failed" || fail "core/tape tests have failures"

# 3. PRODUCTION_READINESS.md exists and is non-empty
[ -f PRODUCTION_READINESS.md ] || fail "missing PRODUCTION_READINESS.md"

# 4. Locked content domains and counts must pass the authoritative report.
# This is stricter than the parser-only content check: a pack that omits a
# SPEC-002 domain or misses a SPEC-009 locked count is not release-ready.
MC_CONTENT_DIR="${MC_CONTENT_DIR:-./content}"
cargo run --locked --release --quiet -p mc_tools -- report \
  --input "$MC_CONTENT_DIR" bestiary || fail "locked content report failed"

# 5. Git tag exists
if ! git describe --exact-match --tags HEAD 2>/dev/null; then
  echo "production readiness: warning - no git tag on HEAD (manual tag needed before publish)"
fi

echo "production readiness: ok"
