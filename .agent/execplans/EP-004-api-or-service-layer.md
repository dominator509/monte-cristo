NODE-META-BEGIN
ID: EP-004
DEPS: EP-003
MAX_ATTEMPTS_PER_MILESTONE: 6
VERIFY: sh scripts/test-e2e.sh
VERIFY_SENTINEL: e2e tests: ok
GREEN_TAG: green/EP-004
NODE-META-END

# EP-004 -- Command bus and tape harness (mc_tape)

## 1. Purpose / Big Picture

Turn the pure core into something every later claim can be tested against. This node
finalises the public simulation API -- `Command` in, `StateView` out, applied only at tick
boundaries -- and builds the input tape: record, replay, checkpoint, hash, and divergence
report. After this node, "does the game do X" becomes "does this tape produce this hash",
which is the mechanism the entire ship gate rests on.

This is also the branch point: EP-005 and EP-006 both depend on this node and on nothing
else, so two agents may take them concurrently.

## 2. Scope

The `Command` and `StateView` contract in mc_core; `crates/mc_tape` in full; the
`mc_tools replay` and `mc_tools record` subcommands; end-to-end tests including the first
real tape covering Act I to the arrest.

## 3. Non-goals

No rendering (EP-005). No fuzzing or confinement (EP-006). No full-campaign golden tape --
that is EP-007, after the shell exists to record it. This node ships the Act I tape only.

## 4. Context and Orientation

SPEC-003 is authoritative. The tick contract (INV-05) is the subtle part: commands apply only
at tick boundaries, which is exactly what makes a tape a simple ascending list of
`(tick, Command)` pairs. Get that wrong and every later determinism claim becomes untestable.

The tape parser is untrusted input and is validated as strictly as a save.

## 5. Files to Read First

- .agent/specs/SPEC-003-api-contracts.md
- .agent/specs/SPEC-001-core-domain.md sections 15, 16
- ARCHITECTURE.md sections 5 (INV-04, INV-05), 6
- SECURITY.md section 12

## 6. Expected Changed Files

- crates/mc_core/src/command.rs
- crates/mc_core/src/view.rs
- crates/mc_core/src/lib.rs
- crates/mc_tape/src/lib.rs
- crates/mc_tape/src/format.rs
- crates/mc_tape/src/record.rs
- crates/mc_tape/src/replay.rs
- crates/mc_tape/src/divergence.rs
- crates/mc_tape/src/error.rs
- crates/mc_tools/src/main.rs
- crates/mc_tools/src/cmd_replay.rs
- crates/mc_tools/src/cmd_record.rs
- crates/mc_core/tests/command_validation.rs
- crates/mc_core/tests/tick_contract.rs
- crates/mc_tape/tests/tape_roundtrip.rs
- crates/mc_tape/tests/e2e_act1.rs
- crates/mc_tape/tests/e2e_determinism.rs
- crates/mc_tape/tests/divergence_report.rs
- crates/mc_tape/tests/forced_failures.rs
- tapes/act1.tape
- tapes/HASHES.txt

## 7. Interfaces and Contracts

`Command` variants exactly as SPEC-003 section 1, with explicit discriminants and
append-only ordering. `StateView` exactly as section 2. Tape format exactly as section 4,
including the magic bytes `MCTAPE01`.

## 8. Milestones

### M1: Command and StateView
GOAL: The two-type contract exists and invalid commands are rejected, not panicked.
READ: SPEC-003 sections 1, 2, 6
CHANGE: crates/mc_core/src/command.rs, crates/mc_core/src/view.rs, crates/mc_core/src/lib.rs,
  crates/mc_core/tests/command_validation.rs
CONTENT: `Command` with explicit discriminants, `StateView` as a read-only borrow,
  `apply_commands` returning `Vec<CoreEvent>` with `CoreEvent::Rejected { command, reason }`
  for anything invalid. `NameYourself` rejected unless Phase2 and all three dossier flags.
RUN: cargo test --locked -p mc_core --test command_validation
EXPECT: test passes, including that every invalid command in the table produces a rejection
  rather than an error or a panic
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-004 MILESTONE_PASS "M1 command bus rejects cleanly"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-004][M1] add Command and StateView contract"

### M2: Tick contract
GOAL: Commands apply only at tick boundaries and no tick is ever dropped or subdivided.
READ: SPEC-003 section 3, ARCHITECTURE.md INV-05
CHANGE: crates/mc_core/tests/tick_contract.rs
CONTENT: a test that submits commands at fractional accumulator positions and asserts they
  take effect at the following tick boundary exactly, and that the accumulator remainder is
  carried rather than discarded across 10,000 irregular frame deltas.
RUN: cargo test --locked -p mc_core --test tick_contract
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-004 MILESTONE_PASS "M2 tick contract holds"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-004][M2] assert the fixed-timestep tick contract"

### M3: Tape format
GOAL: A tape serializes, parses, and rejects malformed input with typed errors.
READ: SPEC-003 section 4, SECURITY.md section 1
CHANGE: crates/mc_tape/src/format.rs, crates/mc_tape/src/error.rs,
  crates/mc_tape/tests/tape_roundtrip.rs, crates/mc_tape/tests/forced_failures.rs
CONTENT: the struct of SPEC-003 section 4 with magic `MCTAPE01`; strictly ascending tick
  validation; content digest match check; bounded allocation; `TapeError` variants exactly as
  SPEC-006. Forced failures: bad magic, non-monotonic ticks, truncation, unknown command
  discriminant, content mismatch.
RUN: cargo test --locked -p mc_tape --test tape_roundtrip --test forced_failures
EXPECT: both pass
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-004 MILESTONE_PASS "M3 tape format round-trips and rejects"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-004][M3] add tape format with strict validation"

### M4: Record and replay
GOAL: `replay` is deterministic and reports the first divergence when one exists.
READ: SPEC-003 section 5
CHANGE: crates/mc_tape/src/record.rs, crates/mc_tape/src/replay.rs,
  crates/mc_tape/src/divergence.rs, crates/mc_tape/tests/divergence_report.rs
CONTENT: `replay(tape) -> ReplayResult` with `final_hash` and
  `first_divergence: Option<(tick, expected, got)>`; checkpoint comparison so a divergence is
  localised to a tick range rather than hunted through millions of ticks. The divergence test
  deliberately mutates one checkpoint and asserts the report names that exact tick.
RUN: cargo test --locked -p mc_tape --test divergence_report
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-004 MILESTONE_PASS "M4 replay and divergence report"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-004][M4] add tape record, replay, and divergence bisection"

### M5: CLI subcommands
GOAL: `mc_tools replay` and `record` work from the command line as COMMANDS.md documents.
READ: COMMANDS.md section 5
CHANGE: crates/mc_tools/src/main.rs, crates/mc_tools/src/cmd_replay.rs, crates/mc_tools/src/cmd_record.rs
CONTENT: clap subcommands `replay --tape <path> [--print-hash|--assert-hash]` and
  `record --out <path>`. `--assert-hash` exits nonzero on divergence and prints the first
  diverging checkpoint. Neither ever rewrites a tape.
RUN:
  cargo run --locked -p mc_tools -- replay --help
EXPECT: help output listing `--tape`, `--print-hash`, `--assert-hash`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-004 MILESTONE_PASS "M5 replay CLI available"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-004][M5] add replay and record CLI subcommands"

### M6: The Act I tape (LF-01)
GOAL: A real tape plays Act I to the arrest and its hash is recorded.
READ: SPEC-000 section 2 (LF-01), SPEC-009 section 9
CHANGE: tapes/act1.tape, tapes/HASHES.txt, crates/mc_tape/tests/e2e_act1.rs
CONTENT: a hand-authored tape (constructed programmatically in the test, then written) that
  drives a new game through Act I to `ACT1_ARREST`. Its terminal hash is appended to
  `tapes/HASHES.txt` as `act1.tape <hash>`. The e2e test replays it and asserts both the flag
  and the hash.
RUN:
  cargo test --locked -p mc_tape --test e2e_act1
  cargo run --locked -p mc_tools -- replay --tape tapes/act1.tape --assert-hash
EXPECT: test passes; the CLI prints `hash: match`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-004 MILESTONE_PASS "M6 hash: match for act1.tape"
FALLBACK: if Act I content from EP-003 is insufficient to reach the arrest, extend the
  content in `content/scenes/act1/` as part of this milestone and note it in the Decision Log.
  Do not stub the arrest.
COMMIT: git add -A && git commit -m "[EP-004][M6] record the Act I tape and assert LF-01"

### M7: Cross-run determinism (LF-09, partial)
GOAL: The same tape produces the same hash in two independent processes and both profiles.
READ: TESTING.md section 6
CHANGE: crates/mc_tape/tests/e2e_determinism.rs
CONTENT: a test that spawns the replay twice as separate processes and compares hashes, and a
  scripted comparison of debug and release replays of the same tape.
RUN:
  cargo test --locked -p mc_tape --test e2e_determinism
  cargo run --locked -p mc_tools -- replay --tape tapes/act1.tape --print-hash
  cargo run --locked --release -p mc_tools -- replay --tape tapes/act1.tape --print-hash
EXPECT: test passes; the two printed hashes are identical
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-004 MILESTONE_PASS "M7 cross-run and cross-profile hashes match"
FALLBACK: none needed. A mismatch here is an INV-01 defect in EP-002 and is fixed there, not
  worked around here.
COMMIT: git add -A && git commit -m "[EP-004][M7] assert cross-run and cross-profile determinism"

### M8: Node verification
GOAL: The e2e gate is green.
READ: COMMANDS.md section 3
CHANGE: (none)
CONTENT: none.
RUN: sh scripts/test-e2e.sh
EXPECT: `e2e tests: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-004 MILESTONE_PASS "M8 e2e tests: ok"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-004][M8] verify command bus and tape harness"

## 9. Validation and Acceptance

| Criterion | Command | Expected |
|---|---|---|
| Invalid commands rejected | `cargo test --locked -p mc_core --test command_validation` | pass |
| Tick contract holds | `cargo test --locked -p mc_core --test tick_contract` | pass |
| Tape round-trips | `cargo test --locked -p mc_tape --test tape_roundtrip` | pass |
| Malformed tape rejected | `cargo test --locked -p mc_tape --test forced_failures` | pass |
| Divergence localised | `cargo test --locked -p mc_tape --test divergence_report` | pass |
| LF-01 proven | `cargo run --locked -p mc_tools -- replay --tape tapes/act1.tape --assert-hash` | `hash: match` |
| Cross-run determinism | `cargo test --locked -p mc_tape --test e2e_determinism` | pass |
| Node gate | `sh scripts/test-e2e.sh` | `e2e tests: ok` |

## 10. Idempotence and Recovery

Replays are read-only and safe to repeat. Recording is not: `record` writes a tape and must
never overwrite a committed one. To re-enter cold: read Progress, find the first unchecked
milestone, re-run the previous milestone's RUN, continue. If `tapes/act1.tape` exists but its
hash does not match `tapes/HASHES.txt`, do **not** re-record it -- investigate, because that
is exactly the signal the whole harness exists to produce.

## 11. Progress

- [ ] M1 command and state view
- [ ] M2 tick contract
- [ ] M3 tape format
- [ ] M4 record and replay
- [ ] M5 CLI subcommands
- [ ] M6 the Act I tape
- [ ] M7 cross-run determinism
- [ ] M8 node verification

## 12. Surprises and Discoveries

<empty>

## 13. Decision Log

<empty>

## 14. Outcomes and Retrospective

- All eight acceptance rows are met. Ledger evidence records command rejection, tick
  contract, tape round-trip and rejection, divergence localization, CLI replay,
  `hash: match`, cross-run determinism, and `e2e tests: ok`; EP-010 re-ran these suites.
- Changed-files audit: 22 paths changed. Every declared tape, CLI, command, test, and tape
  asset path is present except the planned standalone `mc_core/src/view.rs`; the inherited
  implementation keeps the view surface with command/world state. The only additional path
  is the required L6 ledger.
- Retrospective: consolidated state avoided a duplicate view surface without changing the API.
