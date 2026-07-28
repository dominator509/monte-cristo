# PLANS -- the ExecPlan standard

An ExecPlan is a self-contained implementation document for one node. **A new agent with no
prior conversation must be able to complete it from the plan, the laws, and the ledger
alone.** If a plan fails that bar, it is defective and is fixed before it is executed.

## Machine header

Every plan opens with:

    NODE-META-BEGIN
    ID: EP-XXX
    DEPS: <csv or ->
    MAX_ATTEMPTS_PER_MILESTONE: <n>
    VERIFY: <exact node-level verify command>
    VERIFY_SENTINEL: <exact expected line>
    GREEN_TAG: green/EP-XXX
    NODE-META-END

## Required sections, in order

1. Purpose / Big Picture
2. Scope
3. Non-goals
4. Context and Orientation
5. Files to Read First (exact paths)
6. Expected Changed Files (exact paths -- this is the audit list)
7. Interfaces and Contracts (from the vocabulary-locked specs)
8. Milestones
9. Validation and Acceptance (node level)
10. Idempotence and Recovery
11. Progress (one checkbox per milestone)
12. Surprises and Discoveries (empty scaffold)
13. Decision Log (empty scaffold)
14. Outcomes and Retrospective (empty scaffold)

## Milestone grammar -- every milestone, no exceptions

    ### M<k>: <name>
    GOAL: one sentence, observable.
    READ: exact paths to re-read before acting (the re-grounding list).
    CHANGE: exact paths created or modified. Nothing else may change.
    CONTENT: the complete file bodies to transcribe, or anchored edits with exact old text
             and exact new text plus a verification grep, or the exact discovery commands
             whose output fills a named template blank.
    RUN: exact commands, in order.
    EXPECT: the exact sentinel line or lines RUN must produce.
    EVIDENCE: the exact ledger append.
    FALLBACK: the pre-decided alternative for ladder rung 3. A real, simpler implementation,
              never a mock. "none needed" is legal only for trivially safe milestones and
              must be justified in one clause.
    COMMIT: git add -A && git commit -m "[EP-XXX][M<k>] <summary>"

## Execution rules

Milestones run strictly in order. Each ends with RUN and EXPECT; on mismatch, climb the
ladder in LOOPS.md section 5.3. Re-ground at the start of every milestone (LOOPS.md 5.6).
Commit at the end of every milestone. Append MILESTONE_PASS with the observed sentinel in
the detail.

## Validation and acceptance

Node-level acceptance is a list of criteria, each with the exact command that proves it and
the exact expected output. A criterion that cannot be proven by a command is not a criterion;
it is a hope, and it does not belong in a plan.

## Idempotence and recovery

Every plan states how to re-enter the node cold: which files to read, how to determine the
first unchecked milestone, what to re-verify before proceeding, and what to reset if the
working tree is dirty. Assume the agent has no memory of the previous session, because it
does not.

## Progress, Surprises, Decision Log, Outcomes

Progress is checkboxes, one per milestone, checked only after EXPECT was observed. Surprises
records anything the plan did not anticipate, including every failed hypothesis from ladder
rung 2 and above. Decision Log records every pre-decided fork taken and every assumption
made, at the moment it is made. Outcomes is written once at node completion: what shipped,
what was learned, what the next node should know.

## Quality bar

Handed any single plan cold, a lower-tier executor must never need to ask: which file, which
command, what is done, what behaviour, what is out of scope, what if it fails, what if the
repository differs, which files may change, what goes in my final answer.
