pub mod builtin;
pub mod lifecycle;
pub mod node;

use uuid::Uuid;

use node::{NodeInstance, RunResult, RuntimeError};

pub struct MinimalRuntime;

impl MinimalRuntime {
    pub fn run_node(nodes: &mut [NodeInstance], id: Uuid) -> Result<RunResult, RuntimeError> {
        let node = nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or(RuntimeError::NotFound(id))?;
        node.run()
    }
}
