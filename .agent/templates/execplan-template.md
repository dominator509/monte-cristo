NODE-META-BEGIN
ID: EP-XXX
DEPS: -
MAX_ATTEMPTS_PER_MILESTONE: 6
VERIFY: sh scripts/verify.sh
VERIFY_SENTINEL: verify: ok
GREEN_TAG: green/EP-XXX
NODE-META-END

# EP-XXX -- <node name>

## 1. Purpose / Big Picture
<Why this node exists and what the repository can do afterwards that it could not before.>

## 2. Scope
<Exactly what is built here.>

## 3. Non-goals
<Specific things that belong to other nodes. Re-read at the start of every milestone.>

## 4. Context and Orientation
<What state the repository is in when this node starts, and which invariants bind it.>

## 5. Files to Read First
<Exact paths.>

## 6. Expected Changed Files
<Exact paths. This is the audit list at node end. No globs.>

## 7. Interfaces and Contracts
<Names and shapes taken from the vocabulary-locked specs.>

## 8. Milestones

### M1: <name>
GOAL:
READ:
CHANGE:
CONTENT:
RUN:
EXPECT:
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-XXX MILESTONE_PASS "M1 <sentinel>"
FALLBACK:
COMMIT: git add -A && git commit -m "[EP-XXX][M1] <summary>"

## 9. Validation and Acceptance
| Criterion | Command | Expected |
|---|---|---|

## 10. Idempotence and Recovery
<How to re-enter this node cold.>

## 11. Progress
- [ ] M1

## 12. Surprises and Discoveries
<empty>

## 13. Decision Log
<empty>

## 14. Outcomes and Retrospective
<empty>
