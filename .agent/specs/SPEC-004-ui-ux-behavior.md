# SPEC-004 -- Presentation, input, and accessibility (mc_shell)

The shell draws `StateView` and produces `Command`. It owns nothing authoritative (INV-04).

## 1. Render target

Internal 256x224, 16x16 tiles, integer-scaled to the window (1x through 6x, letterboxed).
Non-integer scaling is not offered, because it destroys the pixel grid this art depends on.

Layers: two scrolling background layers plus one overlay, matching the era's constraint
deliberately. An affine layer provides the Mode-7-equivalent effects: the *Pharaon* under
sail, the Mediterranean map, the Monte Cristo grotto reveal, and the Chateau d'If flyover.

## 2. Palette

15-bit colour, act-locked palettes selected by `Act`:

| Act | Palette |
|---|---|
| ACT_I_MARSEILLE | cerulean and sand |
| ACT_II_IF | six greys and one red |
| ACT_III_SEA | sea blue and canvas |
| ACT_IV_TOUR | dust, olive, and lamp |
| ACT_V_ROME | ochre and torch |
| ACT_VI_PARIS | black, gold, wine, gaslight |
| ACT_VII_EPILOGUE | white marble |

Vertical gradient bands on every sky, emulating per-scanline colour change. A high-contrast
interface palette is selectable independently and never alters scene palettes.

## 3. Sprites

24x32 field, 48x64 battle, 64x80 Confidence portraits with eight expression frames each.
22 characters carry portrait sets.

## 4. Screens

Title (preceded by the content advisory), file select, field, battle overlay, Confidence,
menu (party, curriculum, inventory, Web of Debt, ledger, settings), calendar (Act II), season
clock (Act VI), map, epilogue.

## 5. Loading, empty, and error states

- Content load shows a progress bar with the current stage name; a digest failure shows the
  expected and actual digests and exits nonzero.
- An empty save slot reads "empty" and is selectable for a new game only.
- A save write failure shows the typed error and states plainly that the previous save is
  intact.
- A missing content pack shows the expected path and the command to rebake.

## 6. Content advisory

Shown before the title screen on first run, dismissible, and remembered in settings. It
states that the game depicts suicide and the death of a child, faithfully to the source
novel. It can be re-read from the settings menu. This is an acceptance criterion (EP-005 M9),
not a courtesy.

## 7. Input

Fully remappable for keyboard and gamepad. The remap is stored in settings and survives
restart. No action requires more than two simultaneous inputs. Default keyboard: arrows or
WASD to move, Z or Enter to confirm, X or Escape to cancel, C for menu, Shift to run, Tab to
toggle Wait mode.

Input is translated to `Command` at the tick boundary. Held inputs generate repeat commands
at a fixed rate defined in core, not in the shell, so that a replay is unaffected by frame
rate.

## 8. Accessibility (all are ship criteria)

| Requirement | Behaviour | Test |
|---|---|---|
| Remappable input surviving restart | settings round-trip | `input_remap.rs` |
| No colour-only information | every status, damage type, party indicator, and menu state carries a distinct glyph or shape | `glyph_parity.rs` (asserts every status has a unique non-colour glyph) |
| Text speed including instant | four speeds; hold to fast-forward never skips an unread line | `text_speed.rs` |
| High-contrast interface palette | independent of scene palette | `palette_independence.rs` |
| Dyslexia-friendly font | same metrics, so no reflow | `font_metrics.rs` |
| Shake and flash intensity with true zero | zero camera offset and zero full-screen luminance delta | `motion_zero.rs` |
| Captions for informational audio | every cue in the caption table | `caption_coverage.rs` |
| ATB Wait mode | gauges halt whenever a menu is open; toggleable mid-battle | `atb_wait_mode.rs` (core) |

## 9. Audio

Eight-channel sample-based playback, 34 tracks. Music is act-scoped and scene-overridable.
Audio never affects state and never feeds back into core (INV-04); a muted run and a loud run
produce identical hashes, and there is a test that asserts it.

## 10. Frame loop

    poll input -> commands
    accumulate dt
    while accum >= 1/60 { core.apply_commands(); core.step(); accum -= 1/60 }
    view = core.state_view()
    draw(view, interpolation_alpha)

Visual interpolation uses floating point and never returns to core (INV-02).

## 11. Headless mode

`MC_HEADLESS=1` suppresses window and audio device creation and nothing else (INV-10). The
same code path runs the same simulation; only presentation is skipped. This is what lets
live-fire drive the real entry point.

## 12. Validation

Every row in section 8, plus: `advisory_screen.rs`, `no_socket.rs`, `log_redaction.rs`,
`fsroot_confine.rs`, and the e2e entry-point tests in `crates/mc_tape/tests/`.
