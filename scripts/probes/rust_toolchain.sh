#!/usr/bin/env sh
# Read-only probe: the pinned Rust toolchain is installed and active.
set -eu
want="${MC_RUST_VERSION:-}"
[ -n "$want" ] || { echo "probe rust_toolchain: MC_RUST_VERSION not set" >&2; exit 1; }
command -v rustc >/dev/null 2>&1 || { echo "probe rust_toolchain: rustc not found" >&2; exit 1; }
got=$(rustc --version | awk '{print $2}')
[ "$got" = "$want" ] || { echo "probe rust_toolchain: rustc is $got, want $want" >&2; exit 1; }
for c in rustfmt clippy-driver; do
  command -v "$c" >/dev/null 2>&1 || { echo "probe rust_toolchain: missing component: $c" >&2; exit 1; }
done
echo "probe rust_toolchain: ok"
