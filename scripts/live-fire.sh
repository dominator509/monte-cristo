#!/usr/bin/env sh
# 6LAYER live-fire: one scripted proof per core user outcome, against the real entry point
# and real content. This script is the definition of "the software actually works".
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
fail() { echo "live-fire: FAIL - $1" >&2; exit 1; }
mc() { cargo run --locked --release --quiet -p mc_tools -- "$@"; }

[ -f content.pack ] || mc bake --input "${MC_CONTENT_DIR:-./content}" --output content.pack

# LF-01 new-game-to-arrest
mc replay --tape tapes/act1.tape --assert-hash >/dev/null || fail "LF-01 act1 tape hash"
mc replay --tape tapes/act1.tape --require-flag ACT1_ARREST >/dev/null || fail "LF-01 ACT1_ARREST not set"
echo "LF-01 new-game-to-arrest ok"

# LF-02 if-calendar-and-curriculum
mc prove if-calendar --months 168 --faria-at 72 --min-rank3-disciplines 4 || fail "LF-02"
echo "LF-02 if-calendar-and-curriculum ok"

# LF-03 field-encounter-resolves
mc prove field-encounter --region R03 --expect victory --expect-loot --expect-wounds-persist || fail "LF-03"
echo "LF-03 field-encounter-resolves ok"

# LF-04 terrain-gated-spawns
mc prove spawn-gating --rolls 500 --all-regions || fail "LF-04"
echo "LF-04 terrain-gated-spawns ok"

# LF-05 encounter-budget-no-grind
mc prove encounter-budget --reentries 40 --expect-monotonic-decay --expect-floor-zero || fail "LF-05"
echo "LF-05 encounter-budget-no-grind ok"

# LF-06 confidence-scene-gates-story
mc prove confidence-gating || fail "LF-06"
echo "LF-06 confidence-scene-gates-story ok"

# LF-07 save-load-state-identity
mc prove save-identity --mid-battle --restart-process || fail "LF-07"
echo "LF-07 save-load-state-identity ok"

# LF-08 golden-tape-full-run
mc replay --tape tapes/golden-full.tape --assert-hash >/dev/null || fail "LF-08 golden tape hash"
mc replay --tape tapes/golden-full.tape --require-flag EPILOGUE_SAIL >/dev/null || fail "LF-08 epilogue not reached"
echo "LF-08 golden-tape-full-run ok"

# LF-09 determinism-cross-run
a=$(mc replay --tape tapes/golden-full.tape --print-hash | tr -d ' ')
b=$(cargo run --locked --quiet -p mc_tools -- replay --tape tapes/golden-full.tape --print-hash | tr -d ' ')
[ "$a" = "$b" ] || fail "LF-09 release and debug hashes differ: $a vs $b"
echo "LF-09 determinism-cross-run ok"

# LF-10 content-integrity
mc validate --input "${MC_CONTENT_DIR:-./content}" --strict-orphans >/dev/null || fail "LF-10"
echo "LF-10 content-integrity ok"

# LF-11 frame-budget
mc bench --frames 10000 --scene fernand-phase1 --max-p99-step-us 4000 --max-p99-frame-us 16600 || fail "LF-11"
echo "LF-11 frame-budget ok"

# LF-12 final-boss-two-phase
mc prove final-encounter --expect-damage-immune-phase2 --expect-gated-name-yourself || fail "LF-12"
echo "LF-12 final-boss-two-phase ok"
echo "live-fire: ok"
