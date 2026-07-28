# OPERATIONS -- MONTE CRISTO

This is desktop software. "Operations" means the build host, the release drill, and the
support runbooks for a player's machine. There is no service to keep up.

## 1. Local operations

    sh scripts/preflight.sh          # environment truth
    sh scripts/verify.sh             # every gate, in order
    sh scripts/live-fire.sh          # the twelve core proofs
    sh scripts/ledger.sh tail 30     # what the run has been doing
    sh scripts/graph-next.sh         # what happens next
    git log --oneline | head -30     # the other half of the telemetry

## 2. Staging operations

Staging is a clean directory with an extracted tarball. Its only operation is the EP-009 M4
drill: extract, replay the golden tape, compare the hash, delete the directory.

## 3. Production operations

The player runs a binary. The operator's responsibilities are limited to: keeping published
artifacts and their SHA256SUMS intact, retaining the previous version's directory for
rollback, and responding to reports.

## 4. Health checks

| Check | Command | Healthy |
|---|---|---|
| Binary integrity | `sha256sum -c SHA256SUMS` | all OK |
| Content integrity | `./monte-cristo --verify-content` | `content: ok` |
| Simulation integrity | `MC_HEADLESS=1 ./monte-cristo --replay tapes/golden-smoke.tape --assert-hash` | `hash: match` |
| Data directory | `./monte-cristo --check-paths` | three roots resolved and writable |

## 5. Common failure modes and exact diagnostics

| Symptom | Likely cause | Diagnostic | Action |
|---|---|---|---|
| Game will not start, exits immediately | content.pack missing or digest mismatch | `./monte-cristo --verify-content` | reinstall from the tarball; do not hand-edit content.pack |
| "unsupported save version" | the save was written by a newer build | `./monte-cristo --save-info <file>` | install the newer version; never downgrade a save |
| Save fails to write | data directory not writable or full | `./monte-cristo --check-paths`; `df -h` | fix permissions or free space; the previous save is intact |
| Frame rate below 60 | GPU below the reference class, or a compositor forcing vsync off | run with the in-game frame overlay enabled | lower the window scale; report with the overlay numbers |
| Input not recognised | a remap file from an older schema | `./monte-cristo --dump-input-map` | delete settings.ron; defaults are restored |
| Replay hash mismatch on a player machine | a corrupted binary or a tampered content pack | `sha256sum -c SHA256SUMS` | reinstall |

## 6. Troubleshooting the build host

| Symptom | Diagnostic | Action |
|---|---|---|
| A gate fails only in CI | compare `env` output; `sh scripts/preflight.sh` on the runner | fix the missing variable or tool |
| `LAYER_VIOLATION` | `cargo tree -p mc_core --depth 1` | remove the upward import |
| `DETERMINISM_HASH_MISMATCH` | `cargo run -p mc_tools -- replay --tape <t> --print-hash` twice | find the float, hash order, clock, or thread |
| Fuzzer will not build | `rustup toolchain list` | install nightly-2025-01-15, or take the EP-006 fallback |
| Cross-build link error | `sh scripts/probes/graphics_stack.sh` | install mingw-w64 |

## 7. Backup and restore

**Build host:** the repository is the backup; git history plus the green tags reconstruct any
state. The ledger is append-only and must never be rewritten.

**Player:** the save directory under MC_DATA_DIR is the only irreplaceable data. The
documented backup is to copy that directory. `--save-info` prints schema version and digest
for each file so a player can confirm a backup is intact. Restore is copying it back;
because saves carry a digest, a corrupted restore is detected rather than loaded.

**Verification of the backup path** is an EP-010 checklist item performed for real: copy the
directory, corrupt one byte in the copy, confirm the corrupt file is rejected with a typed
error and the good file still loads.

## 8. Scheduled jobs

None. There is no cron, no timer, no background task, and no updater. This is deliberate:
an updater would require network access, which INV-09 forbids.

## 9. Incident triage

See .agent/checklists/incident-response.md. For this project an "incident" is one of: a
published artifact whose checksum does not match, a release that cannot load the previous
version's saves, or a determinism regression discovered after publication. All three are
handled by rollback plus a corrected release; none of them can be hot-fixed remotely,
because there is no remote.

## 10. Escalation

Single-operator project. Escalation is the blocked report in the ExecPlan Progress section
and the run halting. There is no on-call rotation and no pager.

## 11. Maintenance

Dependency updates are deliberate, never automatic: change the pinned version, run
`sh scripts/verify.sh`, confirm the golden tape hash is unchanged, and record an ADR if the
update altered any behaviour. A dependency update that changes the golden tape hash is a
determinism event and is investigated as one.

## 12. Operational safety rules

Never rewrite the ledger. Never force-push. Never delete a green tag. Never hand-edit
content.pack or a save file. Never publish an artifact whose determinism drill did not pass
on all three targets. Never run the MANUAL publish command from an agent session.
