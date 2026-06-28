pub mod cli;
pub mod state;
mod ipc;
pub mod migration;
pub mod persistence;
pub mod runtime;

use std::sync::Mutex;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new().expect("failed to initialize application state");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Mutex::new(app_state))
        .invoke_handler(tauri::generate_handler![
            ipc::get_app_state,
            ipc::add_node,
            ipc::update_node,
            ipc::update_node_ports,
            ipc::update_node_mode,
            ipc::start_node,
            ipc::stop_node,
            ipc::move_node,
            ipc::remove_node,
            ipc::add_edge,
            ipc::remove_edge,
            ipc::validate_connection,
            ipc::run_graph,
            ipc::run_node,
            ipc::check_for_updates,
            ipc::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
