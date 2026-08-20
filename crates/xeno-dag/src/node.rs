//! Strongly-typed DAG node definitions, status state machine, and model assignments.

use serde::{Deserialize, Serialize};

/// Dynamic execution status of a DAG node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    /// Initial state waiting for dependencies.
    Pending,
    /// Actively executing.
    Running,
    /// Finished successfully.
    Success,
    /// Execution failed.
    Failed,
    /// Self-healing retry triggered.
    SelfHealing,
}

impl Default for NodeStatus {
    fn default() -> Self {
        NodeStatus::Pending
    }
}

impl NodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::SelfHealing => "self_healing",
        }
    }
}

/// Assigned LLM model specification for an execution node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignedModel {
    pub provider: String,
    pub model_name: String,
    pub temperature: f32,
}

/// Strongly-typed execution DAG node contract matching Blueprint §13 & F02.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XenoDAGNode {
    pub node_id: String,
    pub label: String,
    pub node_type: String,
    pub status: NodeStatus,
    pub dependencies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_model: Option<AssignedModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_payload: Option<serde_json::Value>,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default)]
    pub duration_ms: u64,
}

fn default_max_retries() -> u32 {
    3
}

impl XenoDAGNode {
    pub fn new(
        node_id: impl Into<String>,
        label: impl Into<String>,
        node_type: impl Into<String>,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            label: label.into(),
            node_type: node_type.into(),
            status: NodeStatus::Pending,
            dependencies: Vec::new(),
            assigned_model: None,
            output_payload: None,
            retry_count: 0,
            max_retries: 3,
            duration_ms: 0,
        }
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn with_assigned_model(
        mut self,
        provider: impl Into<String>,
        model_name: impl Into<String>,
        temperature: f32,
    ) -> Self {
        self.assigned_model = Some(AssignedModel {
            provider: provider.into(),
            model_name: model_name.into(),
            temperature,
        });
        self
    }
}
