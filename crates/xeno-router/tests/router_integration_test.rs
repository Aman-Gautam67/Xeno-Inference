//! Integration tests for Semantic Router, Token Bus, Velocity, Pricing, and Privacy Scrubber.

use futures_util::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use xeno_core::{
    contracts::{ChatMessage, InferenceRequest, PrivacyFilter},
    errors::XenoError,
    types::ProviderKind,
};
use xeno_router::prelude::*;

#[tokio::test]
async fn test_semantic_router_multi_policy_dispatch() {
    let mut router = SemanticRouter::new();

    let mock_fast = Arc::new(MockProvider::new("fast-mock", MockConfig::default()).with_text("fast response"));
    let mock_reasoning = Arc::new(MockProvider::new("reasoning-mock", MockConfig::default()).with_text("reasoning response"));
    let mock_local = Arc::new(MockProvider::new("local-mock", MockConfig::default()).with_text("local response"));

    router.register_provider(mock_fast);
    router.register_provider(mock_reasoning);
    router.register_provider(mock_local);

    let req = InferenceRequest::new("model-a", vec![ChatMessage::user_text("hello")]);

    // Test Speed Priority
    let provider = router.select_provider(&req, RoutingPolicy::SpeedPriority).unwrap();
    assert_eq!(provider.provider_kind(), ProviderKind::Mock);

    // Test Privacy Guard
    let provider = router.select_provider(&req, RoutingPolicy::PrivacyGuard).unwrap();
    assert_eq!(provider.provider_kind(), ProviderKind::Mock);
}

#[tokio::test]
async fn test_router_complete_flow_with_pricing_and_privacy() {
    let mut router = SemanticRouter::new();
    let mock = Arc::new(
        MockProvider::new("mock", MockConfig::default())
            .with_text("Sanitized output")
            .with_thinking("Processing secrets..."),
    );
    router.register_provider(mock.clone());

    // Request containing sensitive PII and secrets
    let prompt = "My AWS key is AKIAIOSFODNN7EXAMPLE and secret is sk-proj-1234567890abcdefghijklmn and IP is 192.168.1.50";
    let req = InferenceRequest::new("claude-3-7-sonnet-20250219", vec![ChatMessage::user_text(prompt)]);

    let resp = router.complete(req, RoutingPolicy::ReasoningPriority).await.unwrap();

    assert_eq!(resp.text_content(), "Sanitized output");
    assert_eq!(resp.thinking_content().as_deref(), Some("Processing secrets..."));
    assert!(resp.metrics.estimated_cost_usd > 0.0 || resp.metrics.prompt_tokens > 0);

    // Verify upstream received sanitized prompt
    let recorded = mock.recorded_requests();
    assert_eq!(recorded.len(), 1);
    let sent_text = recorded[0].messages[0].text_content();
    assert!(sent_text.contains("[REDACTED:AWS_ACCESS_KEY]"));
    assert!(sent_text.contains("[REDACTED:OPENAI_KEY]"));
    assert!(sent_text.contains("[REDACTED:INTERNAL_IP]"));
    assert!(!sent_text.contains("AKIAIOSFODNN7EXAMPLE"));
}

#[tokio::test]
async fn test_router_stream_with_token_bus_broadcasting() {
    let mut router = SemanticRouter::new();
    let mock = Arc::new(
        MockProvider::new("mock", MockConfig {
            chunk_delay: Duration::from_millis(5),
            chunk_size: 8,
            ..MockConfig::default()
        })
        .with_thinking("Planning steps...")
        .with_text("Step 1 complete. Step 2 complete."),
    );
    router.register_provider(mock);

    let mut bus_sub = router.token_bus().subscribe();

    let req = InferenceRequest::new("gpt-4o", vec![ChatMessage::user_text("Run pipeline")]);
    let mut stream = router.stream(req, RoutingPolicy::SpeedPriority).await.unwrap();

    let mut collected_chunks = Vec::new();
    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.unwrap();
        collected_chunks.push(chunk);
    }

    assert!(!collected_chunks.is_empty());

    // Ensure subscriber received every broadcasted chunk
    let mut sub_count = 0;
    while let Ok(_chunk) = bus_sub.try_recv() {
        sub_count += 1;
    }
    assert_eq!(sub_count, collected_chunks.len());
}

#[tokio::test]
async fn test_router_air_gap_violation_rejection() {
    let mut router = SemanticRouter::new();
    // Register only Anthropic cloud provider
    let anthropic = Arc::new(AnthropicProvider::new("mock-key"));
    router.register_provider(anthropic);

    let req = InferenceRequest::new("claude-3-7-sonnet-20250219", vec![ChatMessage::user_text("test")]);

    // Routing with PrivacyGuard when only cloud provider is available must fail with AirGapViolation
    match router.select_provider(&req, RoutingPolicy::PrivacyGuard) {
        Err(XenoError::AirGapViolation { .. }) => {}
        Err(other) => panic!("Expected AirGapViolation, got: {other:?}"),
        Ok(_) => panic!("Expected AirGapViolation error"),
    }
}

#[tokio::test]
async fn test_velocity_calculator_sliding_window_and_ema() {
    let mut calc = TokenVelocityCalculator::new(Duration::from_millis(200), 0.3);

    calc.record_tokens(50);
    sleep(Duration::from_millis(50)).await;
    calc.record_tokens(50);

    let vel = calc.current_velocity();
    assert!(vel > 0.0);

    let ema = calc.ema_velocity();
    assert!(ema > 0.0);

    assert_eq!(calc.total_tokens(), 100);
}

#[tokio::test]
async fn test_privacy_scrubber_custom_patterns() {
    let scrubber = PrivacyScrubber::new();
    let mut filter = PrivacyFilter::default();
    filter.custom_redaction_patterns.push(r"CONFIDENTIAL_PROJECT_\d+".to_string());

    let raw = "Deploying CONFIDENTIAL_PROJECT_9921 to production.";
    let sanitized = scrubber.sanitize_text(raw, &filter);
    assert_eq!(sanitized, "Deploying [REDACTED:CUSTOM] to production.");
}
