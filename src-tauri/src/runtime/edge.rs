use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PORT_OUT: &str = "out";
pub const PORT_IN: &str = "in";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Edge {
    pub id: Uuid,
    pub source_node_id: Uuid,
    pub source_port: String,
    pub target_node_id: Uuid,
    pub target_port: String,
}

impl Edge {
    pub fn new(
        source_node_id: Uuid,
        source_port: impl Into<String>,
        target_node_id: Uuid,
        target_port: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_node_id,
            source_port: source_port.into(),
            target_node_id,
            target_port: target_port.into(),
        }
    }
}

pub fn is_valid_port(port: &str) -> bool {
    port == PORT_OUT || port == PORT_IN
}
