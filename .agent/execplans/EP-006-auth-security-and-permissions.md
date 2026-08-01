NODE-META-BEGIN
ID: EP-006
DEPS: EP-004
MAX_ATTEMPTS_PER_MILESTONE: 6
VERIFY: sh scripts/security-check.sh
VERIFY_SENTINEL: security check: ok
GREEN_TAG: green/EP-006
NODE-META-END

# EP-006 -- Security baseline

## 1. Purpose / Big Picture

There is no authentication in this product -- no accounts, no sessions, no tokens, no remote
resources -- so this node implements the security baseline that genuinely applies to offline
software consuming untrusted files: filesystem confinement, hardened parsers with committed
fuzz corpora, a proven absence of networking, a proven absence of unsafe code in the
deterministic crates, licence and advisory policy, and a committed-secret scan that keeps the
"no secrets" claim true as the project grows.

## 2. Scope

`mc_shell::fsroot`, both parser hardening passes, three fuzz targets with corpora, the
no-socket test, the log-redaction test, and the full body of `scripts/security-check.sh`.

## 3. Non-goals

No authentication or authorization system -- writing one would be inventing a requirement.
No rendering (EP-005, running in parallel). No coverage work (EP-007). Do not edit any file in
EP-005's Expected Changed Files list.

## 4. Context and Orientation

SPEC-005 is authoritative. EP-005 and EP-006 both depend only on EP-004 and may run
concurrently under the lease protocol; coordinate only through git and the ledger.

The important idea in this node is **absence beats prohibition**: mc_core cannot perform I/O
because it has no dependency that can, not because a rule forbids it. Where you can remove a
capability instead of policing it, remove it.

## 5. Files to Read First

- .agent/specs/SPEC-005-auth-and-permissions.md
- SECURITY.md in full
- ARCHITECTURE.md section 5 (INV-07, INV-08, INV-09)
- TESTING.md section 5

## 6. Expected Changed Files

- crates/mc_shell/src/fsroot.rs
- crates/mc_shell/tests/fsroot_confine.rs
- crates/mc_shell/tests/no_socket.rs
- crates/mc_shell/tests/log_redaction.rs
- crates/mc_data/src/save.rs
- crates/mc_data/src/pack.rs
- crates/mc_tape/src/format.rs
- fuzz/Cargo.toml
- fuzz/fuzz_targets/fuzz_save.rs
- fuzz/fuzz_targets/fuzz_content.rs
- fuzz/fuzz_targets/fuzz_tape.rs
- fuzz/corpus/fuzz_save/
- fuzz/corpus/fuzz_content/
- fuzz/corpus/fuzz_tape/
- scripts/security-check.sh
- deny.toml

## 7. Interfaces and Contracts

`confine(root: Root, requested: &Path) -> Result<PathBuf, FsError>` exactly as SPEC-005
section 2. `Root` is `Content | Data | Artifact`. Error variants exactly as SPEC-006 section 2.

## 8. Milestones

### M1: Filesystem confinement
GOAL: One function opens every path, and traversal and symlink escape are rejected.
READ: SPEC-005 section 2, ARCHITECTURE.md INV-07
CHANGE: crates/mc_shell/src/fsroot.rs, crates/mc_shell/tests/fsroot_confine.rs
CONTENT: `confine` canonicalises the requested path, resolves symlinks, and asserts the
  result is inside the canonicalised root. The three roots resolve once at startup from the
  environment and are never re-read. Tests cover `../../etc/passwd`, an absolute path outside
  the root, a symlink pointing outside, and a legitimate nested path.
RUN: cargo test --locked -p mc_shell --test fsroot_confine
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-006 MILESTONE_PASS "M1 confinement holds"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-006][M1] add filesystem confinement with traversal tests"

### M2: Single point of file access
GOAL: No code outside `fsroot` and mc_tools opens a file directly.
READ: SPEC-005 section 2
CHANGE: crates/mc_data/src/save.rs, crates/mc_data/src/pack.rs
CONTENT: route every file access through a caller-supplied already-confined path. mc_data
  takes `&Path` values it does not itself resolve, which is why it needs no confinement of
  its own.
RUN:
  grep -rn 'File::open\|File::create\|fs::read\|fs::write' crates/ --include=*.rs | grep -v 'mc_shell/src/fsroot.rs' | grep -v 'mc_tools/src/' | grep -v '/tests/' | wc -l
EXPECT: `0`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-006 MILESTONE_PASS "M2 single file access point"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-006][M2] route all file access through fsroot"

### M3: Parser hardening
GOAL: Both untrusted parsers bound every allocation and never panic.
READ: SPEC-005 section 1, SPEC-002 section 6
CHANGE: crates/mc_data/src/save.rs, crates/mc_data/src/pack.rs, crates/mc_tape/src/format.rs
CONTENT: every length-prefixed field checked against a declared maximum before allocating;
  every enum discriminant range-checked; digest verified before structural interpretation;
  no `unwrap`, `expect`, or indexing panic on any path reachable from parse input.
RUN:
  cargo test --locked -p mc_data --test forced_failures
  cargo test --locked -p mc_tape --test forced_failures
EXPECT: both pass
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-006 MILESTONE_PASS "M3 parsers hardened"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-006][M3] harden save, pack, and tape parsers"

### M4: Fuzz targets and corpora
GOAL: Three fuzz targets exist with committed corpora and survive a run without a crash.
READ: SPEC-005 section 5, ASSUMPTIONS.md A-07
CHANGE: fuzz/Cargo.toml, fuzz/fuzz_targets/*.rs, fuzz/corpus/*/
CONTENT: `fuzz_save`, `fuzz_content`, and `fuzz_tape`, each parsing arbitrary bytes and
  asserting only that the process does not crash, hang, or exhaust memory. Seed corpora
  built from real valid artifacts plus hand-mutated variants. Crash artifacts stay gitignored;
  corpora are committed.
RUN:
  rustup toolchain install nightly-2025-01-15
  cargo +nightly-2025-01-15 fuzz run fuzz_save -- -max_total_time=120 -runs=100000
  cargo +nightly-2025-01-15 fuzz run fuzz_content -- -max_total_time=120 -runs=100000
  cargo +nightly-2025-01-15 fuzz run fuzz_tape -- -max_total_time=120 -runs=100000
EXPECT: each run completes with `Done` and no crash artifact written
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-006 MILESTONE_PASS "M4 three fuzz targets clean"
FALLBACK: if nightly-2025-01-15 is unavailable (ASSUMPTIONS A-07), build equivalent
  `arbitrary`-driven proptest corpora on stable that exercise the same input space, record an
  ADR naming the reduced assurance, and keep the fuzz targets in the tree for later use.
COMMIT: git add -A && git commit -m "[EP-006][M4] add fuzz targets and committed corpora"

### M5: No network, proven
GOAL: No socket is opened during a full replay, and no networking crate is in the tree.
READ: SPEC-005 section 3, ARCHITECTURE.md INV-09
CHANGE: crates/mc_shell/tests/no_socket.rs, scripts/security-check.sh
CONTENT: a test that replays a tape under a syscall filter (seccomp on Linux, an interposer
  elsewhere) and fails if any socket is created. `security-check.sh` additionally scans
  `cargo tree` for a denylist of networking crates and fails on any hit.
RUN:
  cargo test --locked -p mc_shell --test no_socket
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-006 MILESTONE_PASS "M5 zero sockets during replay"
FALLBACK: if a syscall filter is unavailable on this platform, assert the absence
  structurally instead: no networking crate anywhere in `cargo tree`, plus a link-time check
  that the binary imports no socket symbol. Record an ADR naming the weaker guarantee.
COMMIT: git add -A && git commit -m "[EP-006][M5] prove no socket is opened during a full replay"

### M6: Log redaction
GOAL: No host path, username, or home directory reaches a log line.
READ: SECURITY.md section 8, SPEC-007 section 1
CHANGE: crates/mc_shell/tests/log_redaction.rs
CONTENT: replay a tape with logging enabled and assert the output contains no `/home/`, no
  `/Users/`, and no `C:\Users` sequence, and that paths appear relative to their declared root.
RUN: cargo test --locked -p mc_shell --test log_redaction
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-006 MILESTONE_PASS "M6 logs redacted"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-006][M6] assert log redaction over a full replay"

### M7: Security check script complete
GOAL: `security-check.sh` runs the full hardening checklist of SECURITY.md section 13.
READ: SECURITY.md section 13, SPEC-005 section 10
CHANGE: scripts/security-check.sh, deny.toml
CONTENT: the ten checklist items of SECURITY.md section 13, each failing loudly and naming
  what it found: forbid-unsafe present in mc_core and mc_data; zero unsafe outside the
  ADR-approved list; zero `f32`, `f64`, `HashMap`, `HashSet`, `SystemTime`, `std::thread` in
  mc_core; no networking crate in the tree; no `std::fs` outside `fsroot` and mc_tools;
  committed-secret scan clean; cargo-deny clean; fuzz corpora present and non-empty; the
  no-socket and log-redaction tests present.
RUN: sh scripts/security-check.sh
EXPECT: `security check: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-006 MILESTONE_PASS "M7 security check: ok"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-006][M7] complete the security check gate"

## 9. Validation and Acceptance

| Criterion | Command | Expected |
|---|---|---|
| Traversal rejected | `cargo test --locked -p mc_shell --test fsroot_confine` | pass |
| Single file access point | the M2 grep pipeline | `0` |
| Parsers never panic | `cargo test --locked -p mc_data --test forced_failures` | pass |
| Fuzz corpora present | `ls fuzz/corpus/fuzz_save \| wc -l` | greater than 0 |
| No socket during replay | `cargo test --locked -p mc_shell --test no_socket` | pass |
| Logs redacted | `cargo test --locked -p mc_shell --test log_redaction` | pass |
| Licences inside allowlist | `sh scripts/dependency-audit.sh` | `dependency audit: ok` |
| No unsafe in core and data | `sh scripts/security-check.sh` | `security check: ok` |
| Node gate | `sh scripts/security-check.sh` | `security check: ok` |

## 10. Idempotence and Recovery

Everything here is additive and repeatable; fuzz runs are time-bounded and safe to repeat. To
re-enter cold: read Progress, find the first unchecked milestone, re-run the previous
milestone's RUN, continue. If EP-005 is running concurrently, do not revert files outside this
node's Expected Changed Files list; report a collision instead.

## 11. Progress

- [ ] M1 filesystem confinement
- [ ] M2 single point of file access
- [ ] M3 parser hardening
- [ ] M4 fuzz targets and corpora
- [ ] M5 no network, proven
- [ ] M6 log redaction
- [ ] M7 security check script complete

## 12. Surprises and Discoveries

<empty>

## 13. Decision Log

<empty>

## 14. Outcomes and Retrospective

- All nine acceptance rows are met. The node ledger records confinement, the single access
  point, parser hardening, real fuzz corpora, zero sockets, redaction, and `security check:
  ok`; EP-010 re-observed the security sentinel after correcting the shipped replay read.
- Changed-files audit: 28 paths changed. The declared confinement, fuzz, parser, network,
  redaction, and security-gate surface is present; `deny.toml` was inherited from EP-001
  rather than changed here. Extras are the L6 ledger, Cargo.lock, and approved supporting
  data/shell tests and modules that make the security properties executable.
- Retrospective: the final-review catch proves the static single-access-point gate's value.
