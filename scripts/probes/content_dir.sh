#!/usr/bin/env sh
# Read-only probe: the RON content source root must exist, be a directory, and be writable.
# Side-effect-free: creates and removes one temporary file inside the directory.
set -eu
d="${MC_CONTENT_DIR:-}"
[ -n "$d" ] || { echo "probe content_dir: MC_CONTENT_DIR not set" >&2; exit 1; }
[ -d "$d" ] || { echo "probe content_dir: not a directory: $d" >&2; exit 1; }
t="$d/.6layer-probe.$$"
: > "$t" 2>/dev/null || { echo "probe content_dir: not writable: $d" >&2; exit 1; }
rm -f "$t"
echo "probe content_dir: ok"
