use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::lifecycle::Lifecycle;
use crate::runtime::builtin;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 100.0, y: 100.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum NodeKind {
    Constant { value: String },
    Echo { input: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeInstance {
    pub id: Uuid,
    pub kind: NodeKind,
    pub lifecycle: Lifecycle,
    pub last_output: Option<String>,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunResult {
    pub node_id: Uuid,
    pub output: String,
    pub lifecycle: Lifecycle,
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("node not found: {0}")]
    NotFound(Uuid),
    #[error("lifecycle error: {0}")]
    Lifecycle(#[from] super::lifecycle::LifecycleError),
    #[error("execution failed: {0}")]
    Execution(String),
}

impl NodeInstance {
    pub fn new(kind: NodeKind, position: Option<Position>) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            lifecycle: Lifecycle::Created,
            last_output: None,
            position: position.unwrap_or_default(),
        }
    }

    pub fn run(&mut self, wired_input: Option<&str>) -> Result<RunResult, RuntimeError> {
        self.lifecycle.transition(Lifecycle::Running)?;

        let output = match &self.kind {
            NodeKind::Constant { value } => builtin::constant::execute(value),
            NodeKind::Echo { input } => {
                let effective = wired_input.unwrap_or(input.as_str());
                builtin::echo::execute(effective)
            }
        };

        match output {
            Ok(value) => {
                self.last_output = Some(value.clone());
                self.lifecycle.transition(Lifecycle::Done)?;
                Ok(RunResult {
                    node_id: self.id,
                    output: value,
                    lifecycle: self.lifecycle,
                })
            }
            Err(message) => {
                self.lifecycle.transition(Lifecycle::Failed)?;
                Err(RuntimeError::Execution(message))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_node_outputs_value() {
        let mut node = NodeInstance::new(
            NodeKind::Constant {
                value: "hello".into(),
            },
            None,
        );
        let result = node.run(None).unwrap();
        assert_eq!(result.output, "hello");
        assert_eq!(node.lifecycle, Lifecycle::Done);
    }

    #[test]
    fn echo_node_outputs_input() {
        let mut node = NodeInstance::new(
            NodeKind::Echo {
                input: "world".into(),
            },
            None,
        );
        let result = node.run(None).unwrap();
        assert_eq!(result.output, "world");
    }

    #[test]
    fn echo_prefers_wired_input() {
        let mut node = NodeInstance::new(
            NodeKind::Echo {
                input: "fallback".into(),
            },
            None,
        );
        let result = node.run(Some("wired")).unwrap();
        assert_eq!(result.output, "wired");
    }
}
