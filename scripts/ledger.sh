#!/usr/bin/env sh
# 6LAYER ledger helper. Append-only event writer + status reader.
# The ledger is the single source of runtime truth. Details must not contain " | ".
# Usage:
#   sh scripts/ledger.sh append <AGENT_ID> <NODE|-> <EVENT> [detail...]
#   sh scripts/ledger.sh status <NODE>     -> DONE | BLOCKED | IN_PROGRESS | PENDING
#   sh scripts/ledger.sh tail [n]
set -eu
LEDGER=".agent/state/LEDGER.md"
[ -f "$LEDGER" ] || { echo "ledger.sh: missing $LEDGER (repo not bootstrapped)" >&2; exit 1; }
cmd="${1:-}"
[ -n "$cmd" ] && shift
case "$cmd" in
  append)
    agent="${1:?agent id}"; node="${2:?node id or -}"; event="${3:?event}"; shift 3
    detail="${*:-}"
    ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    printf '%s | %s | %s | %s | %s\n' "$ts" "$agent" "$node" "$event" "$detail" >> "$LEDGER"
    ;;
  status)
    node="${1:?node id}"
    # Terminal states take priority: NODE_DONE and NODE_BLOCKED are final
    # regardless of any subsequent LEASE_RELEASE events.
    line=$(grep -E "\\| $node \\| (NODE_DONE|NODE_BLOCKED) \\|" "$LEDGER" | tail -n 1)
    if [ -n "$line" ]; then
      case "$line" in
        *"| NODE_DONE |"*)    echo DONE ;;
        *"| NODE_BLOCKED |"*) echo BLOCKED ;;
      esac
    else
      line=$(grep -E "\\| $node \\| (LEASE_RELEASE|LEASE) \\|" "$LEDGER" | tail -n 1)
      case "$line" in
        *"| LEASE_RELEASE |"*) echo PENDING ;;
        *"| LEASE |"*)         echo IN_PROGRESS ;;
        *)                     echo PENDING ;;
      esac
    fi
    ;;
  tail)
    n="${1:-30}"
    tail -n "$n" "$LEDGER"
    ;;
  *)
    echo "usage: ledger.sh append|status|tail ..." >&2
    exit 2
    ;;
esac
