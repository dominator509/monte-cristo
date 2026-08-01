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

export MC_CONTENT_DIR="${MC_CONTENT_DIR:-./content}"
export MC_DATA_DIR="${MC_DATA_DIR:-./.local-data}"
export MC_ARTIFACT_DIR="${MC_ARTIFACT_DIR:-./target/artifacts}"
mkdir -p "$MC_CONTENT_DIR" "$MC_DATA_DIR" "$MC_ARTIFACT_DIR"

cargo run --locked --quiet -p mc_tools -- validate --input "${MC_CONTENT_DIR:-./content}" \
  || fail "content validation failed"

[ -f content.pack ] || cargo run --locked --quiet -p mc_tools -- bake \
  --input "${MC_CONTENT_DIR:-./content}" --output content.pack

if [ -f tapes/golden-smoke.tape ]; then
  cargo run --locked --quiet -p mc_tools -- replay --tape tapes/golden-smoke.tape --assert-hash \
    || fail "golden smoke tape hash mismatch"
fi

# Compile outside the timing window, then measure the real release executable.
cargo build --locked --release --quiet -p mc_shell
shell_bin=target/release/monte-cristo
[ -x "$shell_bin" ] || shell_bin=target/release/monte-cristo.exe
[ -x "$shell_bin" ] || fail "release shell executable missing"

# OPERATIONS.md section 4: binary, content, simulation, and data-root integrity.
checksum_file=$(mktemp)
trap 'rm -f "$checksum_file"' EXIT HUP INT TERM
sha256sum "$shell_bin" >"$checksum_file"
sha256sum -c "$checksum_file" >/dev/null \
  || fail "release shell checksum verification failed"

# Cold start budget: under 2.5 s to title (SPEC-008 section 3).
start=$(date +%s%3N)
MC_CONTENT_DIR="$(pwd)" "$shell_bin" --verify-content | grep -q '^content: ok$' \
  || fail "release shell content verification failed"
end=$(date +%s%3N)
elapsed_ms=$((end - start))
[ "$elapsed_ms" -le 2500 ] \
  || fail "cold start check took ${elapsed_ms}ms; budget is 2500ms"

"$shell_bin" --check-paths | grep -q '^paths: ok$' \
  || fail "release shell path check failed"

if [ -f tapes/golden-smoke.tape ]; then
  cargo run --locked --quiet -p mc_tools -- replay \
    --tape tapes/golden-smoke.tape --assert-hash >/dev/null \
    || fail "release simulation integrity failed"
fi

cargo bench --locked -p mc_core --bench battle_step -- --noplot >/dev/null \
  || fail "frame budget failed"

echo "smoke test: reference machine ${MC_REFERENCE_MACHINE:-unset}"
echo "smoke test: ok"
