//! Verify Act I scene files parse correctly.
//! Each #[test] loads one scene RON file and validates it deserializes.

macro_rules! test_scene_file {
    ($file:literal, $name:ident, $expected_id:expr, $expected_act:expr) => {
        #[test]
        fn $name() {
            use mc_data::schema::scene::Scene;
            let ron = include_str!(concat!("../../../content/scenes/act1/", $file));
            let scene: Scene =
                ron::from_str(ron).expect(concat!("Scene ", $file, " should deserialize"));
            assert_eq!(scene.id, $expected_id, "{} id mismatch", $file);
            assert_eq!(scene.act, $expected_act, "{} act mismatch", $file);
            assert!(
                !scene.nodes.is_empty(),
                "{} must have at least one node",
                $file
            );
            println!(
                "PASS: {} (act={:?}, nodes={}, terminal={})",
                scene.id,
                scene.act,
                scene.nodes.len(),
                scene.terminal
            );
        }
    };
}

mod act1_scenes {
    use mc_data::schema::scene::Act;

    test_scene_file!(
        "scn_arrest.ron",
        test_arrest,
        "SCN_ARREST",
        Act::ActI_ARREST
    );
    test_scene_file!(
        "scn_faria_meeting.ron",
        test_faria_meeting,
        "SCN_FARIA_MEETING",
        Act::ActII_CHATEAU
    );
    test_scene_file!(
        "scn_treasure_reveal.ron",
        test_treasure_reveal,
        "SCN_TREASURE_REVEAL",
        Act::ActIII_TREASURE
    );
    test_scene_file!(
        "scn_escape.ron",
        test_escape,
        "SCN_ESCAPE",
        Act::ActIII_TREASURE
    );
    test_scene_file!(
        "scn_arrival.ron",
        test_arrival,
        "SCN_ARRIVAL",
        Act::ActIII_TREASURE
    );
    test_scene_file!(
        "scn_sindbad.ron",
        test_sindbad,
        "SCN_SINDBAD",
        Act::ActIV_TOUR
    );
}
