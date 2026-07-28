# Checklist: agent readiness (plan self-containment audit)

Run before executing any node. Open the plan and confirm each line.

- [ ] NODE-META header present with ID, DEPS, MAX_ATTEMPTS_PER_MILESTONE, VERIFY,
      VERIFY_SENTINEL, GREEN_TAG
- [ ] All fourteen sections present, in order (.agent/PLANS.md)
- [ ] Section 3 Non-goals is non-empty and specific
- [ ] Section 5 Files to Read First lists exact paths that exist: verify with
      `for f in <paths>; do [ -f "$f" ] || echo "MISSING $f"; done`
- [ ] Section 6 Expected Changed Files lists exact paths, no globs, no "etc"
- [ ] Every milestone carries all nine grammar fields: GOAL, READ, CHANGE, CONTENT, RUN,
      EXPECT, EVIDENCE, FALLBACK, COMMIT
- [ ] Every EXPECT names a literal sentinel string, not "should pass"
- [ ] Every FALLBACK names a real simpler implementation, or justifies "none needed" in one
      clause
- [ ] Every RUN command appears in COMMANDS.md or is a plain `cargo`/`git` invocation with
      `--locked` where applicable
- [ ] Section 9 acceptance criteria each name a command and its expected output
- [ ] Section 10 explains cold re-entry: which files, which milestone, what to re-verify
- [ ] Section 11 Progress has exactly one checkbox per milestone
- [ ] Sections 12, 13, 14 exist as empty scaffolds
- [ ] No placeholder token in the plan: `grep -c '{{[A-Z_][A-Z_]*}}' <plan>` returns 0
- [ ] No elision: `grep -nE '\.\.\.|rest omitted|similar to above' <plan>` returns nothing
