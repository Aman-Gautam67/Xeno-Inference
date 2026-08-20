//! # XENO CORE
//!
//! Core primitives, strongly-typed data contracts, event schemas, error taxonomy,
//! telemetry metrics, and graph lifecycle models for the **XENO INFERENCE** platform.
//!
//! This crate establishes the foundational contracts shared across all XENO subsystems:
//! - [`contracts`]: Universal chat messages, multimodal content blocks, tool schemas, and inference requests/responses.
//! - [`events`]: `XenoAgentStepEvent` stream telemetry adhering to Blueprint §13.
//! - [`errors`]: Comprehensive error hierarchy ([`errors::XenoError`], [`errors::InferenceError`], etc.).
//! - [`metrics`]: [`metrics::TokenMetrics`], [`metrics::HardwareStats`], and cost calculation catalog.
//! - [`types`]: [`types::XenoDAGNode`], node lifecycle statuses, security tiers, and provider kind classifications.

pub mod contracts;
pub mod errors;
pub mod events;
pub mod metrics;
pub mod types;

/// Convenient re-exports of common primitives and contracts.
pub mod prelude {
    pub use crate::contracts::{
        ChatMessage, ContentBlock, InferenceRequest, InferenceResponse, MessageRole, PrivacyFilter,
        ProviderConfig, Role, StopReason, StreamChunk, StreamChunkDelta, ToolDefinition,
    };
    pub use crate::errors::{
        AgentError, DAGError, InferenceError, Result, ToolError, XenoError,
    };
    pub use crate::events::{
        AgentRole, BackendType, ExecutionPhase, ObservationPayload, TelemetryPayload,
        ThinkingPayload, ToolCallPayload, XenoAgentStepEvent,
    };
    pub use crate::metrics::{HardwareStats, ModelPricing, PricingCatalog, TokenMetrics};
    pub use crate::types::{
        DAGNodeStatus, DAGNodeType, EventId, ModelAssignment, NodeId, NodeStatus, NodeType,
        ProviderKind, SessionId, ToolSecurityTier, XenoDAGNode,
    };
}

pub use prelude::*;
