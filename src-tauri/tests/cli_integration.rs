use node_ide_lib::cli;

#[test]
fn cli_run_constant_headless() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_node-ide"))
        .args(["--cli", "run", "--kind", "constant", "--value", "hello"])
        .output()
        .expect("failed to run cli");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
}

#[test]
fn cli_run_echo_headless() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_node-ide"))
        .args(["--cli", "run", "--kind", "echo", "--input", "world"])
        .output()
        .expect("failed to run cli");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "world");
}

#[test]
fn cli_migrate_dry_run() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_node-ide"))
        .args(["--cli", "migrate"])
        .output()
        .expect("failed to run cli migrate");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("schema_version"));
}

// Keep CLI module linked in integration tests.
#[allow(dead_code)]
const _CLI_SMOKE: fn() -> i32 = cli::run;
