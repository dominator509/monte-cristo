#!/usr/bin/env sh
# 6LAYER preflight: files, tools, environment, credential probes.
# Must print "preflight: ok" before any graph node may start.
# The ONLY legitimate pre-run stop is a failure here.
set -eu
fail() { echo "preflight: FAIL - $1" >&2; exit 1; }
[ -f AGENTS.md ] && [ -d .agent ] || fail "run from repository root"
for f in AGENTS.md COMMANDS.md PREFLIGHT.md .env.example \
         .agent/GRAPH.md .agent/LOOPS.md .agent/state/LEDGER.md \
         .agent/reality-patterns .agent/reality-allow \
         .agent/MANIFEST.md .agent/PLANS.md .agent/EXECUTION_RULES.md \
         ARCHITECTURE.md TESTING.md SECURITY.md ENVIRONMENT.md \
         DEPLOYMENT.md OPERATIONS.md OBSERVABILITY.md PRODUCTION_READINESS.md \
         RELEASE.md ROLLBACK.md CONTRIBUTING.md PROJECT_BRIEF.md \
         ASSUMPTIONS.md ROADMAP.md DECISIONS.md docs/GAME_DESIGN.md \
         .agent/specs/SPEC-000-product-scope.md \
         .agent/specs/SPEC-009-content-bestiary-and-regions.md \
         .agent/execplans/EP-000-discovery-and-toolchain.md \
         .agent/execplans/EP-010-production-readiness-and-ship.md \
         scripts/ledger.sh scripts/graph-next.sh scripts/reality-gate.sh \
         scripts/verify.sh scripts/live-fire.sh; do
  [ -f "$f" ] || fail "missing required file: $f"
done
for t in git awk grep sed tar mktemp rustc cargo rustup; do
  command -v "$t" >/dev/null 2>&1 || fail "missing required tool: $t"
done
rustc_v=$(rustc --version 2>/dev/null | awk '{print $2}')
[ "$rustc_v" = "1.83.0" ] || fail "rustc is $rustc_v, required exactly 1.83.0 (see ENVIRONMENT.md section 1)"
cargo_v=$(cargo --version 2>/dev/null | awk '{print $2}')
[ "$cargo_v" = "1.83.0" ] || fail "cargo is $cargo_v, required exactly 1.83.0 (see ENVIRONMENT.md section 1)"
command -v sha256sum >/dev/null 2>&1 || command -v shasum >/dev/null 2>&1 || fail "missing required tool: sha256sum or shasum"
[ -f .env ] || fail "missing .env (copy .env.example, fill every REQUIRED value, rerun)"
set -a
. ./.env
set +a
[ "${MC_HEADLESS:-}" = "1" ] || fail "MC_HEADLESS must be exactly 1 in scripted gates"
case "${CARGO_REGISTRY_MODE:-}" in
  online|vendored) : ;;
  *) fail "CARGO_REGISTRY_MODE must be online or vendored (see PREFLIGHT.md section 3)" ;;
esac
TMP=$(mktemp)
trap 'rm -f "$TMP"' EXIT
awk '/^PREFLIGHT-TABLE-BEGIN$/{t=1;next} /^PREFLIGHT-TABLE-END$/{t=0} t && NF' PREFLIGHT.md > "$TMP"
[ -s "$TMP" ] || fail "PREFLIGHT-TABLE missing or empty in PREFLIGHT.md"
if command -v timeout >/dev/null 2>&1 &&
   timeout --version 2>/dev/null | grep -q 'GNU coreutils'; then
  TCMD="timeout 30"
else
  TCMD=""
fi
while IFS='|' read -r var req probe; do
  var=$(printf '%s' "$var" | tr -d ' ')
  req=$(printf '%s' "$req" | tr -d ' ')
  probe=$(printf '%s' "$probe" | tr -d ' ')
  [ -n "$var" ] || continue
  eval "val=\${$var:-}"
  if [ -z "$val" ]; then
    if [ "$req" = "REQUIRED" ]; then fail "env var not set: $var (see PREFLIGHT.md)"; fi
    echo "preflight: optional $var not set; dependent features disabled"
    continue
  fi
  if [ "$probe" != "-" ]; then
    [ -f "$probe" ] || fail "missing probe script: $probe"
    if ! $TCMD sh "$probe" >/dev/null 2>&1; then
      fail "credential probe failed: $var ($probe). Fix the credential, rerun preflight."
    fi
  fi
done < "$TMP"
echo "preflight: ok"
