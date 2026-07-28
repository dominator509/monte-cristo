#[cfg(test)]
mod test_spawn_tables {
    use mc_data::schema::spawn_table::SpawnTable;

    #[test]
    fn r01_s1_deserializes() {
        let ron = include_str!("../../content/spawn_tables/R01-s1.ron");
        let table: SpawnTable = ron::from_str(ron).expect("R01-s1 should deserialize");
        assert_eq!(table.region, "R01");
        assert_eq!(table.chapter_stage, 1);
        assert_eq!(table.pool, 20);
        assert_eq!(table.entries.len(), 20);
        // Verify specific enemies
        assert!(table.entries.iter().any(|e| e.enemy == "ENM_DOCK_RAT"));
        assert!(table.entries.iter().any(|e| e.enemy == "ENM_STREET_BANDIT"));
    }

    #[test]
    fn r03_s1_deserializes() {
        let ron = include_str!("../../content/spawn_tables/R03-s1.ron");
        let table: SpawnTable = ron::from_str(ron).expect("R03-s1 should deserialize");
        assert_eq!(table.region, "R03");
        assert_eq!(table.chapter_stage, 1);
        assert_eq!(table.pool, 26);
        assert_eq!(table.entries.len(), 26);
        // Verify enemies with gates
        let cell_rat = table.entries.iter().find(|e| e.enemy == "ENM_CELL_RAT").expect("CELL_RAT should exist");
        assert_eq!(cell_rat.weight, 15);
        // R03 should have many guard-type enemies
        let guard_count = table.entries.iter().filter(|e| e.enemy.contains("GUARD") || e.enemy.contains("SENTRY") || e.enemy.contains("PATROL")).count();
        assert!(guard_count >= 6, "R03 should have at least 6 guard/sentry/patrol entries, got {}", guard_count);
    }
}
