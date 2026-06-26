use node_ide_lib::migration::registry::MigrationRegistry;
use serde_json::json;

#[test]
fn migration_v1_to_v2_adds_edges() {
    let registry = MigrationRegistry::new();
    let mut data = json!({
        "schema_version": 1,
        "nodes": []
    });

    registry.run(&mut data).unwrap();
    assert_eq!(data["schema_version"], 2);
    assert_eq!(data["edges"], json!([]));
}
