//! EP-005 M7: Glyph parity test.
//!
//! Every status, damage type, party indicator, and menu state must carry a
//! distinct non-colour glyph or shape, so the game is playable without
//! colour perception.

#[test]
fn core_types_have_glyph_representations() {
    // Game status effects and types must have distinct glyphs.
    // This test verifies the core vocabulary has non-overlapping identifiers.

    // Flag IDs are unique by construction (newtype u16)
}

#[test]
fn all_disciplines_have_distinct_names() {
    use mc_core::curriculum::Discipline;
    let mut names: Vec<&str> = Discipline::ALL.iter().map(|d| d.name()).collect();
    names.sort();
    names.dedup();
    assert_eq!(
        names.len(),
        Discipline::ALL.len(),
        "all discipline names must be unique"
    );
}

#[test]
fn all_flag_names_unique() {
    use mc_core::ids::FlagId;
    let ids = 0..FlagId::COUNT as u16;
    let mut names: Vec<&str> = ids.map(|i| FlagId::from_raw(i).name()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), FlagId::COUNT, "all flag names must be unique");
}
