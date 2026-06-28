use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::edge::{PORT_IN, PORT_OUT};
use super::lifecycle::{Lifecycle, LifecycleMode};
use super::protocol::presets::{default_port_decls_for_kind, PortDeclaration};
use super::sdk::{LifecycleHooks, NoopLifecycleHooks};
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
    #[serde(default)]
    pub lifecycle_mode: LifecycleMode,
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
    #[error("{0}")]
    Validation(String),
}

impl NodeInstance {
    pub fn new(kind: NodeKind, position: Option<Position>) -> Self {
        let port_decls = default_port_decls_for_kind(&kind);
        let lifecycle_mode = LifecycleMode::Ephemeral;
        Self {
            id: Uuid::new_v4(),
            kind,
            lifecycle: Lifecycle::initial_for_mode(lifecycle_mode),
            lifecycle_mode,
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

    fn transition(
        &mut self,
        to: Lifecycle,
        hooks: &mut dyn LifecycleHooks,
    ) -> Result<(), RuntimeError> {
        let from = self.lifecycle;
        self.lifecycle
            .transition_with_mode(to, self.lifecycle_mode)?;

        hooks.on_transition(from, to, self.lifecycle_mode);

        match to {
            Lifecycle::Initializing => hooks.on_init(),
            Lifecycle::Running => hooks.on_run(),
            Lifecycle::Stopping => hooks.on_stop(),
            _ => {}
        }

        Ok(())
    }

    fn resting_after_start(&self) -> Lifecycle {
        if self.lifecycle_mode == LifecycleMode::Persistent
            && matches!(&self.kind, NodeKind::Echo { input } if input.is_empty())
        {
            Lifecycle::Waiting
        } else {
            Lifecycle::Idle
        }
    }

    /// Start a persistent node (stopped → initializing → idle/waiting).
    pub fn start(&mut self) -> Result<(), RuntimeError> {
        self.start_with_hooks(&mut NoopLifecycleHooks)
    }

    pub fn start_with_hooks(&mut self, hooks: &mut dyn LifecycleHooks) -> Result<(), RuntimeError> {
        if self.lifecycle_mode != LifecycleMode::Persistent {
            return Err(RuntimeError::Validation(
                "only persistent nodes can be started; switch lifecycle mode in properties"
                    .into(),
            ));
        }

        if self.lifecycle == Lifecycle::Stopped {
            self.transition(Lifecycle::Initializing, hooks)?;
        } else if self.lifecycle != Lifecycle::Idle && self.lifecycle != Lifecycle::Waiting {
            return Err(RuntimeError::Validation(format!(
                "cannot start node in {:?} state",
                self.lifecycle
            )));
        }

        if self.lifecycle == Lifecycle::Initializing {
            let resting = self.resting_after_start();
            self.transition(resting, hooks)?;
        }

        Ok(())
    }

    /// Stop a persistent node (idle/waiting/running → stopping → stopped).
    pub fn stop(&mut self) -> Result<(), RuntimeError> {
        self.stop_with_hooks(&mut NoopLifecycleHooks)
    }

    pub fn stop_with_hooks(&mut self, hooks: &mut dyn LifecycleHooks) -> Result<(), RuntimeError> {
        if self.lifecycle_mode != LifecycleMode::Persistent {
            return Err(RuntimeError::Validation(
                "only persistent nodes can be stopped".into(),
            ));
        }

        match self.lifecycle {
            Lifecycle::Stopped => return Ok(()),
            Lifecycle::Stopping => {
                self.transition(Lifecycle::Stopped, hooks)?;
            }
            Lifecycle::Idle | Lifecycle::Waiting | Lifecycle::Running => {
                self.transition(Lifecycle::Stopping, hooks)?;
                self.transition(Lifecycle::Stopped, hooks)?;
            }
            other => {
                return Err(RuntimeError::Validation(format!(
                    "cannot stop node in {other:?} state"
                )));
            }
        }
        Ok(())
    }

    pub fn run(&mut self, wired_input: Option<&str>) -> Result<RunResult, RuntimeError> {
        self.run_with_hooks(wired_input, &mut NoopLifecycleHooks)
    }

    pub fn run_with_hooks(
        &mut self,
        wired_input: Option<&str>,
        hooks: &mut dyn LifecycleHooks,
    ) -> Result<RunResult, RuntimeError> {
        if self.lifecycle_mode == LifecycleMode::Persistent && self.lifecycle == Lifecycle::Stopped {
            return Err(RuntimeError::Validation(
                "persistent node is stopped; use Start before running".into(),
            ));
        }

        self.transition(Lifecycle::Running, hooks)?;

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
                let terminal = Lifecycle::after_success(self.lifecycle_mode);
                self.transition(terminal, hooks)?;
                Ok(RunResult {
                    node_id: self.id,
                    output: value,
                    lifecycle: self.lifecycle,
                })
            }
            Err(message) => {
                self.transition(Lifecycle::Failed, hooks)?;
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
    fn persistent_constant_returns_to_idle() {
        let mut node = NodeInstance::new(
            NodeKind::Constant {
                value: "hello".into(),
            },
            None,
        );
        node.lifecycle_mode = LifecycleMode::Persistent;
        node.lifecycle = Lifecycle::Stopped;
        node.start().unwrap();
        let result = node.run(None).unwrap();
        assert_eq!(result.output, "hello");
        assert_eq!(node.lifecycle, Lifecycle::Idle);
    }

    #[test]
    fn persistent_echo_empty_input_waits_on_start() {
        let mut node = NodeInstance::new(
            NodeKind::Echo {
                input: "".into(),
            },
            None,
        );
        node.lifecycle_mode = LifecycleMode::Persistent;
        node.lifecycle = Lifecycle::Stopped;
        node.start().unwrap();
        assert_eq!(node.lifecycle, Lifecycle::Waiting);
    }

    #[test]
    fn persistent_stop_preserves_id() {
        let mut node = NodeInstance::new(
            NodeKind::Constant {
                value: "x".into(),
            },
            None,
        );
        let id = node.id;
        node.lifecycle_mode = LifecycleMode::Persistent;
        node.start().unwrap();
        node.stop().unwrap();
        assert_eq!(node.id, id);
        assert_eq!(node.lifecycle, Lifecycle::Stopped);
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
