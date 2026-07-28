/// Debug why Not syntax fails in R01 context
#[test]
fn debug_not_in_context() {
    use mc_data::schema::spawn_table::SpawnTable;
    
    // Test just the Not variant standalone
    let r1 = r#"Not("FLG_ARRESTED")"#;
    let result1 = ron::from_str::<mc_data::schema::enemy::FlagExpr>(r1);
    println!("Not standalone: {:?}", result1);
    
    // Test in SpawnEntry
    let r2 = r#"(enemy: "X", weight: 10, gate: Not("FLG_ARRESTED"))"#;
    let result2 = ron::from_str::<mc_data::schema::spawn_table::SpawnEntry>(r2);
    println!("Not in SpawnEntry: {:?}", result2);
    
    // Test partial table with Not
    let r3 = r#"
SpawnTable(
    region: "R01",
    chapter_stage: 1,
    pool: 20,
    entries: [
        (enemy: "ENM_SOLDIER", weight: 15, gate: Not("FLG_ARRESTED")),
    ],
)
    "#;
    let result3 = ron::from_str::<SpawnTable>(r3);
    println!("Not in small table: {:?}", result3);
    
    // Test if the issue is the trailing comma
    let r4 = r#"Not("FLG_ARRESTED")"#;
    let result4 = ron::from_str::<mc_data::schema::enemy::FlagExpr>(r4);
    println!("Not without trailing: {:?}", result4);
    
    // Test what happens if Not is lowercase
    let r5 = r#"not("FLG_ARRESTED")"#;
    let result5 = ron::from_str::<mc_data::schema::enemy::FlagExpr>(r5);
    println!("Lowercase not: {:?}", result5);
}
