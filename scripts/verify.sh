#!/usr/bin/env sh
# 6LAYER verify: every gate, in order. This is the single definition of green.
# CI invokes only this script (ADR-013).
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
sh scripts/preflight.sh
sh scripts/install.sh
sh scripts/lint.sh
sh scripts/format-check.sh
sh scripts/typecheck.sh
sh scripts/test-unit.sh
sh scripts/test-integration.sh
sh scripts/test-e2e.sh
sh scripts/build.sh
sh scripts/security-check.sh
sh scripts/dependency-audit.sh
sh scripts/reality-gate.sh
sh scripts/smoke-test.sh
sh scripts/live-fire.sh
echo "verify: ok"
