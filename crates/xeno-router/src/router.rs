//! Semantic Router and multi-policy dynamic dispatch engine.

use crate::pricing::CostEstimator;
use crate::privacy::{AirGapEnforcer, PrivacyScrubber};
use crate::provider::{BoxStream, HealthStatus, InferenceProvider};
use crate::token_bus::TokenBus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use xeno_core::{
    contracts::{InferenceRequest, InferenceResponse, StreamChunk},
    errors::XenoError,
    types::ProviderKind,
};

/// Routing policy criteria determining model/provider selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingPolicy {
    /// Routes to ultra-low latency LPU or local 8B models (<50ms TTFT).
    SpeedPriority,
    /// Routes to heavy reasoning models (Claude 3.7 Thinking, o1, DeepSeek R1).
    ReasoningPriority,
    /// Strictly routes to 100% air-gapped local runtimes, redacting sensitive tokens.
    PrivacyGuard,
    /// Minimizes overall financial token expense based on pricing catalog.
    CostOptimizer,
}

/// Dynamic semantic router managing provider registration, policy dispatch, privacy sanitization, and streaming.
#[derive(Clone)]
pub struct SemanticRouter {
    providers: HashMap<ProviderKind, Arc<dyn InferenceProvider>>,
    fallback_chain: Vec<ProviderKind>,
    token_bus: TokenBus,
    cost_estimator: CostEstimator,
    privacy_scrubber: PrivacyScrubber,
    air_gap_enforcer: AirGapEnforcer,
}

impl Default for SemanticRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticRouter {
    /// Constructs a new [`SemanticRouter`].
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            fallback_chain: vec![
                ProviderKind::Anthropic,
                ProviderKind::Openai,
                ProviderKind::Deepseek,
                ProviderKind::Groq,
                ProviderKind::Google,
                ProviderKind::Local,
                ProviderKind::Mock,
            ],
            token_bus: TokenBus::default(),
            cost_estimator: CostEstimator::default(),
            privacy_scrubber: PrivacyScrubber::new(),
            air_gap_enforcer: AirGapEnforcer::new(),
        }
    }

    /// Registers an inference provider backend.
    pub fn register_provider(&mut self, provider: Arc<dyn InferenceProvider>) {
        self.providers.insert(provider.provider_kind(), provider);
    }

    /// Registers a provider with builder pattern.
    pub fn with_provider(mut self, provider: Arc<dyn InferenceProvider>) -> Self {
        self.register_provider(provider);
        self
    }

    /// Configures custom fallback provider hierarchy.
    pub fn set_fallback_chain(&mut self, chain: Vec<ProviderKind>) {
        self.fallback_chain = chain;
    }

    /// Returns a reference to the shared token bus.
    pub fn token_bus(&self) -> &TokenBus {
        &self.token_bus
    }

    /// Returns a reference to the cost estimator.
    pub fn cost_estimator(&self) -> &CostEstimator {
        &self.cost_estimator
    }

    /// Selects the best available provider given an inference request and routing policy.
    pub fn select_provider(
        &self,
        req: &InferenceRequest,
        policy: RoutingPolicy,
    ) -> Result<Arc<dyn InferenceProvider>, XenoError> {
        // 1. If privacy guard policy is active or air-gap filter is enabled, enforce local only
        let air_gap = policy == RoutingPolicy::PrivacyGuard
            || req
                .privacy_filter
                .as_ref()
                .map(|f| f.air_gap_mode)
                .unwrap_or(false);

        if air_gap {
            if let Some(provider) = self.providers.get(&ProviderKind::Local) {
                return Ok(provider.clone());
            }
            if let Some(provider) = self.providers.get(&ProviderKind::Mock) {
                return Ok(provider.clone());
            }
            return Err(XenoError::AirGapViolation {
                mode: "AirGapEnforced".into(),
                target: "No local or mock provider registered for air-gap routing".into(),
            });
        }

        // 2. Policy-specific selection logic
        match policy {
            RoutingPolicy::SpeedPriority => {
                // Prefer Groq, then Local, then Mock
                for kind in [
                    ProviderKind::Groq,
                    ProviderKind::Local,
                    ProviderKind::Google,
                    ProviderKind::Mock,
                ] {
                    if let Some(provider) = self.providers.get(&kind) {
                        return Ok(provider.clone());
                    }
                }
            }
            RoutingPolicy::ReasoningPriority => {
                // Prefer Claude 3.7 Thinking, o1/o3, DeepSeek R1
                for kind in [
                    ProviderKind::Anthropic,
                    ProviderKind::Deepseek,
                    ProviderKind::Openai,
                    ProviderKind::Google,
                ] {
                    if let Some(provider) = self.providers.get(&kind) {
                        return Ok(provider.clone());
                    }
                }
            }
            RoutingPolicy::CostOptimizer => {
                // Prefer Local (free), then DeepSeek ($0.14), then Groq, then Google, then OpenAI, then Anthropic
                for kind in [
                    ProviderKind::Local,
                    ProviderKind::Mock,
                    ProviderKind::Deepseek,
                    ProviderKind::Groq,
                    ProviderKind::Google,
                    ProviderKind::Openai,
                    ProviderKind::Anthropic,
                ] {
                    if let Some(provider) = self.providers.get(&kind) {
                        return Ok(provider.clone());
                    }
                }
            }
            RoutingPolicy::PrivacyGuard => unreachable!(),
        }

        // 3. Fallback to any registered provider in fallback chain
        for kind in &self.fallback_chain {
            if let Some(provider) = self.providers.get(kind) {
                return Ok(provider.clone());
            }
        }

        Err(XenoError::Internal(
            "No matching inference provider found in registry".into(),
        ))
    }

    /// Executes complete inference with pre-flight privacy redaction and automatic fallback.
    pub async fn complete(
        &self,
        mut req: InferenceRequest,
        policy: RoutingPolicy,
    ) -> Result<InferenceResponse, XenoError> {
        let filter = req.privacy_filter.clone().unwrap_or_default();

        // 1. Sanitize request text
        self.privacy_scrubber.sanitize_request(&mut req, &filter);

        // 2. Select primary provider
        let provider = self.select_provider(&req, policy)?;

        // 3. Validate air-gap constraints
        self.air_gap_enforcer
            .validate_provider_kind(provider.provider_kind(), filter.air_gap_mode)?;

        // 4. Execute with retry/fallback
        match provider.complete(&req).await {
            Ok(mut resp) => {
                self.cost_estimator.enrich_metrics(&resp.model, &mut resp.metrics);
                Ok(resp)
            }
            Err(err) if err.is_retryable() => {
                // Try fallback providers
                for kind in &self.fallback_chain {
                    if *kind == provider.provider_kind() {
                        continue;
                    }
                    if let Some(fallback) = self.providers.get(kind) {
                        if filter.air_gap_mode && fallback.provider_kind().is_cloud() {
                            continue;
                        }
                        if let Ok(mut fallback_resp) = fallback.complete(&req).await {
                            self.cost_estimator
                                .enrich_metrics(&fallback_resp.model, &mut fallback_resp.metrics);
                            return Ok(fallback_resp);
                        }
                    }
                }
                Err(err)
            }
            Err(err) => Err(err),
        }
    }

    /// Initiates streaming inference wrapped by the token bus with real-time velocity and cost tracking.
    pub async fn stream(
        &self,
        mut req: InferenceRequest,
        policy: RoutingPolicy,
    ) -> Result<BoxStream<StreamChunk>, XenoError> {
        let filter = req.privacy_filter.clone().unwrap_or_default();

        // 1. Sanitize request text
        self.privacy_scrubber.sanitize_request(&mut req, &filter);

        // 2. Select primary provider
        let provider = self.select_provider(&req, policy)?;

        // 3. Validate air-gap constraints
        self.air_gap_enforcer
            .validate_provider_kind(provider.provider_kind(), filter.air_gap_mode)?;

        let prompt_tokens = req
            .messages
            .iter()
            .map(|m| (m.text_content().len() / 4).max(1) as u32)
            .sum::<u32>()
            .max(10);

        let upstream = provider.stream(&req).await?;

        // Wrap stream in TokenBus for live TTFT, velocity, metrics enrichment and broadcast
        let wrapped = self.token_bus.wrap_stream(
            req.model.clone(),
            upstream,
            prompt_tokens,
            Some(self.cost_estimator.clone()),
        );

        Ok(wrapped)
    }

    /// Performs health checks across all registered providers.
    pub async fn health_check_all(&self) -> HashMap<ProviderKind, HealthStatus> {
        let mut results = HashMap::new();
        for (&kind, provider) in &self.providers {
            let status = provider
                .health_check()
                .await
                .unwrap_or_else(|e| HealthStatus::Unhealthy {
                    reason: e.to_string(),
                });
            results.insert(kind, status);
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::mock::{MockConfig, MockProvider};
    use xeno_core::contracts::ChatMessage;

    #[tokio::test]
    async fn test_semantic_router_privacy_guard_enforces_local() {
        let mut router = SemanticRouter::new();
        let mock = Arc::new(MockProvider::new("mock", MockConfig::default()));
        router.register_provider(mock);

        let req = InferenceRequest::new("test-model", vec![ChatMessage::user_text("test")]);
        let provider = router.select_provider(&req, RoutingPolicy::PrivacyGuard).unwrap();
        assert_eq!(provider.provider_kind(), ProviderKind::Mock);
    }

    #[tokio::test]
    async fn test_semantic_router_complete_with_scrubber() {
        let mut router = SemanticRouter::new();
        let mock = Arc::new(MockProvider::new("mock", MockConfig::default()).with_text("pong"));
        router.register_provider(mock.clone());

        let req = InferenceRequest::new(
            "test-model",
            vec![ChatMessage::user_text("My key is sk-1234567890abcdef1234567890")],
        );

        let resp = router.complete(req, RoutingPolicy::SpeedPriority).await.unwrap();
        assert_eq!(resp.text_content(), "pong");

        // Verify the recorded request was sanitized
        let recorded = mock.recorded_requests();
        assert_eq!(recorded.len(), 1);
        let sanitized_text = recorded[0].messages[0].text_content();
        assert!(sanitized_text.contains("[REDACTED:OPENAI_KEY]"));
    }
}
