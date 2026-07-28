#[test]
fn test_not_syntax_variants() {
    use mc_data::schema::enemy::FlagExpr;

    // Try 1: plain Not("X")
    let r1 = r#"Not("FLG_ARRESTED")"#;
    println!("Trying: {}", r1);
    match ron::from_str::<FlagExpr>(r1) {
        Ok(v) => println!("  OK: {:?}", v),
        Err(e) => println!("  FAIL: {:?}", e),
    }

    // Try 2: with serde rename
    let r2 = r#"Not("FLG_ARRESTED")"#;
    println!("Trying: {}", r2);
    match ron::from_str::<FlagExpr>(r2) {
        Ok(v) => println!("  OK: {:?}", v),
        Err(e) => println!("  FAIL: {:?}", e),
    }

    // Try 3: In context of SpawnEntry
    use mc_data::schema::spawn_table::SpawnEntry;
    let r3 = r#"(enemy: "X", weight: 10, gate: Not("FLG_ARRESTED"))"#;
    println!("Trying: {}", r3);
    match ron::from_str::<SpawnEntry>(r3) {
        Ok(v) => println!("  OK: {:?}", v),
        Err(e) => println!("  FAIL: {:?}", e),
    }

    // Try 4: With parentheses around the variant
    let r4 = r#"(enemy: "X", weight: 10, gate: (Not("FLG_ARRESTED")))"#;
    println!("Trying: {}", r4);
    match ron::from_str::<SpawnEntry>(r4) {
        Ok(v) => println!("  OK: {:?}", v),
        Err(e) => println!("  FAIL: {:?}", e),
    }
}
