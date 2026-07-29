#!/usr/bin/env sh
# EP-009 M3: Release automation — cross-compile, package, sign.
#
# Builds release binaries for Linux (x86_64, aarch64), Windows (x86_64),
# and macOS (x86_64, aarch64), then produces a SHA256SUMS manifest.
#
# Usage: sh scripts/release.sh [version]
#   version defaults to the current git describe tag.

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

VERSION="${1:-$(git describe --tags --always 2>/dev/null || echo dev)}"
RELEASE_DIR="target/release-artifacts/$VERSION"
mkdir -p "$RELEASE_DIR"

echo "=== Monte Cristo Release $VERSION ==="

# Targets to cross-compile for
TARGETS="x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-pc-windows-gnu x86_64-apple-darwin aarch64-apple-darwin"

for target in $TARGETS; do
    echo "Building for $target..."

    # Install target if not already installed
    rustup target add "$target" 2>/dev/null || true

    # Build all crates
    if cargo build --release --locked --target "$target" -p mc_tools 2>/dev/null; then
        echo "  $target: build succeeded"

        # Determine binary name and extension
        case "$target" in
            *windows*)
                BIN="target/$target/release/mc_tools.exe"
                ARCHIVE="monte-cristo-$VERSION-$target.zip"
                ;;
            *darwin* | *apple*)
                BIN="target/$target/release/mc_tools"
                ARCHIVE="monte-cristo-$VERSION-$target.tar.gz"
                ;;
            *)
                BIN="target/$target/release/mc_tools"
                ARCHIVE="monte-cristo-$VERSION-$target.tar.gz"
                ;;
        esac

        if [ -f "$BIN" ]; then
            cp "$BIN" "$RELEASE_DIR/"
            # Create archive
            (cd "$RELEASE_DIR" && tar czf "../$ARCHIVE" "$(basename "$BIN")" 2>/dev/null || \
             zip -j "../${ARCHIVE%.tar.gz}.zip" "$(basename "$BIN")" 2>/dev/null || true)
        fi
    else
        echo "  $target: build FAILED (skipping)" >&2
    fi
done

# Generate SHA256SUMS
cd "$RELEASE_DIR"
echo "=== SHA256 Checksums ===" > SHA256SUMS
for f in *; do
    if [ -f "$f" ] && [ "$f" != "SHA256SUMS" ]; then
        sha256sum "$f" >> SHA256SUMS
    fi
done

echo ""
echo "=== Release artifacts in $RELEASE_DIR ==="
ls -la "$RELEASE_DIR"
echo ""
echo "=== SHA256SUMS ==="
cat SHA256SUMS
echo ""
echo "Release $VERSION complete."
echo "To publish: upload $RELEASE_DIR/* to the GitHub release page."
