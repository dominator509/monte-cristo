#!/usr/bin/env sh
# 6LAYER lint gate: clippy at deny level, POSIX shell check, crate layer enforcement.
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
cargo clippy --locked --workspace --all-targets -- -D warnings

# Every shipped script must be POSIX sh with no bashisms.
for f in scripts/*.sh scripts/probes/*.sh; do
  [ -f "$f" ] || continue
  sh -n "$f" || { echo "lint: FAIL - $f is not POSIX-clean" >&2; exit 1; }
done

# Crate layer DAG (ARCHITECTURE.md section 4). An upward import is signature LAYER_VIOLATION.
layer_check() {
  crate="$1"; shift
  for forbidden in "$@"; do
    if cargo tree -p "$crate" --depth 1 2>/dev/null | grep -q "^[^ ]* $forbidden\b\|[ ]$forbidden v"; then
      echo "lint: FAIL - LAYER_VIOLATION: $crate depends on $forbidden" >&2
      exit 1
    fi
  done
}
if [ -d crates/mc_core ]; then
  layer_check mc_core mc_data mc_tape mc_shell mc_tools macroquad tracing
  layer_check mc_data mc_tape mc_shell mc_tools macroquad
  layer_check mc_tape mc_data mc_shell mc_tools macroquad
  layer_check mc_shell mc_tools
fi
echo "lint: ok"
