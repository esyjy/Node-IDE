use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::edge::{PORT_IN, PORT_OUT};
use super::lifecycle::Lifecycle;
use super::protocol::presets::{default_port_decls_for_kind, PortDeclaration};
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
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeKind {
    Constant { value: String },
    JsonConstant { value: String },
    Echo { input: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeInstance {
    pub id: Uuid,
    pub kind: NodeKind,
    pub lifecycle: Lifecycle,
    pub last_output: Option<String>,
    pub position: Position,
    #[serde(default = "default_empty_port_decls")]
    pub port_decls: HashMap<String, PortDeclaration>,
}

fn default_empty_port_decls() -> HashMap<String, PortDeclaration> {
    HashMap::new()
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
        let port_decls = default_port_decls_for_kind(&kind);
        Self {
            id: Uuid::new_v4(),
            kind,
            lifecycle: Lifecycle::Created,
            last_output: None,
            position: position.unwrap_or_default(),
            port_decls,
        }
    }

    pub fn ensure_port_decls(&mut self) {
        if self.port_decls.is_empty() {
            self.port_decls = default_port_decls_for_kind(&self.kind);
        }
    }

    pub fn port_decl(&self, port_id: &str) -> Option<&PortDeclaration> {
        self.port_decls.get(port_id)
    }

    pub fn has_port(&self, port_id: &str) -> bool {
        match &self.kind {
            NodeKind::Constant { .. } | NodeKind::JsonConstant { .. } => port_id == PORT_OUT,
            NodeKind::Echo { .. } => port_id == PORT_IN || port_id == PORT_OUT,
        }
    }

    pub fn run(&mut self, wired_input: Option<&str>) -> Result<RunResult, RuntimeError> {
        self.lifecycle.transition(Lifecycle::Running)?;

        let output = match &self.kind {
            NodeKind::Constant { value } | NodeKind::JsonConstant { value } => {
                builtin::constant::execute(value)
            }
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
    use crate::runtime::protocol::presets::{HowPreset, WhatPreset};

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
    fn json_constant_has_json_port_decl() {
        let node = NodeInstance::new(
            NodeKind::JsonConstant {
                value: r#"{"a":1}"#.into(),
            },
            None,
        );
        let out = node.port_decl(PORT_OUT).unwrap();
        assert_eq!(out.what, WhatPreset::Json);
        assert_eq!(out.how, HowPreset::Single);
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
