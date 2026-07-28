# COMMANDS.md -- the only legal command source

Coding agents must not invent commands. If a command is missing or stale, update this file
first, citing repository evidence, with a Decision Log entry in the active ExecPlan.

## 1. Working directory rule

Every command in this file runs from the repository root, the directory containing
AGENTS.md and .agent/. Scripts assert this and fail loudly otherwise. Never `cd` into a
crate directory to run a gate; use `-p <crate>` instead.

## 2. Non-interactive environment (export at session start, verbatim)

    export CI=true
    export GIT_TERMINAL_PROMPT=0
    export GIT_PAGER=cat
    export PAGER=cat
    export DEBIAN_FRONTEND=noninteractive
    export CARGO_TERM_COLOR=never
    export CARGO_INCREMENTAL=0
    export RUST_BACKTRACE=1
    export MC_HEADLESS=1

Every script in scripts/ exports this block itself, so a gate is safe even if you forget.
`MC_HEADLESS=1` is what stops mc_shell from trying to open a window inside automation.

## 3. Gates (each prints an exact sentinel; nonzero exit on any failure)

| Purpose | Command | Sentinel |
|---|---|---|
| Install / fetch deps | `sh scripts/install.sh` | `install: ok` |
| Preflight | `sh scripts/preflight.sh` | `preflight: ok` |
| Lint | `sh scripts/lint.sh` | `lint: ok` |
| Format check | `sh scripts/format-check.sh` | `format check: ok` |
| Typecheck | `sh scripts/typecheck.sh` | `typecheck: ok` |
| Unit tests | `sh scripts/test-unit.sh` | `unit tests: ok` |
| Integration tests | `sh scripts/test-integration.sh` | `integration tests: ok` |
| End-to-end tests | `sh scripts/test-e2e.sh` | `e2e tests: ok` |
| Build | `sh scripts/build.sh` | `build: ok` |
| Security check | `sh scripts/security-check.sh` | `security check: ok` |
| Dependency audit | `sh scripts/dependency-audit.sh` | `dependency audit: ok` |
| Reality gate | `sh scripts/reality-gate.sh` | `reality gate: ok` |
| Smoke test | `sh scripts/smoke-test.sh` | `smoke test: ok` |
| Live fire | `sh scripts/live-fire.sh` | `live-fire: ok` |
| Full verify | `sh scripts/verify.sh` | `verify: ok` |
| Production readiness | `sh scripts/production-readiness-check.sh` | `production readiness: ok` |

`scripts/verify.sh` runs, in this order: preflight, lint, format-check, typecheck, unit,
integration, e2e, build, security-check, dependency-audit, reality-gate, smoke, live-fire.

## 4. Ledger and scheduler

    sh scripts/ledger.sh append <AGENT_ID> <NODE|-> <EVENT> <detail>
    sh scripts/ledger.sh status <NODE>
    sh scripts/ledger.sh tail 30
    sh scripts/graph-next.sh

AGENT_ID is `<platform>-<short-handle>`, for example `claude-code-a1` or `codex-b2`.

## 5. Project-specific development commands

    # Bake the RON content tree into the binary content pack.
    cargo run --locked -p mc_tools -- bake --input "$MC_CONTENT_DIR" --output content.pack

    # Validate content without baking: schema, references, orphans, supernatural lint.
    cargo run --locked -p mc_tools -- validate --input "$MC_CONTENT_DIR"

    # Record an input tape from a live session (developer use, never in automation).
    cargo run --locked -p mc_tools -- record --out tapes/scratch.tape

    # Replay a tape headlessly and print the terminal state hash.
    cargo run --locked -p mc_tools -- replay --tape tapes/golden-full.tape --print-hash

    # Replay a tape and assert it matches its recorded hash (this is what CI runs).
    cargo run --locked -p mc_tools -- replay --tape tapes/golden-full.tape --assert-hash

    # Bestiary and region reports used by EP-002 and EP-007 acceptance.
    cargo run --locked -p mc_tools -- report bestiary
    cargo run --locked -p mc_tools -- report encounter-budget

    # Frame-budget bench (criterion), the source of the LF-11 numbers.
    cargo bench --locked -p mc_core -- --save-baseline current

## 6. Local run (backgrounded, with readiness probe and kill path)

The game is a desktop application, not a service, so the only backgrounded form is the
headless replay harness used by live-fire.

    # start
    MC_HEADLESS=1 cargo run --locked --release -p mc_tools -- serve-replay --port-file .local-data/replay.port &
    echo $! > .local-data/replay.pid

    # readiness probe (bounded: 30 attempts, 2s apart)
    i=0; while [ $i -lt 30 ]; do [ -s .local-data/replay.port ] && break; i=$((i+1)); sleep 2; done
    [ -s .local-data/replay.port ] || { echo "READINESS_TIMEOUT_replay" >&2; exit 1; }

    # kill
    kill "$(cat .local-data/replay.pid)" 2>/dev/null || true; rm -f .local-data/replay.pid .local-data/replay.port

## 7. Database and migrations

Not applicable. This project has no database. The analogous concern is the SAVE SCHEMA
VERSION, and its migration command is:

    cargo run --locked -p mc_tools -- save-migrate --dir "$MC_DATA_DIR/saves" --dry-run
    cargo run --locked -p mc_tools -- save-migrate --dir "$MC_DATA_DIR/saves" --apply

Save migrations are expand-migrate-contract and are covered by round-trip tests in EP-003.

## 8. Forbidden commands

Never run: an interactive REPL; an editor; a pager; `cargo watch` or any foreground watch
mode; `git push --force` or any history rewrite; `git clean -x` outside a milestone's
declared recovery; `rm -rf` on any path outside `target/`, `.local-data/`, or a path a
milestone explicitly names; `cargo update` (it defeats the pinned lockfile); `cargo install`
without `--locked`; any command that prompts for input; anything involving npm, node, yarn,
or pnpm, none of which appear anywhere in this project by design.

## 9. Adapter parity check

    for f in AGENTS.md CLAUDE.md .hermes/6layer.md .openclaw/6layer.md; do awk '/PRIME-BLOCK-BEGIN/,/PRIME-BLOCK-END/' "$f" | cksum; done

All cksum lines must be identical. If you add an adapter, add it to this command in the same
commit.

## 10. Recovery

When a gate fails, do not improvise. Go to .agent/LOOPS.md section 5.3 and climb the ladder.
For a failing validation command specifically, use the procedure in
.agent/prompts/debug-validation-failure.md.
