# SPEC-003 -- Command bus, state view, and tape format

The only channel between the shell and the core (INV-04). Also the reason every gameplay
claim in this project is testable.

## 1. Command

    enum Command {
      // field
      Move(Dir), Interact, OpenMenu, CloseMenu,
      // battle
      SelectAction(ActorId, Action), ConfirmTarget(TargetId), CancelSelection,
      SetWaitMode(bool),
      // scene
      SceneAdvance, SceneChoose(ChoiceIdx),
      // act II calendar
      CalendarAct(CalendarAction),
      // act VI
      SeasonAct(CampaignId, CampaignAction),
      // personas and travel
      SwapPersona(PersonaId), FastTravel(RegionId),
      // the final encounter
      NameYourself,
      // meta
      Save(SaveSlot), Load(SaveSlot),
    }

`apply_commands` validates each command against the current state and **rejects** invalid
ones with a `Rejection` in the returned event list. It never asserts and never panics, which
is what makes replaying a malformed tape safe (SECURITY.md section 12).

`NameYourself` is rejected unless the final encounter is in Phase2 **and**
`MORCERF_YANINA_DOSSIER`, `MORCERF_ALBERT_WITHDRAWN`, and `MERCEDES_RECOGNITION` are all set
(INV-14). This gameplay rule lives at the same boundary as the safety rules, deliberately, so
that a crafted tape cannot bypass it.

## 2. StateView

A read-only projection produced once per frame:

    struct StateView<'a> {
      tick: u64,
      act: Act, region: RegionId, map: MapId,
      party: &'a Party,
      battle: Option<BattleView<'a>>,
      scene: Option<SceneView<'a>>,
      hud: HudView,
      events: &'a [CoreEvent],
      state_hash: Option<[u8; 32]>,   // Some only at checkpoint ticks
    }

The shell may read it and may not mutate through it. There is no callback into the shell and
no second channel.

## 3. The tick contract (INV-05)

Commands are applied only at tick boundaries. One tick is exactly 1/60 second of simulated
time. The shell accumulates real elapsed time and calls `step` the correct number of times; it
may drop frames but may never drop, subdivide, or double a tick. Accumulated remainder is
carried, never discarded.

## 4. Tape format

    Tape {
      magic: b"MCTAPE01",
      seed: u128,
      content_digest: [u8; 32],
      start: TapeStart::NewGame | TapeStart::FromSave(Save),
      entries: Vec<(tick: u64, command: Command)>,   // strictly ascending by tick
      checkpoints: Vec<(tick: u64, state_hash: [u8; 32])>,
      final_hash: [u8; 32],
    }

Rules: entries strictly ascending; a tape whose `content_digest` does not match the loaded
pack is refused (a content change invalidates a tape, which is correct and is why the
CHANGELOG Determinism subsection exists); checkpoints let a divergence be bisected to a tick
range instead of hunting through 8.6 million ticks.

The tape parser is untrusted input and is validated as strictly as a save (INV-08).

## 5. Replay contract

    replay(tape) -> ReplayResult { final_hash, first_divergence: Option<(tick, expected, got)> }

`--assert-hash` exits nonzero on any divergence and prints the first diverging checkpoint. It
never "re-records" anything; re-recording is a deliberate human act, documented in the
CHANGELOG (CONTRIBUTING.md section 4).

## 6. Error contract

Every public function returns `Result<T, E>` with a `thiserror` enum. No public function
panics. `apply_commands` returns rejections as events rather than errors, because a rejected
command is normal gameplay (pressing a button that does nothing), not a fault.

## 7. Versioning of the contract

`Command` and `StateView` are internal, so there is no external compatibility promise --
except through tapes. Adding a `Command` variant is backward compatible for existing tapes.
Removing or reordering variants breaks them, so variants are append-only and the discriminant
is explicit.

## 8. Validation

| Behaviour | Test |
|---|---|
| invalid command is rejected, not panicked | `crates/mc_core/tests/command_validation.rs` |
| NameYourself gated on three flags | `crates/mc_core/tests/final_encounter.rs` |
| tick boundary application | `crates/mc_core/tests/tick_contract.rs` |
| tape round-trips | `crates/mc_tape/tests/tape_roundtrip.rs` |
| malformed tape is a typed error | `crates/mc_tape/tests/forced_failures.rs` |
| replay is identical twice | `crates/mc_tape/tests/e2e_determinism.rs` |
| checkpoint bisection localises divergence | `crates/mc_tape/tests/divergence_report.rs` |
