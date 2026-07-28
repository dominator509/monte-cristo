# RELEASE -- MONTE CRISTO

## 1. Release types

| Type | Meaning | Save compatibility |
|---|---|---|
| patch (0.0.x) | fixes only; no content change that alters the golden tape hash | full |
| minor (0.x.0) | content additions, balance changes, new tapes | forward: old saves load |
| major (x.0.0) | save schema break or a deliberate determinism change | migration required |

A change that alters the golden tape hash is never a patch, even if the code change looks
small. The hash is the definition of behaviour here.

## 2. Versioning

Semantic versioning against observable game behaviour, not against the Rust API. The version
lives in `VERSION` at the repository root and is the single source; Cargo.toml versions are
derived from it by the build script and asserted equal by `scripts/build.sh`.

The save schema has its own independent integer version, bumped only when the save shape
changes, and recorded in SPEC-002.

## 3. Changelog

`CHANGELOG.md`, Keep-a-Changelog shape, one section per version with Added, Changed, Fixed,
and a mandatory **Determinism** subsection that states either "golden tape hash unchanged"
or the old and new hashes with the reason. That subsection is not optional and its absence
fails the release checklist.

## 4. Branch strategy

Trunk-based. `main` is always releasable because the ship gate is the merge gate. Release
candidates are tags on main, never long-lived branches. No release branch exists to drift.

## 5. Release-candidate criteria

- `sh scripts/verify.sh` green from scratch in one session
- all twelve live-fire proofs passing
- golden tape hash matches, or the change is documented in the Determinism subsection
- coverage floors met
- `cargo deny check advisories bans licenses sources` clean
- previous-version save fixtures migrate cleanly
- artifacts build for all three targets and each replays to the Linux hash

## 6. Release checklist

1. Update `VERSION`.
2. Update `CHANGELOG.md`, including the Determinism subsection.
3. `sh scripts/verify.sh` -> `verify: ok`
4. `sh scripts/production-readiness-check.sh` -> `production readiness: ok`
5. `sh scripts/build.sh` -> three tarballs plus SHA256SUMS in `$MC_ARTIFACT_DIR`
6. EP-009 M4 cross-target determinism drill -> all three hashes match
7. `git tag -a "v$(cat VERSION)" -m "MONTE CRISTO v$(cat VERSION)"`
8. Print the MANUAL publish command. Stop.

## 7. Smoke after release

From a clean directory, against the published files:

    sha256sum -c SHA256SUMS
    tar xzf monte-cristo-<version>-x86_64-unknown-linux-gnu.tar.gz
    ./monte-cristo --version
    ./monte-cristo --verify-content
    MC_HEADLESS=1 ./monte-cristo --replay tapes/golden-smoke.tape --assert-hash

## 8. Approvals

Auto-Deploy Authorization is `no`, so exactly one human approval exists and it is the
publish step itself. No other approval is required: when the gates pass, the artifact is
correct by the project's own definition, and the human is deciding to publish rather than
re-checking the machine.

## 9. Release notes

Written for players, not for the repository. State what is new, what is fixed, whether saves
carry forward, and whether the version changes behaviour in a way that invalidates a
recorded tape (which matters to speedrunners and is the reason the Determinism subsection
exists).

## 10. Post-release monitoring

There is no telemetry, so monitoring means: the published checksums still verify, and player
reports. Because the game is deterministic, a report accompanied by a save reproduces
exactly on the developer's machine, which is the entire triage path.
