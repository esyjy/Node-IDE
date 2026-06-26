use std::fs;
use std::path::PathBuf;

use node_ide_lib::cli;
use node_ide_lib::runtime::graph::{self, run_graph, GraphError};

fn isolated_data_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("node-ide-test-{name}-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp data dir");
    dir
}

fn run_cli(data_dir: &PathBuf, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_node-ide"))
        .env("NODE_IDE_DATA_DIR", data_dir)
        .args(args)
        .output()
        .expect("failed to run cli")
}

#[test]
fn cli_run_constant_headless() {
    let data_dir = isolated_data_dir("run-constant");
    let output = run_cli(&data_dir, &["--cli", "run", "--kind", "constant", "--value", "hello"]);

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
    let _ = fs::remove_dir_all(data_dir);
}

#[test]
fn cli_run_echo_headless() {
    let data_dir = isolated_data_dir("run-echo");
    let output = run_cli(&data_dir, &["--cli", "run", "--kind", "echo", "--input", "world"]);

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "world");
    let _ = fs::remove_dir_all(data_dir);
}

#[test]
fn cli_migrate_dry_run_includes_port_decls() {
    let data_dir = isolated_data_dir("migrate");
    let project_path = data_dir.join("project.json");
    fs::write(
        &project_path,
        r#"{"schema_version":2,"nodes":[{"id":"00000000-0000-0000-0000-000000000001","kind":{"kind":"constant","value":"hi"},"lifecycle":"created","last_output":null,"position":{"x":0.0,"y":0.0}}],"edges":[]}"#,
    )
    .expect("write v2 project");

    let output = run_cli(&data_dir, &["--cli", "migrate"]);

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("port_decls"));
    assert!(stdout.contains("schema_version"));
    let _ = fs::remove_dir_all(data_dir);
}

#[test]
fn cli_resolve_json_to_text_rejects() {
    let data_dir = isolated_data_dir("resolve");
    let output = run_cli(
        &data_dir,
        &[
            "--cli",
            "resolve",
            "--source-what",
            "json",
            "--source-how",
            "single",
            "--target-what",
            "text",
            "--target-how",
            "single",
        ],
    );

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"compatible\": false"));
    assert!(stdout.contains("What"));
    let _ = fs::remove_dir_all(data_dir);
}

#[test]
fn wired_constant_to_echo_graph() {
    let (mut nodes, edges) = cli::sample_wired_graph();
    let result = run_graph(&mut nodes, &edges).expect("graph should run");
    assert_eq!(result.node_results.len(), 2);
    assert_eq!(result.node_results[1].output, "wired");
    assert_eq!(result.deliveries.len(), 1);
    assert_eq!(result.deliveries[0].envelope.sequence, 1);
}

#[test]
fn json_constant_to_echo_rejected() {
    let (nodes, edges) = cli::sample_incompatible_graph();
    let edge = &edges[0];
    let err = graph::validate_new_edge(&nodes, &[], edge).unwrap_err();
    assert!(matches!(err, GraphError::IncompatiblePorts { .. }));
}

#[allow(dead_code)]
const _CLI_SMOKE: fn() -> i32 = cli::run;
