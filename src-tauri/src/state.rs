use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::migration::registry::MigrationRegistry;
use crate::migration::{backup_before_update, restore_on_migration_failure, run_migrations_on_project};
use crate::persistence::project::{self, Project, ProjectError};
use crate::runtime::edge::Edge;
use crate::runtime::graph::{self, ConnectionValidation, GraphError, GraphRunResult};
use crate::runtime::lifecycle::LifecycleMode;
use crate::runtime::node::{NodeInstance, NodeKind, Position, RunResult, RuntimeError};
use crate::runtime::protocol::presets::PortDeclaration;
use crate::runtime::runner::{GraphRunner, GUI_PACING_MS, NoopObserver};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("project error: {0}")]
    Project(#[from] ProjectError),
    #[error("runtime error: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("graph error: {0}")]
    Graph(#[from] GraphError),
    #[error("{0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStateSnapshot {
    pub schema_version: u32,
    pub nodes: Vec<NodeInstance>,
    pub edges: Vec<Edge>,
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
            edges: self.project.edges.clone(),
            project_path: self.project_path.display().to_string(),
        }
    }

    pub(crate) fn persist(&self) -> Result<(), AppError> {
        self.project.save(&self.project_path)?;
        Ok(())
    }

    pub fn add_node(&mut self, kind: NodeKind, position: Option<Position>) -> Result<NodeInstance, AppError> {
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
        for (port_id, decl) in crate::runtime::protocol::presets::default_port_decls_for_kind(&node.kind)
        {
            node.port_decls.entry(port_id).or_insert(decl);
        }
        let updated = node.clone();
        self.persist()?;
        Ok(updated)
    }

    pub fn update_node_ports(
        &mut self,
        id: Uuid,
        port_decls: std::collections::HashMap<String, PortDeclaration>,
    ) -> Result<(), AppError> {
        let node = self
            .project
            .nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| AppError::Validation(format!("node not found: {id}")))?;

        for (port_id, decl) in port_decls {
            if !node.has_port(&port_id) {
                return Err(AppError::Validation(format!(
                    "port '{port_id}' is not declared on this node"
                )));
            }
            node.port_decls.insert(port_id, decl);
        }
        self.persist()?;
        Ok(())
    }

    pub fn validate_connection(
        &self,
        source_node_id: Uuid,
        source_port: String,
        target_node_id: Uuid,
        target_port: String,
    ) -> Result<ConnectionValidation, AppError> {
        graph::validate_connection(
            &self.project.nodes,
            source_node_id,
            &source_port,
            target_node_id,
            &target_port,
        )
        .map_err(AppError::from)
    }

    pub fn move_node(&mut self, id: Uuid, position: Position) -> Result<(), AppError> {
        let node = self
            .project
            .nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| AppError::Validation(format!("node not found: {id}")))?;

        node.position = position;
        self.persist()?;
        Ok(())
    }

    pub fn remove_node(&mut self, id: Uuid) -> Result<(), AppError> {
        let before = self.project.nodes.len();
        self.project.nodes.retain(|n| n.id != id);
        if self.project.nodes.len() == before {
            return Err(AppError::Validation(format!("node not found: {id}")));
        }
        self.project
            .edges
            .retain(|e| e.source_node_id != id && e.target_node_id != id);
        self.persist()?;
        Ok(())
    }

    pub fn add_edge(
        &mut self,
        source_node_id: Uuid,
        source_port: String,
        target_node_id: Uuid,
        target_port: String,
    ) -> Result<Edge, AppError> {
        let edge = Edge::new(source_node_id, source_port, target_node_id, target_port);
        graph::validate_new_edge(&self.project.nodes, &self.project.edges, &edge)?;
        self.project.edges.push(edge.clone());
        self.persist()?;
        Ok(edge)
    }

    pub fn remove_edge(&mut self, id: Uuid) -> Result<(), AppError> {
        let before = self.project.edges.len();
        self.project.edges.retain(|e| e.id != id);
        if self.project.edges.len() == before {
            return Err(AppError::Validation(format!("edge not found: {id}")));
        }
        self.persist()?;
        Ok(())
    }

    pub fn update_node_mode(&mut self, id: Uuid, mode: LifecycleMode) -> Result<(), AppError> {
        let node = self
            .project
            .nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| AppError::Validation(format!("node not found: {id}")))?;

        if node.lifecycle == crate::runtime::lifecycle::Lifecycle::Running {
            return Err(AppError::Validation(
                "cannot change lifecycle mode while node is running".into(),
            ));
        }

        node.lifecycle_mode = mode;
        if node.lifecycle != crate::runtime::lifecycle::Lifecycle::Failed {
            node.lifecycle = crate::runtime::lifecycle::Lifecycle::initial_for_mode(mode);
        }

        self.persist()?;
        Ok(())
    }

    pub fn start_node(&mut self, id: Uuid) -> Result<NodeInstance, AppError> {
        let node = self
            .project
            .nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| AppError::Validation(format!("node not found: {id}")))?;

        node.start().map_err(AppError::from)?;
        let updated = node.clone();
        self.persist()?;
        Ok(updated)
    }

    pub fn start_node_with_hooks(
        &mut self,
        id: Uuid,
        hooks: &mut dyn crate::runtime::sdk::LifecycleHooks,
    ) -> Result<NodeInstance, AppError> {
        let node = self
            .project
            .nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| AppError::Validation(format!("node not found: {id}")))?;

        node.start_with_hooks(hooks).map_err(AppError::from)?;
        let updated = node.clone();
        self.persist()?;
        Ok(updated)
    }

    pub fn stop_node(&mut self, id: Uuid) -> Result<NodeInstance, AppError> {
        let node = self
            .project
            .nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| AppError::Validation(format!("node not found: {id}")))?;

        node.stop().map_err(AppError::from)?;
        let updated = node.clone();
        self.persist()?;
        Ok(updated)
    }

    pub fn stop_node_with_hooks(
        &mut self,
        id: Uuid,
        hooks: &mut dyn crate::runtime::sdk::LifecycleHooks,
    ) -> Result<NodeInstance, AppError> {
        let node = self
            .project
            .nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or_else(|| AppError::Validation(format!("node not found: {id}")))?;

        node.stop_with_hooks(hooks).map_err(AppError::from)?;
        let updated = node.clone();
        self.persist()?;
        Ok(updated)
    }

    pub fn run_node(&mut self, id: Uuid) -> Result<RunResult, AppError> {
        let mut observer = NoopObserver;
        let result =
            GraphRunner::run_node_observed(&mut self.project.nodes, id, None, &mut observer)?;
        self.persist()?;
        Ok(result)
    }

    pub fn run_graph(&mut self) -> Result<GraphRunResult, AppError> {
        let edges = self.project.edges.clone();
        let mut observer = NoopObserver;
        let result = GraphRunner::run_graph_observed(
            &mut self.project.nodes,
            &edges,
            &mut observer,
            GUI_PACING_MS,
        )?;
        self.persist()?;
        Ok(result)
    }

    pub fn run_graph_headless(&mut self) -> Result<GraphRunResult, AppError> {
        let edges = self.project.edges.clone();
        let mut observer = NoopObserver;
        let result = GraphRunner::run_graph_observed(
            &mut self.project.nodes,
            &edges,
            &mut observer,
            crate::runtime::runner::CLI_PACING_MS,
        )?;
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
            edges: project.edges,
            project_path: self.project_path.display().to_string(),
        })
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new().expect("failed to initialize app state")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::node::NodeKind;
    use crate::runtime::protocol::presets::{HowPreset, PortDeclaration, WhatPreset};
    use std::collections::HashMap;

    #[test]
    fn update_node_preserves_custom_port_decls() {
        let mut state = AppState {
            project: Project::default(),
            project_path: std::env::temp_dir().join("node-ide-test-state"),
            migration_registry: MigrationRegistry::new(),
        };

        let node = state
            .add_node(
                NodeKind::Echo { input: "hello".into() },
                Some(Position { x: 0.0, y: 0.0 }),
            )
            .unwrap();

        let mut custom_decls = HashMap::new();
        custom_decls.insert(
            "in".into(),
            PortDeclaration::new(WhatPreset::Json, HowPreset::Single),
        );
        state.update_node_ports(node.id, custom_decls).unwrap();

        state
            .update_node(node.id, NodeKind::Echo { input: "updated".into() })
            .unwrap();

        let updated = state.project.nodes.iter().find(|n| n.id == node.id).unwrap();
        assert_eq!(updated.kind, NodeKind::Echo { input: "updated".into() });
        assert_eq!(
            updated.port_decls.get("in").map(|d| &d.what),
            Some(&WhatPreset::Json)
        );
    }
}
