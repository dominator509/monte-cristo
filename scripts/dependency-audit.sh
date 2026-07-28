#!/usr/bin/env sh
# 6LAYER dependency gate: advisories, bans, licences, sources.
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
[ -f deny.toml ] || { echo "dependency audit: FAIL - deny.toml missing" >&2; exit 1; }
cargo deny check advisories
cargo deny check bans
cargo deny check licenses
cargo deny check sources
echo "dependency audit: ok"
