NODE-META-BEGIN
ID: EP-001
DEPS: EP-000
MAX_ATTEMPTS_PER_MILESTONE: 6
VERIFY: sh scripts/verify.sh
VERIFY_SENTINEL: verify: ok
GREEN_TAG: green/EP-001
NODE-META-END

# EP-001 -- Foundation

## 1. Purpose / Big Picture

Raise the five-crate workspace and prove that every gate in COMMANDS.md is real by running
the whole of `verify.sh` green against a skeleton. After this node the repository has a
committed lockfile, enforced formatting and linting, an enforced crate-layer DAG, one real
passing test, and a CI entry point that adds nothing of its own. No game logic yet.

## 2. Scope

Workspace manifests; pinned dependencies and committed Cargo.lock; rustfmt and clippy
configuration; the crate-layer check; `deny.toml`; one real unit test that asserts something
true; the `VERSION` file; the CI entry point; `verify.sh` green end to end.

## 3. Non-goals

No domain logic. No content. No rendering. No save format. No tape format. The five crates
exist with their module skeletons and nothing else. Specifically: do not implement `Fx`, do
not implement the ATB, do not write RON. Those are EP-002 and EP-003.

## 4. Context and Orientation

EP-000 left a pinned toolchain, three writable roots, and a green preflight. The crate layer
DAG in ARCHITECTURE.md section 4 is the load-bearing structure of this node: getting it wrong
here is expensive to fix later, because every subsequent node builds on top of it.

## 5. Files to Read First

- ARCHITECTURE.md sections 3, 4, 5
- ENVIRONMENT.md section 2
- TESTING.md sections 1, 8
- SECURITY.md section 7
- COMMANDS.md

## 6. Expected Changed Files

- Cargo.toml
- Cargo.lock
- VERSION
- deny.toml
- clippy.toml
- rustfmt.toml
- crates/mc_core/Cargo.toml
- crates/mc_core/src/lib.rs
- crates/mc_data/Cargo.toml
- crates/mc_data/src/lib.rs
- crates/mc_tape/Cargo.toml
- crates/mc_tape/src/lib.rs
- crates/mc_shell/Cargo.toml
- crates/mc_shell/src/main.rs
- crates/mc_tools/Cargo.toml
- crates/mc_tools/src/main.rs
- .ci/verify.yml

## 7. Interfaces and Contracts

Crate names and their permitted dependencies are fixed by ARCHITECTURE.md section 4. Exact
dependency versions are fixed by ENVIRONMENT.md section 2. Neither may be varied here.

## 8. Milestones

### M1: Workspace manifest and VERSION
GOAL: `cargo metadata` resolves a five-member workspace.
READ: ARCHITECTURE.md section 3, ENVIRONMENT.md section 2
CHANGE: Cargo.toml, VERSION
CONTENT: create Cargo.toml with exactly this content:

[workspace]
resolver = "2"
members = ["crates/mc_core", "crates/mc_data", "crates/mc_tape", "crates/mc_shell", "crates/mc_tools"]

[workspace.package]
edition = "2021"
rust-version = "1.83.0"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
serde = { version = "=1.0.215", default-features = false, features = ["derive"] }
postcard = { version = "=1.0.10", default-features = false, features = ["alloc"] }
blake3 = "=1.5.5"
indexmap = "=2.7.0"
thiserror = "=2.0.6"
ron = "=0.8.1"
macroquad = "=0.4.13"
tracing = "=0.1.41"
tracing-subscriber = { version = "=0.3.19", features = ["json"] }
clap = { version = "=4.5.23", features = ["derive"] }
proptest = "=1.5.0"
insta = "=1.41.1"
criterion = "=0.5.1"
arbitrary = "=1.4.1"

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true

Create VERSION containing exactly: 0.1.0
RUN:
  cargo metadata --locked --format-version 1 > /dev/null || cargo metadata --format-version 1 > /dev/null
EXPECT: exit code 0 and no output
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-001 MILESTONE_PASS "M1 workspace resolves"
FALLBACK: if a pinned version does not exist on this registry, use the nearest existing patch
  of the same minor, update ENVIRONMENT.md section 2 in the same commit, and record an ADR.
COMMIT: git add -A && git commit -m "[EP-001][M1] create workspace manifest and VERSION"

### M2: Five crate skeletons with the layer DAG
GOAL: All five crates compile and their dependency edges match ARCHITECTURE.md section 4.
READ: ARCHITECTURE.md section 4
CHANGE: crates/*/Cargo.toml, crates/*/src/lib.rs, crates/mc_shell/src/main.rs, crates/mc_tools/src/main.rs
CONTENT:
  crates/mc_core/Cargo.toml depends on: serde, postcard, blake3, indexmap, thiserror. Nothing
    else. It must NOT depend on any other project crate, on macroquad, on tracing, on ron, or
    on clap.
  crates/mc_core/src/lib.rs begins with exactly these two lines:
    #![forbid(unsafe_code)]
    //! INV-01: pure, deterministic, no I/O. See ARCHITECTURE.md section 5.
  crates/mc_data/Cargo.toml depends on: mc_core (path), serde, postcard, ron, blake3, thiserror.
  crates/mc_data/src/lib.rs begins with `#![forbid(unsafe_code)]`.
  crates/mc_tape/Cargo.toml depends on: mc_core (path), serde, postcard, blake3, thiserror.
  crates/mc_shell/Cargo.toml depends on: mc_core, mc_data, mc_tape (paths), macroquad,
    tracing, tracing-subscriber, thiserror.
  crates/mc_tools/Cargo.toml depends on: mc_core, mc_data, mc_tape (paths), clap, ron,
    tracing, tracing-subscriber, thiserror.
  Each src file contains only the header, module declarations that exist, and nothing else.
  Do not add placeholder functions -- an empty crate is honest, a stubbed one is not.
RUN:
  cargo build --locked --workspace
  cargo tree -p mc_core --depth 1
EXPECT: build succeeds; the `cargo tree` output for mc_core contains none of `mc_data`,
  `mc_tape`, `mc_shell`, `mc_tools`, `macroquad`, `tracing`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-001 MILESTONE_PASS "M2 five crates build, layer clean"
FALLBACK: none needed -- the dependency table is fully specified and admits no ambiguity.
COMMIT: git add -A && git commit -m "[EP-001][M2] create five crate skeletons with enforced layering"

### M3: Lockfile committed
GOAL: Cargo.lock is committed and every later command uses `--locked`.
READ: AGENTS.md section 10
CHANGE: Cargo.lock
CONTENT: none -- the file is generated.
RUN:
  cargo generate-lockfile
  git add Cargo.lock
  cargo build --locked --workspace
EXPECT: build succeeds with `--locked` and no lockfile modification
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-001 MILESTONE_PASS "M3 lockfile committed"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-001][M3] commit Cargo.lock"

### M4: Formatting and linting configuration
GOAL: `format-check.sh` and `lint.sh` both reach their sentinels.
READ: CONTRIBUTING.md section 3
CHANGE: rustfmt.toml, clippy.toml
CONTENT:
  rustfmt.toml with exactly:
    edition = "2021"
    max_width = 100
  clippy.toml with exactly:
    msrv = "1.83.0"
    too-many-arguments-threshold = 8
RUN:
  cargo fmt --all
  sh scripts/format-check.sh
  sh scripts/lint.sh
EXPECT: `format check: ok` then `lint: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-001 MILESTONE_PASS "M4 format check: ok / lint: ok"
FALLBACK: if a clippy lint fires on skeleton code that cannot be satisfied without inventing
  logic, add the specific lint to `clippy.toml` with a comment naming why, and record an ADR.
  Never add a blanket `#![allow(clippy::all)]`.
COMMIT: git add -A && git commit -m "[EP-001][M4] add formatting and lint configuration"

### M5: Dependency policy
GOAL: `dependency-audit.sh` reaches its sentinel with the licence allowlist enforced.
READ: SECURITY.md section 7, DECISIONS.md ADR-014
CHANGE: deny.toml
CONTENT: create deny.toml with exactly:

[advisories]
yanked = "deny"

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "Zlib", "Unicode-3.0"]
confidence-threshold = 0.9

[bans]
multiple-versions = "deny"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"

RUN:
  sh scripts/dependency-audit.sh
EXPECT: `dependency audit: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-001 MILESTONE_PASS "M5 dependency audit: ok"
FALLBACK: if `multiple-versions = "deny"` fails on an unavoidable transitive duplicate, add a
  narrow `skip` entry naming that exact crate and version pair, with an ADR. Never relax it to
  "warn" globally, and never relax the licence list at all -- that is STOP condition (c).
COMMIT: git add -A && git commit -m "[EP-001][M5] add cargo-deny policy"

### M6: One real passing test
GOAL: The test harness is proven by a test that asserts something true rather than `true`.
READ: TESTING.md section 1
CHANGE: crates/mc_core/src/lib.rs
CONTENT: add to crates/mc_core/src/lib.rs a public const and a test module:

/// The number of regions in the campaign. SPEC-009 section 1 is authoritative.
pub const REGION_COUNT: usize = 15;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_count_matches_spec() {
        // INV-06: content is data, but the region count is a structural fact the
        // bake validates against, so it lives in code as the single source.
        assert_eq!(REGION_COUNT, 15);
    }
}

RUN:
  sh scripts/test-unit.sh
EXPECT: `unit tests: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-001 MILESTONE_PASS "M6 unit tests: ok"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-001][M6] prove the test harness with a real test"

### M7: Typecheck, build, and reality gate
GOAL: The remaining skeleton gates reach their sentinels.
READ: COMMANDS.md section 3
CHANGE: (none)
CONTENT: none.
RUN:
  sh scripts/typecheck.sh
  sh scripts/build.sh
  sh scripts/reality-gate.sh
  sh scripts/security-check.sh
EXPECT: `typecheck: ok`, `build: ok`, `reality gate: ok`, `security check: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-001 MILESTONE_PASS "M7 four gates ok"
FALLBACK: if the reality gate fires on a legitimate identifier in skeleton code, rename the
  identifier. Adding a line to `.agent/reality-allow` is the last resort and requires a
  Decision Log entry (L5 rule).
COMMIT: git add -A && git commit -m "[EP-001][M7] typecheck, build, reality gate, security check green"

### M8: CI entry point
GOAL: CI invokes `scripts/verify.sh` and contains no gate logic of its own (ADR-013).
READ: DECISIONS.md ADR-013
CHANGE: .ci/verify.yml
CONTENT: create .ci/verify.yml with exactly:

# Self-hosted CI entry point. ADR-013: this file contains no gate logic.
# The single source of truth for what "green" means is scripts/verify.sh.
name: verify
on: [push]
jobs:
  verify:
    runs-on: self-hosted
    steps:
      - uses: actions/checkout@v4
      - run: sh scripts/verify.sh

RUN:
  grep -cE 'cargo (test|clippy|fmt|build)' .ci/verify.yml
EXPECT: `0`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-001 MILESTONE_PASS "M8 ci contains no gate logic"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-001][M8] add CI entry point that only calls verify.sh"

### M9: Full verify green
GOAL: `verify.sh` runs every gate in order and reaches `verify: ok` on the skeleton.
READ: COMMANDS.md section 3
CHANGE: (none)
CONTENT: none.
RUN:
  sh scripts/verify.sh
EXPECT: `verify: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-001 MILESTONE_PASS "M9 verify: ok"
FALLBACK: none needed -- a failure here is a failure of an individual gate and is fixed in
  the milestone that owns that gate.
COMMIT: git add -A && git commit -m "[EP-001][M9] full verify green on the skeleton"

## 9. Validation and Acceptance

| Criterion | Command | Expected |
|---|---|---|
| Workspace has five members | `cargo metadata --locked --format-version 1 \| grep -c '"name":"mc_'` | at least 5 |
| Layer DAG holds | `cargo tree -p mc_core --depth 1` | contains no other project crate |
| Lockfile committed | `git ls-files Cargo.lock` | `Cargo.lock` |
| Formatting enforced | `sh scripts/format-check.sh` | `format check: ok` |
| Linting enforced | `sh scripts/lint.sh` | `lint: ok` |
| Licence policy enforced | `sh scripts/dependency-audit.sh` | `dependency audit: ok` |
| A real test passes | `sh scripts/test-unit.sh` | `unit tests: ok` |
| CI adds nothing | `grep -cE 'cargo (test\|clippy\|fmt\|build)' .ci/verify.yml` | `0` |
| Full gate green | `sh scripts/verify.sh` | `verify: ok` |
| No unsafe in core | `grep -c 'forbid(unsafe_code)' crates/mc_core/src/lib.rs` | `1` |

## 10. Idempotence and Recovery

All milestones are file-creation or configuration and are safe to repeat. To re-enter cold:
read Progress, find the first unchecked milestone, re-run the previous milestone's RUN, and
continue. If the tree is dirty, reset to the last `[EP-001][M<k>]` commit and redo that
milestone from the top. Cargo.lock must never be regenerated after M3 without an explicit
decision, because a silent lockfile change is a supply-chain event.

## 11. Progress

- [ ] M1 workspace manifest and VERSION
- [ ] M2 five crate skeletons with the layer DAG
- [ ] M3 lockfile committed
- [ ] M4 formatting and linting configuration
- [ ] M5 dependency policy
- [ ] M6 one real passing test
- [ ] M7 typecheck, build, and reality gate
- [ ] M8 CI entry point
- [ ] M9 full verify green

## 12. Surprises and Discoveries

<empty>

## 13. Decision Log

<empty>

## 14. Outcomes and Retrospective

- All ten acceptance rows are met. The node ledger records the five-member workspace,
  clean dependency direction, committed lockfile, format/lint/dependency/unit sentinels,
  zero duplicated CI gate logic, and `verify: ok`; the 2026-08-01 clean verify rechecked
  every stable gate.
- Changed-files audit: 25 paths changed from `green/EP-000` to `green/EP-001`. All 17
  declared paths are present. The eight additional paths are the L6 ledger, ENVIRONMENT.md,
  PREFLIGHT.md, and dependency, security, and ledger gate scripts. They are recorded
  historical Hermes baseline deviations, not hidden changes.
- Retrospective: later work strengthened the gates without changing the five-crate layout.
