# How to Use This Blueprint Pack

MONTE CRISTO -- 6LAYER blueprint pack, v2 GRAPHLOCK.

## 1. Materialize

This pack ships as a directory tree, already materialized. Drop it into an empty repository
root. If you ever need to move it as a single transcript instead, concatenate the files
between `=== FILE: <path> ===` and `=== END FILE ===` markers and split them with this
splitter, saved as `unpack.sh`:

    #!/usr/bin/env sh
    # 6LAYER pack splitter: materializes files from a pack transcript.
    set -eu
    pack="${1:-BLUEPRINT_PACK.md}"
    [ -f "$pack" ] || { echo "unpack: missing $pack" >&2; exit 1; }
    awk '
      /^=== FILE: /{
        path=substr($0, 11)
        sub(/ ===$/, "", path)
        cmd="mkdir -p \"$(dirname \"" path "\")\""
        system(cmd)
        printf "" > path
        out=1
        next
      }
      /^=== END FILE ===$/{ out=0; close(path); next }
      out { print >> path }
    ' "$pack"
    echo "unpack: ok"

Run: `sh unpack.sh && rm BLUEPRINT_PACK.md unpack.sh`. Alternatively, paste the pack to any
coding agent and instruct it to materialize every FILE block exactly, byte for byte -- the
markers make either path lossless.

## 2. Bootstrap (the single interactive moment)

    git init
    git add -A && git commit -m "[6LAYER] bootstrap blueprint pack"
    chmod +x scripts/*.sh scripts/probes/*.sh

Open PREFLIGHT.md. This project has **no credentials** -- there is no API key, account, or
service anywhere in it. What you must supply is: the pinned Rust toolchain (1.83.0), three
cargo subcommands at their exact versions, three writable directories, and a registry mode.

    cp .env.example .env
    # edit .env: set MC_CONTENT_DIR, MC_ARTIFACT_DIR, MC_DATA_DIR,
    #            MC_GRAPHICS_STACK, CARGO_REGISTRY_MODE, MC_REFERENCE_MACHINE
    mkdir -p ./content ./target/artifacts ./.local-data
    rustup toolchain install 1.83.0
    rustup component add rustfmt clippy --toolchain 1.83.0
    cargo install --locked cargo-deny@0.16.2 cargo-fuzz@0.12.0 cargo-llvm-cov@0.6.15
    sh scripts/preflight.sh          # must print: preflight: ok

Do not proceed past a failing preflight. It names the exact missing item, and every node of
the graph assumes these facts hold.

## 3. Launch hands-off

Give any agent the contents of `.agent/prompts/run-graph.md`.

- **Claude Code:** `claude -p "$(cat .agent/prompts/run-graph.md)"` with the platform's
  non-interactive or auto-approval mode enabled per its current documentation.
- **Codex CLI:** `codex --cd . --ask-for-approval never --sandbox workspace-write "$(cat .agent/prompts/run-graph.md)"`
- **Hermes, OpenClaw, or any other agent:** paste `run-graph.md` as the task.

The instruction text is identical everywhere; only the runner's own auto-approval flag
differs. Any agent that can read files, edit files, and run commands qualifies.

## 4. Observe without interfering

    tail -f .agent/state/LEDGER.md
    git log --oneline

That is the run's entire telemetry. Do not chat with a running agent; the repository is the
only channel.

## 5. Relay or parallel operation

Stop any agent at any time. The lease-plus-ledger protocol makes the next launch -- same
platform or different -- resume losslessly via the same `run-graph.md` prompt. Two agents
pointed at the repository coordinate automatically through the lease rules. EP-005 and EP-006
are the one deliberate parallel branch: both depend only on EP-004, so two agents can take
them concurrently.

## 6. If it blocks

`sh scripts/graph-next.sh` prints `BLOCKED <id>`. Read the blocked report in that ExecPlan's
Progress section, make the one named decision, append a ledger note, reset the node per its
Idempotence and Recovery section, and relaunch.

## 7. Single-node and maintenance modes

Use `.agent/prompts/execute-active-execplan.md`, `continue-execplan.md`,
`debug-validation-failure.md`, and `final-review.md` for surgical work under the same laws.
Never implement from ROADMAP.md, and evolve plans only through the documented update rules
with ledger entries.

## 8. Ship decision

`RUN_COMPLETE` in the ledger plus `production readiness: ok` is the ship decision.
Auto-Deploy Authorization for this project is **no**, so the run ends at a proven, tagged
artifact set and prints one MANUAL publish command. Executing that command is the only
remaining human action -- and it exists on purpose, because it is where a human judges the
things no gate can: art, music, writing, and balance.

## 9. What is unusual about this particular pack

The subject is a 40-hour narrative RPG, which is normally the least testable software there
is. The architecture exists to change that: `mc_core` is a pure, headless, deterministic
state machine, so the entire game is a function of (seed, content pack, input tape). That
turns "the campaign is completable" into `replay --assert-hash`, and it is why the ship gate
can be a script rather than a play session.

The single property that makes all of it work is determinism. If a hash test fails, find the
cause -- a float in a state path, a hash-ordered iteration, a clock, or a thread. Never
re-record a tape to make a test pass. That is the one process violation this pack treats as
unforgivable, and `.agent/LOOPS.md` says so in three separate places for a reason.
