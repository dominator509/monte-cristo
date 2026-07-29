pub mod encounter;
pub mod enemy;
pub mod item;
pub mod region;
pub mod scene;
pub mod spawn_table;

#[cfg(test)]
mod tests {
    use crate::schema::enemy::Enemy;
    use crate::schema::item::Item;
    use crate::schema::region::Region;
    use crate::schema::scene::Scene;
    use crate::schema::spawn_table::SpawnTable;

    #[test]
    fn enemy_deserialize_roundtrip() {
        let ron = r#"
            Enemy(
                id: "ENM_CELL_RAT",
                name_key: "enemy.cell_rat.name",
                family: VERMIN,
                region_affinity: ["R03"],
                gate: Always,
                stats: Stats(hp: 14, atk: 6, def: 3, spd: 22),
                resist: [],
                abilities: ["ABL_BITE"],
                loot: [],
                xp: 3,
                tier: 1,
                sprite: "spr/enemy/cell_rat",
            )
        "#;
        let enemy: Enemy = ron::from_str(ron).expect("Enemy should deserialize");
        assert_eq!(enemy.id, "ENM_CELL_RAT");
        assert_eq!(enemy.stats.hp, 14);
    }

    #[test]
    fn region_deserialize() {
        let ron = r#"
            Region(
                id: "R03",
                name_key: "region.r03.name",
                description_key: "region.r03.desc",
                tier: 1,
                connections: ["R04"],
                locked: false,
                gate: Always,
            )
        "#;
        let region: Region = ron::from_str(ron).expect("Region should deserialize");
        assert_eq!(region.id, "R03");
    }

    #[test]
    fn scene_deserialize() {
        let ron = r#"
            Scene(
                id: "SCN_TEST",
                act: ActI_ARREST,
                participants: ["CHR_EDMOND"],
                nodes: [
                    Node(
                        id: "n0",
                        text_key: "scene.test.n0",
                        choices: [
                            Choice(text_key: "test.choice.1", to: "n1"),
                        ],
                    ),
                ],
                terminal: false,
            )
        "#;
        let scene: Scene = ron::from_str(ron).expect("Scene should deserialize");
        assert_eq!(scene.id, "SCN_TEST");
        assert_eq!(scene.nodes.len(), 1);
    }

    #[test]
    fn spawn_table_deserialize() {
        let ron = r#"
            SpawnTable(
                region: "R03",
                chapter_stage: 1,
                pool: 26,
                entries: [
                    (enemy: "ENM_CELL_RAT", weight: 30, gate: Always),
                ],
            )
        "#;
        let table: SpawnTable = ron::from_str(ron).expect("SpawnTable should deserialize");
        assert_eq!(table.region, "R03");
        assert_eq!(table.entries.len(), 1);
    }

    #[test]
    fn item_deserialize() {
        let ron = r#"
            Item(
                id: "ITM_POTION",
                name_key: "item.potion.name",
                description_key: "item.potion.desc",
                item_type: Consumable,
                heal_hp: Some(50),
                effect: None,
                usable_in: [],
                value: 10,
            )
        "#;
        let item: Item = ron::from_str(ron).expect("Item should deserialize");
        assert_eq!(item.id, "ITM_POTION");
        assert_eq!(item.heal_hp, Some(50));
    }
}
