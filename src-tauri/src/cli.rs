use clap::{Parser, Subcommand};

use crate::runtime::edge::{Edge, PORT_IN, PORT_OUT};
use crate::runtime::node::{NodeInstance, NodeKind, Position};
use crate::state::AppState;

#[derive(Parser)]
#[command(name = "node-ide", about = "Node-IDE CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a built-in node headlessly (ignores graph wiring)
    Run {
        #[arg(long)]
        kind: String,
        #[arg(long, default_value = "hello")]
        value: String,
        #[arg(long, default_value = "")]
        input: String,
    },
    /// Run the persisted graph headlessly
    RunGraph,
    /// Print persisted project state
    State,
    /// Dry-run migrations on project.json
    Migrate,
}

pub fn run() -> i32 {
    let args: Vec<String> = std::env::args()
        .filter(|arg| arg != "--cli")
        .collect();
    run_with_args(args)
}

pub fn run_with_args(args: Vec<String>) -> i32 {
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Run { kind, value, input } => match run_headless(&kind, &value, &input) {
            Ok(output) => {
                println!("{output}");
                0
            }
            Err(error) => {
                eprintln!("error: {error}");
                1
            }
        },
        Commands::RunGraph => match AppState::new() {
            Ok(mut state) => match state.run_graph() {
                Ok(result) => match serde_json::to_string_pretty(&result) {
                    Ok(json) => {
                        println!("{json}");
                        0
                    }
                    Err(error) => {
                        eprintln!("error: {error}");
                        1
                    }
                },
                Err(error) => {
                    eprintln!("error: {error}");
                    1
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                1
            }
        },
        Commands::State => match AppState::new() {
            Ok(state) => {
                let snapshot = state.snapshot();
                match serde_json::to_string_pretty(&snapshot) {
                    Ok(json) => {
                        println!("{json}");
                        0
                    }
                    Err(error) => {
                        eprintln!("error: {error}");
                        1
                    }
                }
            }
            Err(error) => {
                eprintln!("error: {error}");
                1
            }
        },
        Commands::Migrate => match AppState::new() {
            Ok(state) => match state.migrate_dry_run() {
                Ok(snapshot) => match serde_json::to_string_pretty(&snapshot) {
                    Ok(json) => {
                        println!("{json}");
                        0
                    }
                    Err(error) => {
                        eprintln!("error: {error}");
                        1
                    }
                },
                Err(error) => {
                    eprintln!("error: {error}");
                    1
                }
            },
            Err(error) => {
                eprintln!("error: {error}");
                1
            }
        },
    }
}

fn run_headless(kind: &str, value: &str, input: &str) -> Result<String, String> {
    let node_kind = match kind {
        "constant" => NodeKind::Constant {
            value: value.to_string(),
        },
        "echo" => NodeKind::Echo {
            input: input.to_string(),
        },
        other => return Err(format!("unknown node kind: {other}")),
    };

    let mut node = NodeInstance::new(node_kind, None);
    let result = node.run(None).map_err(|e| e.to_string())?;
    Ok(result.output)
}

/// Build a Constant → Echo graph in memory for tests.
pub fn sample_wired_graph() -> (Vec<NodeInstance>, Vec<Edge>) {
    let constant = NodeInstance::new(
        NodeKind::Constant {
            value: "wired".into(),
        },
        Some(Position { x: 0.0, y: 0.0 }),
    );
    let echo = NodeInstance::new(
        NodeKind::Echo {
            input: "fallback".into(),
        },
        Some(Position { x: 200.0, y: 0.0 }),
    );
    let constant_id = constant.id;
    let echo_id = echo.id;
    let edge = Edge::new(constant_id, PORT_OUT, echo_id, PORT_IN);
    (vec![constant, echo], vec![edge])
}
