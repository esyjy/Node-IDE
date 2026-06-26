use serde_json::{json, Value};

use super::registry::MigrationError;

pub struct V1ToV2Migrator;

impl super::registry::Migrator for V1ToV2Migrator {
    fn from_version(&self) -> u32 {
        1
    }

    fn to_version(&self) -> u32 {
        2
    }

    fn migrate(&self, data: &mut Value) -> Result<(), MigrationError> {
        if data.get("edges").is_none() {
            data["edges"] = json!([]);
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
    fn migrates_v1_project_to_v2() {
        let registry = MigrationRegistry::new();
        let mut data = json!({
            "schema_version": 1,
            "nodes": []
        });

        registry.run(&mut data).unwrap();
        assert_eq!(data["schema_version"], CURRENT_SCHEMA_VERSION);
        assert!(data.get("edges").is_some());
        assert_eq!(data["edges"], json!([]));
    }
}
