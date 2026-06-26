// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|arg| arg == "--cli") {
        let code = node_ide_lib::cli::run();
        std::process::exit(code);
    }

    node_ide_lib::run();
}
