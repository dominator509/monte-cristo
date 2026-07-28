#[test]
fn full_table_with_gates() {
    use mc_data::schema::spawn_table::SpawnTable;
    let ron = r#"
        SpawnTable(
            region: "R01",
            chapter_stage: 1,
            pool: 20,
            entries: [
                (enemy: "ENM_DOCK_RAT", weight: 25, gate: Always),
                (enemy: "ENM_SOLDIER", weight: 15, gate: Not("FLG_ARRESTED")),
                (enemy: "ENM_GENDARME", weight: 10, gate: Not("FLG_ARRESTED")),
            ],
        )
    "#;
    let table: SpawnTable = ron::from_str(ron).expect("SpawnTable with gates should deserialize");
    assert_eq!(table.entries.len(), 3);
    let soldier = &table.entries[1];
    assert_eq!(soldier.enemy, "ENM_SOLDIER");
    let gendarme = &table.entries[2];
    assert_eq!(gendarme.enemy, "ENM_GENDARME");
}
