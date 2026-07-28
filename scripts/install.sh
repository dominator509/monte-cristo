#!/usr/bin/env sh
# 6LAYER install gate: fetch and build dependencies from the committed lockfile.
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
[ -f Cargo.toml ] || { echo "install: FAIL - no workspace manifest" >&2; exit 1; }
[ -f Cargo.lock ] || { echo "install: FAIL - Cargo.lock missing; it is committed at EP-001 M3" >&2; exit 1; }
if [ "${CARGO_REGISTRY_MODE:-online}" = "vendored" ]; then
  cargo fetch --locked --offline
else
  cargo fetch --locked
fi
echo "install: ok"
