//! Adversarial, edge-case, and fault-injection tests for xeno-router.

use futures_util::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use xeno_core::contracts::{ChatMessage, InferenceRequest, PrivacyFilter};
use xeno_router::prelude::*;

#[tokio::test]
async fn test_adversarial_pii_and_secret_redaction_complex() {
    let scrubber = PrivacyScrubber::new();
    let filter = PrivacyFilter::default();

    let text_with_multiple_secrets = r#"
    Here is an RSA key:
    -----BEGIN RSA PRIVATE KEY-----
    MIIEowIBAAKCAQEA0Y1+
    FakeKeyData1234567890abcdef
    -----END RSA PRIVATE KEY-----

    And an OpenAI project key: sk-proj-abcdef1234567890abcdef1234567890abcdef1234567890
    And an Anthropic key: sk-ant-api03-abcdef1234567890abcdef1234567890abcdef1234567890-abcdef
    And a GitHub PAT: ghp_123456789012345678901234567890123456
    And a fine-grained token: github_pat_11AAAAAAA0123456789012345678901234567890123456789012345678901234567890123456789012
    And a JWT token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.doNotLeakThisSignature12345
    And an internal IP: 10.0.4.12
    "#;

    let sanitized = scrubber.sanitize_text(text_with_multiple_secrets, &filter);

    assert!(sanitized.contains("[REDACTED:PRIVATE_SSH_KEY]"));
    assert!(sanitized.contains("[REDACTED:OPENAI_KEY]"));
    assert!(sanitized.contains("[REDACTED:ANTHROPIC_KEY]"));
    assert!(sanitized.contains("[REDACTED:GITHUB_PAT]"));
    assert!(sanitized.contains("[REDACTED:JWT_TOKEN]"));
    assert!(sanitized.contains("[REDACTED:INTERNAL_IP]"));

    assert!(!sanitized.contains("BEGIN RSA PRIVATE KEY"));
    assert!(!sanitized.contains("sk-proj-"));
    assert!(!sanitized.contains("sk-ant-"));
    assert!(!sanitized.contains("ghp_"));
    assert!(!sanitized.contains("eyJhbGciOiJIUzI1Ni"));
    assert!(!sanitized.contains("10.0.4.12"));
}

#[tokio::test]
async fn test_adversarial_token_bus_dropped_subscribers() {
    let bus = TokenBus::new(16);

    // Create a subscriber and immediately drop it
    {
        let _sub = bus.subscribe();
    }

    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let upstream: BoxStream<_> = Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx));

    tokio::spawn(async move {
        for i in 0..10 {
            tx.send(Ok(xeno_core::contracts::StreamChunk::text(i, format!("Chunk {i}"))))
                .await
                .unwrap();
        }
    });

    let mut wrapped = bus.wrap_stream("model-x".into(), upstream, 50, None);

    let mut count = 0;
    while let Some(chunk_res) = wrapped.next().await {
        let chunk = chunk_res.unwrap();
        assert!(chunk.partial_metrics.is_some());
        count += 1;
    }

    assert_eq!(count, 10);
}

#[tokio::test]
async fn test_adversarial_velocity_bursts() {
    let mut calc = TokenVelocityCalculator::new(Duration::from_millis(100), 0.5);

    // Burst 10,000 tokens
    calc.record_tokens(10_000);
    assert_eq!(calc.total_tokens(), 10_000);
    assert!(calc.current_velocity() > 0.0);
    assert!(calc.ema_velocity() > 0.0);

    // Zero tokens record
    calc.record_tokens(0);
    assert_eq!(calc.total_tokens(), 10_000);
}

#[tokio::test]
async fn test_adversarial_router_fallback_on_injected_error() {
    let mut router = SemanticRouter::new();

    // Primary mock will fail
    let failing_mock = Arc::new(MockProvider::new("failing", MockConfig::default()));
    failing_mock.inject_error("Temporary upstream 503 service unavailable");

    // Fallback mock will succeed
    let healthy_mock = Arc::new(MockProvider::new("healthy", MockConfig::default()).with_text("Recovered from fallback!"));

    router.register_provider(failing_mock);
    router.register_provider(healthy_mock);
    router.set_fallback_chain(vec![ProviderKind::Mock]);

    let req = InferenceRequest::new("model-fallback", vec![ChatMessage::user_text("test")]);
    let resp = router.complete(req, RoutingPolicy::SpeedPriority).await.unwrap();

    assert_eq!(resp.text_content(), "Recovered from fallback!");
}
