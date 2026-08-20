//! Error taxonomy and structured failure models for XENO INFERENCE.

use thiserror::Error;

/// Result type alias using [`XenoError`].
pub type Result<T> = std::result::Result<T, XenoError>;

/// Root error enum encompassing all system, provider, agent, tool, and DAG failures.
#[derive(Debug, Error)]
pub enum XenoError {
    #[error("Inference failure: {0}")]
    Inference(#[from] InferenceError),

    #[error("Tool execution failure: {0}")]
    Tool(#[from] ToolError),

    #[error("Agent harness failure: {0}")]
    Agent(#[from] AgentError),

    #[error("DAG execution failure: {0}")]
    DAG(#[from] DAGError),

    #[error("Authentication failed for provider '{provider}': {message}")]
    Authentication {
        provider: String,
        message: String,
    },

    #[error("Rate limit exceeded for provider '{provider}': retry after {retry_after_secs:?} seconds")]
    RateLimit {
        provider: String,
        retry_after_secs: Option<u64>,
    },

    #[error("Context length exceeded: requested {requested_tokens} tokens, max allowed is {max_tokens}")]
    ContextLengthExceeded {
        requested_tokens: u32,
        max_tokens: u32,
    },

    #[error("Model '{model}' not found or unavailable on provider '{provider}'")]
    ModelNotFound {
        provider: String,
        model: String,
    },

    #[error("Provider '{provider}' upstream error ({status_code}): {message}")]
    UpstreamError {
        provider: String,
        status_code: u16,
        message: String,
    },

    #[error("Inference request timed out after {timeout_ms}ms")]
    Timeout {
        timeout_ms: u64,
    },

    #[error("Network connection error: {message}")]
    NetworkError {
        message: String,
    },

    #[error("Air-gap policy violation: outgoing network connection blocked in mode '{mode}' to target '{target}'")]
    AirGapViolation {
        mode: String,
        target: String,
    },

    #[error("Privacy filter triggered: sensitive secret detected ({rule_name}) in prompt payload")]
    PrivacyViolation {
        rule_name: String,
    },

    #[error("Stream interrupted prematurely: {reason}")]
    StreamInterrupted {
        reason: String,
    },

    #[error("Serialization / Deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid request configuration: {0}")]
    InvalidRequest(String),

    #[error("Internal engine error: {0}")]
    Internal(String),
}

impl XenoError {
    /// Determines if an error is transient and can be retried.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::RateLimit { .. } => true,
            Self::Timeout { .. } => true,
            Self::NetworkError { .. } => true,
            Self::StreamInterrupted { .. } => true,
            Self::UpstreamError { status_code, .. } => *status_code >= 500 || *status_code == 429,
            Self::Inference(InferenceError::RequestFailed { status, .. }) => {
                *status >= 500 || *status == 429
            }
            Self::Inference(InferenceError::StreamingFailed(_)) => true,
            Self::Tool(ToolError::Timeout { .. }) => true,
            _ => false,
        }
    }

    /// Returns a machine-readable static error code string.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Inference(_) => "INFERENCE_ERROR",
            Self::Tool(_) => "TOOL_ERROR",
            Self::Agent(_) => "AGENT_ERROR",
            Self::DAG(_) => "DAG_ERROR",
            Self::Authentication { .. } => "AUTH_FAILED",
            Self::RateLimit { .. } => "RATE_LIMIT_EXCEEDED",
            Self::ContextLengthExceeded { .. } => "CONTEXT_OVERFLOW",
            Self::ModelNotFound { .. } => "MODEL_NOT_FOUND",
            Self::UpstreamError { .. } => "UPSTREAM_ERROR",
            Self::Timeout { .. } => "REQUEST_TIMEOUT",
            Self::NetworkError { .. } => "NETWORK_ERROR",
            Self::AirGapViolation { .. } => "AIRGAP_VIOLATION",
            Self::PrivacyViolation { .. } => "PRIVACY_VIOLATION",
            Self::StreamInterrupted { .. } => "STREAM_INTERRUPTED",
            Self::JsonError(_) => "JSON_PARSE_ERROR",
            Self::InvalidRequest(_) => "INVALID_REQUEST",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Convenience constructor for authentication errors.
    pub fn auth(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Authentication {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Convenience constructor for rate limit errors.
    pub fn rate_limit(provider: impl Into<String>, retry_after_secs: Option<u64>) -> Self {
        Self::RateLimit {
            provider: provider.into(),
            retry_after_secs,
        }
    }

    /// Convenience constructor for upstream errors.
    pub fn upstream(provider: impl Into<String>, status_code: u16, message: impl Into<String>) -> Self {
        Self::UpstreamError {
            provider: provider.into(),
            status_code,
            message: message.into(),
        }
    }
}

/// Errors occurring during model inference and token streaming.
#[derive(Debug, Error)]
pub enum InferenceError {
    #[error("Provider '{0}' is not registered or supported")]
    ProviderNotFound(String),

    #[error("Model '{model}' is unavailable: {reason}")]
    ModelUnavailable {
        model: String,
        reason: String,
    },

    #[error("Token streaming failed: {0}")]
    StreamingFailed(String),

    #[error("Token budget limit exceeded: current {current}, max {limit}")]
    TokenLimitExceeded {
        current: u32,
        limit: u32,
    },

    #[error("Provider '{provider}' does not support feature '{feature}'")]
    UnsupportedFeature {
        provider: String,
        feature: String,
    },

    #[error("Malformed provider response: {0}")]
    MalformedResponse(String),

    #[error("Inference HTTP request failed with status {status}: {message}")]
    RequestFailed {
        status: u16,
        message: String,
    },
}

/// Errors occurring during tool invocation, sandboxing, or AST validation.
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool '{0}' was not found in registry")]
    NotFound(String),

    #[error("Permission denied executing tool '{tool}' (Tier {tier}): {reason}")]
    PermissionDenied {
        tool: String,
        tier: String,
        reason: String,
    },

    #[error("Tool '{tool}' execution failed: {error}")]
    ExecutionFailed {
        tool: String,
        error: String,
    },

    #[error("Tool '{tool}' execution timed out after {timeout_ms}ms")]
    Timeout {
        tool: String,
        timeout_ms: u64,
    },

    #[error("Invalid arguments for tool '{tool}': {details}")]
    InvalidArguments {
        tool: String,
        details: String,
    },

    #[error("AST syntax validation failed on file '{file}': {reason}")]
    AstValidationFailed {
        file: String,
        reason: String,
    },

    #[error("Sandbox security violation for tool '{tool}': {reason}")]
    SandboxViolation {
        tool: String,
        reason: String,
    },

    #[error("Process was terminated with signal/code '{signal_or_code}' during execution of tool '{tool}'")]
    ProcessKilled {
        tool: String,
        signal_or_code: String,
    },
}

/// Errors occurring within the autonomous agent harness (PAORV) and swarm orchestrator.
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Goal planning failed: {0}")]
    GoalPlanningFailed(String),

    #[error("Maximum step iterations reached ({iterations}/{max})")]
    MaxIterationsReached {
        iterations: u32,
        max: u32,
    },

    #[error("Swarm consensus check failed (agreement: {agreement_pct:.1}%, required: {required_pct:.1}%): {reason}")]
    ConsensusFailed {
        agreement_pct: f32,
        required_pct: f32,
        reason: String,
    },

    #[error("Agent memory subsystem error: {0}")]
    MemoryError(String),

    #[error("Role transition from '{from}' to '{to}' failed: {reason}")]
    RoleTransitionFailed {
        from: String,
        to: String,
        reason: String,
    },

    #[error("Verification gate failed: {0}")]
    VerificationFailed(String),

    #[error("Self-healing loop exhausted after {attempts} attempts: {error}")]
    SelfHealingExhausted {
        attempts: u32,
        error: String,
    },
}

/// Errors occurring in dynamic DAG construction, execution, and grafting.
#[derive(Debug, Error)]
pub enum DAGError {
    #[error("Cycle detected in execution graph: {0}")]
    CycleDetected(String),

    #[error("Node '{0}' not found in DAG")]
    NodeNotFound(String),

    #[error("Unmet dependency for node '{node_id}': '{unmet_dependency}' has not succeeded")]
    DependencyUnmet {
        node_id: String,
        unmet_dependency: String,
    },

    #[error("Invalid node state transition for '{node_id}' from '{from}' to '{to}'")]
    InvalidTransition {
        node_id: String,
        from: String,
        to: String,
    },

    #[error("DAG execution failed at node '{node_id}': {reason}")]
    ExecutionFailed {
        node_id: String,
        reason: String,
    },

    #[error("Dynamic subgraph grafting failed: {0}")]
    GraftingFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_and_codes() {
        let err = XenoError::ModelNotFound {
            provider: "groq".into(),
            model: "unknown-model".into(),
        };
        assert_eq!(err.error_code(), "MODEL_NOT_FOUND");
        assert!(err.to_string().contains("groq"));

        let ctx_err = XenoError::ContextLengthExceeded {
            requested_tokens: 150_000,
            max_tokens: 128_000,
        };
        assert_eq!(ctx_err.error_code(), "CONTEXT_OVERFLOW");

        let priv_err = XenoError::PrivacyViolation {
            rule_name: "aws_access_key".into(),
        };
        assert_eq!(priv_err.error_code(), "PRIVACY_VIOLATION");

        let inv_err = XenoError::InvalidRequest("temperature out of range".into());
        assert_eq!(inv_err.error_code(), "INVALID_REQUEST");

        let int_err = XenoError::Internal("corrupted memory buffer".into());
        assert_eq!(int_err.error_code(), "INTERNAL_ERROR");
    }
}

