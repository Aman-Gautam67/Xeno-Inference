//! Real-time DAG streaming event models for user interfaces and telemetry.

use crate::node::NodeStatus;
use serde::{Deserialize, Serialize};

/// Type of DAG mutation or state transition event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DAGEventType {
    NodeAdded,
    StatusChanged,
    SubgraphGrafted,
    OutputAttached,
    ExecutionCompleted,
}

/// Strongly-typed event emitted over the DAG broadcast channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DAGNodeEvent {
    pub event_id: String,
    pub timestamp: u64,
    pub node_id: String,
    pub event_type: DAGEventType,
    pub status: NodeStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

impl DAGNodeEvent {
    pub fn new(
        node_id: impl Into<String>,
        event_type: DAGEventType,
        status: NodeStatus,
        message: impl Into<String>,
    ) -> Self {
        let ts = chrono::Utc::now().timestamp_millis() as u64;
        Self {
            event_id: format!("dag-evt-{}", uuid::Uuid::new_v4()),
            timestamp: ts,
            node_id: node_id.into(),
            event_type,
            status,
            message: message.into(),
            payload: None,
        }
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }
}
