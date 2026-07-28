#!/usr/bin/env sh
# Read-only probe: the three pinned cargo subcommands are installed at their exact versions.
# MC_CARGO_TOOLS is "cargo-deny,cargo-fuzz,cargo-llvm-cov" versions in that order.
set -eu
spec="${MC_CARGO_TOOLS:-}"
[ -n "$spec" ] || { echo "probe cargo_tools: MC_CARGO_TOOLS not set" >&2; exit 1; }
want_deny=$(printf '%s' "$spec" | cut -d, -f1)
want_fuzz=$(printf '%s' "$spec" | cut -d, -f2)
want_cov=$(printf '%s' "$spec" | cut -d, -f3)
[ -n "$want_deny" ] && [ -n "$want_fuzz" ] && [ -n "$want_cov" ] || {
  echo "probe cargo_tools: MC_CARGO_TOOLS must be three comma-separated versions" >&2; exit 1; }
check() {
  sub="$1"; want="$2"
  got=$(cargo "$sub" --version 2>/dev/null | awk '{print $2}' || true)
  [ -n "$got" ] || { echo "probe cargo_tools: cargo-$sub not installed" >&2; exit 1; }
  [ "$got" = "$want" ] || { echo "probe cargo_tools: cargo-$sub is $got, want $want" >&2; exit 1; }
}
check deny "$want_deny"
check fuzz "$want_fuzz"
check llvm-cov "$want_cov"
echo "probe cargo_tools: ok"
