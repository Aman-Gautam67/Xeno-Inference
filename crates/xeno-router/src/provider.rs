//! Unified Inference Provider trait and core provider types.

use async_trait::async_trait;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use xeno_core::{
    contracts::{InferenceRequest, InferenceResponse, StreamChunk},
    errors::XenoError,
    types::ProviderKind,
};

/// Type alias for pinned boxed stream yielding [`StreamChunk`] items or [`XenoError`].
pub type BoxStream<T> = Pin<Box<dyn Stream<Item = Result<T, XenoError>> + Send>>;

/// Model metadata, context windows, and pricing information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    /// Model identifier (e.g., "claude-3-7-sonnet-20250219", "gpt-4o", "deepseek-reasoner").
    pub id: String,
    /// Provider family to which this model belongs.
    pub provider: ProviderKind,
    /// Maximum context window in tokens.
    pub context_window: u32,
    /// Maximum output/completion tokens allowed.
    pub max_output_tokens: u32,
    /// Whether the model supports chunked streaming.
    pub supports_streaming: bool,
    /// Whether the model supports JSON Schema tool/function calling.
    pub supports_tools: bool,
    /// Whether the model produces Chain-of-Thought / reasoning tokens.
    pub supports_reasoning: bool,
    /// Pricing per 1,000,000 input tokens in USD.
    pub input_cost_per_million: f64,
    /// Pricing per 1,000,000 output tokens in USD.
    pub output_cost_per_million: f64,
}

impl ModelInfo {
    /// Constructs a basic [`ModelInfo`] description.
    pub fn new(
        id: impl Into<String>,
        provider: ProviderKind,
        context_window: u32,
        max_output_tokens: u32,
        input_cost_per_million: f64,
        output_cost_per_million: f64,
    ) -> Self {
        Self {
            id: id.into(),
            provider,
            context_window,
            max_output_tokens,
            supports_streaming: true,
            supports_tools: true,
            supports_reasoning: false,
            input_cost_per_million,
            output_cost_per_million,
        }
    }

    /// Sets reasoning capability flag.
    pub fn with_reasoning(mut self, supports_reasoning: bool) -> Self {
        self.supports_reasoning = supports_reasoning;
        self
    }

    /// Sets tools capability flag.
    pub fn with_tools(mut self, supports_tools: bool) -> Self {
        self.supports_tools = supports_tools;
        self
    }
}

/// Operational health and connectivity status of a provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// Provider is fully reachable and operational.
    Healthy,
    /// Provider is operational with degraded performance or elevated latency.
    Degraded { reason: String },
    /// Provider is unreachable, credentials invalid, or endpoint offline.
    Unhealthy { reason: String },
}

impl HealthStatus {
    /// Returns true if the provider is [`HealthStatus::Healthy`].
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Returns true if the provider can accept requests (Healthy or Degraded).
    pub fn is_available(&self) -> bool {
        !matches!(self, Self::Unhealthy { .. })
    }
}

/// Unified asynchronous interface implemented by all local and cloud inference backends.
#[async_trait]
pub trait InferenceProvider: Send + Sync {
    /// Returns the provider family classification.
    fn provider_kind(&self) -> ProviderKind;

    /// Provider instance label or identifier.
    fn name(&self) -> &str;

    /// Alias for [`InferenceProvider::provider_kind`].
    fn provider_type(&self) -> ProviderKind {
        self.provider_kind()
    }

    /// Retrieves capabilities and pricing metadata for a supported model.
    fn model_info(&self, model: &str) -> Result<ModelInfo, XenoError>;

    /// Validates endpoint connectivity, latency, and authentication credentials.
    async fn health_check(&self) -> Result<HealthStatus, XenoError>;

    /// Executes a non-streaming inference request and returns the completed response.
    async fn complete(&self, req: &InferenceRequest) -> Result<InferenceResponse, XenoError>;

    /// Streams an inference response chunk-by-chunk over an async stream.
    async fn stream(&self, req: &InferenceRequest) -> Result<BoxStream<StreamChunk>, XenoError>;

    /// Alias for [`InferenceProvider::complete`].
    async fn infer(&self, req: &InferenceRequest) -> Result<InferenceResponse, XenoError> {
        self.complete(req).await
    }

    /// Alias for [`InferenceProvider::stream`].
    async fn infer_stream(
        &self,
        req: &InferenceRequest,
    ) -> Result<BoxStream<StreamChunk>, XenoError> {
        self.stream(req).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info_builder() {
        let info = ModelInfo::new("claude-3-7-sonnet", ProviderKind::Anthropic, 200_000, 8_192, 3.0, 15.0)
            .with_reasoning(true)
            .with_tools(true);

        assert_eq!(info.id, "claude-3-7-sonnet");
        assert_eq!(info.provider, ProviderKind::Anthropic);
        assert!(info.supports_reasoning);
        assert!(info.supports_tools);
        assert_eq!(info.context_window, 200_000);
    }

    #[test]
    fn test_health_status() {
        let healthy = HealthStatus::Healthy;
        assert!(healthy.is_healthy());
        assert!(healthy.is_available());

        let degraded = HealthStatus::Degraded {
            reason: "High latency".into(),
        };
        assert!(!degraded.is_healthy());
        assert!(degraded.is_available());

        let unhealthy = HealthStatus::Unhealthy {
            reason: "Connection refused".into(),
        };
        assert!(!unhealthy.is_healthy());
        assert!(!unhealthy.is_available());
    }
}
