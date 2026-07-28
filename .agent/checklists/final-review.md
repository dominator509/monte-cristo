# Checklist: final review

- [ ] `cargo clean` then `sh scripts/verify.sh` from scratch -> every sentinel through
      `verify: ok`, observed in this session
- [ ] `sh scripts/reality-gate.sh` -> `reality gate: ok`
- [ ] `sh scripts/live-fire.sh` -> `live-fire: ok` with LF-01 through LF-12 each reported
- [ ] Expected-files audit for every node:
      `git diff --name-only green/EP-<prev>..green/EP-<this>` equals section 6 of that plan
- [ ] Every ExecPlan acceptance criterion marked met with recorded evidence
- [ ] Every ExecPlan has a written Outcomes and Retrospective section
- [ ] `sh scripts/production-readiness-check.sh` -> `production readiness: ok`
- [ ] Every DOC line in PRODUCTION_READINESS.md verified by opening the named file
- [ ] ARCHITECTURE.md invariants still match the code (section 14 review checklist)
- [ ] ENVIRONMENT.md dependency table matches Cargo.lock exactly
- [ ] PREFLIGHT.md machine table matches what scripts/preflight.sh actually checks
- [ ] COMMANDS.md matches the contents of scripts/
- [ ] Every spec's Validation table names test files that exist
- [ ] `grep -rn '{{[A-Z_][A-Z_]*}}' --include='*.md' . | grep -v '^\./docs/6Layer-MasterPrompt'`
      returns nothing (the master prompt copy is provenance and keeps its skeleton tokens)
- [ ] Ledger has no unresolved NODE_BLOCKED and no dangling LEASE
- [ ] Final report written per AGENTS.md section 16
