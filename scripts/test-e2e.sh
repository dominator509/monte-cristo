#!/usr/bin/env sh
# 6LAYER e2e gate: tape replay through the real command bus, plus recorded hash assertions.
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
cargo test --locked -p mc_tape --tests

# Every committed tape must still match its recorded hash (TESTING.md section 6 item 5).
if [ -f tapes/HASHES.txt ]; then
  while read -r tape hash; do
    [ -n "$tape" ] || continue
    case "$tape" in \#*) continue ;; esac
    hash=$(printf '%s' "$hash" | tr -d '\r')
    [ -f "tapes/$tape" ] || { echo "e2e tests: FAIL - missing tape: tapes/$tape" >&2; exit 1; }
    got=$(cargo run --locked --quiet -p mc_tools -- replay --tape "tapes/$tape" --print-hash | tr -d ' ')
    [ "$got" = "$hash" ] || {
      echo "e2e tests: FAIL - DETERMINISM_HASH_MISMATCH for $tape: want $hash got $got" >&2
      echo "Do NOT re-record the tape. See .agent/LOOPS.md section 5.3." >&2
      exit 1
    }
  done < tapes/HASHES.txt
fi
echo "e2e tests: ok"
