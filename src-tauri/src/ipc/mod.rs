use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_updater::UpdaterExt;
use uuid::Uuid;

use crate::runtime::envelope::MessageEnvelope;
use crate::runtime::graph::{ConnectionValidation, GraphRunResult, MessageDelivery};
use crate::runtime::lifecycle::{Lifecycle, LifecycleMode};
use crate::runtime::node::{NodeKind, Position, RunResult};
use crate::runtime::protocol::presets::PortDeclaration;
use crate::runtime::runner::{GraphRunner, GUI_PACING_MS, LifecycleObserver};
use crate::runtime::sdk::LifecycleHooks;
use crate::state::{AppError, AppState, AppStateSnapshot};

#[derive(Debug, Clone, Serialize)]
pub struct LifecycleEvent {
    pub node_id: Uuid,
    pub lifecycle: Lifecycle,
    pub previous: Option<Lifecycle>,
    pub lifecycle_mode: LifecycleMode,
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

struct LifecycleEmitter {
    app: AppHandle,
    pending: HashMap<Uuid, LifecycleEvent>,
    last_flush: Instant,
}

impl LifecycleEmitter {
    fn new(app: AppHandle) -> Self {
        Self {
            app,
            pending: HashMap::new(),
            last_flush: Instant::now(),
        }
    }

    fn queue(&mut self, event: LifecycleEvent) {
        self.pending.insert(event.node_id, event);
        if self.last_flush.elapsed() >= Duration::from_millis(16) {
            self.flush();
        }
    }

    fn flush(&mut self) {
        for (_, event) in self.pending.drain() {
            let _ = self.app.emit("node:lifecycle", event);
        }
        self.last_flush = Instant::now();
    }
}

impl LifecycleObserver for LifecycleEmitter {
    fn on_lifecycle(
        &mut self,
        node_id: Uuid,
        previous: Lifecycle,
        current: Lifecycle,
        mode: LifecycleMode,
    ) {
        self.queue(LifecycleEvent {
            node_id,
            lifecycle: current,
            previous: Some(previous),
            lifecycle_mode: mode,
        });
    }

    fn on_output(&mut self, node_id: Uuid, output: &str) {
        let _ = self.app.emit(
            "node:output",
            OutputEvent {
                node_id,
                output: output.to_string(),
            },
        );
    }

    fn on_delivery(&mut self, delivery: &MessageDelivery) {
        let _ = self.app.emit(
            "message:delivered",
            MessageDeliveredEvent {
                edge_id: delivery.edge_id,
                envelope: delivery.envelope.clone(),
            },
        );
    }
}

struct EmitterHooks<'a> {
    emitter: &'a mut LifecycleEmitter,
    node_id: Uuid,
}

impl LifecycleHooks for EmitterHooks<'_> {
    fn on_transition(&mut self, from: Lifecycle, to: Lifecycle, mode: LifecycleMode) {
        self.emitter
            .on_lifecycle(self.node_id, from, to, mode);
    }
}

fn emit_lifecycle_simple(
    app: &AppHandle,
    node_id: Uuid,
    lifecycle: Lifecycle,
    mode: LifecycleMode,
) {
    let _ = app.emit(
        "node:lifecycle",
        LifecycleEvent {
            node_id,
            lifecycle,
            previous: None,
            lifecycle_mode: mode,
        },
    );
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageDeliveredEvent {
    pub edge_id: uuid::Uuid,
    pub envelope: MessageEnvelope,
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
pub fn get_app_state(state: State<'_, Mutex<AppState>>) -> Result<AppStateSnapshot, String> {
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
    state: State<'_, Mutex<AppState>>,
    request: AddNodeRequest,
) -> Result<AppStateSnapshot, String> {
    let kind = parse_kind(&request.kind, request.value, request.input)?;
    let position = Position {
        x: request.x.unwrap_or(100.0),
        y: request.y.unwrap_or(100.0),
    };

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let node = guard.add_node(kind, Some(position)).map_err(app_error)?;
    emit_lifecycle_simple(&app, node.id, node.lifecycle, node.lifecycle_mode);
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
    state: State<'_, Mutex<AppState>>,
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
pub fn move_node(
    state: State<'_, Mutex<AppState>>,
    id: Uuid,
    x: f64,
    y: f64,
) -> Result<AppStateSnapshot, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard
        .move_node(id, Position { x, y })
        .map_err(app_error)?;
    Ok(guard.snapshot())
}

#[tauri::command]
pub fn remove_node(
    state: State<'_, Mutex<AppState>>,
    id: Uuid,
) -> Result<AppStateSnapshot, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.remove_node(id).map_err(app_error)?;
    Ok(guard.snapshot())
}

#[derive(Debug, Deserialize)]
pub struct ValidateConnectionRequest {
    pub source_node_id: Uuid,
    pub source_port: String,
    pub target_node_id: Uuid,
    pub target_port: String,
}

#[tauri::command]
pub fn validate_connection(
    state: State<'_, Mutex<AppState>>,
    request: ValidateConnectionRequest,
) -> Result<ConnectionValidation, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    guard
        .validate_connection(
            request.source_node_id,
            request.source_port,
            request.target_node_id,
            request.target_port,
        )
        .map_err(app_error)
}

#[derive(Debug, Deserialize)]
pub struct PortDeclInput {
    pub what: String,
    pub how: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNodePortsRequest {
    pub id: Uuid,
    pub port_decls: std::collections::HashMap<String, PortDeclInput>,
}

#[tauri::command]
pub fn update_node_ports(
    state: State<'_, Mutex<AppState>>,
    request: UpdateNodePortsRequest,
) -> Result<AppStateSnapshot, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let parsed: std::collections::HashMap<String, PortDeclaration> = request
        .port_decls
        .into_iter()
        .map(|(port, input)| {
            PortDeclaration::from_ids(&input.what, &input.how)
                .map(|decl| (port, decl))
                .map_err(|e| e.to_string())
        })
        .collect::<Result<_, _>>()?;

    guard
        .update_node_ports(request.id, parsed)
        .map_err(app_error)?;
    Ok(guard.snapshot())
}

#[derive(Debug, Deserialize)]
pub struct UpdateNodeModeRequest {
    pub id: Uuid,
    pub mode: String,
}

#[tauri::command]
pub fn update_node_mode(
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
    request: UpdateNodeModeRequest,
) -> Result<AppStateSnapshot, String> {
    let mode = parse_lifecycle_mode(&request.mode)?;
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard
        .update_node_mode(request.id, mode)
        .map_err(app_error)?;
    let node = guard
        .project
        .nodes
        .iter()
        .find(|n| n.id == request.id)
        .ok_or_else(|| "node not found".to_string())?;
    emit_lifecycle_simple(&app, node.id, node.lifecycle, node.lifecycle_mode);
    Ok(guard.snapshot())
}

#[tauri::command]
pub fn start_node(
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
    id: Uuid,
) -> Result<AppStateSnapshot, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let mut emitter = LifecycleEmitter::new(app);
    {
        let mut hooks = EmitterHooks {
            emitter: &mut emitter,
            node_id: id,
        };
        guard.start_node_with_hooks(id, &mut hooks).map_err(app_error)?;
    }
    emitter.flush();
    Ok(guard.snapshot())
}

#[tauri::command]
pub fn stop_node(
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
    id: Uuid,
) -> Result<AppStateSnapshot, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let mut emitter = LifecycleEmitter::new(app);
    {
        let mut hooks = EmitterHooks {
            emitter: &mut emitter,
            node_id: id,
        };
        guard.stop_node_with_hooks(id, &mut hooks).map_err(app_error)?;
    }
    emitter.flush();
    Ok(guard.snapshot())
}

#[derive(Debug, Deserialize)]
pub struct AddEdgeRequest {
    pub source_node_id: Uuid,
    pub source_port: String,
    pub target_node_id: Uuid,
    pub target_port: String,
}

#[tauri::command]
pub fn add_edge(
    state: State<'_, Mutex<AppState>>,
    request: AddEdgeRequest,
) -> Result<AppStateSnapshot, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard
        .add_edge(
            request.source_node_id,
            request.source_port,
            request.target_node_id,
            request.target_port,
        )
        .map_err(app_error)?;
    Ok(guard.snapshot())
}

#[tauri::command]
pub fn remove_edge(
    state: State<'_, Mutex<AppState>>,
    id: Uuid,
) -> Result<AppStateSnapshot, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    guard.remove_edge(id).map_err(app_error)?;
    Ok(guard.snapshot())
}

#[tauri::command]
pub async fn run_graph(
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<GraphRunResult, String> {
    let edges = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        guard.project.edges.clone()
    };

    let order = {
        let guard = state.lock().map_err(|e| e.to_string())?;
        crate::runtime::graph::topological_order(&guard.project.nodes, &edges)
            .map_err(|e| app_error(AppError::from(e)))?
    };

    let mut result = GraphRunResult {
        node_results: Vec::new(),
        deliveries: Vec::new(),
    };

    let mut emitter = LifecycleEmitter::new(app.clone());

    for (i, node_id) in order.iter().enumerate() {
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_millis(GUI_PACING_MS));
        }

        let step = {
            let mut guard = state.lock().map_err(|e| e.to_string())?;
            let wired_input = crate::runtime::graph::incoming_payload(
                *node_id,
                &edges,
                &guard.project.nodes,
            )
            .map_err(|e| app_error(AppError::from(e)))?;

            GraphRunner::run_node_observed(
                &mut guard.project.nodes,
                *node_id,
                wired_input.as_deref(),
                &mut emitter,
            )
            .map_err(|e| app_error(AppError::from(e)))?
        };

        result.node_results.push(step.clone());

        if let Ok(Some((edge_id, envelope))) = {
            let guard = state.lock().map_err(|e| e.to_string())?;
            crate::runtime::graph::wired_input_envelope(*node_id, &edges, &guard.project.nodes)
        } {
            let delivery = MessageDelivery { edge_id, envelope };
            emitter.on_delivery(&delivery);
            result.deliveries.push(delivery);
        }
    }

    emitter.flush();

    {
        let guard = state.lock().map_err(|e| e.to_string())?;
        guard.persist().map_err(app_error)?;
    }

    Ok(result)
}

#[tauri::command]
pub fn run_node(
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
    id: Uuid,
) -> Result<RunResult, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let mut emitter = LifecycleEmitter::new(app);

    match GraphRunner::run_node_observed(&mut guard.project.nodes, id, None, &mut emitter) {
        Ok(result) => {
            emitter.flush();
            guard.persist().map_err(app_error)?;
            Ok(result)
        }
        Err(error) => {
            emitter.flush();
            let _ = guard.persist();
            Err(app_error(AppError::from(error)))
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
pub async fn install_update(app: AppHandle, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
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
        "json_constant" => Ok(NodeKind::JsonConstant {
            value: value.unwrap_or_else(|| "{}".into()),
        }),
        "echo" => Ok(NodeKind::Echo {
            input: input.unwrap_or_default(),
        }),
        other => Err(format!("unknown node kind: {other}")),
    }
}

fn parse_lifecycle_mode(mode: &str) -> Result<LifecycleMode, String> {
    match mode {
        "ephemeral" => Ok(LifecycleMode::Ephemeral),
        "persistent" => Ok(LifecycleMode::Persistent),
        other => Err(format!("unknown lifecycle mode: {other}")),
    }
}

fn app_error(error: AppError) -> String {
    error.to_string()
}
