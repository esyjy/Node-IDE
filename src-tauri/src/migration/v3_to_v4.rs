use serde_json::Value;

use super::registry::MigrationError;

pub struct V3ToV4Migrator;

impl super::registry::Migrator for V3ToV4Migrator {
    fn from_version(&self) -> u32 {
        3
    }

    fn to_version(&self) -> u32 {
        4
    }

    fn migrate(&self, data: &mut Value) -> Result<(), MigrationError> {
        let Some(nodes) = data.get_mut("nodes").and_then(|v| v.as_array_mut()) else {
            return Ok(());
        };

        for node in nodes.iter_mut() {
            if node.get("lifecycle_mode").is_none() {
                node["lifecycle_mode"] = Value::String("ephemeral".into());
            }

            if let Some(lc) = node.get("lifecycle").and_then(|v| v.as_str()) {
                if lc == "created" {
                    node["lifecycle"] = Value::String("idle".into());
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::migration::registry::MigrationRegistry;
    use crate::persistence::project::CURRENT_SCHEMA_VERSION;
    use serde_json::json;

    #[test]
    fn migrates_v3_project_to_v4() {
        let registry = MigrationRegistry::new();
        let mut data = json!({
            "schema_version": 3,
            "nodes": [{
                "id": "00000000-0000-0000-0000-000000000001",
                "kind": { "kind": "constant", "value": "hi" },
                "lifecycle": "created",
                "last_output": null,
                "position": { "x": 0.0, "y": 0.0 },
                "port_decls": {}
            }],
            "edges": []
        });

        registry.run(&mut data).unwrap();
        assert_eq!(data["schema_version"], CURRENT_SCHEMA_VERSION);
        assert_eq!(data["nodes"][0]["lifecycle"], "idle");
        assert_eq!(data["nodes"][0]["lifecycle_mode"], "ephemeral");
    }
}
