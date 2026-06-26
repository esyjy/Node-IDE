use serde_json::Value;
use thiserror::Error;

use crate::persistence::project::CURRENT_SCHEMA_VERSION;

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("migration failed at version {from} -> {to}: {message}")]
    StepFailed {
        from: u32,
        to: u32,
        message: String,
    },
    #[error("no migrator registered for {from} -> {to}")]
    MissingMigrator { from: u32, to: u32 },
    #[error("schema version {0} is newer than supported {1}")]
    UnsupportedVersion(u32, u32),
}

pub trait Migrator: Send + Sync {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn migrate(&self, data: &mut Value) -> Result<(), MigrationError>;
}

pub struct MigrationRegistry {
    migrators: Vec<Box<dyn Migrator>>,
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationRegistry {
    pub fn new() -> Self {
        Self {
            migrators: Vec::new(),
        }
    }

    pub fn register(&mut self, migrator: Box<dyn Migrator>) {
        self.migrators.push(migrator);
    }

    pub fn run(&self, data: &mut Value) -> Result<(), MigrationError> {
        let current = data
            .get("schema_version")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        if current > CURRENT_SCHEMA_VERSION {
            return Err(MigrationError::UnsupportedVersion(
                current,
                CURRENT_SCHEMA_VERSION,
            ));
        }

        let mut version = current;
        while version < CURRENT_SCHEMA_VERSION {
            let next = version + 1;
            let migrator = self
                .migrators
                .iter()
                .find(|m| m.from_version() == version && m.to_version() == next)
                .ok_or(MigrationError::MissingMigrator {
                    from: version,
                    to: next,
                })?;

            migrator.migrate(data).map_err(|e| match e {
                MigrationError::StepFailed { .. } => e,
                other => MigrationError::StepFailed {
                    from: version,
                    to: next,
                    message: other.to_string(),
                },
            })?;

            data["schema_version"] = Value::from(next);
            version = next;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_registry_noop_at_current_version() {
        let registry = MigrationRegistry::new();
        let mut data = json!({ "schema_version": CURRENT_SCHEMA_VERSION, "nodes": [] });
        registry.run(&mut data).unwrap();
        assert_eq!(data["schema_version"], CURRENT_SCHEMA_VERSION);
    }
}
