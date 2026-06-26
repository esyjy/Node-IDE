use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::runtime::node::NodeInstance;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
pub const MAX_NODES_V1: usize = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub schema_version: u32,
    pub nodes: Vec<NodeInstance>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            nodes: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("v1 allows at most {MAX_NODES_V1} node on canvas")]
    NodeLimitExceeded,
}

impl Project {
    pub fn load(path: &Path) -> Result<Self, ProjectError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let project: Project = serde_json::from_str(&content)?;
        Ok(project)
    }

    pub fn save(&self, path: &Path) -> Result<(), ProjectError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn can_add_node(&self) -> bool {
        self.nodes.len() < MAX_NODES_V1
    }
}

pub fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.nodeide.app")
}

pub fn project_path() -> PathBuf {
    app_data_dir().join("project.json")
}

pub fn backups_dir() -> PathBuf {
    app_data_dir().join("backups")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_project_has_schema_version() {
        let project = Project::default();
        assert_eq!(project.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn node_limit_v1() {
        let project = Project::default();
        assert!(project.can_add_node());
    }
}
