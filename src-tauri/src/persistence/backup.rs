use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("project file does not exist: {0}")]
    NotFound(PathBuf),
}

pub fn backup_project(project_path: &Path, backups_dir: &Path) -> Result<PathBuf, BackupError> {
    if !project_path.exists() {
        return Err(BackupError::NotFound(project_path.to_path_buf()));
    }

    fs::create_dir_all(backups_dir)?;
    let timestamp = Utc::now().format("%Y%m%dT%H%M%S");
    let backup_path = backups_dir.join(format!("pre-update-{timestamp}.json"));
    fs::copy(project_path, &backup_path)?;
    Ok(backup_path)
}

pub fn restore_latest_backup(backups_dir: &Path, project_path: &Path) -> Result<(), BackupError> {
    let mut entries: Vec<PathBuf> = fs::read_dir(backups_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();

    entries.sort();
    let latest = entries
        .pop()
        .ok_or_else(|| BackupError::NotFound(backups_dir.to_path_buf()))?;

    fs::copy(&latest, project_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn backup_and_restore_roundtrip() {
        let dir = std::env::temp_dir().join(format!("node-ide-backup-{}", uuid::Uuid::new_v4()));
        let backups = dir.join("backups");
        let project = dir.join("project.json");
        fs::create_dir_all(&dir).unwrap();
        let mut file = fs::File::create(&project).unwrap();
        writeln!(file, r#"{{"schema_version":1,"nodes":[]}}"#).unwrap();

        let backup = backup_project(&project, &backups).unwrap();
        assert!(backup.exists());

        writeln!(file, r#"{{"broken":true}}"#).unwrap();
        drop(file);
        fs::write(&project, r#"{"broken":true}"#).unwrap();

        restore_latest_backup(&backups, &project).unwrap();
        let restored = fs::read_to_string(&project).unwrap();
        assert!(restored.contains("schema_version"));

        let _ = fs::remove_dir_all(&dir);
    }
}
