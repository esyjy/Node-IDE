use node_ide_lib::migration::registry::MigrationRegistry;
use node_ide_lib::persistence::project::CURRENT_SCHEMA_VERSION;
use serde_json::json;

#[test]
fn migration_v1_to_v2_adds_edges() {
    let registry = MigrationRegistry::new();
    let mut data = json!({
        "schema_version": 1,
        "nodes": []
    });

    registry.run(&mut data).unwrap();
    assert_eq!(data["schema_version"], CURRENT_SCHEMA_VERSION);
    assert_eq!(data["edges"], json!([]));
}

#[test]
fn migration_v2_to_v3_adds_port_decls() {
    let registry = MigrationRegistry::new();
    let mut data = json!({
        "schema_version": 2,
        "nodes": [{
            "id": "00000000-0000-0000-0000-000000000001",
            "kind": { "kind": "constant", "value": "hi" },
            "lifecycle": "created",
            "last_output": null,
            "position": { "x": 0.0, "y": 0.0 }
        }],
        "edges": []
    });

    registry.run(&mut data).unwrap();
    assert_eq!(data["schema_version"], CURRENT_SCHEMA_VERSION);
    assert!(data["nodes"][0].get("port_decls").is_some());
    assert_eq!(data["nodes"][0]["lifecycle"], "idle");
    assert_eq!(data["nodes"][0]["lifecycle_mode"], "ephemeral");
}
