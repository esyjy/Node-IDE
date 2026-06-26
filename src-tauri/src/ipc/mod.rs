use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_updater::UpdaterExt;
use uuid::Uuid;

use crate::runtime::lifecycle::Lifecycle;
use crate::runtime::node::{NodeKind, Position, RunResult};
use crate::state::{AppError, AppState, AppStateSnapshot};

#[derive(Debug, Clone, Serialize)]
pub struct LifecycleEvent {
    pub node_id: Uuid,
    pub lifecycle: Lifecycle,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputEvent {
    pub node_id: Uuid,
    pub output: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateStatusEvent {
    pub phase: String,
    pub message: Option<String>,
}

fn emit_lifecycle(app: &AppHandle, node_id: Uuid, lifecycle: Lifecycle) {
    let _ = app.emit(
        "node:lifecycle",
        LifecycleEvent {
            node_id,
            lifecycle,
        },
    );
}

fn emit_output(app: &AppHandle, node_id: Uuid, output: &str) {
    let _ = app.emit(
        "node:output",
        OutputEvent {
            node_id,
            output: output.to_string(),
        },
    );
}

fn emit_update_status(app: &AppHandle, phase: &str, message: Option<String>) {
    let _ = app.emit(
        "update:status",
        UpdateStatusEvent {
            phase: phase.to_string(),
            message,
        },
    );
}

#[tauri::command]
pub fn get_app_state(state: State<'_, std::sync::Mutex<AppState>>) -> Result<AppStateSnapshot, String> {
    state
        .lock()
        .map_err(|e| e.to_string())
        .map(|s| s.snapshot())
}

#[derive(Debug, Deserialize)]
pub struct AddNodeRequest {
    pub kind: String,
    pub value: Option<String>,
    pub input: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
}

#[tauri::command]
pub fn add_node(
    app: AppHandle,
    state: State<'_, std::sync::Mutex<AppState>>,
    request: AddNodeRequest,
) -> Result<AppStateSnapshot, String> {
    let kind = parse_kind(&request.kind, request.value, request.input)?;
    let position = Position {
        x: request.x.unwrap_or(100.0),
        y: request.y.unwrap_or(100.0),
    };

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let node = guard.add_node(kind, Some(position)).map_err(app_error)?;
    emit_lifecycle(&app, node.id, node.lifecycle);
    Ok(guard.snapshot())
}

#[derive(Debug, Deserialize)]
pub struct UpdateNodeRequest {
    pub id: Uuid,
    pub kind: String,
    pub value: Option<String>,
    pub input: Option<String>,
}

#[tauri::command]
pub fn update_node(
    state: State<'_, std::sync::Mutex<AppState>>,
    request: UpdateNodeRequest,
) -> Result<AppStateSnapshot, String> {
    let kind = parse_kind(&request.kind, request.value, request.input)?;
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard
        .update_node(request.id, kind)
        .map_err(app_error)?;
    Ok(guard.snapshot())
}

#[tauri::command]
pub fn remove_node(
    state: State<'_, std::sync::Mutex<AppState>>,
    id: Uuid,
) -> Result<AppStateSnapshot, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.remove_node(id).map_err(app_error)?;
    Ok(guard.snapshot())
}

#[tauri::command]
pub fn run_node(
    app: AppHandle,
    state: State<'_, std::sync::Mutex<AppState>>,
    id: Uuid,
) -> Result<RunResult, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    emit_lifecycle(&app, id, Lifecycle::Running);

    match guard.run_node(id) {
        Ok(result) => {
            emit_output(&app, result.node_id, &result.output);
            emit_lifecycle(&app, result.node_id, result.lifecycle);
            Ok(result)
        }
        Err(error) => {
            emit_lifecycle(&app, id, Lifecycle::Failed);
            Err(app_error(error))
        }
    }
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    emit_update_status(&app, "checking", None);

    let current_version = app.package_info().version.to_string();

    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await {
        Ok(Some(update)) => {
            emit_update_status(&app, "ready", Some(format!("Update {} available", update.version)));
            Ok(UpdateInfo {
                available: true,
                current_version,
                latest_version: Some(update.version.clone()),
                notes: update.body.clone(),
            })
        }
        Ok(None) => {
            emit_update_status(&app, "idle", Some("No updates available".into()));
            Ok(UpdateInfo {
                available: false,
                current_version,
                latest_version: None,
                notes: None,
            })
        }
        Err(error) => {
            emit_update_status(&app, "error", Some(error.to_string()));
            Err(error.to_string())
        }
    }
}

#[tauri::command]
pub async fn install_update(app: AppHandle, state: State<'_, std::sync::Mutex<AppState>>) -> Result<(), String> {
    emit_update_status(&app, "downloading", None);

    {
        let guard = state.lock().map_err(|e| e.to_string())?;
        guard.backup_before_update().map_err(app_error)?;
    }

    let updater = app.updater().map_err(|e| e.to_string())?;
    let Some(update) = updater.check().await.map_err(|e| e.to_string())? else {
        return Err("No update available".into());
    };

    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())?;

    {
        let mut guard = state.lock().map_err(|e| e.to_string())?;
        guard.run_migrations().map_err(app_error)?;
    }

    emit_update_status(&app, "restarting", Some("Applying update...".into()));
    app.restart();
    #[allow(unreachable_code)]
    Ok(())
}

fn parse_kind(kind: &str, value: Option<String>, input: Option<String>) -> Result<NodeKind, String> {
    match kind {
        "constant" => Ok(NodeKind::Constant {
            value: value.unwrap_or_else(|| "hello".into()),
        }),
        "echo" => Ok(NodeKind::Echo {
            input: input.unwrap_or_default(),
        }),
        other => Err(format!("unknown node kind: {other}")),
    }
}

fn app_error(error: AppError) -> String {
    error.to_string()
}
