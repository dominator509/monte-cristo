# SPEC-010 -- Narrative structure and Confidences

## 1. What a Confidence is, and is not

A Confidence is an authored branching dialogue scene. It **is not** a combat system. It has
no hit points, no turn order, no action gauge, no resource meter, no win condition, and no
fail state (ADR-008, SPEC-001 section 12).

Its only outputs: set or clear flags; adjust `trust[char]` by a small integer; adjust `mask`;
grant or consume an item; select the next node; and, at exit, select which variant a later
scene will use.

If an implementation gives a Confidence a battle-shaped interface, it violates this spec and
fails review. This is stated in the spec and not only in the ADR because the spec is what the
executor reads.

## 2. Count and distribution

45 Confidences, approximately 210,000 words.

| Act | Confidences | Notable scenes |
|---|---|---|
| ACT_I_MARSEILLE | 5 | Morrel's promise; the betrothal; the arbour (optional witness); Villefort's interrogation |
| ACT_II_IF | 7 | Faria's arrival; the reconstruction of the conspiracy; Faria's death |
| ACT_III_SEA | 4 | Jacopo's rations; the archive purchase |
| ACT_IV_TOUR | 9 | Caderousse and the diamond; Morrel's rescue; Haydee's sale; Bertuccio's confession |
| ACT_V_ROME | 5 | Peppino's pardon; Vampa's terms; Albert's invitation |
| ACT_VI_PARIS | 12 | Beauchamp; Auteuil; the Chamber of Peers (played as Haydee); Mercedes' plea; Benedetto's trial; Edouard |
| ACT_VII_EPILOGUE | 3 | Cell 34; Maximilien's ordeal; the sail |

## 3. Trust thresholds

Four characters carry a decisive threshold. Below it, the alternate scene variant plays.

| Character | Threshold | Decides |
|---|---|---|
| CHR_MERCEDES | 20 | whether `MERCEDES_RECOGNITION` is set, which is one of the three final-boss gates |
| CHR_ALBERT | 15 | whether `MORCERF_ALBERT_WITHDRAWN` is set at the Opera |
| CHR_HAYDEE | 25 | whether she testifies at the Chamber of Peers |
| CHR_MAXIMILIEN | 20 | the variant of the grotto scene in the epilogue |

Trust is never displayed as a number. The player learns it from how people speak to them,
which is the correct instrument for this novel.

## 4. Mask

One global `i16`, 0 to 100, starting at 100. It drops only at scripted moments -- being seen
without a persona, spending catastrophically in public, an unmasked act of violence. Its only
mechanical effects are persona-to-map access and which of four framing-text variants the
epilogue uses.

## 5. Personas

`PSN_BUSONI`, `PSN_WILMORE`, `PSN_SINBAD`, `PSN_COUNT`, `PSN_EDMOND`.

Each gates map and scene access. `PSN_EDMOND` becomes available exactly twice, both in the
final act; one of those uses is `Command::NameYourself` at the final boss.

## 6. Key items are not ammunition

Documents (`ITM_REGISTER_PAGE`, `ITM_YANINA_DEED`, `ITM_AUTEUIL_BOX`,
`ITM_NOIRTIER_CONFESSION`, `ITM_BENEDETTO_BAPTISM`, `ITM_SPADA_CODICIL`) unlock scenes,
regions, and flags. They are consumed by `on_exit` effects. They deal no damage, have no
combat use, and appear in the key-item menu rather than in battle inventory.

## 7. The Edouard scene

`SCN_EDOUARD`, act ACT_VI_PARIS. `ITM_ANTIDOTE_EDOUARD` is present in the player's inventory
and has `usable_in: []`. The item is real and visible; it cannot be used. `content_invariants.rs`
asserts both facts, so that a future contributor cannot "fix" what looks like a bug.

## 8. The final confrontation

`SCN_FERNAND_ARRIVES` leads into the three-phase encounter (SPEC-001 section 15). Phase 2's
resolving command is `NameYourself`; the line it produces is the novel's, and the encounter
ends there. Phase 3 is `SCN_MORCERF_PURSUIT`, a scripted run with no combat, ending at a lit
window going dark. The player does not enter the room.

## 9. The epilogue

`SCN_CELL_34` (no enemies; the R14 trash mobs are already trivial by design),
`SCN_MAXIMILIEN_ORDEAL`, `SCN_THE_SAIL`. Exactly one scene in the entire content tree carries
`terminal: true`, and it is `SCN_THE_SAIL`. The Mercy Ledger -- a hidden count of the Count's
graces (Morrel, Peppino, Valentine, Danglars, Albert, Haydee) -- selects among four framing-
text variants and changes nothing that happens.

## 10. Validation

| Behaviour | Test |
|---|---|
| a Confidence has no combat fields | `crates/mc_data/tests/scene_schema.rs` |
| flags and trust set correctly on exit | `crates/mc_core/tests/confidence_flags.rs` |
| a later scene resolves to its variant | `crates/mc_core/tests/confidence_flags.rs` (LF-06) |
| exactly one terminal scene | `crates/mc_data/tests/content_invariants.rs` |
| Edouard antidote unusable | `crates/mc_data/tests/content_invariants.rs` |
| reserved flags unused | `crates/mc_data/tests/content_invariants.rs` |
| final gates require all three flags | `crates/mc_core/tests/final_encounter.rs` (LF-12) |
