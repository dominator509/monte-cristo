# ROLLBACK -- MONTE CRISTO

## 1. Triggers

Roll back a published release if any of these is true:

- a published tarball's checksum does not verify
- the shipped binary replays the golden tape to a different hash than recorded
- a save written by the previous version fails to load
- content.pack fails its own digest check on a clean install
- a crash reproduces on a clean install within the first playable minutes

Do **not** roll back for: a balance complaint, a performance report on hardware below the
reference class, or a cosmetic defect. Those are the next release.

## 2. Decision owner

The operator. Single-owner project; there is no committee and no approval chain.

## 3. Application rollback

Previous version directories are retained on the publish host. Rollback is republishing the
previous directory as current and removing the bad version's listing:

    # MANUAL, performed by the operator, never by an agent session
    ln -sfn /srv/releases/monte-cristo/<previous-version> /srv/releases/monte-cristo/current
    mv /srv/releases/monte-cristo/<bad-version> /srv/releases/monte-cristo/.withdrawn-<bad-version>

Then verify:

    sha256sum -c SHA256SUMS
    MC_HEADLESS=1 ./monte-cristo --replay tapes/golden-smoke.tape --assert-hash

## 4. Data rollback

There is no server-side data. The player's saves are the only data and they are never
touched by a rollback. The one hazard is a player who has already saved under a newer save
schema: those saves will refuse to load on the older build, by design (INV-13), with a typed
`UnsupportedVersion` error rather than silent corruption. The release notes for a withdrawn
version must say so plainly.

## 5. Configuration rollback

Settings files are forward and backward tolerant: unknown keys are ignored, missing keys take
defaults. No configuration rollback is required. If a settings file is unreadable the game
restores defaults and logs a warning rather than failing to start.

## 6. Feature-flag rollback

Not applicable. There are no runtime feature flags, because INV-10 forbids configuration
that alters behaviour. The only cargo features are `debug-overlay` and platform selection,
neither of which is in a release build's default set.

## 7. Repository rollback (during the build run)

Different mechanism, same discipline. From ladder rung 4 only:

    git reset --hard <last green tag or last [EP-XXX][M<k-1>] commit>
    sh scripts/ledger.sh append <AGENT_ID> EP-XXX ROLLBACK "reset to <ref>"

Rollback never crosses a completed node's green tag. After rolling back, re-enter the
milestone on its declared FALLBACK path, once.

## 8. Verification after rollback

- [ ] `sha256sum -c SHA256SUMS` verifies for the restored version
- [ ] `./monte-cristo --verify-content` prints `content: ok`
- [ ] golden smoke tape replays to the recorded hash
- [ ] a save from two versions back still loads
- [ ] the withdrawn version is no longer reachable from the published index

## 9. Communication

Update the release notes of the withdrawn version with a one-paragraph statement: what was
wrong, who is affected, what to do, and whether saves are safe. No telemetry means no way to
notify anyone directly, so the note must be findable and plain.

## 10. Postmortem

Required for every rollback, appended to DECISIONS.md as an ADR. It must answer: what gate
should have caught this and did not; what test now exists so that it would; and whether the
gate was ever weakened, skipped, or asserted from memory rather than observed. That last
question is the one that matters, because every one of this pack's guarantees rests on
gates being genuinely run.

## 11. Rollback drill

EP-009 M5 performs a real drill, not a tabletop one: publish version N to a temporary
directory tree, publish N+1, detect a deliberately corrupted N+1, execute the section 3
commands, and run the section 8 verification. Evidence is recorded in the EP-009 Progress
section. A drill that was not actually performed does not count.
