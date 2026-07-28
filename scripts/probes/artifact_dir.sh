#!/usr/bin/env sh
# Read-only probe: the release artifact staging root must exist, be a directory, and be writable.
# Side-effect-free: creates and removes one temporary file inside the directory.
set -eu
d="${MC_ARTIFACT_DIR:-}"
[ -n "$d" ] || { echo "probe artifact_dir: MC_ARTIFACT_DIR not set" >&2; exit 1; }
[ -d "$d" ] || { echo "probe artifact_dir: not a directory: $d" >&2; exit 1; }
t="$d/.6layer-probe.$$"
: > "$t" 2>/dev/null || { echo "probe artifact_dir: not writable: $d" >&2; exit 1; }
rm -f "$t"
echo "probe artifact_dir: ok"
