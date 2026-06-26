use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::migration::registry::MigrationRegistry;
use crate::migration::{backup_before_update, restore_on_migration_failure, run_migrations_on_project};
use crate::persistence::project::{self, Project, ProjectError};
use crate::runtime::node::{NodeInstance, NodeKind, Position, RunResult, RuntimeError};
use crate::runtime::MinimalRuntime;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("project error: {0}")]
    Project(#[from] ProjectError),
    #[error("runtime error: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("{0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStateSnapshot {
    pub schema_version: u32,
    pub nodes: Vec<NodeInstance>,
    pub project_path: String,
}

pub struct AppState {
    pub project: Project,
    pub project_path: PathBuf,
    pub migration_registry: MigrationRegistry,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        let project_path = project::project_path();
        let migration_registry = MigrationRegistry::new();
        let project = run_migrations_on_project(&migration_registry, &project_path)
            .map_err(|e| AppError::Validation(e.to_string()))?;

        Ok(Self {
            project,
            project_path,
            migration_registry,
        })
    }

    pub fn snapshot(&self) -> AppStateSnapshot {
        AppStateSnapshot {
            schema_version: self.project.schema_version,
            nodes: self.project.nodes.clone(),
            project_path: self.project_path.display().to_string(),
        }
    }

    fn persist(&self) -> Result<(), AppError> {
        self.project.save(&self.project_path)?;
        Ok(())
    }

    pub fn add_node(&mut self, kind: NodeKind, position: Option<Position>) -> Result<NodeInstance, AppError> {
        if !self.project.can_add_node() {
            return Err(AppError::Validation(
                "v1 allows only one node on the canvas".into(),
            ));
        }

        let node = NodeInstance::new(kind, position);
        self.project.nodes.push(node.clone());
        self.persist()?;
        Ok(node)
    }

    pub fn update_node(&mut self, id: Uuid, kind: NodeKind) -> Result<NodeInstance, AppError> {
        let node = self
            .project
            .nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| AppError::Validation(format!("node not found: {id}")))?;

        node.kind = kind;
        let updated = node.clone();
        self.persist()?;
        Ok(updated)
    }

    pub fn remove_node(&mut self, id: Uuid) -> Result<(), AppError> {
        let before = self.project.nodes.len();
        self.project.nodes.retain(|n| n.id != id);
        if self.project.nodes.len() == before {
            return Err(AppError::Validation(format!("node not found: {id}")));
        }
        self.persist()?;
        Ok(())
    }

    pub fn run_node(&mut self, id: Uuid) -> Result<RunResult, AppError> {
        let result = MinimalRuntime::run_node(&mut self.project.nodes, id)?;
        self.persist()?;
        Ok(result)
    }

    pub fn backup_before_update(&self) -> Result<(), AppError> {
        backup_before_update(&self.project_path)
            .map_err(|e| AppError::Validation(e.to_string()))
    }

    pub fn run_migrations(&mut self) -> Result<(), AppError> {
        match run_migrations_on_project(&self.migration_registry, &self.project_path) {
            Ok(project) => {
                self.project = project;
                Ok(())
            }
            Err(error) => {
                let _ = restore_on_migration_failure(&self.project_path);
                Err(AppError::Validation(error.to_string()))
            }
        }
    }

    pub fn migrate_dry_run(&self) -> Result<AppStateSnapshot, AppError> {
        let content = if self.project_path.exists() {
            std::fs::read_to_string(&self.project_path)
                .map_err(|e| AppError::Validation(e.to_string()))?
        } else {
            serde_json::to_string(&Project::default())
                .map_err(|e| AppError::Validation(e.to_string()))?
        };
        let mut value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| AppError::Validation(e.to_string()))?;
        self.migration_registry
            .run(&mut value)
            .map_err(|e| AppError::Validation(e.to_string()))?;
        let project: Project = serde_json::from_value(value)
            .map_err(|e| AppError::Validation(e.to_string()))?;
        Ok(AppStateSnapshot {
            schema_version: project.schema_version,
            nodes: project.nodes,
            project_path: self.project_path.display().to_string(),
        })
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new().expect("failed to initialize app state")
    }
}
