NODE-META-BEGIN
ID: EP-005
DEPS: EP-004
MAX_ATTEMPTS_PER_MILESTONE: 6
VERIFY: sh scripts/test-e2e.sh
VERIFY_SENTINEL: e2e tests: ok
GREEN_TAG: green/EP-005
NODE-META-END

# EP-005 -- Presentation shell (mc_shell)

## 1. Purpose / Big Picture

Draw the game. macroquad, a 256x224 internal render target integer-scaled to the window, two
scrolling background layers plus an overlay, an affine layer for the Mode-7-equivalent
moments, act-locked 15-bit palettes, tilemaps, sprites, the battle interface, menus, the
Confidence presentation, audio, remappable input, and the full accessibility surface
including the content advisory screen.

The shell owns nothing authoritative. It reads `StateView` and produces `Command` (INV-04).
A muted run and a loud run produce identical state hashes, and there is a test that says so.

## 2. Scope

`crates/mc_shell` in full, plus its tests. Everything visible and audible.

## 3. Non-goals

No simulation logic of any kind. If you are tempted to compute a game outcome in the shell,
that outcome belongs in mc_core. No parser hardening or filesystem confinement -- that is
EP-006, running in parallel; until it lands, mc_shell uses a single provisional path helper
that EP-006 replaces with `fsroot::confine`. No golden full-campaign tape (EP-007).

## 4. Context and Orientation

SPEC-004 is authoritative. EP-005 and EP-006 both depend only on EP-004, so another agent may
hold EP-006 concurrently. Do not touch `crates/mc_shell/src/fsroot.rs` -- it belongs to
EP-006 and editing it will collide.

`MC_HEADLESS=1` must suppress window and audio device creation and nothing else (INV-10).
Every scripted gate sets it, which is what allows live-fire to drive the real entry point.

## 5. Files to Read First

- .agent/specs/SPEC-004-ui-ux-behavior.md
- .agent/specs/SPEC-003-api-contracts.md sections 2, 3
- ARCHITECTURE.md sections 5 (INV-04, INV-05, INV-10), 6
- docs/GAME_DESIGN.md section 7

## 6. Expected Changed Files

- crates/mc_shell/src/main.rs
- crates/mc_shell/src/app.rs
- crates/mc_shell/src/config.rs
- crates/mc_shell/src/render/mod.rs
- crates/mc_shell/src/render/target.rs
- crates/mc_shell/src/render/palette.rs
- crates/mc_shell/src/render/tilemap.rs
- crates/mc_shell/src/render/sprite.rs
- crates/mc_shell/src/render/affine.rs
- crates/mc_shell/src/ui/mod.rs
- crates/mc_shell/src/ui/battle.rs
- crates/mc_shell/src/ui/menu.rs
- crates/mc_shell/src/ui/confidence.rs
- crates/mc_shell/src/ui/advisory.rs
- crates/mc_shell/src/ui/text.rs
- crates/mc_shell/src/input/mod.rs
- crates/mc_shell/src/input/remap.rs
- crates/mc_shell/src/audio.rs
- crates/mc_shell/src/a11y.rs
- crates/mc_shell/assets/fonts/
- crates/mc_shell/tests/input_remap.rs
- crates/mc_shell/tests/motion_zero.rs
- crates/mc_shell/tests/advisory_screen.rs
- crates/mc_shell/tests/text_speed.rs
- crates/mc_shell/tests/palette_independence.rs
- crates/mc_shell/tests/font_metrics.rs
- crates/mc_shell/tests/caption_coverage.rs
- crates/mc_shell/tests/audio_hash_stability.rs
- crates/mc_data/tests/glyph_parity.rs
- crates/mc_shell/Cargo.toml

## 7. Interfaces and Contracts

Render target, palettes, sprite sizes, screens, input defaults, and every accessibility row
come from SPEC-004 and are not varied here. `StateView` is read-only; there is no path by
which the shell writes core state except by submitting a `Command`.

## 8. Milestones

### M1: Window, render target, and headless mode
GOAL: The binary opens a 256x224 integer-scaled window, and does not when MC_HEADLESS=1.
READ: SPEC-004 sections 1, 11
CHANGE: crates/mc_shell/src/main.rs, crates/mc_shell/src/app.rs,
  crates/mc_shell/src/config.rs, crates/mc_shell/src/render/target.rs,
  crates/mc_shell/Cargo.toml
CONTENT: a macroquad entry point rendering into a 256x224 offscreen target, blitted with
  nearest-neighbour integer scaling (1x through 6x) and letterboxing. Non-integer scaling is
  not offered. `MC_HEADLESS=1` skips window and audio device creation and runs the same frame
  loop headlessly. Configuration is validated once at startup and never partially applied.
RUN:
  cargo build --locked -p mc_shell
  MC_HEADLESS=1 cargo run --locked -p mc_shell -- --version
EXPECT: build succeeds; the version line prints and the process exits 0 without opening a window
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-005 MILESTONE_PASS "M1 shell builds, headless clean"
FALLBACK: if macroquad cannot hold the frame budget on the reference machine (ASSUMPTIONS
  A-01), switch the blit path to direct wgpu as recorded in ADR-004 and note it in the
  Decision Log. The shell owns nothing authoritative, so the swap is contained.
COMMIT: git add -A && git commit -m "[EP-005][M1] add shell entry point and 256x224 render target"

### M2: Palettes, tilemaps, sprites, affine layer
GOAL: A map draws with its act-locked palette and the affine layer works.
READ: SPEC-004 sections 2, 3, docs/GAME_DESIGN.md section 7
CHANGE: crates/mc_shell/src/render/palette.rs, crates/mc_shell/src/render/tilemap.rs,
  crates/mc_shell/src/render/sprite.rs, crates/mc_shell/src/render/affine.rs,
  crates/mc_shell/src/render/mod.rs, crates/mc_shell/tests/palette_independence.rs
CONTENT: 15-bit colour with the seven act palettes of SPEC-004 section 2; vertical gradient
  bands on skies; two scrolling background layers plus one overlay; 16x16 tiles; 24x32 field
  and 48x64 battle sprites; an affine layer for the Pharaon, the Mediterranean map, the
  grotto reveal, and the If flyover. The high-contrast interface palette is selected
  independently and never alters scene palettes, which the test asserts.
RUN: cargo test --locked -p mc_shell --test palette_independence
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-005 MILESTONE_PASS "M2 palettes and layers"
FALLBACK: if the affine layer cannot be implemented within budget, ship the four affine
  moments as pre-rendered scrolling sequences, record an ADR, and keep the layer interface so
  it can be filled later. A reduced-but-real effect is a legitimate fallback.
COMMIT: git add -A && git commit -m "[EP-005][M2] add palette system, tilemaps, sprites, affine layer"

### M3: Input and remapping
GOAL: Every action is remappable for keyboard and gamepad and the remap survives restart.
READ: SPEC-004 section 7
CHANGE: crates/mc_shell/src/input/mod.rs, crates/mc_shell/src/input/remap.rs,
  crates/mc_shell/tests/input_remap.rs
CONTENT: the SPEC-004 section 7 defaults; no action requiring more than two simultaneous
  inputs; translation to `Command` at the tick boundary; held-input repeat rate defined in
  core, not in the shell, so frame rate cannot affect a replay. The remap round-trips through
  the settings file.
RUN: cargo test --locked -p mc_shell --test input_remap
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-005 MILESTONE_PASS "M3 input remap survives restart"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-005][M3] add remappable input surviving restart"

### M4: Battle interface and menus
GOAL: A battle is playable and every menu screen exists.
READ: SPEC-004 sections 4, 5
CHANGE: crates/mc_shell/src/ui/mod.rs, crates/mc_shell/src/ui/battle.rs,
  crates/mc_shell/src/ui/menu.rs, crates/mc_shell/src/ui/text.rs
CONTENT: ATB gauges, target selection, tech and item lists, dual and triple tech offers when
  all participants are ready. Menus: party, curriculum, inventory, Web of Debt, ledger,
  settings, plus the Act II calendar and the Act VI season clock screens. Loading, empty, and
  error states exactly as SPEC-004 section 5, including that a save write failure states
  plainly that the previous save is intact.
RUN: cargo build --locked -p mc_shell && sh scripts/lint.sh
EXPECT: build succeeds; `lint: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-005 MILESTONE_PASS "M4 battle ui and menus"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-005][M4] add battle interface and menu screens"

### M5: Confidence presentation
GOAL: A Confidence renders as a scene, with no combat interface anywhere in it.
READ: SPEC-010 sections 1, 2, SPEC-004 section 4
CHANGE: crates/mc_shell/src/ui/confidence.rs
CONTENT: 64x80 portraits with eight expression frames, a dialogue box, and a choice list.
  There is no gauge, no meter, no hit point display, and no turn indicator, because the
  underlying type cannot express them (ADR-008). Text speed has four settings including
  instant, and hold-to-fast-forward never skips an unread line.
RUN:
  cargo test --locked -p mc_shell --test text_speed
  grep -cE 'hp|gauge|atb|turn_order' crates/mc_shell/src/ui/confidence.rs
EXPECT: test passes; the grep prints `0`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-005 MILESTONE_PASS "M5 confidence ui, zero combat affordances"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-005][M5] add Confidence scene presentation"

### M6: Audio
GOAL: Sound plays and cannot affect a replay hash.
READ: SPEC-004 section 9
CHANGE: crates/mc_shell/src/audio.rs, crates/mc_shell/tests/audio_hash_stability.rs
CONTENT: eight-channel sample-based playback, act-scoped music with scene override. Audio
  reads `StateView` and never returns anything to core. The test replays a tape muted and
  unmuted and asserts identical hashes.
RUN: cargo test --locked -p mc_shell --test audio_hash_stability
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-005 MILESTONE_PASS "M6 audio does not affect hash"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-005][M6] add audio with proven hash stability"

### M7: Accessibility surface
GOAL: Every row of SPEC-004 section 8 is implemented and tested.
READ: SPEC-004 section 8, PRODUCTION_READINESS.md accessibility block
CHANGE: crates/mc_shell/src/a11y.rs, crates/mc_shell/assets/fonts/,
  crates/mc_shell/tests/motion_zero.rs, crates/mc_shell/tests/font_metrics.rs,
  crates/mc_shell/tests/caption_coverage.rs, crates/mc_data/tests/glyph_parity.rs
CONTENT: shake and flash sliders with a true zero that produces zero camera offset and zero
  full-screen luminance delta; a dyslexia-friendly bitmap font at identical metrics so no
  layout reflows; captions for every information-bearing audio cue; and a glyph-parity check
  asserting every status, damage type, party indicator, and menu state carries a distinct
  non-colour glyph or shape.
RUN:
  cargo test --locked -p mc_shell --test motion_zero --test font_metrics --test caption_coverage
  cargo test --locked -p mc_data --test glyph_parity
EXPECT: all pass
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-005 MILESTONE_PASS "M7 accessibility surface complete"
FALLBACK: if a licence-compatible dyslexia-friendly bitmap font at the exact metrics cannot
  be sourced, generate one from the default font's metrics with widened apertures and record
  an ADR naming the provenance. Never ship a font whose licence is outside the allowlist.
COMMIT: git add -A && git commit -m "[EP-005][M7] add accessibility surface with tests"

### M8: Content advisory
GOAL: The advisory appears before the title screen on first run and is remembered.
READ: SPEC-004 section 6, ASSUMPTIONS.md A-12
CHANGE: crates/mc_shell/src/ui/advisory.rs, crates/mc_shell/tests/advisory_screen.rs
CONTENT: a dismissible screen shown before the title on first run, stating that the game
  depicts suicide and the death of a child faithfully to the source novel, re-readable from
  settings, and remembered in the settings file. The test asserts it appears on a fresh
  profile, does not appear on the second run, and is reachable from settings.
RUN: cargo test --locked -p mc_shell --test advisory_screen
EXPECT: test passes
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-005 MILESTONE_PASS "M8 advisory screen verified"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-005][M8] add content advisory screen"

### M9: Node verification
GOAL: The shell drives the real entry point end to end.
READ: COMMANDS.md section 3
CHANGE: (none)
CONTENT: none.
RUN:
  sh scripts/test-e2e.sh
  sh scripts/lint.sh
EXPECT: `e2e tests: ok` then `lint: ok`
EVIDENCE: sh scripts/ledger.sh append <AGENT_ID> EP-005 MILESTONE_PASS "M9 e2e tests: ok"
FALLBACK: none needed.
COMMIT: git add -A && git commit -m "[EP-005][M9] verify presentation shell"

## 9. Validation and Acceptance

| Criterion | Command | Expected |
|---|---|---|
| Headless suppresses window only | `MC_HEADLESS=1 cargo run --locked -p mc_shell -- --version` | exits 0, no window |
| Palette independence | `cargo test --locked -p mc_shell --test palette_independence` | pass |
| Remap survives restart | `cargo test --locked -p mc_shell --test input_remap` | pass |
| No combat affordance in Confidences | `grep -cE 'hp\|gauge\|atb\|turn_order' crates/mc_shell/src/ui/confidence.rs` | `0` |
| Audio cannot change a hash | `cargo test --locked -p mc_shell --test audio_hash_stability` | pass |
| Motion zero is truly zero | `cargo test --locked -p mc_shell --test motion_zero` | pass |
| Glyph parity | `cargo test --locked -p mc_data --test glyph_parity` | pass |
| Captions complete | `cargo test --locked -p mc_shell --test caption_coverage` | pass |
| Advisory shown and remembered | `cargo test --locked -p mc_shell --test advisory_screen` | pass |
| Node gate | `sh scripts/test-e2e.sh` | `e2e tests: ok` |

## 10. Idempotence and Recovery

Rendering work is regenerable; no milestone here writes irreplaceable state. To re-enter
cold: read Progress, find the first unchecked milestone, re-run the previous milestone's RUN,
continue. If EP-006 is running concurrently, do not touch `crates/mc_shell/src/fsroot.rs` or
any file in EP-006's Expected Changed Files list; if you find one dirty, leave it and report
it rather than reverting another node's work.

## 11. Progress

- [ ] M1 window, render target, headless mode
- [ ] M2 palettes, tilemaps, sprites, affine layer
- [ ] M3 input and remapping
- [ ] M4 battle interface and menus
- [ ] M5 Confidence presentation
- [ ] M6 audio
- [ ] M7 accessibility surface
- [ ] M8 content advisory
- [ ] M9 node verification

## 12. Surprises and Discoveries

<empty>

## 13. Decision Log

<empty>

## 14. Outcomes and Retrospective

<empty>
