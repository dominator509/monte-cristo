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
WORKSPACE_VER=$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\([^"]*\)"/\1/p' Cargo.toml)
[ "$VER" = "$WORKSPACE_VER" ] || {
  echo "build: FAIL - VERSION $VER does not match workspace version $WORKSPACE_VER" >&2
  exit 1
}
OUT="${MC_ARTIFACT_DIR:-target/artifacts}"
mkdir -p "$OUT"
rm -f "$OUT"/SHA256SUMS

TARGETS="x86_64-unknown-linux-gnu x86_64-pc-windows-gnu aarch64-apple-darwin"
HOST=$(rustc -vV | sed -n 's/^host: //p')
if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1 &&
   [ -x /c/ProgramData/mingw64/mingw64/bin/x86_64-w64-mingw32-gcc.exe ]; then
  PATH="/c/ProgramData/mingw64/mingw64/bin:$PATH"
  export PATH
fi
built=0
for t in $TARGETS; do
  if rustup target list --installed 2>/dev/null | grep -q "^$t$"; then
    if [ "$t" = "x86_64-unknown-linux-gnu" ] && [ "$HOST" != "$t" ]; then
      if ! command -v x86_64-linux-gnu-gcc >/dev/null 2>&1; then
        echo "build: cross toolchain not available for $t, skipping (install x86_64-linux-gnu-gcc)"
        continue
      fi
      CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
        cargo build --locked --release -p mc_shell --target "$t"
      built=$((built + 1))
      continue
    fi
    if [ "$t" = "x86_64-pc-windows-gnu" ] && [ "$HOST" != "$t" ]; then
      if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
        echo "build: cross toolchain not available for $t, skipping (install mingw-w64)"
        continue
      fi
      CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
        cargo build --locked --release -p mc_shell --target "$t"
      built=$((built + 1))
      continue
    fi
    # For aarch64-apple-darwin cross-builds, verify the full external toolchain works.
    # A native arm64 macOS runner uses its installed Xcode linker directly.
    if [ "$t" = "aarch64-apple-darwin" ] && [ "$HOST" != "$t" ]; then
      # Check if the full cross toolchain works (zig with macOS SDK, or osxcross)
      if ! (command -v aarch64-apple-darwin-cc >/dev/null 2>&1 || \
            (command -v zig >/dev/null 2>&1 && \
             echo 'int main(void){}' | zig cc -target aarch64-macos-none -framework Foundation -x c - -o /dev/null 2>/dev/null)); then
        echo "build: cross toolchain not available for $t, skipping (install osxcross or zig + macOS SDK)"
        continue
      fi
      CARGO_FEATURE_PURE=1 cargo build --locked --release -p mc_shell --target "$t"
    else
      cargo build --locked --release -p mc_shell --target "$t"
    fi
    built=$((built + 1))
  else
    echo "build: target not installed, skipping: $t" >&2
  fi
done
[ "$built" -gt 0 ] || { echo "build: FAIL - no target could be built" >&2; exit 1; }

cargo run --locked --release -p mc_tools -- bake --input "${MC_CONTENT_DIR:-./content}" --output content.pack
[ -f content.pack ] || { echo "build: FAIL - content.pack missing after bake" >&2; exit 1; }
[ -f content.pack.blake3 ] || {
  echo "build: FAIL - content.pack.blake3 missing after bake" >&2
  exit 1
}
[ -f LICENSE ] || { echo "build: FAIL - LICENSE missing" >&2; exit 1; }
[ -f docs/artifact-README.txt ] || {
  echo "build: FAIL - docs/artifact-README.txt missing" >&2
  exit 1
}

for t in $TARGETS; do
  bin="target/$t/release/monte-cristo"
  [ -f "$bin" ] || bin="target/$t/release/monte-cristo.exe"
  [ -f "$bin" ] || continue
  stage=$(mktemp -d)
  cp "$bin" "$stage/"
  cp content.pack content.pack.blake3 LICENSE "$stage/"
  cargo deny list -f tsv > "$stage/THIRD-PARTY-LICENSES.txt"
  [ -s "$stage/THIRD-PARTY-LICENSES.txt" ] || {
    echo "build: FAIL - generated third-party license file is empty" >&2
    exit 1
  }
  cp docs/artifact-README.txt "$stage/README.txt"
  tar --force-local -czf "$OUT/monte-cristo-$VER-$t.tar.gz" -C "$stage" .
  rm -rf "$stage"
done

available=0
archive_names=""
for t in $TARGETS; do
  archive="$OUT/monte-cristo-$VER-$t.tar.gz"
  [ -f "$archive" ] || continue
  case "$t" in
    x86_64-pc-windows-gnu) archive_bin="monte-cristo.exe" ;;
    *) archive_bin="monte-cristo" ;;
  esac
  actual_members=$(tar --force-local -tzf "$archive" | sed 's|^\./||' | grep -v '^$' | sort)
  expected_members=$(printf '%s\n' \
    "$archive_bin" content.pack content.pack.blake3 LICENSE \
    THIRD-PARTY-LICENSES.txt README.txt | sort)
  [ "$actual_members" = "$expected_members" ] || {
    echo "build: FAIL - unexpected members in $archive" >&2
    printf 'expected:\n%s\nactual:\n%s\n' "$expected_members" "$actual_members" >&2
    exit 1
  }
  available=$((available + 1))
  archive_names="$archive_names $(basename "$archive")"
done
[ "$available" -gt 0 ] || { echo "build: FAIL - no artifacts staged" >&2; exit 1; }

( cd "$OUT" && if command -v sha256sum >/dev/null 2>&1; then
    # shellcheck disable=SC2086 -- archive_names is a controlled list of fixed filenames.
    sha256sum $archive_names > SHA256SUMS
  else
    # shellcheck disable=SC2086 -- archive_names is a controlled list of fixed filenames.
    shasum -a 256 $archive_names > SHA256SUMS
  fi )
if [ "$available" -eq 3 ]; then
  echo "build: ok"
else
  echo "build: partial ($available/3 artifacts; $built target(s) built on this host)"
fi
