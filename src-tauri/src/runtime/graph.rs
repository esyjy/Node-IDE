use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::edge::{Edge, PORT_IN, PORT_OUT};
use super::envelope::MessageEnvelope;
use super::node::{NodeInstance, RunResult};
use super::protocol::presets::PortDeclaration;
use super::protocol::resolve::{resolve_ports, ResolutionOutcome};
use super::protocol::presets::Axis;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionValidation {
    pub compatible: bool,
    pub axis: Option<Axis>,
    pub reason: Option<String>,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageDelivery {
    pub edge_id: Uuid,
    pub envelope: MessageEnvelope,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphRunResult {
    pub node_results: Vec<RunResult>,
    pub deliveries: Vec<MessageDelivery>,
}

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("node not found: {0}")]
    NodeNotFound(Uuid),
    #[error("edge would create a cycle")]
    CycleDetected,
    #[error("cannot connect a node to itself")]
    SelfLoop,
    #[error("invalid port: {0} (v2 allows only 'out' and 'in')")]
    InvalidPort(String),
    #[error("target port '{port}' on node {node_id} is already connected")]
    DuplicateTarget { node_id: Uuid, port: String },
    #[error("upstream node {0} has no output to deliver")]
    MissingUpstreamOutput(Uuid),
    #[error("{axis}: {reason} — Hint: {hint}")]
    IncompatiblePorts {
        axis: Axis,
        reason: String,
        hint: String,
    },
    #[error("port '{port}' not declared on node {node_id}")]
    PortNotDeclared { node_id: Uuid, port: String },
    #[error("runtime error: {0}")]
    Runtime(#[from] super::node::RuntimeError),
}

pub fn validate_new_edge(nodes: &[NodeInstance], edges: &[Edge], candidate: &Edge) -> Result<(), GraphError> {
    if !super::edge::is_valid_port(&candidate.source_port)
        || !super::edge::is_valid_port(&candidate.target_port)
    {
        return Err(GraphError::InvalidPort(format!(
            "{} -> {}",
            candidate.source_port, candidate.target_port
        )));
    }

    if candidate.source_port != PORT_OUT || candidate.target_port != PORT_IN {
        return Err(GraphError::InvalidPort(format!(
            "v2 connections must be out -> in (got {} -> {})",
            candidate.source_port, candidate.target_port
        )));
    }

    if candidate.source_node_id == candidate.target_node_id {
        return Err(GraphError::SelfLoop);
    }

    if !nodes.iter().any(|n| n.id == candidate.source_node_id) {
        return Err(GraphError::NodeNotFound(candidate.source_node_id));
    }
    if !nodes.iter().any(|n| n.id == candidate.target_node_id) {
        return Err(GraphError::NodeNotFound(candidate.target_node_id));
    }

    if edges.iter().any(|e| {
        e.target_node_id == candidate.target_node_id && e.target_port == candidate.target_port
    }) {
        return Err(GraphError::DuplicateTarget {
            node_id: candidate.target_node_id,
            port: candidate.target_port.clone(),
        });
    }

    if would_create_cycle(edges, candidate) {
        return Err(GraphError::CycleDetected);
    }

    validate_port_compatibility(nodes, candidate)?;

    Ok(())
}

pub fn validate_connection(
    nodes: &[NodeInstance],
    source_node_id: Uuid,
    source_port: &str,
    target_node_id: Uuid,
    target_port: &str,
) -> Result<ConnectionValidation, GraphError> {
    if source_port != PORT_OUT || target_port != PORT_IN {
        return Ok(ConnectionValidation {
            compatible: false,
            axis: None,
            reason: Some(format!(
                "Connections must be out → in (got {source_port} → {target_port})"
            )),
            hint: Some("Drag from an output handle (right) to an input handle (left).".into()),
        });
    }

    if source_node_id == target_node_id {
        return Ok(ConnectionValidation {
            compatible: false,
            axis: None,
            reason: Some("Cannot connect a node to itself.".into()),
            hint: Some("Connect two different nodes.".into()),
        });
    }

    let source = nodes
        .iter()
        .find(|n| n.id == source_node_id)
        .ok_or(GraphError::NodeNotFound(source_node_id))?;
    let target = nodes
        .iter()
        .find(|n| n.id == target_node_id)
        .ok_or(GraphError::NodeNotFound(target_node_id))?;

    match port_resolution(source, source_port, target, target_port) {
        Ok(()) => Ok(ConnectionValidation {
            compatible: true,
            axis: None,
            reason: None,
            hint: None,
        }),
        Err(GraphError::IncompatiblePorts { axis, reason, hint }) => Ok(ConnectionValidation {
            compatible: false,
            axis: Some(axis),
            reason: Some(reason),
            hint: Some(hint),
        }),
        Err(other) => Err(other),
    }
}

fn validate_port_compatibility(nodes: &[NodeInstance], candidate: &Edge) -> Result<(), GraphError> {
    let source = nodes
        .iter()
        .find(|n| n.id == candidate.source_node_id)
        .ok_or(GraphError::NodeNotFound(candidate.source_node_id))?;
    let target = nodes
        .iter()
        .find(|n| n.id == candidate.target_node_id)
        .ok_or(GraphError::NodeNotFound(candidate.target_node_id))?;

    port_resolution(source, &candidate.source_port, target, &candidate.target_port)
}

fn port_resolution(
    source: &NodeInstance,
    source_port: &str,
    target: &NodeInstance,
    target_port: &str,
) -> Result<(), GraphError> {
    if !source.has_port(source_port) {
        return Err(GraphError::PortNotDeclared {
            node_id: source.id,
            port: source_port.to_string(),
        });
    }
    if !target.has_port(target_port) {
        return Err(GraphError::PortNotDeclared {
            node_id: target.id,
            port: target_port.to_string(),
        });
    }

    let source_decl = source
        .port_decl(source_port)
        .ok_or_else(|| GraphError::PortNotDeclared {
            node_id: source.id,
            port: source_port.to_string(),
        })?;
    let target_decl = target
        .port_decl(target_port)
        .ok_or_else(|| GraphError::PortNotDeclared {
            node_id: target.id,
            port: target_port.to_string(),
        })?;

    match resolve_ports(source_decl, target_decl) {
        ResolutionOutcome::Compatible => Ok(()),
        ResolutionOutcome::Reject { axis, reason, hint } => Err(GraphError::IncompatiblePorts {
            axis,
            reason,
            hint,
        }),
    }
}

pub fn resolve_declarations(
    source_what: &str,
    source_how: &str,
    target_what: &str,
    target_how: &str,
) -> ConnectionValidation {
    let source = match PortDeclaration::from_ids(source_what, source_how) {
        Ok(decl) => decl,
        Err(reason) => {
            return ConnectionValidation {
                compatible: false,
                axis: None,
                reason: Some(reason),
                hint: Some("Use presets: any, text, json, bytes and single, stream, request-response, broadcast.".into()),
            };
        }
    };
    let target = match PortDeclaration::from_ids(target_what, target_how) {
        Ok(decl) => decl,
        Err(reason) => {
            return ConnectionValidation {
                compatible: false,
                axis: None,
                reason: Some(reason),
                hint: Some("Use presets: any, text, json, bytes and single, stream, request-response, broadcast.".into()),
            };
        }
    };

    match resolve_ports(&source, &target) {
        ResolutionOutcome::Compatible => ConnectionValidation {
            compatible: true,
            axis: None,
            reason: None,
            hint: None,
        },
        ResolutionOutcome::Reject { axis, reason, hint } => ConnectionValidation {
            compatible: false,
            axis: Some(axis),
            reason: Some(reason),
            hint: Some(hint),
        },
    }
}

pub fn would_create_cycle(edges: &[Edge], candidate: &Edge) -> bool {
    let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for edge in edges {
        adjacency
            .entry(edge.source_node_id)
            .or_default()
            .push(edge.target_node_id);
    }
    adjacency
        .entry(candidate.source_node_id)
        .or_default()
        .push(candidate.target_node_id);

    let mut visited = HashMap::new();
    let mut stack = HashMap::new();

    fn dfs(
        node: Uuid,
        adjacency: &HashMap<Uuid, Vec<Uuid>>,
        visited: &mut HashMap<Uuid, bool>,
        stack: &mut HashMap<Uuid, bool>,
    ) -> bool {
        visited.insert(node, true);
        stack.insert(node, true);

        if let Some(neighbors) = adjacency.get(&node) {
            for &next in neighbors {
                if !visited.get(&next).copied().unwrap_or(false) {
                    if dfs(next, adjacency, visited, stack) {
                        return true;
                    }
                } else if stack.get(&next).copied().unwrap_or(false) {
                    return true;
                }
            }
        }

        stack.insert(node, false);
        false
    }

    for &start in adjacency.keys() {
        if !visited.get(&start).copied().unwrap_or(false) && dfs(start, &adjacency, &mut visited, &mut stack) {
            return true;
        }
    }

    false
}

pub fn topological_order(nodes: &[NodeInstance], edges: &[Edge]) -> Result<Vec<Uuid>, GraphError> {
    let node_ids: Vec<Uuid> = nodes.iter().map(|n| n.id).collect();
    let mut in_degree: HashMap<Uuid, usize> = node_ids.iter().copied().map(|id| (id, 0)).collect();

    for edge in edges {
        if in_degree.contains_key(&edge.target_node_id) {
            *in_degree.get_mut(&edge.target_node_id).unwrap() += 1;
        }
    }

    let mut queue: VecDeque<Uuid> = in_degree
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(id, _)| *id)
        .collect();

    let mut order = Vec::with_capacity(node_ids.len());

    while let Some(id) = queue.pop_front() {
        order.push(id);
        for edge in edges.iter().filter(|e| e.source_node_id == id) {
            if let Some(deg) = in_degree.get_mut(&edge.target_node_id) {
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(edge.target_node_id);
                }
            }
        }
    }

    if order.len() != node_ids.len() {
        return Err(GraphError::CycleDetected);
    }

    Ok(order)
}

pub fn run_graph(nodes: &mut [NodeInstance], edges: &[Edge]) -> Result<GraphRunResult, GraphError> {
    crate::runtime::runner::GraphRunner::run_graph_observed(
        nodes,
        edges,
        &mut crate::runtime::runner::NoopObserver,
        crate::runtime::runner::CLI_PACING_MS,
    )
}

pub(crate) fn incoming_payload(
    target_id: Uuid,
    edges: &[Edge],
    nodes: &[NodeInstance],
) -> Result<Option<String>, GraphError> {
    let incoming: Vec<&Edge> = edges
        .iter()
        .filter(|e| e.target_node_id == target_id && e.target_port == PORT_IN)
        .collect();

    if incoming.is_empty() {
        return Ok(None);
    }

    if incoming.len() > 1 {
        return Err(GraphError::DuplicateTarget {
            node_id: target_id,
            port: PORT_IN.into(),
        });
    }

    let edge = incoming[0];
    let source = nodes
        .iter()
        .find(|n| n.id == edge.source_node_id)
        .ok_or(GraphError::NodeNotFound(edge.source_node_id))?;

    let payload = source
        .last_output
        .clone()
        .ok_or(GraphError::MissingUpstreamOutput(edge.source_node_id))?;

    Ok(Some(payload))
}

pub(crate) fn wired_input_envelope(
    target_id: Uuid,
    edges: &[Edge],
    nodes: &[NodeInstance],
) -> Result<Option<(Uuid, MessageEnvelope)>, GraphError> {
    let Some(edge) = edges
        .iter()
        .find(|e| e.target_node_id == target_id && e.target_port == PORT_IN)
    else {
        return Ok(None);
    };

    let source = nodes
        .iter()
        .find(|n| n.id == edge.source_node_id)
        .ok_or(GraphError::NodeNotFound(edge.source_node_id))?;

    let payload = source
        .last_output
        .clone()
        .ok_or(GraphError::MissingUpstreamOutput(edge.source_node_id))?;

    Ok(Some((
        edge.id,
        MessageEnvelope::single_shot(edge.source_node_id, edge.source_port.clone(), payload),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::node::{NodeKind, NodeInstance};

    #[test]
    fn constant_to_echo_graph_run() {
        let constant = NodeInstance::new(
            NodeKind::Constant {
                value: "wired".into(),
            },
            None,
        );
        let echo = NodeInstance::new(NodeKind::Echo { input: "fallback".into() }, None);
        let constant_id = constant.id;
        let echo_id = echo.id;

        let edge = Edge::new(constant_id, PORT_OUT, echo_id, PORT_IN);
        let mut nodes = vec![constant, echo];

        let result = run_graph(&mut nodes, &[edge]).unwrap();
        assert_eq!(result.node_results.len(), 2);
        assert_eq!(result.node_results[1].output, "wired");
        assert_eq!(result.deliveries.len(), 1);
        assert_eq!(result.deliveries[0].envelope.sequence, 1);
    }

    #[test]
    fn rejects_cycle() {
        let a = NodeInstance::new(NodeKind::Constant { value: "a".into() }, None);
        let b = NodeInstance::new(NodeKind::Echo { input: "".into() }, None);
        let edge1 = Edge::new(a.id, PORT_OUT, b.id, PORT_IN);
        let edge2 = Edge::new(b.id, PORT_OUT, a.id, PORT_IN);
        let nodes = vec![a, b];

        assert!(validate_new_edge(&nodes, &[edge1.clone()], &edge2).is_err());
    }

    #[test]
    fn rejects_incompatible_what() {
        let constant = NodeInstance::new(
            NodeKind::JsonConstant {
                value: "{}".into(),
            },
            None,
        );
        let echo = NodeInstance::new(NodeKind::Echo { input: "".into() }, None);
        let edge = Edge::new(constant.id, PORT_OUT, echo.id, PORT_IN);
        let nodes = vec![constant, echo];

        match validate_new_edge(&nodes, &[], &edge) {
            Err(GraphError::IncompatiblePorts { axis, .. }) => assert_eq!(axis, Axis::What),
            other => panic!("expected incompatible ports, got {other:?}"),
        }
    }
}
