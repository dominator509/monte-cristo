#!/usr/bin/env sh
# Read-only probe: the saves, settings, logs, and crash report root must exist, be a directory, and be writable.
# Side-effect-free: creates and removes one temporary file inside the directory.
set -eu
d="${MC_DATA_DIR:-}"
[ -n "$d" ] || { echo "probe data_dir: MC_DATA_DIR not set" >&2; exit 1; }
[ -d "$d" ] || { echo "probe data_dir: not a directory: $d" >&2; exit 1; }
t="$d/.6layer-probe.$$"
: > "$t" 2>/dev/null || { echo "probe data_dir: not writable: $d" >&2; exit 1; }
rm -f "$t"
echo "probe data_dir: ok"
