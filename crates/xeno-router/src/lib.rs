//! # XENO ROUTER
//!
//! Unified Multi-Provider Inference Router, Streaming Token Bus, Real-Time Velocity & Cost Engine,
//! Privacy & Secret Redaction Scrubber, and Air-Gap Enforcer for the **XENO INFERENCE** platform.
//!
//! This crate implements Features F06–F19:
//! - [`provider`]: Unified asynchronous [`InferenceProvider`] trait, [`ModelInfo`], and [`HealthStatus`].
//! - [`providers`]: Concrete backend adapters for Local OpenAI (llama.cpp/vLLM/Ollama), Anthropic Claude 3.7 Messages API v1, OpenAI, Gemini, Groq, DeepSeek V3/R1, and Mock testing.
//! - [`token_bus`]: Real-time Tokio async streaming [`TokenBus`] with monotonic TTFT and chunk broadcasting.
//! - [`velocity`]: [`TokenVelocityCalculator`] implementing sliding window and Exponential Moving Average (EMA) tokens/sec tracking.
//! - [`pricing`]: Real-time USD [`CostEstimator`] for prompt, completion, and reasoning tokens.
//! - [`privacy`]: Pre-flight [`PrivacyScrubber`] and socket-level [`AirGapEnforcer`].
//! - [`router`]: Multi-policy [`SemanticRouter`] supporting Speed, Reasoning, Privacy, and Cost optimization policies.

pub mod pricing;
pub mod privacy;
pub mod provider;
pub mod providers;
pub mod router;
pub mod token_bus;
pub mod velocity;

/// Convenient re-exports for the XENO router subsystem.
pub mod prelude {
    pub use crate::pricing::CostEstimator;
    pub use crate::privacy::{AirGapEnforcer, PrivacyScrubber};
    pub use crate::provider::{BoxStream, HealthStatus, InferenceProvider, ModelInfo};
    pub use crate::providers::{
        AnthropicProvider, DeepSeekProvider, GeminiProvider, GroqProvider, LocalOpenAIProvider,
        MockConfig, MockProvider, OpenAIProvider, SseEvent,
    };
    pub use crate::router::{RoutingPolicy, SemanticRouter};
    pub use crate::token_bus::TokenBus;
    pub use crate::velocity::TokenVelocityCalculator;
    pub use xeno_core::types::ProviderKind;
}

pub use prelude::*;
