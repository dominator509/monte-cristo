NODE-META-BEGIN
ID: EP-000
DEPS: -
MAX_ATTEMPTS_PER_MILESTONE: 6
VERIFY: sh scripts/preflight.sh
VERIFY_SENTINEL: preflight: ok
GREEN_TAG: green/EP-000
NODE-META-END

# EP-000 -- Discovery and toolchain

## 1. Purpose / Big Picture

Make the environment incapable of being the cause of a later failure. This is a greenfield
repository, so there is nothing to inventory except the toolchain, the directories, and the
registry mode. After this node, every tool is pinned and version-asserted, the three
confinement roots exist, the repository is initialised with the pack committed, and the
scheduler and ledger work. Nothing here builds game code.

## 2. Scope

Toolchain pinning and verification; directory creation; registry mode resolution including
the vendored path; git initialisation and the bootstrap commit; proof that ledger.sh and
graph-next.sh operate; proof that preflight.sh reaches its sentinel.

## 3. Non-goals

No crates. No Cargo.toml. No game code, content, or tests. No CI configuration. Do not create
`crates/`, do not run `cargo build`, do not author any RON. Those are EP-001 and later. If
you find yourself writing Rust in this node, stop and re-read this section.

## 4. Context and Orientation

The repository contains only the blueprint pack. Repository Status is Greenfield, so there
are no loud-fail placeholder scripts to replace -- every script in `scripts/` is real and
final as shipped. The three environment roots (INV-07) are declared in `.env` and must exist
before any later node can write anything.

## 5. Files to Read First

- AGENTS.md
- COMMANDS.md
- PREFLIGHT.md
- ENVIRONMENT.md
- .agent/GRAPH.md
- .agent/LOOPS.md
- .env.example

## 6. Expected Changed Files

- .env
- rust-toolchain.toml
- .agent/state/LEDGER.md
- .cargo/config.toml  (only when CARGO_REGISTRY_MODE=vendored)
- ASSUMPTIONS.md      (only if a recorded assumption proves false)

## 7. Interfaces and Contracts

Sentinels: `preflight: ok`. Ledger grammar per .agent/GRAPH.md. Environment variable names
and shapes per ENVIRONMENT.md section 3, which is authoritative -- do not invent a variable.

## 8. Milestones

### M1: Repository initialisation and bootstrap commit
GOAL: The pack is under version control and the ledger is writable.
READ: AGENTS.md, .agent/GRAPH.md
CHANGE: .agent/state/LEDGER.md
CONTENT: No file bodies. Run the commands below exactly.
RUN:
  git init 2>/dev/null || true
  git add -A
  git commit -m "[6LAYER] bootstrap blueprint pack" || true
  sh scripts/ledger.sh append <AGENT_ID> EP-000 LEASE "node start"
  sh scripts/ledger.sh tail 5
EXPECT: the tail output ends with a line containing `| EP-000 | LEASE | node start`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-000 MILESTONE_PASS "M1 ledger writable"
FALLBACK: none needed -- git init and an append are trivially safe and idempotent.
COMMIT: git add -A && git commit -m "[EP-000][M1] initialise repository and seed lease"

### M2: Toolchain pin
GOAL: rustc, cargo, rustfmt, and clippy are pinned to exactly 1.83.0.
READ: ENVIRONMENT.md section 1
CHANGE: rust-toolchain.toml
CONTENT: create rust-toolchain.toml with exactly this content:

[toolchain]
channel = "1.83.0"
components = ["rustfmt", "clippy"]
targets = ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-gnu", "aarch64-apple-darwin"]
profile = "minimal"

RUN:
  rustup toolchain install 1.83.0
  rustup component add rustfmt clippy --toolchain 1.83.0
  rustc --version
  cargo --version
EXPECT: `rustc --version` output contains `1.83.0`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-000 MILESTONE_PASS "M2 rustc 1.83.0 pinned"
FALLBACK: if rustup is unavailable, install the exact toolchain by the platform's documented
  offline installer and record it in the Decision Log. Never proceed on a different version;
  a different compiler threatens INV-01.
COMMIT: git add -A && git commit -m "[EP-000][M2] pin rust toolchain to 1.83.0"

### M3: Cargo subcommands
GOAL: cargo-deny, cargo-fuzz, and cargo-llvm-cov are installed at their pinned versions.
READ: ENVIRONMENT.md section 1, PREFLIGHT.md section 1.2
CHANGE: (none -- tooling is installed outside the repository)
CONTENT: none.
RUN:
  cargo install --locked cargo-deny@0.16.2
  cargo install --locked cargo-fuzz@0.12.0
  cargo install --locked cargo-llvm-cov@0.6.15
  sh scripts/probes/cargo_tools.sh
EXPECT: `probe cargo_tools: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-000 MILESTONE_PASS "M3 probe cargo_tools: ok"
FALLBACK: if a pinned version fails to build on this host, install the nearest patch release
  of the same minor version, update MC_CARGO_TOOLS and ENVIRONMENT.md section 1 in the same
  commit, and record an ADR. Never widen to a different minor version silently.
COMMIT: git add -A && git commit -m "[EP-000][M3] install pinned cargo subcommands"

### M4: Confinement roots
GOAL: The three declared directories exist and are writable.
READ: PREFLIGHT.md section 2, ENVIRONMENT.md section 3
CHANGE: .env
CONTENT: copy .env.example to .env and set MC_CONTENT_DIR, MC_ARTIFACT_DIR, MC_DATA_DIR,
  MC_RUST_VERSION=1.83.0, MC_CARGO_TOOLS=0.16.2,0.12.0,0.6.15, MC_GRAPHICS_STACK to the
  value matching this host, CARGO_REGISTRY_MODE, MC_REFERENCE_MACHINE to a short identifier
  of this machine, and MC_HEADLESS=1.
RUN:
  cp -n .env.example .env || true
  mkdir -p ./content ./target/artifacts ./.local-data
  sh scripts/probes/content_dir.sh
  sh scripts/probes/artifact_dir.sh
  sh scripts/probes/data_dir.sh
EXPECT: three lines: `probe content_dir: ok`, `probe artifact_dir: ok`, `probe data_dir: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-000 MILESTONE_PASS "M4 three roots ok"
FALLBACK: if a chosen path is not writable, choose a different writable path inside the
  repository, update .env, and record it in the Decision Log. Never chmod a system directory.
COMMIT: git add -A && git commit -m "[EP-000][M4] create and verify confinement roots"

### M5: Registry mode
GOAL: The build can fetch dependencies in the declared mode.
READ: PREFLIGHT.md section 3
CHANGE: .cargo/config.toml (only in vendored mode)
CONTENT: in vendored mode only, create .cargo/config.toml with exactly:

[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"

RUN:
  sh scripts/probes/registry_mode.sh
EXPECT: `probe registry_mode: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-000 MILESTONE_PASS "M5 probe registry_mode: ok"
FALLBACK: if `vendored` was chosen and `vendor/` is empty, switch CARGO_REGISTRY_MODE to
  `online` for this run, record an ADR, and note that EP-001 M2 will run `cargo vendor` once
  the workspace exists so that a later offline build is possible.
COMMIT: git add -A && git commit -m "[EP-000][M5] resolve cargo registry mode"

### M6: Graphics stack
GOAL: The host can link mc_shell against its windowing and audio stack.
READ: PREFLIGHT.md section 1.4
CHANGE: (none)
CONTENT: none.
RUN:
  sh scripts/probes/graphics_stack.sh
EXPECT: `probe graphics_stack: ok`
FALLBACK: install the platform development packages named in PREFLIGHT.md section 1.4. If the
  host genuinely cannot provide them, this is STOP condition (a): report the exact missing
  packages and stop. Do not proceed to EP-001 with a host that cannot build the shell.
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-000 MILESTONE_PASS "M6 probe graphics_stack: ok"
COMMIT: git add -A && git commit -m "[EP-000][M6] verify graphics stack"

### M7: Preflight green and scheduler proven
GOAL: preflight.sh reaches its sentinel and graph-next.sh dispatches correctly.
READ: PREFLIGHT.md section 6, .agent/GRAPH.md
CHANGE: (none)
CONTENT: none.
RUN:
  sh scripts/preflight.sh
  sh scripts/graph-next.sh
EXPECT: `preflight: ok` from the first command; `RESUME EP-000` from the second (this node
  holds the lease, which is the correct answer and proves the scheduler reads the ledger).
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-000 MILESTONE_PASS "M7 preflight: ok"
FALLBACK: none needed -- if preflight fails it names the exact missing item, which is fixed
  in the milestone that owns it, not here.
COMMIT: git add -A && git commit -m "[EP-000][M7] preflight green and scheduler proven"

## 9. Validation and Acceptance

| Criterion | Command | Expected |
|---|---|---|
| Toolchain pinned | `rustc --version` | contains `1.83.0` |
| Subcommands pinned | `sh scripts/probes/cargo_tools.sh` | `probe cargo_tools: ok` |
| Roots exist and writable | `sh scripts/probes/data_dir.sh` | `probe data_dir: ok` |
| Registry resolved | `sh scripts/probes/registry_mode.sh` | `probe registry_mode: ok` |
| Graphics stack ready | `sh scripts/probes/graphics_stack.sh` | `probe graphics_stack: ok` |
| Preflight green | `sh scripts/preflight.sh` | `preflight: ok` |
| Scheduler works | `sh scripts/graph-next.sh` | one line, `RESUME EP-000` |
| No game code created | `ls crates 2>/dev/null \| wc -l` | `0` |

## 10. Idempotence and Recovery

Every milestone here is idempotent: `mkdir -p`, `cp -n`, `rustup install`, and
`cargo install --locked` are all safe to repeat. To re-enter cold: read this file's Progress
section, find the first unchecked milestone, re-run the previous milestone's RUN to confirm
its EXPECT still holds, then continue. If the working tree is dirty, `git checkout -- .`
is safe here because no milestone in this node produces work that cannot be regenerated by
re-running it.

## 11. Progress

- [ ] M1 repository initialisation and bootstrap commit
- [ ] M2 toolchain pin
- [ ] M3 cargo subcommands
- [ ] M4 confinement roots
- [ ] M5 registry mode
- [ ] M6 graphics stack
- [ ] M7 preflight green and scheduler proven

## 12. Surprises and Discoveries

<empty>

## 13. Decision Log

<empty>

## 14. Outcomes and Retrospective

<empty>
