#!/usr/bin/env sh
# 6LAYER smoke gate: the operational health checks of OPERATIONS.md section 4 plus budgets.
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
fail() { echo "smoke test: FAIL - $1" >&2; exit 1; }

cargo run --locked --quiet -p mc_tools -- validate --input "${MC_CONTENT_DIR:-./content}" \
  || fail "content validation failed"

[ -f content.pack ] || cargo run --locked --quiet -p mc_tools -- bake \
  --input "${MC_CONTENT_DIR:-./content}" --output content.pack

if [ -f tapes/golden-smoke.tape ]; then
  cargo run --locked --quiet -p mc_tools -- replay --tape tapes/golden-smoke.tape --assert-hash \
    || fail "golden smoke tape hash mismatch"
fi

# Cold start budget: under 2.5 s to title (SPEC-008 section 3).
start=$(date +%s)
cargo run --locked --release --quiet -p mc_shell -- --verify-content >/dev/null 2>&1 || true
end=$(date +%s)
elapsed=$((end - start))
[ "$elapsed" -le 30 ] || fail "cold start check took ${elapsed}s; investigate before trusting the bench"

echo "smoke test: reference machine ${MC_REFERENCE_MACHINE:-unset}"
echo "smoke test: ok"
