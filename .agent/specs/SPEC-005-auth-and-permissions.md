# SPEC-005 -- Security baseline

**Authentication and authorization are not applicable to this product.** There are no
accounts, no sessions, no tokens, and no remote resources. This is recorded explicitly so a
future reader does not assume it was forgotten.

What EP-006 implements instead is the security baseline that genuinely applies to offline
software that consumes untrusted files. Every item below is a real control with a real test.

## 1. Trust boundaries

Exactly three inbound boundaries: save files, content packs, and input tapes. All three are
untrusted byte streams. All three parsers must: never panic, never allocate on an unvalidated
length, verify a digest before structural interpretation, and fail with a typed error.

## 2. Filesystem confinement (INV-07)

One function:

    mc_shell::fsroot::confine(root: Root, requested: &Path) -> Result<PathBuf, FsError>

It canonicalises `requested`, resolves symlinks, and asserts the result is inside the
canonicalised root. `Root` is one of `Content`, `Data`, `Artifact`, resolved once at startup
from the three environment variables.

No other code in the project calls `File::open`, `File::create`, `fs::read`, or
`fs::write` directly. `scripts/security-check.sh` greps for those calls outside
`crates/mc_shell/src/fsroot.rs` and `crates/mc_tools/src/` and fails on a hit.

Traversal, absolute-path injection, and symlink escape are all fuzz and unit targets.

## 3. No network (INV-09)

Absence beats prohibition: no crate in the dependency tree provides networking, which
`scripts/security-check.sh` asserts by scanning `cargo tree` for a denylist of networking
crates. Additionally, `crates/mc_shell/tests/no_socket.rs` replays a golden tape with a
syscall filter (seccomp on Linux; a `LD_PRELOAD` interposer elsewhere) and fails if any
socket is created.

## 4. No unsafe

`#![forbid(unsafe_code)]` in mc_core and mc_data. Any `unsafe` in mc_shell, mc_tape, or
mc_tools requires an ADR naming the reason and the review. `scripts/security-check.sh` counts
`unsafe` occurrences and compares against the ADR-approved list.

## 5. Fuzzing

Two required targets with committed corpora:

    fuzz/fuzz_targets/fuzz_save.rs        // Save::parse over arbitrary bytes
    fuzz/fuzz_targets/fuzz_content.rs     // ContentPack::parse over arbitrary bytes

Plus one for the tape parser. Each must survive a minimum run without a crash, a hang, or an
out-of-memory. Corpora live in `fuzz/corpus/<target>/` and are committed; crash artifacts are
gitignored. Every historical crash becomes a permanent corpus entry and a unit test.

Fallback if the nightly toolchain is unavailable (ASSUMPTIONS A-07): `arbitrary`-driven
proptest corpora on stable, which is weaker but real.

## 6. Secrets

There are none. `scripts/security-check.sh` nonetheless scans the whole tree for
high-entropy strings and common credential shapes and fails on a hit, so the property stays
true as the project grows.

## 7. Dependency policy

`deny.toml`: advisories at high or critical fail; licences restricted to MIT, Apache-2.0,
BSD-2-Clause, BSD-3-Clause, Zlib, Unicode-3.0; duplicate major versions denied; only the
official registry or the vendored mirror as a source. A licence outside the list is STOP
condition (c).

## 8. Audit logging

The security-relevant events logged locally: content digest verification result at startup,
save digest verification result on each load, every `fsroot::confine` rejection with the
requested path relative to the root, and every parser rejection with its typed error. These
are `warn` level and are never suppressed.

## 9. Abuse prevention

The realistic abuse case for offline software is a maliciously crafted save or content pack
shared between players. The controls are sections 1, 2, and 5. There is no rate limit to
apply, no account to lock, and no server to protect.

## 10. Validation

| Control | Test |
|---|---|
| parser never panics | `fuzz/` targets plus `forced_failures.rs` in mc_data and mc_tape |
| digest before interpretation | `crates/mc_data/tests/save_roundtrip.rs` |
| traversal rejected | `crates/mc_shell/tests/fsroot_confine.rs` |
| symlink escape rejected | `crates/mc_shell/tests/fsroot_confine.rs` |
| no socket during full replay | `crates/mc_shell/tests/no_socket.rs` |
| no unsafe in core and data | `scripts/security-check.sh` |
| no networking crate in the tree | `scripts/security-check.sh` |
| no secrets committed | `scripts/security-check.sh` |
| licences inside the allowlist | `scripts/dependency-audit.sh` |
| logs redacted | `crates/mc_shell/tests/log_redaction.rs` |
