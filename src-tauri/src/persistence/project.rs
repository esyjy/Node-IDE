use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::runtime::edge::Edge;
use crate::runtime::node::NodeInstance;

pub const CURRENT_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub schema_version: u32,
    pub nodes: Vec<NodeInstance>,
    #[serde(default)]
    pub edges: Vec<Edge>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
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
}

pub fn app_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("NODE_IDE_DATA_DIR") {
        return PathBuf::from(dir);
    }
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
    fn default_project_has_schema_version_and_edges() {
        let project = Project::default();
        assert_eq!(project.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(project.edges.is_empty());
    }
}
