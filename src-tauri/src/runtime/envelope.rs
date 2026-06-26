use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// v2 wire-format seed: single-shot messages carry source + sequence for v10 streaming.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageEnvelope {
    pub source_node_id: Uuid,
    pub source_port: String,
    pub sequence: u64,
    pub payload: String,
}

impl MessageEnvelope {
    pub fn single_shot(source_node_id: Uuid, source_port: impl Into<String>, payload: String) -> Self {
        Self {
            source_node_id,
            source_port: source_port.into(),
            sequence: 1,
            payload,
        }
    }
}
