use serde_json::Value;

use crate::runtime::node::NodeKind;
use crate::runtime::protocol::presets::default_port_decls_for_kind;

use super::registry::MigrationError;

pub struct V2ToV3Migrator;

impl super::registry::Migrator for V2ToV3Migrator {
    fn from_version(&self) -> u32 {
        2
    }

    fn to_version(&self) -> u32 {
        3
    }

    fn migrate(&self, data: &mut Value) -> Result<(), MigrationError> {
        let Some(nodes) = data.get_mut("nodes").and_then(|v| v.as_array_mut()) else {
            return Ok(());
        };

        for node in nodes.iter_mut() {
            if node.get("port_decls").is_some() {
                continue;
            }

            let kind_tag = node
                .get("kind")
                .and_then(|k| k.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("constant");

            let kind = match kind_tag {
                "constant" => NodeKind::Constant {
                    value: node
                        .get("kind")
                        .and_then(|k| k.get("value"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("hello")
                        .to_string(),
                },
                "json_constant" => NodeKind::JsonConstant {
                    value: node
                        .get("kind")
                        .and_then(|k| k.get("value"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("{}")
                        .to_string(),
                },
                _ => NodeKind::Echo {
                    input: node
                        .get("kind")
                        .and_then(|k| k.get("input"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                },
            };

            let decls = default_port_decls_for_kind(&kind);
            node["port_decls"] = serde_json::to_value(decls).map_err(|e| {
                MigrationError::StepFailed {
                    from: 2,
                    to: 3,
                    message: e.to_string(),
                }
            })?;
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
    fn migrates_v2_project_to_v3_with_port_decls() {
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
        assert_eq!(data["nodes"][0]["port_decls"]["out"]["what"]["preset"], "text");
    }
}
