#!/usr/bin/env sh
# 6LAYER integration test gate: real files on a real filesystem.
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
cargo test --locked -p mc_data --tests
cargo test --locked -p mc_shell --tests

# No test may leave residue in the working tree (TESTING.md section 7).
residue=$(git status --porcelain 2>/dev/null || true)
if [ -n "$residue" ]; then
  echo "integration tests: FAIL - tests left files behind:" >&2
  printf '%s\n' "$residue" >&2
  exit 1
fi
echo "integration tests: ok"
