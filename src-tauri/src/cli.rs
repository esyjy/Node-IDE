use clap::{Parser, Subcommand};
use uuid::Uuid;

use crate::runtime::edge::{Edge, PORT_IN, PORT_OUT};
use crate::runtime::graph;
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
    /// Resolve port declarations without a graph
    Resolve {
        #[arg(long)]
        source_what: String,
        #[arg(long)]
        source_how: String,
        #[arg(long)]
        target_what: String,
        #[arg(long)]
        target_how: String,
    },
    /// Validate a connection between two nodes in the persisted graph
    ValidateConnection {
        #[arg(long)]
        from: Uuid,
        #[arg(long)]
        to: Uuid,
    },
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
                Ok(result) => print_json(&result),
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
            Ok(state) => print_json(&state.snapshot()),
            Err(error) => {
                eprintln!("error: {error}");
                1
            }
        },
        Commands::Migrate => match AppState::new() {
            Ok(state) => match state.migrate_dry_run() {
                Ok(snapshot) => print_json(&snapshot),
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
        Commands::Resolve {
            source_what,
            source_how,
            target_what,
            target_how,
        } => {
            let result = graph::resolve_declarations(
                &source_what,
                &source_how,
                &target_what,
                &target_how,
            );
            print_json(&result)
        }
        Commands::ValidateConnection { from, to } => match AppState::new() {
            Ok(state) => match state.validate_connection(from, PORT_OUT.into(), to, PORT_IN.into()) {
                Ok(result) => print_json(&result),
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

fn print_json<T: serde::Serialize>(value: &T) -> i32 {
    match serde_json::to_string_pretty(value) {
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

fn run_headless(kind: &str, value: &str, input: &str) -> Result<String, String> {
    let node_kind = match kind {
        "constant" => NodeKind::Constant {
            value: value.to_string(),
        },
        "json_constant" => NodeKind::JsonConstant {
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

#[allow(dead_code)]
pub fn sample_incompatible_graph() -> (Vec<NodeInstance>, Vec<Edge>) {
    let json_constant = NodeInstance::new(
        NodeKind::JsonConstant {
            value: "{}".into(),
        },
        Some(Position { x: 0.0, y: 0.0 }),
    );
    let echo = NodeInstance::new(
        NodeKind::Echo {
            input: "".into(),
        },
        Some(Position { x: 200.0, y: 0.0 }),
    );
    let edge = Edge::new(json_constant.id, PORT_OUT, echo.id, PORT_IN);
    (vec![json_constant, echo], vec![edge])
}
