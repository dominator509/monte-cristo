#!/usr/bin/env sh
# Read-only probe: the declared crate registry mode is usable.
# online   -> the registry index is reachable via a metadata read (no download, no write)
# vendored -> vendor/ is populated and .cargo/config.toml redirects to it
set -eu
mode="${CARGO_REGISTRY_MODE:-}"
case "$mode" in
  online)
    command -v cargo >/dev/null 2>&1 || { echo "probe registry_mode: cargo not found" >&2; exit 1; }
    cargo search --limit 1 serde >/dev/null 2>&1 || {
      echo "probe registry_mode: registry unreachable in online mode" >&2; exit 1; }
    ;;
  vendored)
    [ -d vendor ] || { echo "probe registry_mode: vendored mode but vendor/ missing" >&2; exit 1; }
    n=$(ls vendor 2>/dev/null | wc -l)
    [ "$n" -gt 0 ] || { echo "probe registry_mode: vendor/ is empty" >&2; exit 1; }
    [ -f .cargo/config.toml ] || { echo "probe registry_mode: missing .cargo/config.toml" >&2; exit 1; }
    grep -q 'vendored-sources' .cargo/config.toml || {
      echo "probe registry_mode: .cargo/config.toml does not redirect to vendored-sources" >&2; exit 1; }
    ;;
  *)
    echo "probe registry_mode: CARGO_REGISTRY_MODE must be online or vendored" >&2; exit 1 ;;
esac
echo "probe registry_mode: ok"
