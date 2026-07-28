/// Test that the R01-s1.ron file deserializes correctly
#[test]
fn r01_s1_from_file() {
    use mc_data::schema::spawn_table::SpawnTable;
    let ron = include_str!("../../../content/spawn_tables/R01-s1.ron");
    let table: SpawnTable = ron::from_str(ron).expect("R01-s1 should deserialize");
    assert_eq!(table.region, "R01");
    assert_eq!(table.chapter_stage, 1);
    assert_eq!(table.pool, 20);
    assert_eq!(table.entries.len(), 20);
    // Check each entry has the right structure
    for entry in &table.entries {
        assert!(!entry.enemy.is_empty());
        assert!(entry.weight > 0);
    }
}
