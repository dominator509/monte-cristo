# Checklist: implementation (run per milestone)

**Before acting (the re-grounding loop, LOOPS.md 5.6):**
- [ ] Re-read the milestone block itself
- [ ] Re-read the node's Non-goals
- [ ] `sh scripts/ledger.sh tail 15`

**While working:**
- [ ] Only the paths in CHANGE are touched
- [ ] Where CONTENT gives a file body, it is transcribed exactly, not improved
- [ ] Where CONTENT gives an anchored edit, the exact old text is found first and the
      verification grep is run after
- [ ] Every name used (API, flag, identifier, command) was read from a file or transcribed
      from the pack -- none invented
- [ ] New comments carry the "why" and cite an invariant number where relevant
- [ ] No `f32`, `f64`, `HashMap`, `HashSet`, `SystemTime`, or `thread` added to mc_core
- [ ] No `std::fs` call added outside `fsroot::confine` or mc_tools
- [ ] No `cfg` added that changes behaviour rather than presentation

**After RUN:**
- [ ] The exact EXPECT sentinel appeared in real output in this session
- [ ] `git status --porcelain` shows only CHANGE paths
- [ ] `git diff --name-only HEAD~1` after commit shows only CHANGE paths
- [ ] EVIDENCE ledger line appended with the observed sentinel in the detail
- [ ] COMMIT made with the exact `[EP-XXX][Mk]` format
- [ ] Progress checkbox ticked
- [ ] Any surprise recorded in Surprises and Discoveries
- [ ] Any decision recorded in the Decision Log at the moment it was made
