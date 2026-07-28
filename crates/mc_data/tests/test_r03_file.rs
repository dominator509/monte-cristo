/// Test that the R03-s1.ron file deserializes correctly
#[test]
fn r03_s1_from_file() {
    use mc_data::schema::spawn_table::SpawnTable;
    let ron = include_str!("../../../content/spawn_tables/R03-s1.ron");
    let table: SpawnTable = ron::from_str(ron).expect("R03-s1 should deserialize");
    assert_eq!(table.region, "R03");
    assert_eq!(table.chapter_stage, 1);
    assert_eq!(table.pool, 26);
    assert_eq!(table.entries.len(), 26);
}
