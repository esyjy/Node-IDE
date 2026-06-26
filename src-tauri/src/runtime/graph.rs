use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::edge::{Edge, PORT_IN, PORT_OUT};
use super::envelope::MessageEnvelope;
use super::node::{NodeInstance, RunResult};

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

    Ok(())
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
    let order = topological_order(nodes, edges)?;
    let mut deliveries = Vec::new();
    let mut node_results = Vec::new();

    for node_id in order {
        let wired_input = incoming_payload(node_id, edges, nodes)?;

        let idx = nodes
            .iter()
            .position(|n| n.id == node_id)
            .ok_or(GraphError::NodeNotFound(node_id))?;

        let result = nodes[idx].run(wired_input.as_deref())?;
        node_results.push(result);

        if let Some((edge_id, envelope)) = wired_input_envelope(node_id, edges, nodes)? {
            deliveries.push(MessageDelivery { edge_id, envelope });
        }
    }

    Ok(GraphRunResult {
        node_results,
        deliveries,
    })
}

fn incoming_payload(
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

fn wired_input_envelope(
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
}
