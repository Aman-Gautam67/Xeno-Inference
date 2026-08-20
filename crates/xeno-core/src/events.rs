//! Event stream data models matching Blueprint §13 for agent step telemetry.

use crate::metrics::TokenMetrics;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Swarm role classification of the agent emitting an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    /// High-level goal planning and budget orchestration.
    Commander,
    /// System architecture, interface contracts, and schema design.
    Architect,
    /// High-speed code implementation and AST patching.
    Coder,
    /// Dynamic test execution and regression validation.
    Tester,
    /// Vulnerability scanning, prompt injection audit, and security gates.
    Security,
}

impl AgentRole {
    /// String representation of the role.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Commander => "commander",
            Self::Architect => "architect",
            Self::Coder => "coder",
            Self::Tester => "tester",
            Self::Security => "security",
        }
    }
}

/// Discrete phase of the Plan-Act-Observe-Reflect-Verify (PAORV) agentic loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPhase {
    /// Model reasoning and cognitive stream decomposition.
    Thinking,
    /// Tool selection and action schema emission.
    ToolCall,
    /// Tool execution output and environment feedback ingestion.
    Observation,
    /// Anomaly evaluation and hypothesis refinement.
    Reflection,
    /// Formal verification passed and task committed.
    Verified,
}

impl ExecutionPhase {
    /// String representation of the execution phase.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Thinking => "thinking",
            Self::ToolCall => "tool_call",
            Self::Observation => "observation",
            Self::Reflection => "reflection",
            Self::Verified => "verified",
        }
    }
}

/// Detailed cognitive stream payload for reasoning phases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingPayload {
    /// Unfiltered thinking/CoT token content.
    pub raw_tokens: String,
    /// Elapsed duration in milliseconds for this reasoning step.
    pub elapsed_ms: u64,
    /// Token generation velocity during thinking.
    pub tokens_per_second: f64,
    /// Branch identifier in speculative reasoning trees.
    pub branch_id: String,
    /// Indicates if this cognitive path was pruned.
    pub is_pruned: bool,
}

impl ThinkingPayload {
    /// Constructs a new [`ThinkingPayload`].
    pub fn new(
        raw_tokens: impl Into<String>,
        elapsed_ms: u64,
        tokens_per_second: f64,
        branch_id: impl Into<String>,
        is_pruned: bool,
    ) -> Self {
        Self {
            raw_tokens: raw_tokens.into(),
            elapsed_ms,
            tokens_per_second,
            branch_id: branch_id.into(),
            is_pruned,
        }
    }
}

/// Tool invocation specification payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallPayload {
    /// Unique invocation call identifier.
    pub call_id: String,
    /// Name of the invoked tool.
    pub tool_name: String,
    /// Formatted JSON input arguments.
    pub arguments: serde_json::Value,
    /// Optional MCP server label if invoked via Model Context Protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_server: Option<String>,
}

impl ToolCallPayload {
    /// Constructs a standard tool call payload.
    pub fn new(
        call_id: impl Into<String>,
        tool_name: impl Into<String>,
        arguments: serde_json::Value,
    ) -> Self {
        Self {
            call_id: call_id.into(),
            tool_name: tool_name.into(),
            arguments,
            mcp_server: None,
        }
    }

    /// Constructs an MCP tool call payload.
    pub fn with_mcp(
        call_id: impl Into<String>,
        tool_name: impl Into<String>,
        arguments: serde_json::Value,
        mcp_server: impl Into<String>,
    ) -> Self {
        Self {
            call_id: call_id.into(),
            tool_name: tool_name.into(),
            arguments,
            mcp_server: Some(mcp_server.into()),
        }
    }
}

/// Tool observation feedback payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationPayload {
    /// Process exit code (0 for success).
    pub exit_code: i32,
    /// Captured standard output.
    pub stdout: String,
    /// Captured standard error output.
    pub stderr: String,
    /// Unified diff snippet for file edits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_snippet: Option<String>,
    /// Indicates whether in-memory AST syntax validation passed.
    pub ast_validation_passed: bool,
}

impl ObservationPayload {
    /// Constructs a successful observation payload.
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            exit_code: 0,
            stdout: stdout.into(),
            stderr: String::new(),
            diff_snippet: None,
            ast_validation_passed: true,
        }
    }

    /// Constructs a failure observation payload.
    pub fn failure(exit_code: i32, stderr: impl Into<String>) -> Self {
        Self {
            exit_code,
            stdout: String::new(),
            stderr: stderr.into(),
            diff_snippet: None,
            ast_validation_passed: false,
        }
    }
}

/// Low-level backend runtime classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendType {
    /// Local llama.cpp GGUF runtime.
    LocalGguf,
    /// Apple MLX hardware acceleration.
    LocalMlx,
    /// Local vLLM / PagedAttention engine.
    LocalVllm,
    /// Remote commercial Cloud API.
    CloudApi,
}

/// Telemetry and resource utilization stats embedded inside step events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryPayload {
    /// Model identifier utilized.
    pub model_used: String,
    /// Runtime backend type.
    pub backend_type: BackendType,
    /// GPU VRAM allocated in bytes.
    pub vram_allocated_bytes: u64,
    /// Prompt tokens ingested.
    pub prompt_tokens: u32,
    /// Completion tokens emitted.
    pub completion_tokens: u32,
    /// Reasoning tokens produced.
    pub reasoning_tokens: u32,
    /// Total financial cost in USD.
    pub estimated_cost_usd: f64,
    /// Time-To-First-Token in milliseconds.
    pub ttft_ms: u64,
    /// Total execution duration in milliseconds.
    pub total_latency_ms: u64,
    /// Generation velocity in tokens/second.
    pub tokens_per_second: f64,
}

impl TelemetryPayload {
    /// Constructs a [`TelemetryPayload`] from a [`TokenMetrics`] object.
    pub fn from_metrics(
        model_used: impl Into<String>,
        backend_type: BackendType,
        vram_allocated_bytes: u64,
        metrics: &TokenMetrics,
    ) -> Self {
        Self {
            model_used: model_used.into(),
            backend_type,
            vram_allocated_bytes,
            prompt_tokens: metrics.prompt_tokens,
            completion_tokens: metrics.completion_tokens,
            reasoning_tokens: metrics.reasoning_tokens,
            estimated_cost_usd: metrics.estimated_cost_usd,
            ttft_ms: metrics.ttft_ms,
            total_latency_ms: metrics.total_duration_ms,
            tokens_per_second: metrics.tokens_per_second,
        }
    }
}

/// Canonical agent execution event schema matching Blueprint §13.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XenoAgentStepEvent {
    /// Unique event identifier.
    pub event_id: Uuid,
    /// UTC timestamp of event creation.
    pub timestamp: DateTime<Utc>,
    /// Workflow session identifier.
    pub session_id: String,
    /// Swarm role executing this step.
    pub agent_role: AgentRole,
    /// Active lifecycle phase.
    pub execution_phase: ExecutionPhase,

    /// Optional cognitive reasoning payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingPayload>,

    /// Optional tool call invocation payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<ToolCallPayload>,

    /// Optional tool observation output payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation: Option<ObservationPayload>,

    /// Execution telemetry & resource consumption.
    pub telemetry: TelemetryPayload,
}

impl XenoAgentStepEvent {
    /// Constructs a new thinking phase event.
    pub fn thinking_step(
        session_id: impl Into<String>,
        agent_role: AgentRole,
        thinking: ThinkingPayload,
        telemetry: TelemetryPayload,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_id: session_id.into(),
            agent_role,
            execution_phase: ExecutionPhase::Thinking,
            thinking: Some(thinking),
            tool_call: None,
            observation: None,
            telemetry,
        }
    }

    /// Constructs a new tool call phase event.
    pub fn tool_call_step(
        session_id: impl Into<String>,
        agent_role: AgentRole,
        tool_call: ToolCallPayload,
        telemetry: TelemetryPayload,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_id: session_id.into(),
            agent_role,
            execution_phase: ExecutionPhase::ToolCall,
            thinking: None,
            tool_call: Some(tool_call),
            observation: None,
            telemetry,
        }
    }

    /// Constructs a new observation phase event.
    pub fn observation_step(
        session_id: impl Into<String>,
        agent_role: AgentRole,
        observation: ObservationPayload,
        telemetry: TelemetryPayload,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_id: session_id.into(),
            agent_role,
            execution_phase: ExecutionPhase::Observation,
            thinking: None,
            tool_call: None,
            observation: Some(observation),
            telemetry,
        }
    }

    /// Constructs a new verified completion event.
    pub fn verified_step(
        session_id: impl Into<String>,
        agent_role: AgentRole,
        telemetry: TelemetryPayload,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_id: session_id.into(),
            agent_role,
            execution_phase: ExecutionPhase::Verified,
            thinking: None,
            tool_call: None,
            observation: None,
            telemetry,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roles_and_phases_as_str() {
        assert_eq!(AgentRole::Commander.as_str(), "commander");
        assert_eq!(AgentRole::Architect.as_str(), "architect");
        assert_eq!(AgentRole::Coder.as_str(), "coder");
        assert_eq!(AgentRole::Tester.as_str(), "tester");
        assert_eq!(AgentRole::Security.as_str(), "security");

        assert_eq!(ExecutionPhase::Thinking.as_str(), "thinking");
        assert_eq!(ExecutionPhase::ToolCall.as_str(), "tool_call");
        assert_eq!(ExecutionPhase::Observation.as_str(), "observation");
        assert_eq!(ExecutionPhase::Reflection.as_str(), "reflection");
        assert_eq!(ExecutionPhase::Verified.as_str(), "verified");
    }

    #[test]
    fn test_tool_call_payload_constructors() {
        let tc1 = ToolCallPayload::new("c1", "tool_a", serde_json::json!({"x": 1}));
        assert_eq!(tc1.call_id, "c1");
        assert_eq!(tc1.mcp_server, None);

        let tc2 = ToolCallPayload::with_mcp("c2", "tool_b", serde_json::json!({"y": 2}), "server_b");
        assert_eq!(tc2.mcp_server.as_deref(), Some("server_b"));
    }

    #[test]
    fn test_observation_payload_failure() {
        let obs = ObservationPayload::failure(1, "syntax error in file");
        assert_eq!(obs.exit_code, 1);
        assert_eq!(obs.stderr, "syntax error in file");
        assert!(!obs.ast_validation_passed);
    }
}

