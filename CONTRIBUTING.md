# CONTRIBUTING -- MONTE CRISTO

## 1. Setup

See ENVIRONMENT.md section 4. In short: pin the toolchain, install the three cargo
subcommands, create the three directories, fill `.env`, and get `sh scripts/preflight.sh` to
print `preflight: ok`.

## 2. Branch rules

Trunk-based. Work on short-lived branches or directly on main; either way the gate is the
same and `main` must always be releasable. No force-push, no history rewrite, no amend of a
pushed milestone commit.

## 3. Coding standards

- Rust 2021 edition, `rustfmt` default configuration, `clippy` at deny level with the
  allowances listed in `clippy.toml` and nowhere else.
- `#![forbid(unsafe_code)]` at the top of mc_core and mc_data. `unsafe` anywhere else needs
  an ADR.
- No `f32`, `f64`, `HashMap`, `HashSet`, `SystemTime`, or `thread` in mc_core. Not
  discouraged: absent. If the type is not imported, the mistake cannot be made.
- Errors are typed with `thiserror`. `unwrap` and `expect` are permitted only in tests,
  benches, and `main` startup where a failure is genuinely fatal and the message names the
  cause.
- **Comments carry the "why", not the "what", and cite the invariant they uphold** in the
  form `INV-07`. A comment that restates the code is noise; a comment that explains why the
  loop iterates a `BTreeMap` instead of a `HashMap` is load-bearing documentation.

Example of the expected comment style:

    // INV-03: BTreeMap, not HashMap. Spawn order feeds the RNG stream, so
    // iteration order is state-affecting and must not depend on hash seeds.
    let mut eligible: BTreeMap<EnemyId, Weight> = BTreeMap::new();

## 4. Test requirements

Every change ships with the tests required by TESTING.md section 4. A change to anything in
`step::ORDER` also updates the determinism property test. A change that alters observable
behaviour updates the affected tape hashes in `tapes/HASHES.txt` **and** says so in the
CHANGELOG Determinism subsection. Silently re-recording a tape to make a test pass is the
single worst thing you can do in this repository, because it converts a caught regression
into a shipped one.

## 5. Documentation requirements

- A new mechanism updates its spec in `.agent/specs/` before the code lands.
- A new invariant gets a number in ARCHITECTURE.md section 5 and is never renumbered.
- A new dependency updates ENVIRONMENT.md section 2 and Cargo.lock in the same commit.
- A new content type updates its locked vocabulary table in the relevant spec.
- A pre-decided fork or an assumption gets an ADR in DECISIONS.md at the moment it is made.

## 6. Commit format

    [EP-XXX][M<k>] <imperative summary>

Example: `[EP-002][M5] add terrain-gated spawn eligibility with region affinity table`

One commit per milestone, nothing left uncommitted between milestones. Outside of a graph
run, use `[maint] <summary>`.

## 7. Pull-request checklist

- [ ] `sh scripts/verify.sh` green from scratch
- [ ] tests added per TESTING.md section 4
- [ ] golden tape hash unchanged, or changed with an explanation in the CHANGELOG
- [ ] no new `f32`/`f64`/`HashMap`/`HashSet` in mc_core
- [ ] no new `std::fs` call outside `fsroot::confine`
- [ ] no new `cfg` that changes behaviour
- [ ] specs updated before code where a mechanism changed
- [ ] comments cite invariants where relevant
- [ ] scope matches the milestone's CHANGE list exactly

## 8. Review checklist

- [ ] Does this change threaten INV-01 in any way? (floats, hash order, clock, threads)
- [ ] Does content express behaviour that should be a core mechanism? (INV-06)
- [ ] Is any gate weakened, skipped, or asserted from memory rather than observed?
- [ ] Is a test asserting on a double of the thing under test?
- [ ] Does a comment explain why, or merely restate what?
- [ ] Would a new agent, cold, with only the plan and the laws, have done this the same way?

## 9. Agent-specific rules

Agents follow AGENTS.md, which supersedes this file wherever they differ. The most common
mistakes an agent makes in this repository are: adding a `HashMap` in mc_core because it is
the obvious choice; re-recording a tape to make a hash test pass; and "helpfully" refactoring
outside the milestone's CHANGE list. All three are caught, all three are reverted, and all
three are avoidable by reading the milestone before acting (LOOPS.md section 5.6).
