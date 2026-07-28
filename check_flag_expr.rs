// Minimal test to check FlagExpr deserialization
fn main() {
    // Test basic unit variant
    let always = ron::from_str::<mc_data::schema::enemy::FlagExpr>("Always").unwrap();
    println!("Always: {:?}", always);
    
    // Test Not variant with string
    let not = ron::from_str::<mc_data::schema::enemy::FlagExpr>(r#"Not("FLG_ARRESTED")"#).unwrap();
    println!("Not: {:?}", not);
    
    // Test All variant
    let all = ron::from_str::<mc_data::schema::enemy::FlagExpr>(r#"All(["FLG_ARRESTED"])"#).unwrap();
    println!("All: {:?}", all);
    
    // Test Any variant
    let any = ron::from_str::<mc_data::schema::enemy::FlagExpr>(r#"Any(["FLG_ARRESTED"])"#).unwrap();
    println!("Any: {:?}", any);
    
    // Test the full spawn table entry
    let entry = ron::from_str::<mc_data::schema::spawn_table::SpawnEntry>(
        r#"(enemy: "ENM_DOCK_RAT", weight: 25, gate: Not("FLG_ARRESTED"))"#
    ).unwrap();
    println!("Entry: {:?}", entry);
}
