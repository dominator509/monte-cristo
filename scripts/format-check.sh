#!/usr/bin/env sh
# 6LAYER format gate: rustfmt in check mode across the workspace.
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
cargo fmt --all -- --check
echo "format check: ok"
