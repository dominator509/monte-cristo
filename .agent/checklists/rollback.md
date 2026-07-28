# Checklist: rollback

**Trigger confirmed (one of):**
- [ ] a published tarball's checksum does not verify
- [ ] the shipped binary replays the golden tape to a different hash
- [ ] a previous-version save fails to load
- [ ] content.pack fails its digest check on a clean install
- [ ] a crash reproduces on a clean install in the first playable minutes

**Execute (MANUAL, operator only, never from an agent session):**
- [ ] Repoint `current` to the previous version directory
- [ ] Move the bad version to `.withdrawn-<version>`

**Verify:**
- [ ] `sha256sum -c SHA256SUMS` verifies for the restored version
- [ ] `./monte-cristo --verify-content` -> `content: ok`
- [ ] `MC_HEADLESS=1 ./monte-cristo --replay tapes/golden-smoke.tape --assert-hash` ->
      `hash: match`
- [ ] a save from two versions back still loads
- [ ] the withdrawn version is unreachable from the published index

**Communicate:**
- [ ] Release notes of the withdrawn version updated: what was wrong, who is affected, what
      to do, whether saves are safe

**Close:**
- [ ] Postmortem ADR appended to DECISIONS.md answering: which gate should have caught this;
      what test now exists so it would; and whether any gate was weakened, skipped, or
      asserted from memory rather than observed

**Repository rollback during a build run (different mechanism):**
- [ ] Only from ladder rung 4
- [ ] `git reset --hard <last green tag or previous milestone commit>`
- [ ] ROLLBACK ledger event appended naming the ref
- [ ] Did not cross a completed node's green tag
- [ ] Re-entered the milestone on its declared FALLBACK, once
