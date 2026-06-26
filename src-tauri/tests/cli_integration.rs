use std::fs;
use std::path::PathBuf;

use node_ide_lib::cli;
use node_ide_lib::runtime::graph::run_graph;

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
fn cli_migrate_dry_run_includes_edges() {
    let data_dir = isolated_data_dir("migrate");
    let project_path = data_dir.join("project.json");
    fs::write(
        &project_path,
        r#"{"schema_version":1,"nodes":[]}"#,
    )
    .expect("write v1 project");

    let output = run_cli(&data_dir, &["--cli", "migrate"]);

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("schema_version"));
    assert!(stdout.contains("edges"));
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

#[allow(dead_code)]
const _CLI_SMOKE: fn() -> i32 = cli::run;
