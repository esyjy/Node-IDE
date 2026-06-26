use clap::{Parser, Subcommand};

use crate::runtime::node::{NodeInstance, NodeKind};
use crate::state::AppState;

#[derive(Parser)]
#[command(name = "node-ide", about = "Node-IDE CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a built-in node headlessly
    Run {
        #[arg(long)]
        kind: String,
        #[arg(long, default_value = "hello")]
        value: String,
        #[arg(long, default_value = "")]
        input: String,
    },
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
    let result = node.run().map_err(|e| e.to_string())?;
    Ok(result.output)
}
