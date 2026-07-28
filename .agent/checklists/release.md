# Checklist: release

- [ ] `VERSION` updated
- [ ] `CHANGELOG.md` updated, including the mandatory **Determinism** subsection stating
      either "golden tape hash unchanged" or both hashes and the reason
- [ ] `sh scripts/verify.sh` -> `verify: ok`
- [ ] `sh scripts/production-readiness-check.sh` -> `production readiness: ok`
- [ ] `sh scripts/build.sh` -> three tarballs plus SHA256SUMS in `$MC_ARTIFACT_DIR`
- [ ] `ls "$MC_ARTIFACT_DIR"` shows all three target triples and SHA256SUMS
- [ ] `sha256sum -c "$MC_ARTIFACT_DIR/SHA256SUMS"` verifies
- [ ] Cross-target determinism drill: each extracted artifact replays
      `tapes/golden-full.tape` to the hash recorded on Linux
- [ ] `THIRD-PARTY-LICENSES.txt` present in every tarball and inside the allowlist
- [ ] Previous-version save fixtures migrate: `cargo run --locked -p mc_tools -- save-migrate
      --dir tests/fixtures/saves-v1 --dry-run`
- [ ] Rollback drill evidence present in EP-009 Progress M5
- [ ] `git tag -a "v$(cat VERSION)"` created; the tag did not already exist
- [ ] MANUAL publish command printed and **not executed** (Auto-Deploy Authorization is `no`)
- [ ] `RUN_COMPLETE` appended to the ledger
