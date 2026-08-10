NODE-META-BEGIN
ID: EP-010
DEPS: EP-009
MAX_ATTEMPTS_PER_MILESTONE: 6
VERIFY: sh scripts/production-readiness-check.sh
VERIFY_SENTINEL: production readiness: ok
GREEN_TAG: green/EP-010
NODE-META-END

# EP-010 -- Production readiness and ship

## 1. Purpose / Big Picture

Prove the whole thing from scratch and stop in the right place. Full verify from a clean
build, the reality gate, all twelve live-fire proofs, the security, performance,
accessibility, and privacy reviews against their specs, backup and restore verified for real,
the deployment dry run, the documentation review, the ship gate, and the version tag.

Because Auto-Deploy Authorization is `no`, this node ends by printing one MANUAL publish
command and appending `RUN_COMPLETE`. It does not publish.

## 2. Scope

Verification, review, tagging, and the ship gate. No new functionality of any kind.

## 3. Non-goals

No feature work. No content changes. No gate changes. No publishing. If a review finds a
defect, the correct response is to fix it in the node that owns it and re-enter this node,
not to patch it here.

## 4. Context and Orientation

SPEC-008 and PRODUCTION_READINESS.md are authoritative. The evidence rule governs this entire
node: a gate passes only if it was run in this session and its sentinel appeared in real
output. Cached green is not green, and a remembered pass is a fabrication.

## 5. Files to Read First

- .agent/specs/SPEC-008-production-readiness.md
- PRODUCTION_READINESS.md in full
- AGENTS.md section 15
- .agent/checklists/final-review.md
- .agent/prompts/final-review.md

## 6. Expected Changed Files

- PRODUCTION_READINESS.md  (checkboxes only)
- .agent/execplans/EP-000-discovery-and-toolchain.md  (Outcomes section only)
- .agent/execplans/EP-001-foundation.md  (Outcomes section only)
- .agent/execplans/EP-002-core-domain.md  (Outcomes section only)
- .agent/execplans/EP-003-data-and-persistence.md  (Outcomes section only)
- .agent/execplans/EP-004-api-or-service-layer.md  (Outcomes section only)
- .agent/execplans/EP-005-user-interface-or-client.md  (Outcomes section only)
- .agent/execplans/EP-006-auth-security-and-permissions.md  (Outcomes section only)
- .agent/execplans/EP-007-testing-hardening.md  (Outcomes section only)
- .agent/execplans/EP-008-observability-and-operations.md  (Outcomes section only)
- .agent/execplans/EP-009-deployment-and-release.md  (Outcomes section only)
- .agent/state/LEDGER.md

## 7. Interfaces and Contracts

The ship gate is the ten-item list in AGENTS.md section 15 and SPEC-008 section 1, and no
item may be substituted or reordered. Sentinels are exactly `verify: ok`,
`reality gate: ok`, `live-fire: ok`, and `production readiness: ok`. The twelve proofs report
individually as `LF-01 ... ok` through `LF-12 ... ok`. The tag format is `v<VERSION>`. The
MANUAL publish command is the one in DEPLOYMENT.md section 5, printed verbatim and never
executed by an agent session.

## 8. Milestones

### M1: Verify from scratch
GOAL: Every gate passes from a clean build in this session.
READ: COMMANDS.md section 3, SPEC-008 section 2
CHANGE: (none)
CONTENT: none.
RUN:
  cargo clean
  sh scripts/verify.sh
EXPECT: every sentinel in order, ending `verify: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-010 MILESTONE_PASS "M1 verify: ok from clean"
FALLBACK: none needed. A failure here is owned by the node that owns the failing gate.
COMMIT: git add -A && git commit -m "[EP-010][M1] verify from scratch"

### M2: Reality and live fire
GOAL: The reality gate and all twelve proofs pass.
READ: AGENTS.md section 9, SPEC-000 section 2
CHANGE: (none)
CONTENT: none.
RUN:
  sh scripts/reality-gate.sh
  sh scripts/live-fire.sh
EXPECT: `reality gate: ok`; twelve `LF-xx ... ok` lines then `live-fire: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-010 MILESTONE_PASS "M2 reality gate: ok, live-fire: ok"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-010][M2] reality gate and live fire green"

### M3: Expected-files audit and acceptance walk
GOAL: Every node changed exactly what it declared and met every criterion with evidence.
READ: .agent/checklists/final-review.md
CHANGE: (ExecPlan Outcomes sections only)
CONTENT: for each node, diff the files changed between its green tag and its predecessor's
  against section 6 of its plan, and record any difference. Then open each plan's section 9
  and confirm every criterion is marked met with the observed output recorded, not merely
  ticked. Write the Outcomes and Retrospective section of every plan.
RUN:
  for n in 001 002 003 004 005 006 007 008 009; do p=$(printf '%03d' $((10#$n - 1))); echo "== EP-$n"; git diff --name-only "green/EP-$p".."green/EP-$n"; done
EXPECT: each listing matches that node's Expected Changed Files section
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-010 MILESTONE_PASS "M3 expected-files audit clean"
FALLBACK: none needed. A discrepancy is recorded and explained, not hidden.
COMMIT: git add -A && git commit -m "[EP-010][M3] expected-files audit and acceptance walk"

### M4: Reviews against the specs
GOAL: Security, performance, accessibility, and privacy each reviewed against their spec.
READ: SECURITY.md, SPEC-004 section 8, SPEC-008 section 3, PROJECT_BRIEF.md privacy section
CHANGE: (none)
CONTENT: none.
RUN:
  sh scripts/security-check.sh
  sh scripts/dependency-audit.sh
  cargo bench --locked -p mc_core -- battle_step
  cargo test --locked -p mc_shell --test motion_zero --test advisory_screen --test log_redaction
  cargo test --locked -p mc_data --test glyph_parity --test backup_restore
EXPECT: `security check: ok`; `dependency audit: ok`; bench p99 under 4.0 ms; all named tests pass
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-010 MILESTONE_PASS "M4 four reviews clean"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-010][M4] security, performance, accessibility, privacy reviews"

### M5: Deployment dry run and documentation review
GOAL: The artifact set verifies on a clean extraction, and the documentation is true.
READ: DEPLOYMENT.md sections 8, 10, SPEC-008 section 5
CHANGE: (none)
CONTENT: none.
RUN:
  sha256sum -c "$MC_ARTIFACT_DIR/SHA256SUMS"
  d=$(mktemp -d); tar xzf "$MC_ARTIFACT_DIR"/monte-cristo-*-x86_64-unknown-linux-gnu.tar.gz -C "$d"; "$d"/monte-cristo --version; "$d"/monte-cristo --verify-content; MC_HEADLESS=1 "$d"/monte-cristo --replay tapes/golden-smoke.tape --assert-hash; rm -rf "$d"
  grep -rn '{{[A-Z_][A-Z_]*}}' --include='*.md' . | grep -v '^\./docs/6Layer-MasterPrompt' | wc -l
EXPECT: checksums OK; `content: ok`; `hash: match`; the grep count is `0`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-010 MILESTONE_PASS "M5 dry run and docs verified"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-010][M5] deployment dry run and documentation review"

### M6: Ship gate, tag, and MANUAL stop
GOAL: The ship gate passes, the release is tagged, and the run stops without publishing.
READ: AGENTS.md section 15, DEPLOYMENT.md section 5
CHANGE: PRODUCTION_READINESS.md, .agent/state/LEDGER.md
CONTENT: tick every line of PRODUCTION_READINESS.md with its observed evidence. Then tag.
  Then PRINT the MANUAL publish command from DEPLOYMENT.md section 5 without executing it,
  stating plainly that Auto-Deploy Authorization is `no`.
RUN:
  sh scripts/production-readiness-check.sh
  git tag -a "v$(cat VERSION)" -m "MONTE CRISTO v$(cat VERSION)"
  echo "MANUAL: rsync -av --checksum \"$MC_ARTIFACT_DIR\"/ /srv/releases/monte-cristo/\"$(cat VERSION)\"/"
  sh scripts/ledger.sh append <AGENT_ID> EP-010 RUN_COMPLETE "ship gate passed, manual publish pending"
EXPECT: `production readiness: ok`; the tag is created; the MANUAL line is printed and not executed
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-010 MILESTONE_PASS "M6 production readiness: ok"
FALLBACK: none needed. If the tag already exists, that is STOP condition (b) territory: do not
  force it, report it.
COMMIT: git add -A && git commit -m "[EP-010][M6] ship gate passed and release tagged"

## 9. Validation and Acceptance

| Criterion | Command | Expected |
|---|---|---|
| Fresh verify | `cargo clean && sh scripts/verify.sh` | `verify: ok` |
| Reality gate | `sh scripts/reality-gate.sh` | `reality gate: ok` |
| Twelve proofs | `sh scripts/live-fire.sh` | `live-fire: ok` |
| Expected files per node | the M3 loop | matches each plan's section 6 |
| Security review | `sh scripts/security-check.sh` | `security check: ok` |
| Dependency review | `sh scripts/dependency-audit.sh` | `dependency audit: ok` |
| Performance review | `cargo bench --locked -p mc_core -- battle_step` | p99 under 4.0 ms |
| Accessibility review | `cargo test --locked -p mc_shell --test motion_zero` | pass |
| Privacy review | `cargo test --locked -p mc_shell --test log_redaction` | pass |
| Backup and restore | `cargo test --locked -p mc_data --test backup_restore` | pass |
| Artifacts verify | `sha256sum -c "$MC_ARTIFACT_DIR/SHA256SUMS"` | OK for every line |
| No placeholders in agent-facing docs | the M5 placeholder scan | `0` |
| Ship gate | `sh scripts/production-readiness-check.sh` | `production readiness: ok` |
| Not published | the MANUAL line appears in output and was not executed | confirmed |

## 10. Idempotence and Recovery

Every milestone except tagging is read-only verification and is safe to repeat. Tagging is
not: if `v<VERSION>` already exists, stop and report rather than forcing. To re-enter cold:
read Progress, find the first unchecked milestone, and re-run from there; re-running the
verification milestones is always correct and is in fact what the evidence rule requires.

## 11. Progress

- [x] M1 verify from scratch — `cargo clean` removed 5,881 files / 2.6 GiB; the
  subsequent `sh scripts/verify.sh` exited 0 after 447.5 seconds with all subordinate
  gates green and the final `verify: ok` sentinel.
- [x] M2 reality and live fire — `reality gate: ok`; LF-01 through LF-12 each
  reported `ok`; `live-fire: ok`.
- [x] M3 expected-files audit and acceptance walk — audited `green/EP-000` through
  `green/EP-009`; every acceptance row is recorded met in its Outcomes section. Historical
  file-list deviations are enumerated rather than hidden, including the user-approved
  EP-007 98-path Hermes exception.
- [x] M4 reviews against the specs — `security check: ok`; `dependency audit: ok`;
  p99 core/frame 0.000400 ms; shell accessibility/privacy suites 10 passed; data
  glyph/backup suites 5 passed.
- [x] M5 deployment dry run and documentation review — all three manifest entries `OK`;
  final native run 30723274849 clean-extracted Linux/macOS and printed version 0.1.0,
  `content: ok`, `paths: ok`, and `hash: match`; clean Windows extraction printed version,
  content, and hash sentinels; agent-doc placeholder count 0.
- [x] M6 ship gate, tag, and MANUAL stop — 0.1.1 metadata, clean verification, native
  runner artifacts, readiness, and local Windows extraction all passed; the tag and final
  ledger events are recorded below.
  BLOCKED before mutation: annotated tag `v0.1.0` already exists locally and on `origin`,
  pointing to commit `cfc2957065da2049bee2e3c19829b47c201e9c2f` (tag object
  `c66e57bd7597b0d8357bdee97da0e4f4b3199bf3`, dated 2026-07-29). The corrected M5
  candidate is `8cc62cb2de0e611e86bdecb32e0f0029800ebe1e`; the tag was not moved or overwritten.
  Continuation rung-2 audit on 2026-08-01 killed the hypothesis that external state had
  removed or corrected the tag: `VERSION` remains `0.1.0`; local and remote refs still peel
  to `cfc2957`; the target is an ancestor 39 commits behind candidate `4d4092a`; and
  `gh release view v0.1.0` reports no GitHub Release object. The conflict is the remote Git
  tag itself, and it remains untouched.

### NODE_BLOCKED report — EP-010 M6

1. **Exact blocker:** `VERSION` requires creation of `v0.1.0`, but that annotated tag
   already exists locally and on `origin` at pre-ship commit `cfc2957`; replacing it is a
   destructive external history change that M6 explicitly forbids without human authority.
2. **Full evidence:**
   - `sh scripts/preflight.sh` — exit 0, `preflight: ok` on all three consecutive audits.
   - `sh scripts/graph-next.sh` — exit 0, `NEXT EP-010`; M6 is the first unchecked milestone.
   - `Get-Content VERSION` — `0.1.0` on all three audits.
   - `git for-each-ref refs/tags/v0.1.0` — local annotated object
     `c66e57bd7597b0d8357bdee97da0e4f4b3199bf3`, peeled commit
     `cfc2957065da2049bee2e3c19829b47c201e9c2f`, dated 2026-07-29.
   - `git ls-remote --tags origin v0.1.0 v0.1.0^{}` — the remote returns those same two
     object IDs; the conflict is not local-only.
   - `git rev-list --count v0.1.0^{}..HEAD` — `39` on rung 2 and `40` on rung 3; current
     candidate before this report is `b9ef8a0a4a28123092f315e36a056bec1a12b64b`.
   - `gh release view v0.1.0 --json tagName,url` — exit 1, `release not found`; there is no
     GitHub Release object, but the remote Git tag remains an irreversible published ref.
   - `git status --short --branch` plus `git rev-list --left-right --count
     origin/master...master` — branch clean before the ledger lease and `0 0` synchronized.
3. **Error signature and hypotheses:** signature `VERSION_TAG_ALREADY_EXISTS_REMOTE`
   recurred on three consecutive goal turns. Rung-1 hypothesis that the tag was absent was
   disproved by local and remote refs. Rung-2 hypothesis that external state might remove or
   correct it was disproved by identical object IDs. Rung-3 observation that no GitHub
   Release object exists reduces collateral scope but does not authorize deleting a remote
   Git ref.
4. **Rungs climbed:** rung 1 performed the initial no-mutation provenance check; rung 2
   isolated ancestry, remote equality, candidate distance, and Release-object state; rung 3
   re-ran the authoritative boot and remote-ref checks. No code or tag diff was attempted,
   no force push occurred, and the milestone fallback is explicitly `none`.
5. **Smallest human decision that unblocks:** either authorize reopening release planning
   to bump `VERSION` and release metadata to `0.1.1`, or explicitly authorize deletion and
   replacement of both local and remote `v0.1.0` tags.
  6. **Recommended default:** bump to `0.1.1`. It preserves immutable published history and
  permits the final clean verification, readiness checklist, tag, `green/EP-010`, and
  `RUN_COMPLETE` to proceed. The broader graphical-facelift objective also remains
  unverified and cannot be implemented inside M6 because feature work is a declared non-goal.

### M6 resumed by explicit human authorization — 2026-08-09

The user explicitly authorized a version bump to `0.1.1`. This reopens the blocked release
decision without mutating or deleting the existing remote `v0.1.0` tag. M6 resumes as a
fresh blocked audit under the recorded `LEASE_TAKEOVER` event; the graph-control scripts
remain unchanged. The visual audit is limited to the existing shell presentation boundary
and will not alter the deterministic core or release gates.

## 12. Surprises and Discoveries

- M1's first clean verify exposed `clippy::format_collect` in the EP-009 release CLI hex
  formatter. Earlier targeted tests compiled successfully, but only the from-scratch lint
  gate exercised the repository-wide `-D warnings` policy.
- M1's next clean verify passed the test suites but `scripts/security-check.sh` found that
  the shipped shell's `--replay` path used `std::fs::read` directly. This contradicted
  ARCHITECTURE.md INV-07 even though the native release smoke had passed.
- The first narrow compile of the confinement correction showed that `std::fs` remains
  required by the separately confined `--check-paths` write probe. Restoring the module
  import, while leaving the tape read on `fsroot::read`, was the smallest correction.
- M5's documentation walk found that `--verify-content` ignored `MC_CONTENT_DIR`,
  `--save-info` accepted an unconfined path, and operator replay examples did not declare
  the tape's trusted root. Passing gates therefore did not yet make the published command
  surface consistent with INV-07.
- The first direct post-download `build.sh` call did not inherit ignored `.env` values and
  correctly aggregated only the default-root Windows artifact. Re-running with the same
  explicit `MC_ARTIFACT_DIR` used by the clean verifier preserved both downloaded native
  archives, rebuilt Windows, regenerated the manifest, and printed `build: ok`.
- M6's mandatory pre-tag check found `v0.1.0` already published to the remote and pointing
  at a pre-EP-010 commit. The plan explicitly classifies this as irreversible-action STOP
  territory; no readiness checkbox, release tag, or RUN_COMPLETE claim was fabricated.
- The first 0.1.1 clean-gate attempt used a Git-Bash-incompatible Windows artifact path in
  the process environment. Preflight correctly rejected it before graph execution; the
  native archives were moved outside `target` and the `.env` artifact root was set to the
  portable `C:/tmp/mc-0.1.1-artifacts` spelling before retrying.
- The next retry showed that the release tar step receives the Windows-style `C:/...` value
  with an escaped drive prefix under this shell bridge. The fallback is the existing ignored
  `.local-data` root with a relative artifact path, which is accepted by both preflight and
  the tar implementation and survives `cargo clean`.
- The verifier canonicalizes that relative root before invoking the build script; the RTK
  shell bridge then rewrites the POSIX path back to `C\:/...` for `tar`. Direct Git Bash
  preserves the canonical `/c/...` form and is the narrowest environment-specific fallback.

## 13. Decision Log

- 2026-08-01, M1 owning-node correction: fix the EP-009-owned
  `crates/mc_shell/src/main.rs` formatter exactly as Clippy recommends, using `fmt::Write`
  into one preallocated `String`. The repository has no mechanism to reopen a green tag
  without rewriting append-only history, so record this final-review scope exception and
  rerun EP-010 M1 from `cargo clean`; do not weaken or suppress the lint.
- 2026-08-01, M1 confinement correction: route the shipped shell's tape read through
  `fsroot::read(Root::Data, ...)` and make the authorized native workflow explicitly
  designate the repository workspace as `MC_DATA_DIR` for its read-only golden-tape
  replay. Keep these two EP-009-owned paths as a pre-recorded EP-010 scope exception,
  rebuild every native artifact from the corrected commit, and rerun M1 from clean.
- 2026-08-01, M3 historical-boundary audit: literal equality is false for several inherited
  green-tag boundaries. Follow M3's stated fallback by recording each discrepancy and its
  evidence in the affected Outcomes section. Treat only the user's explicit 98-path EP-007
  grandfathering and already-recorded node Decision Log exceptions as authorized; do not
  rewrite tags or describe the audit as exact equality.
- 2026-08-01, M5 confinement-complete correction: confine content verification to
  `MC_CONTENT_DIR`, save inspection and replay to `MC_DATA_DIR`, and update the release
  workflow, smoke gate, artifact README, operator docs, and incident/rollback checklists to
  declare those roots explicitly. These are pre-recorded EP-005/006/008/009-owned scope
  exceptions required to make documentation and shipped behavior agree with INV-07. Rebuild
  all three artifacts from the corrected source before accepting M5.
- 2026-08-01, M5 native execution equivalence: this Windows host cannot execute the Linux
  archive locally. Use the explicitly authorized Ubuntu native job 91430451211, which
  performs clean extraction plus checksum, version, confined content verification, path
  probes, and full golden replay, as the platform-native execution of M5's Linux command.
  Keep the local three-entry manifest check and Windows clean-extraction smoke as independent
  evidence; do not claim Windows executed an ELF binary.
- 2026-08-09, M6 human authorization: the user explicitly authorized bumping the release
  metadata to `0.1.1`. Preserve the published `v0.1.0` tag, update only release metadata and
  the bounded shell presentation audit, then verify and tag `v0.1.1`.

## 14. Outcomes and Retrospective

EP-010 M6 completed under the user-authorized `0.1.1` patch release. The immutable remote
`v0.1.0` tag was preserved. The final clean verifier printed `verify: ok`; standalone
`reality gate: ok`, `live-fire: ok` (LF-01 through LF-12), and
`production readiness: ok` were observed. Native workflow `31349697192` passed Linux and
macOS jobs on the 0.1.1 release candidate, each reporting archive checksum, version,
content, path, and golden-hash success. The local Windows archive extracted and reported
`Monte Cristo v0.1.1`, `content: ok`, `paths: ok`, and `hash: match`.

The M6 plan's declared change list was extended by the explicit human authorization and the
recorded Decision Log: release metadata (`VERSION`, workspace and local package manifests,
`Cargo.lock`, changelog, artifact README), the bounded shell presentation facelift, and
append-only audit evidence were necessary to make the 0.1.1 release truthful. No runtime
networking, core-state, or tape behavior changed. The final artifact manifest contains all
three target archives and every `sha256sum -c` line is `OK`.

Manual publish remains intentionally unexecuted because Auto-Deploy Authorization is `no`:
`rsync -av --checksum "$MC_ARTIFACT_DIR"/ /srv/releases/monte-cristo/"$(cat VERSION)"/`
