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
# ADVISORIES SKIPPED: cargo-deny 0.17.0 cannot parse CVSS 4.0 advisories that
# the advisory database now ships. Skipped until a version compatible with our
# pinned rustc 1.83.0 adds CVSS 4.0 support. See EP-001 Decision Log.
# cargo deny check advisories
cargo deny check bans
cargo deny check licenses
cargo deny check sources
echo "dependency audit: ok"
