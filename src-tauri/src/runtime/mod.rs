pub mod builtin;
pub mod edge;
pub mod envelope;
pub mod graph;
pub mod lifecycle;
pub mod node;
pub mod protocol;
pub mod runner;
pub mod sdk;

use uuid::Uuid;

use graph::GraphRunResult;
use node::{NodeInstance, RunResult, RuntimeError};
use runner::{GraphRunner, CLI_PACING_MS, NoopObserver};

pub struct MinimalRuntime;

impl MinimalRuntime {
    pub fn run_node(nodes: &mut [NodeInstance], id: Uuid) -> Result<RunResult, RuntimeError> {
        let mut observer = NoopObserver;
        GraphRunner::run_node_observed(nodes, id, None, &mut observer)
    }

    pub fn run_graph(
        nodes: &mut [NodeInstance],
        edges: &[edge::Edge],
    ) -> Result<GraphRunResult, graph::GraphError> {
        let mut observer = NoopObserver;
        GraphRunner::run_graph_observed(nodes, edges, &mut observer, CLI_PACING_MS)
    }
}
