//! Common types, DAG node models, provider kinds, and lifecycle enums.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier types for sessions, nodes, and events.
pub type SessionId = String;
pub type NodeId = String;
pub type EventId = Uuid;

/// Functional classification of execution nodes within the real-time execution DAG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DAGNodeType {
    /// Commander or planning agent node.
    Orchestrator,
    /// Specialized subagent worker node (Architect, Coder, Tester, Security).
    Subagent,
    /// Direct tool or sandbox execution node.
    ToolExec,
    /// Verification, lint, consensus, or AST sanity gate node.
    VerificationGate,
    /// Generated output code, diff, or document artifact node.
    Artifact,
}

/// Type alias for [`DAGNodeType`].
pub type NodeType = DAGNodeType;

/// Execution lifecycle states of a DAG node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DAGNodeStatus {
    /// Awaiting dependency completion.
    Pending,
    /// Actively executing in the runtime.
    Running,
    /// Completed successfully with valid outputs.
    Success,
    /// Execution failed with error.
    Failed,
    /// Active self-healing / recursive retry loop.
    SelfHealing,
}

/// Type alias for [`DAGNodeStatus`].
pub type NodeStatus = DAGNodeStatus;

impl DAGNodeStatus {
    /// Returns true if the status represents a finished terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Success | Self::Failed)
    }

    /// Returns true if the node is currently in progress.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::SelfHealing)
    }
}

/// Supported provider families for local runtimes and hyperscale cloud APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    /// Local engine (llama.cpp, vLLM, Ollama, MLX).
    Local,
    /// Anthropic (Claude 3.7 Sonnet, Claude 3.5 Haiku).
    Anthropic,
    /// OpenAI (GPT-4o, o1, o3-mini).
    Openai,
    /// Google (Gemini 2.0 Flash, Gemini 2.0 Pro).
    Google,
    /// DeepSeek (DeepSeek V3, DeepSeek R1).
    Deepseek,
    /// Groq (LPU Ultra-fast inference).
    Groq,
    /// Deterministic mock provider for tests.
    Mock,
}

impl ProviderKind {
    /// Returns true if this is an external cloud API provider.
    pub fn is_cloud(&self) -> bool {
        matches!(
            self,
            Self::Anthropic | Self::Openai | Self::Google | Self::Deepseek | Self::Groq
        )
    }

    /// Returns true if this is a local runtime.
    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local | Self::Mock)
    }

    /// Canonical string identifier for the provider kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Anthropic => "anthropic",
            Self::Openai => "openai",
            Self::Google => "google",
            Self::Deepseek => "deepseek",
            Self::Groq => "groq",
            Self::Mock => "mock",
        }
    }
}

/// Model assignment configuration for a DAG node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelAssignment {
    /// Provider family to use.
    pub provider: ProviderKind,
    /// Specific model name/identifier.
    pub model_name: String,
    /// Sampling temperature (0.0 - 2.0).
    pub temperature: f32,
}

impl ModelAssignment {
    /// Creates a new model assignment.
    pub fn new(provider: ProviderKind, model_name: impl Into<String>, temperature: f32) -> Self {
        Self {
            provider,
            model_name: model_name.into(),
            temperature,
        }
    }
}

/// Represents a discrete node within the real-time execution DAG.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XenoDAGNode {
    /// Unique identifier for this node.
    pub node_id: String,
    /// Human-readable label / task description.
    pub label: String,
    /// Semantic node classification.
    pub node_type: DAGNodeType,
    /// Current execution lifecycle state.
    pub status: DAGNodeStatus,
    /// Upstream dependency node IDs that must succeed before this node runs.
    pub dependencies: Vec<String>,
    /// Assigned inference model configuration.
    pub assigned_model: ModelAssignment,
    /// Output payload produced by this node upon completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_payload: Option<serde_json::Value>,
    /// UTC timestamp when the node was instantiated.
    pub created_at: DateTime<Utc>,
    /// UTC timestamp when the node reached a terminal status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

impl XenoDAGNode {
    /// Constructs a new [`XenoDAGNode`] in `Pending` status.
    pub fn new(
        node_id: impl Into<String>,
        label: impl Into<String>,
        node_type: DAGNodeType,
        assigned_model: ModelAssignment,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            label: label.into(),
            node_type,
            status: DAGNodeStatus::Pending,
            dependencies: Vec::new(),
            assigned_model,
            output_payload: None,
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Adds a single dependency to this node.
    pub fn with_dependency(mut self, dependency_id: impl Into<String>) -> Self {
        self.dependencies.push(dependency_id.into());
        self
    }

    /// Adds multiple dependencies to this node.
    pub fn with_dependencies(mut self, deps: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.dependencies.extend(deps.into_iter().map(Into::into));
        self
    }

    /// Transitions node status to a new state and updates completion timestamp if terminal.
    pub fn transition_to(&mut self, new_status: DAGNodeStatus) {
        self.status = new_status;
        if new_status.is_terminal() && self.completed_at.is_none() {
            self.completed_at = Some(Utc::now());
        }
    }

    /// Sets the output payload and marks node as succeeded.
    pub fn set_output(&mut self, payload: serde_json::Value) {
        self.output_payload = Some(payload);
        self.transition_to(DAGNodeStatus::Success);
    }

    /// Returns true if this node has reached a terminal status.
    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }

    /// Computes total execution duration in milliseconds if completed.
    pub fn duration_ms(&self) -> Option<i64> {
        self.completed_at
            .map(|completed| (completed - self.created_at).num_milliseconds())
    }
}

/// Tool execution security tiers for authorization and sandboxing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolSecurityTier {
    /// Tier 1: Safe read-only commands (e.g. `cat`, `ls`, `cargo check`).
    Tier1Safe,
    /// Tier 2: Guarded file modifications with automatic diff snapshots.
    Tier2Guarded,
    /// Tier 3: Destructive commands requiring elevated permission (e.g. `rm -rf`, `sudo`).
    Tier3Destructive,
}

impl ToolSecurityTier {
    /// Returns numeric tier level (1, 2, or 3).
    pub fn level(&self) -> u8 {
        match self {
            Self::Tier1Safe => 1,
            Self::Tier2Guarded => 2,
            Self::Tier3Destructive => 3,
        }
    }

    /// Returns human-readable tier label.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tier1Safe => "Tier 1 (Safe Read-Only)",
            Self::Tier2Guarded => "Tier 2 (Guarded Operation)",
            Self::Tier3Destructive => "Tier 3 (Destructive / Elevated)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_serde() {
        let n_type = DAGNodeType::VerificationGate;
        let json_str = serde_json::to_string(&n_type).unwrap();
        assert_eq!(json_str, "\"verification_gate\"");
        let de: DAGNodeType = serde_json::from_str(&json_str).unwrap();
        assert_eq!(de, n_type);
    }

    #[test]
    fn test_provider_kind_as_str() {
        assert_eq!(ProviderKind::Local.as_str(), "local");
        assert_eq!(ProviderKind::Anthropic.as_str(), "anthropic");
        assert_eq!(ProviderKind::Openai.as_str(), "openai");
        assert_eq!(ProviderKind::Google.as_str(), "google");
        assert_eq!(ProviderKind::Deepseek.as_str(), "deepseek");
        assert_eq!(ProviderKind::Groq.as_str(), "groq");
        assert_eq!(ProviderKind::Mock.as_str(), "mock");
    }
}

