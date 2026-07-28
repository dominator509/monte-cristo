# EXECUTION RULES -- one page

- **One active node.** At most one node is IN_PROGRESS repo-wide. The ledger's LEASE event
  says who holds it.
- **No hidden context.** Everything you need is in the pack. If it is not written down, it is
  not a requirement.
- **Never implement from ROADMAP.md.** Implementation happens only through the graph.
- **Continue by default.** Finish the node. Do not ask for next steps, preferences, or
  confirmation.
- **STOP list only.** The five conditions in AGENTS.md section 5. Nothing else is a stop.
- **Anti-drift.** The milestone's CHANGE list is exhaustive. Revert anything outside it. No
  broad refactors, no unrelated cleanup, no opportunistic renaming.
- **Anti-hallucination.** Never invent an API, command, flag, environment variable, content
  identifier, or story flag. Read the file that defines it or transcribe it from the pack.
  Commands come only from COMMANDS.md.
- **Transcription over composition.** When a milestone gives you a file body, transcribe it
  exactly. Do not improve it.
- **Anti-fixation.** Climb the ladder in LOOPS.md 5.3. Never apply the same fix twice.
- **Evidence before edits.** Read the file before you touch it. Confirm the name exists.
- **Evidence before done.** A gate passes only if you ran it in this session and saw the
  sentinel. Claiming a pass from memory is fabrication.
- **Diff review.** After every milestone: `git status --porcelain` and
  `git diff --name-only HEAD~1`.
- **Boot sequence.** Every session starts with the PRIME BLOCK's seven steps. No exceptions,
  including when you think you remember the state.
- **Ledger duties.** LEASE on start, HEARTBEAT every 15 minutes and after every milestone,
  MILESTONE_PASS with the observed sentinel, SIG on every failure, NODE_DONE or
  NODE_BLOCKED at the end, LEASE_RELEASE if you stop for any other reason.
- **Determinism is sacred.** In this project specifically: a hash mismatch is never worked
  around, never quarantined, and never fixed by re-recording the tape.
- **Final response.** Node completed; changed files versus expected; commands run with
  observed sentinels; acceptance status per criterion; decisions; assumptions; risks; ship-gate
  status.
