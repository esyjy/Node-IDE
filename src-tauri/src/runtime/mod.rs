pub mod builtin;
pub mod edge;
pub mod envelope;
pub mod graph;
pub mod lifecycle;
pub mod node;
pub mod protocol;
pub mod sdk;

use uuid::Uuid;

use graph::GraphRunResult;
use node::{NodeInstance, RunResult, RuntimeError};

pub struct MinimalRuntime;

impl MinimalRuntime {
    pub fn run_node(nodes: &mut [NodeInstance], id: Uuid) -> Result<RunResult, RuntimeError> {
        let node = nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or(RuntimeError::NotFound(id))?;
        node.run(None)
    }

    pub fn run_graph(
        nodes: &mut [NodeInstance],
        edges: &[edge::Edge],
    ) -> Result<GraphRunResult, graph::GraphError> {
        graph::run_graph(nodes, edges)
    }
}
