# Checklist: incident response

For this project an incident is one of exactly three things: a published artifact whose
checksum does not match, a release that cannot load the previous version's saves, or a
determinism regression discovered after publication. There is no server, so nothing can be
hot-fixed remotely.

**Detect**
- [ ] Reproduce from a clean install of the published artifact
- [ ] `sha256sum -c SHA256SUMS`
- [ ] `./monte-cristo --verify-content`
- [ ] `MC_HEADLESS=1 ./monte-cristo --replay tapes/golden-smoke.tape --assert-hash`
- [ ] If a player supplied a save and a tape, reproduce exactly on the development machine

**Triage**
- [ ] Classify: artifact integrity / save compatibility / determinism regression
- [ ] Determine the last known-good published version
- [ ] Determine how many versions are affected

**Mitigate**
- [ ] Execute .agent/checklists/rollback.md
- [ ] Confirm the withdrawn version is unreachable

**Communicate**
- [ ] Update the withdrawn version's release notes with the plain statement
- [ ] State clearly whether saves written under the bad version are safe

**Resolve**
- [ ] Fix on trunk with a regression test that would have caught it
- [ ] `sh scripts/verify.sh` -> `verify: ok`
- [ ] Golden tape hash accounted for in the CHANGELOG Determinism subsection

**Verify**
- [ ] Full release checklist for the corrected version
- [ ] Cross-target determinism drill passes on all three targets

**Document**
- [ ] Postmortem ADR in DECISIONS.md
- [ ] The specific question answered: was any gate weakened, skipped, or asserted from memory?

**Follow up**
- [ ] The new regression test is in the permanent suite, not a one-off script
- [ ] If a fuzz crash was involved, the input is now a committed corpus entry and a unit test
