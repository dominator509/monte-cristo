# SECURITY -- MONTE CRISTO

## 1. Security goals

This is offline single-player software with no accounts, no network, and no secrets. That
removes most of the usual attack surface and leaves a small, sharp one: **two parsers that
consume untrusted bytes, and a filesystem the software must not wander around in.** Almost
everything below is about those two things. The rest is about keeping the "no network, no
secrets" claims true rather than merely asserted.

## 2. Threat model summary

| Threat | Vector | Control |
|---|---|---|
| Malicious save file (downloaded, shared, or corrupted) | `MC_DATA_DIR/saves/*.sav` | Typed-error parser, fuzzed with a committed corpus, blake3 digest, schema version refusal, no unbounded allocation (INV-08) |
| Malicious or tampered content pack | `content.pack` | Content addressing plus digest verification at load; release builds refuse loose RON |
| Path traversal via a save name or content path | any user-influenced path component | `fsroot::confine` canonicalises and asserts containment in one of three roots (INV-07); it is the only place the software opens a file |
| Exfiltration of local data | any socket | No networking dependency exists in the tree at all (INV-09); an integration test asserts zero socket syscalls across a full replay |
| Supply-chain compromise | a crate dependency | Exact pinned versions, committed Cargo.lock, `cargo-deny` advisory and licence gates on every verify |
| Memory-safety defect | `unsafe` | `#![forbid(unsafe_code)]` in mc_core and mc_data; any `unsafe` elsewhere requires an ADR |
| Accidental secret introduction | a future contributor | Committed-secret scan over the whole tree on every verify, even though there are no secrets today |
| Log leakage of host paths | local logs | Redaction of any absolute path outside the declared roots, asserted by a test over a full replay's log output |

## 3. Authentication and authorization

Not applicable: no accounts, no sessions, no tokens, no remote resource. Recording this
explicitly so a later reader does not assume it was forgotten. EP-006 therefore implements
the baseline above rather than an auth system.

## 4. Input validation at every trust boundary

There are exactly three inbound trust boundaries.

1. **Save files.** Length-prefixed fields are validated against a maximum before allocation.
   Enum discriminants are checked against their valid range. The digest is verified before
   any structural interpretation. A newer schema version is refused, never guessed.
2. **Content packs.** The digest is verified first. Every reference is resolved against the
   pack's own index; a dangling reference is a load failure, not a null. Counts are bounded.
3. **Command-line arguments and environment.** Paths are confined (INV-07); the three root
   variables are canonicalised once at startup and never re-read.

Player keyboard and gamepad input is not a trust boundary in the security sense -- it is a
finite set of enumerated commands -- but the tape parser that replays recorded input **is**,
and it is validated as strictly as a save file.

## 5. Output encoding

The only outputs are rendered frames, audio, local log lines, and files this software wrote.
Log lines are structured JSON with values escaped by the serializer; no log line is built by
string concatenation of untrusted content.

## 6. Secret management

There are no secrets in this project. To keep that true: `.env` is gitignored; no credential
appears in any file; `scripts/security-check.sh` runs a committed-secret scan over the whole
tree on every verify and fails on a hit. If a future change genuinely requires a secret, it
must be added to PREFLIGHT.md with a probe and read from the environment only -- never
committed, never logged, never included in a crash report.

## 7. Dependency security policy

`cargo-deny` runs on every verify with these gates:

- **Advisories:** any advisory of severity high or critical fails the build. A waiver
  requires an ADR in DECISIONS.md naming the advisory identifier, the reason the project is
  not exposed, and the compensating control. Waivers carry an expiry date.
- **Licences:** allowlist is MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, Zlib, Unicode-3.0.
  Anything else is STOP condition (c) in AGENTS.md. There is no waiver path for licences.
- **Bans:** duplicate major versions of the same crate are denied, to keep the tree small
  and the audit surface honest.
- **Sources:** only the official crate registry, or the vendored mirror when
  CARGO_REGISTRY_MODE is `vendored`.

## 8. Log redaction rules

Never logged: absolute filesystem paths outside the three declared roots, the operator's
home directory or username, environment variable values, and the full content of a save
file. Paths inside a declared root are logged relative to that root. A redaction test
replays a golden tape with logging enabled and asserts that the output contains no `/home/`,
no `C:\Users`, and no `/Users/` sequence.

## 9. Data protection and privacy

No personal data is collected, stored, or transmitted. The only files written are saves,
settings, logs, and local crash reports under MC_DATA_DIR. There is no identifier of any
kind, no fingerprint, and no usage record. Uninstalling is deleting the directories, and
DEPLOYMENT.md names exactly which ones.

## 10. Production-data rules

There is no production data, because there is no server. The nearest equivalent is a
player's save directory. Rules: no test ever writes into MC_DATA_DIR; every test uses its
own temporary directory; the save migration command has a `--dry-run` that is the default in
every documented invocation; a migration never deletes the pre-migration file, it renames it
with a `.bak` suffix and reports the path.

## 11. Safe-migration rules

Save schema migrations follow expand-migrate-contract: add the new field with a default,
ship a version that reads both shapes, migrate on load, and only remove the old field a
version later. Every migration is reversible in the sense that the `.bak` file is retained.
Round-trip tests and an insta snapshot of the wire bytes pin the format at each version.

## 12. API security

Not applicable -- no network API. The internal command bus is nonetheless validated:
`apply_commands` rejects any command that is invalid for the current state rather than
asserting, which is what makes the tape replayer robust against a malformed tape. Notably,
`Command::NameYourself` is rejected unless the three dossier flags are set (INV-14); this is
a gameplay rule enforced at the same boundary as the safety rules, deliberately, so that it
cannot be bypassed by a crafted tape.

## 13. Hardening checklist (wired into scripts/security-check.sh)

- [ ] `#![forbid(unsafe_code)]` present in mc_core and mc_data
- [ ] zero `unsafe` blocks outside crates with an ADR
- [ ] zero `HashMap`, `HashSet`, `f32`, `f64`, `SystemTime`, `thread` in mc_core
- [ ] zero networking crates anywhere in `cargo tree`
- [ ] zero `std::fs` calls outside `mc_shell::fsroot` and `mc_tools`
- [ ] committed-secret scan clean
- [ ] `cargo-deny check advisories bans licenses sources` clean
- [ ] fuzz corpora present and non-empty for both parsers
- [ ] no-socket integration test present and passing
- [ ] log-redaction test present and passing

## 14. Security STOP conditions

Subset of AGENTS.md section 5. Stop and report if: a dependency's licence falls outside the
allowlist; an advisory is high or critical and no compensating control exists; a change would
require `unsafe` in mc_core or mc_data; a change would require network access at runtime; or
a change would require reading or writing outside the three declared roots.
