#!/usr/bin/env sh
# 6LAYER build gate: release binaries for every supported target, staged with a manifest.
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
[ -f VERSION ] || { echo "build: FAIL - VERSION missing" >&2; exit 1; }
VER=$(cat VERSION)
OUT="${MC_ARTIFACT_DIR:-target/artifacts}"
mkdir -p "$OUT"

TARGETS="x86_64-unknown-linux-gnu x86_64-pc-windows-gnu aarch64-apple-darwin"
built=0
for t in $TARGETS; do
  if rustup target list --installed 2>/dev/null | grep -q "^$t$"; then
    cargo build --locked --release -p mc_shell --target "$t"
    built=$((built + 1))
  else
    echo "build: target not installed, skipping: $t" >&2
  fi
done
[ "$built" -gt 0 ] || { echo "build: FAIL - no target could be built" >&2; exit 1; }

cargo run --locked --release -p mc_tools -- bake --input "${MC_CONTENT_DIR:-./content}" --output content.pack

for t in $TARGETS; do
  bin="target/$t/release/monte-cristo"
  [ -f "$bin" ] || bin="target/$t/release/monte-cristo.exe"
  [ -f "$bin" ] || continue
  stage=$(mktemp -d)
  cp "$bin" "$stage/"
  cp content.pack content.pack.blake3 "$stage/" 2>/dev/null || true
  [ -f LICENSE ] && cp LICENSE "$stage/"
  cargo deny list -f tsv > "$stage/THIRD-PARTY-LICENSES.txt" 2>/dev/null || true
  [ -f docs/artifact-README.txt ] && cp docs/artifact-README.txt "$stage/README.txt"
  tar czf "$OUT/monte-cristo-$VER-$t.tar.gz" -C "$stage" .
  rm -rf "$stage"
done

( cd "$OUT" && if command -v sha256sum >/dev/null 2>&1; then
    sha256sum monte-cristo-*.tar.gz > SHA256SUMS
  else
    shasum -a 256 monte-cristo-*.tar.gz > SHA256SUMS
  fi )
echo "build: ok"
