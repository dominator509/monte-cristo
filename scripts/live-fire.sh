#!/usr/bin/env sh
# 6LAYER live-fire: one scripted proof per core user outcome, using real commands.
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
MC="cargo run --locked --release --quiet -p mc_tools --"

# LF-01 new-game-to-arrest
# Two-part proof:
# 1. Tape determinism: act1.tape replays to its recorded hash (proves determinism of recorded commands)
# 2. Flag assertion: prove_act1_arrest loads real content scenes and proves FLG_ARRESTED
#    is set by the scene system when the arrest scene is traversed.
$MC replay --tape tapes/act1.tape --assert-hash >/dev/null || fail "LF-01 act1 tape hash mismatch"
$MC prove act1-arrest >/dev/null || fail "LF-01 act1 arrest flag not set"
echo "LF-01 new-game-to-arrest ok"

# LF-02 if-calendar-and-curriculum
$MC prove if-calendar --months 168 --faria-at 72 --min-rank3-disciplines 4 >/dev/null || fail "LF-02"
echo "LF-02 if-calendar-and-curriculum ok"

# LF-03 field-encounter-resolves
$MC prove field-encounter --region R03 >/dev/null || fail "LF-03"
echo "LF-03 field-encounter-resolves ok"

# LF-04 terrain-gated-spawns
$MC prove spawn-gating --rolls 500 --all-regions >/dev/null || fail "LF-04"
echo "LF-04 terrain-gated-spawns ok"

# LF-05 encounter-budget-no-grind
$MC prove encounter-budget --reentries 40 >/dev/null || fail "LF-05"
echo "LF-05 encounter-budget-no-grind ok"

# LF-06 confidence-scene-gates-story
$MC prove confidence-gating >/dev/null || fail "LF-06"
echo "LF-06 confidence-scene-gates-story ok"

# LF-07 save-load-state-identity
$MC prove save-identity >/dev/null || fail "LF-07"
echo "LF-07 save-load-state-identity ok"

# LF-08 golden-tape-full-run
if [ -f tapes/golden-full.tape ]; then
    $MC replay --tape tapes/golden-full.tape --assert-hash >/dev/null || fail "LF-08 golden tape hash"
    $MC prove epilogue >/dev/null || fail "LF-08 epilogue content incomplete"
    echo "LF-08 golden-tape-full-run ok"
else
    echo "LF-08 golden-tape-full-run SKIP (tape not yet recorded)"
fi

# LF-09 determinism-cross-run
HASH1=$($MC replay --tape tapes/act1.tape --print-hash 2>/dev/null | tr -d ' ')
HASH2=$($MC replay --tape tapes/act1.tape --print-hash 2>/dev/null | tr -d ' ')
[ "$HASH1" = "$HASH2" ] || fail "LF-09 hashes differ: $HASH1 vs $HASH2"
echo "LF-09 determinism-cross-run ok"

# LF-10 content-integrity and locked corpus
$MC validate --input ./content >/dev/null || fail "LF-10"
$MC report --input ./content bestiary >/dev/null || fail "LF-10 locked content corpus"
echo "LF-10 content-integrity ok"

# LF-11 frame-budget
cargo bench --locked -p mc_core --bench battle_step -- --noplot >/dev/null ||
    fail "LF-11 frame budget"
echo "LF-11 frame-budget ok"

# LF-12 final-boss-two-phase
$MC prove final-encounter >/dev/null || fail "LF-12"
echo "LF-12 final-boss-two-phase ok"

echo "live-fire: ok"
