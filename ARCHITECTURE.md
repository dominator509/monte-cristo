# ARCHITECTURE -- MONTE CRISTO

L2 SPECIFICATION. Concrete rules only. Every architectural invariant is numbered and is
cited by number in production code comments, in the form `INV-07`.

## 1. Purpose

Make a 40-hour narrative RPG mechanically verifiable. Every design decision below exists to
serve one property: **the game is a pure function of (seed, content pack, input tape)**. If
that holds, every gameplay claim becomes a test, and the ship gate can be a script.

## 2. System overview

    +-------------------------------------------------------------+
    |  mc_shell    presentation: window, GPU, audio, input, frames |
    |              reads core state, writes core commands          |
    +----------------------------+--------------------------------+
                                 | Command / StateView (no other channel)
    +----------------------------v--------------------------------+
    |  mc_core     the game. Pure. Deterministic. No I/O.          |
    |              world, party, ATB, bestiary, spawns, budget,    |
    |              curriculum, poison, flags, scenes, save model   |
    +----------------------------^--------------------------------+
                                 |
              +------------------+------------------+
              |                                     |
    +---------+---------+                 +---------+---------+
    |  mc_data          |                 |  mc_tape          |
    |  RON -> bake ->   |                 |  record / replay  |
    |  content.pack     |                 |  tapes, hashing   |
    +-------------------+                 +-------------------+
              |                                     |
              +------------------+------------------+
                                 |
                        +--------v---------+
                        |  mc_tools  CLI   |
                        |  bake, validate, |
                        |  replay, report  |
                        +------------------+

## 3. Repository map

    /crates/mc_core        pure simulation. No dependency capable of I/O.
    /crates/mc_data        content schema, RON parse, bake, content.pack reader
    /crates/mc_tape        input tape format, recorder, replayer, state hashing
    /crates/mc_shell       macroquad presentation, the only binary a player runs
    /crates/mc_tools       developer and CI CLI
    /content               RON content sources (maps, bestiary, scenes, items, tables)
    /tapes                 committed input tapes, including tapes/golden-full.tape
    /fuzz                  cargo-fuzz targets and committed corpora
    /scripts               every gate, every probe. POSIX sh only.
    /.agent                the pack: laws, graph, plans, specs, checklists, state
    /docs                  GAME_DESIGN.md and the filled master prompt

## 4. Layer law for the code

The crate dependency graph is a DAG and is enforced, not merely documented.

| Crate | May depend on | May never depend on |
|---|---|---|
| mc_core | nothing project-local | mc_data, mc_tape, mc_shell, mc_tools |
| mc_data | mc_core | mc_tape, mc_shell, mc_tools |
| mc_tape | mc_core | mc_data, mc_shell, mc_tools |
| mc_shell | mc_core, mc_data, mc_tape | mc_tools |
| mc_tools | all of the above | -- |

Enforcement: `scripts/lint.sh` runs `cargo tree -p <crate> --depth 1` for each crate and
fails on any forbidden edge, with signature `LAYER_VIOLATION`.

## 5. Architectural invariants

These are numbered permanently. Code comments cite them. Tests reference them.

**INV-01 Determinism.** mc_core is a pure function of its state and its inputs. It contains
no floating point in state-affecting arithmetic, no hash-order-dependent iteration, no
system clock, no thread, no I/O, and no ambient randomness. The only randomness is a
PCG64 generator seeded from the save and advanced explicitly. Violating INV-01 is the
highest-severity defect class in this project because it invalidates LF-07 through LF-09
simultaneously.

**INV-02 Fixed point.** All state-affecting arithmetic uses `Fx` (Q16.16, i32 backing).
Floating point exists only in mc_shell, for rendering interpolation and audio, and never
flows back into core state. There is no `f32` or `f64` anywhere in mc_core's public or
private types.

**INV-03 Ordered iteration.** Any iteration whose order can affect state uses `Vec`,
`BTreeMap`, or `IndexMap`. `HashMap` and `HashSet` are forbidden in mc_core entirely, not
merely discouraged, so that the mistake is impossible rather than caught.

**INV-04 One channel.** mc_shell communicates with mc_core through exactly two types:
`Command` in, `StateView` out. There is no shared mutable state, no callback into the shell,
and no back-channel. A rendering concern never becomes a simulation concern.

**INV-05 Fixed timestep.** Simulation advances in ticks of exactly 1/60 second. The shell
may drop or repeat frames; it may never drop, subdivide, or double a tick. Accumulated time
is carried, never discarded.

**INV-06 Content is data.** No map, enemy, encounter, item, ability, scene, or line of
dialogue is expressed in Rust. All of it lives in `/content` as RON and is baked into
content.pack. A code change is never required to change content, and content can never
introduce behaviour that is not already a mechanism in mc_core.

**INV-07 Confined filesystem.** Every path the software opens is canonicalised and asserted
to be inside one of exactly three roots: MC_CONTENT_DIR, MC_DATA_DIR, MC_ARTIFACT_DIR. The
check happens in one function, `mc_shell::fsroot::confine`, and no other code calls
`std::fs::File::open` directly.

**INV-08 Untrusted input.** Save files and content packs are untrusted. Their parsers never
panic, never allocate based on an unvalidated length field, and reject malformed input with
a typed error. Both are fuzz targets with committed corpora.

**INV-09 No network.** The software opens no socket, ever. mc_core and mc_data have no
networking dependency at all (absence beats prohibition), and an integration test asserts
zero socket syscalls across a full golden-tape replay.

**INV-10 No behavioural configuration.** No `cfg`, feature flag, or environment variable
alters simulation behaviour. `MC_HEADLESS` suppresses window creation in mc_shell and
nothing else. Configuration may differ between environments; behaviour may not.

**INV-11 Terrain gates the bestiary.** Enemy eligibility is a pure function
`eligible(region, flags) -> Vec<EnemyId>` with no other inputs. Content declares region
affinity and a flag gate per enemy; the function has no special cases and no exceptions.

**INV-12 No grind.** Experience awarded for a repeated encounter in a region-chapter decays
by 30 percent compounding from a finite spawn budget and floors at zero. This lives in
mc_core, not in content convention, so it cannot be authored around.

**INV-13 Save schema is versioned and forward-checked.** Every save carries a schema version
and a blake3 digest. Loading a newer schema is refused with a typed error, never guessed at.
Migrations are expand-migrate-contract and are proven by round-trip tests.

**INV-14 Phase-2 gating.** The final encounter's phase 2 cannot be resolved by damage. The
resolution requires `Command::NameYourself`, which the command bus rejects unless all three
dossier flags are set. This is core logic, not content, and is proven by LF-12.

## 6. Runtime flow

    frame:
      shell polls input        -> Vec<Command>
      shell accumulates time   -> n = floor(accum / (1/60))
      for _ in 0..n:
          core.apply_commands(&commands)   // commands applied at tick boundary only
          core.step()                      // exactly one tick
      shell reads StateView    -> draws (may interpolate visually, never mutates)

Commands are applied only at tick boundaries, never mid-tick, which is what makes the tape
format a simple list of (tick, command) pairs.

## 7. State management rules

Core state is one owned tree, `World`, with no interior mutability and no reference cycles.
Sub-systems are plain functions over `&mut World`. There is no ECS, no event bus, and no
observer pattern in mc_core, because all three introduce order-dependence that threatens
INV-01. Systems run in a fixed, declared order defined once in `mc_core::step::ORDER`.

## 8. Persistence boundaries

mc_core defines the save model and can serialize it, but cannot write it. mc_shell and
mc_tools perform the actual file I/O through the confinement function of INV-07. This is
why the save path can be tested headlessly without a filesystem in core tests.

## 9. Content pipeline

    /content/**.ron  --(mc_data::bake)-->  content.pack (+ .blake3)  --(mc_data::load)-->  ContentDb

The bake performs, in order: parse, schema validation, vocabulary check against the locked
identifier tables in .agent/specs/*, reference resolution, orphan detection, the
supernatural lint (INV-06 plus design law L1), region-affinity check, and content-addressing.
A release build refuses to read loose RON files; only content.pack is loadable.

## 10. Security boundaries

Two untrusted parsers (save, content pack), one confinement point (INV-07), zero network
(INV-09), zero secrets. See SECURITY.md.

## 11. Observability boundaries

Logging is a shell and tools concern. mc_core emits no logs; it returns structured events in
`StateView::events` and the shell decides whether to record them. This keeps core free of a
logging dependency and therefore of a clock. See OBSERVABILITY.md.

## 12. Forbidden moves

- Putting any behaviour in content that is not already a mechanism in core.
- Adding a system to `step::ORDER` without adding it to the determinism property test.
- Using `f32`/`f64`/`HashMap`/`HashSet`/`SystemTime`/`thread` anywhere in mc_core.
- Opening a file outside `fsroot::confine`.
- Adding a `cfg` that changes simulation behaviour.
- Making the shell authoritative about anything.
- Introducing a second communication channel between shell and core.
- "Temporarily" disabling a gate.

## 13. How to add things

**A feature:** find or write its spec in .agent/specs/; add the mechanism to mc_core with a
unit test and, if it is order-sensitive, an entry in the determinism property test; expose
it through `Command`/`StateView`; add its content schema to mc_data; author content; add a
live-fire assertion if it touches a core outcome.

**A dependency:** confirm nothing in the tree already does it; check the licence against the
allowed set; pin the exact version; add a comment naming the invariant it serves; commit
Cargo.lock; add it to ENVIRONMENT.md; run `sh scripts/dependency-audit.sh`.

**A content type:** add the RON schema in mc_data, add its identifier table to the relevant
spec (identifiers are locked vocabulary), extend the bake validator with its reference and
orphan rules, and add it to `mc_tools report`.

**A save field:** bump the schema version, write the expand-migrate-contract migration, add
a round-trip test and an insta snapshot of the wire bytes, and extend the save fuzz corpus.

## 14. Architecture review checklist

- [ ] Does mc_core still compile with `#![forbid(unsafe_code)]` and no I/O dependency?
- [ ] `cargo tree` layer check clean for all five crates?
- [ ] Any new `f32`, `f64`, `HashMap`, `HashSet` in mc_core? (must be zero)
- [ ] Any new system in `step::ORDER` without a determinism test?
- [ ] Any new `std::fs` call outside `fsroot::confine`?
- [ ] Any new `cfg` that changes behaviour rather than presentation?
- [ ] Does the golden tape still match its hash?
- [ ] Do all twelve live-fire proofs still pass?
