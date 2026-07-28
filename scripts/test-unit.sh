#!/usr/bin/env sh
# 6LAYER unit test gate: library and property tests.
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
cargo test --locked --workspace --lib
cargo test --locked -p mc_core --tests
echo "unit tests: ok"
