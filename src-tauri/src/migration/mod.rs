pub mod registry;
pub mod v1_to_v2;

use std::fs;
use std::path::Path;

use serde_json::Value;
use thiserror::Error;

use crate::migration::registry::{MigrationError, MigrationRegistry};
use crate::persistence::backup::{self, BackupError};
use crate::persistence::project::{self, Project, ProjectError};

#[derive(Debug, Error)]
pub enum MigrationRunError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("migration error: {0}")]
    Migration(#[from] MigrationError),
    #[error("project error: {0}")]
    Project(#[from] ProjectError),
    #[error("backup error: {0}")]
    Backup(#[from] BackupError),
}

pub fn run_migrations_on_project(
    registry: &MigrationRegistry,
    project_path: &Path,
) -> Result<Project, MigrationRunError> {
    if !project_path.exists() {
        return Ok(Project::default());
    }

    let content = fs::read_to_string(project_path)?;
    let mut value: Value = serde_json::from_str(&content)?;
    registry.run(&mut value)?;

    let project: Project = serde_json::from_value(value)?;
    project.save(project_path)?;
    Ok(project)
}

pub fn backup_before_update(project_path: &Path) -> Result<(), MigrationRunError> {
    let backups = project::backups_dir();
    if project_path.exists() {
        backup::backup_project(project_path, &backups)?;
    }
    Ok(())
}

pub fn restore_on_migration_failure(project_path: &Path) -> Result<(), MigrationRunError> {
    let backups = project::backups_dir();
    if backups.exists() {
        backup::restore_latest_backup(&backups, project_path)?;
    }
    Ok(())
}
