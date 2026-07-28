//! Quick integration test for content file parsing.

#[test]
fn parse_all_content_files() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("content");

    // Parse bestiary files
    let bestiary_dir = base.join("bestiary");
    for entry in std::fs::read_dir(&bestiary_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map_or(false, |e| e == "ron") {
            let content = std::fs::read_to_string(&path).unwrap();
            let enemy: mc_data::schema::enemy::Enemy = ron::from_str(&content)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
            println!("OK enemy: {}", enemy.id);
        }
    }

    // Parse regions
    let regions_dir = base.join("regions");
    for entry in std::fs::read_dir(&regions_dir).unwrap() {
        let path = entry.unwrap().path();
        let content = std::fs::read_to_string(&path).unwrap();
        let _region: mc_data::schema::region::Region = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
    }

    // Parse scenes
    let scenes_dir = base.join("scenes").join("act1");
    for entry in std::fs::read_dir(&scenes_dir).unwrap() {
        let path = entry.unwrap().path();
        let content = std::fs::read_to_string(&path).unwrap();
        let _scene: mc_data::schema::scene::Scene = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
    }

    // Parse spawn tables
    let spawn_dir = base.join("spawn_tables");
    for entry in std::fs::read_dir(&spawn_dir).unwrap() {
        let path = entry.unwrap().path();
        let content = std::fs::read_to_string(&path).unwrap();
        let _table: mc_data::schema::spawn_table::SpawnTable = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
    }
}
