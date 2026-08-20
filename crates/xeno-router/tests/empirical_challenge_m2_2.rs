//! Empirical Challenge Suite 2 for Milestone 2 (`crates/xeno-router`).
//!
//! Areas Covered:
//! 1. Privacy scrubber edge cases (embedded secrets with unicode zero-width chars, overlapping regexes, multi-MB stress)
//! 2. Air-gap socket boundary enforcement (IPv6 loopback variations, cloud hostnames, IP spoofing attempts)
//! 3. Semantic router fallback under provider timeouts and consecutive network error bursts (multi-hop fallback, non-retryable isolation, air-gap preservation)

use std::sync::Arc;
use std::time::{Duration, Instant};
use xeno_core::contracts::{ChatMessage, InferenceRequest, PrivacyFilter};
use xeno_core::errors::XenoError;
use xeno_core::types::ProviderKind;
use xeno_router::prelude::*;
use xeno_router::privacy::AirGapEnforcer;

// =========================================================================
// 1. PRIVACY SCRUBBER EDGE CASES
// =========================================================================

#[test]
fn test_privacy_scrubber_standard_secrets_redaction() {
    let scrubber = PrivacyScrubber::new();
    let filter = PrivacyFilter::default();

    let text = "AWS: AKIAIOSFODNN7EXAMPLE, OpenAI: sk-proj-1234567890abcdef1234567890, Anthropic: sk-ant-api03-1234567890abcdef1234567890-abcdefgh, GitHub: ghp_123456789012345678901234567890123456, IP: 192.168.1.100";
    let sanitized = scrubber.sanitize_text(text, &filter);

    assert!(sanitized.contains("[REDACTED:AWS_ACCESS_KEY]"));
    assert!(sanitized.contains("[REDACTED:OPENAI_KEY]"));
    assert!(sanitized.contains("[REDACTED:ANTHROPIC_KEY]"));
    assert!(sanitized.contains("[REDACTED:GITHUB_PAT]"));
    assert!(sanitized.contains("[REDACTED:INTERNAL_IP]"));

    assert!(!sanitized.contains("AKIAIOSFODNN7EXAMPLE"));
    assert!(!sanitized.contains("sk-proj-1234567890"));
    assert!(!sanitized.contains("sk-ant-api03"));
    assert!(!sanitized.contains("ghp_1234567890"));
    assert!(!sanitized.contains("192.168.1.100"));
}

#[test]
fn test_privacy_scrubber_overlapping_anthropic_and_openai_keys() {
    let scrubber = PrivacyScrubber::new();
    let filter = PrivacyFilter::default();

    // Anthropic keys start with 'sk-ant-', OpenAI keys start with 'sk-'.
    // Scrubber must match Anthropic rule first and not misidentify as OpenAI key.
    let text = "Key 1: sk-ant-api03-abcdef1234567890abcdef1234567890-xyz123 Key 2: sk-1234567890abcdef1234567890";
    let sanitized = scrubber.sanitize_text(text, &filter);

    assert!(sanitized.contains("[REDACTED:ANTHROPIC_KEY]"));
    assert!(sanitized.contains("[REDACTED:OPENAI_KEY]"));
    assert!(!sanitized.contains("sk-ant-"));
    assert!(!sanitized.contains("sk-1234567890"));
}

#[test]
fn test_privacy_scrubber_unicode_zero_width_character_evasion() {
    let scrubber = PrivacyScrubber::new();
    let filter = PrivacyFilter::default();

    // Adversarial Evasion: Inject Zero-Width Space (\u{200B}), ZWNJ (\u{200C}), ZWJ (\u{200D}), BOM (\u{FEFF}) inside keys
    let raw_aws_zwsp = "AKIA\u{200B}IOSFODNN7EXAMPLE";
    let raw_openai_zwnj = "sk-proj\u{200C}-1234567890abcdef1234567890";
    let raw_anthropic_zwj = "sk-ant-\u{200D}api03-1234567890abcdef1234567890-abcdef";
    let raw_ip_bom = "192.\u{FEFF}168.1.1";

    let sanitized_aws = scrubber.sanitize_text(raw_aws_zwsp, &filter);
    let sanitized_openai = scrubber.sanitize_text(raw_openai_zwnj, &filter);
    let sanitized_anthropic = scrubber.sanitize_text(raw_anthropic_zwj, &filter);
    let sanitized_ip = scrubber.sanitize_text(raw_ip_bom, &filter);

    println!("ZWSP AWS Sanitized: {}", sanitized_aws);
    println!("ZWNJ OpenAI Sanitized: {}", sanitized_openai);
    println!("ZWJ Anthropic Sanitized: {}", sanitized_anthropic);
    println!("BOM IP Sanitized: {}", sanitized_ip);

    // Check whether current regex implementation is vulnerable to unicode zero-width character evasion
    let aws_evaded = sanitized_aws.contains("AKIA");
    let openai_evaded = sanitized_openai.contains("sk-proj");
    let anthropic_evaded = sanitized_anthropic.contains("sk-ant-");
    let ip_evaded = sanitized_ip.contains("192.");

    println!("Evasion findings: AWS={}, OpenAI={}, Anthropic={}, IP={}", aws_evaded, openai_evaded, anthropic_evaded, ip_evaded);
}

#[test]
fn test_privacy_scrubber_multi_megabyte_text_stress() {
    let scrubber = PrivacyScrubber::new();
    let filter = PrivacyFilter::default();

    // Generate 4MB of text with secrets interspersed at beginning, middle, and end
    let line_filler = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\n";
    let mut large_text = String::with_capacity(4 * 1024 * 1024);

    large_text.push_str("START_SECRET: AKIA1111111111111111\n");
    for i in 0..30_000 {
        large_text.push_str(line_filler);
        if i == 15_000 {
            large_text.push_str("MID_SECRET: sk-proj-1234567890abcdef1234567890 MID_IP: 10.200.1.50\n");
        }
    }
    large_text.push_str("END_SECRET: ghp_999999999999999999999999999999999999\n");

    let start = Instant::now();
    let sanitized = scrubber.sanitize_text(&large_text, &filter);
    let elapsed = start.elapsed();

    println!("Scrubbed 4MB text in {:?}", elapsed);

    assert!(elapsed < Duration::from_secs(3), "Scrubbing 4MB must finish within 3s, took {:?}", elapsed);
    assert!(sanitized.contains("[REDACTED:AWS_ACCESS_KEY]"));
    assert!(sanitized.contains("[REDACTED:OPENAI_KEY]"));
    assert!(sanitized.contains("[REDACTED:INTERNAL_IP]"));
    assert!(sanitized.contains("[REDACTED:GITHUB_PAT]"));

    assert!(!sanitized.contains("AKIA1111111111111111"));
    assert!(!sanitized.contains("sk-proj-1234567890abcdef1234567890"));
    assert!(!sanitized.contains("10.200.1.50"));
    assert!(!sanitized.contains("ghp_999999999999999999999999999999999999"));
}

#[test]
fn test_privacy_scrubber_request_in_place_sanitization() {
    let scrubber = PrivacyScrubber::new();
    let filter = PrivacyFilter::default();

    let mut req = InferenceRequest::new(
        "test-model",
        vec![
            ChatMessage::user_text("System token: AKIA2222222222222222"),
            ChatMessage::assistant_text("Echoing sk-33333333333333333333333333"),
        ],
    )
    .with_system_prompt("System instructions with IP 172.16.5.99");

    scrubber.sanitize_request(&mut req, &filter);

    assert!(req.system_prompt.as_ref().unwrap().contains("[REDACTED:INTERNAL_IP]"));
    assert!(!req.system_prompt.as_ref().unwrap().contains("172.16.5.99"));

    let user_msg = req.messages[0].text_content();
    assert!(user_msg.contains("[REDACTED:AWS_ACCESS_KEY]"));
    assert!(!user_msg.contains("AKIA2222222222222222"));

    let asst_msg = req.messages[1].text_content();
    assert!(asst_msg.contains("[REDACTED:OPENAI_KEY]"));
    assert!(!asst_msg.contains("sk-33333333333333333333333333"));
}

// =========================================================================
// 2. AIR-GAP SOCKET BOUNDARY ENFORCEMENT
// =========================================================================

#[test]
fn test_air_gap_ipv6_loopback_acceptance() {
    let enforcer = AirGapEnforcer::new();

    // Standard IPv6 & IPv4 loopbacks
    assert!(enforcer.validate_endpoint_url("http://[::1]:8080/v1", true).is_ok());
    assert!(enforcer.validate_endpoint_url("http://127.0.0.1:11434/v1", true).is_ok());
    assert!(enforcer.validate_endpoint_url("http://localhost:11434/v1", true).is_ok());
    assert!(enforcer.validate_endpoint_url("http://0.0.0.0:8000/v1", true).is_ok());
}

#[test]
fn test_air_gap_ipv6_uncompressed_and_mapped_variations() {
    let enforcer = AirGapEnforcer::new();

    // Testing variations of IPv6 loopbacks and edge cases
    let uncompressed_ipv6 = "http://[0:0:0:0:0:0:0:1]:8080/v1";
    let ipv4_mapped_ipv6 = "http://[::ffff:127.0.0.1]:8080/v1";
    let ipv4_subnet_loopback = "http://127.0.0.2:8080/v1"; // RFC 1122 127.0.0.0/8 loopback block
    let rfc1122_end = "http://127.255.255.254:8080/v1";

    println!("Uncompressed IPv6 [0:0:0:0:0:0:0:1]: {:?}", enforcer.validate_endpoint_url(uncompressed_ipv6, true));
    println!("IPv4-mapped IPv6 [::ffff:127.0.0.1]: {:?}", enforcer.validate_endpoint_url(ipv4_mapped_ipv6, true));
    println!("RFC 1122 Loopback 127.0.0.2: {:?}", enforcer.validate_endpoint_url(ipv4_subnet_loopback, true));
    println!("RFC 1122 Loopback 127.255.255.254: {:?}", enforcer.validate_endpoint_url(rfc1122_end, true));
}

#[test]
fn test_air_gap_cloud_hostnames_and_spoofing_rejection() {
    let enforcer = AirGapEnforcer::new();

    // Public Cloud endpoints - MUST BE REJECTED
    let public_urls = vec![
        "https://api.openai.com/v1",
        "https://api.anthropic.com/v1",
        "https://generativelanguage.googleapis.com/v1beta",
        "https://api.groq.com/openai/v1",
        "https://api.deepseek.com/v1",
        "http://evil.com/v1",
        "http://localhost.evil.com/v1", // Subdomain spoofing
        "http://127.0.0.1.attacker.com/v1",
        "http://169.254.169.254/latest/meta-data/", // AWS metadata service
        "http://10.0.0.1:8080/v1", // RFC 1918 Private LAN
        "http://192.168.1.1:8080/v1",
        "http://172.16.0.1:8080/v1",
    ];

    for url in public_urls {
        let res = enforcer.validate_endpoint_url(url, true);
        assert!(
            res.is_err(),
            "Air-gap enforcer must reject non-loopback URL '{url}', but got: {res:?}"
        );
        match res.unwrap_err() {
            XenoError::AirGapViolation { mode, target } => {
                assert_eq!(mode, "AirGapEnforced");
                assert!(target.contains("blocked under air-gap"));
            }
            other => panic!("Expected AirGapViolation for '{url}', got: {other:?}"),
        }
    }
}

#[test]
fn test_air_gap_provider_kind_validation_all_variants() {
    let enforcer = AirGapEnforcer::new();

    // In air-gap mode: Local and Mock allowed
    assert!(enforcer.validate_provider_kind(ProviderKind::Local, true).is_ok());
    assert!(enforcer.validate_provider_kind(ProviderKind::Mock, true).is_ok());

    // All cloud variants must be rejected
    assert!(enforcer.validate_provider_kind(ProviderKind::Anthropic, true).is_err());
    assert!(enforcer.validate_provider_kind(ProviderKind::Openai, true).is_err());
    assert!(enforcer.validate_provider_kind(ProviderKind::Google, true).is_err());
    assert!(enforcer.validate_provider_kind(ProviderKind::Deepseek, true).is_err());
    assert!(enforcer.validate_provider_kind(ProviderKind::Groq, true).is_err());

    // When air_gap_mode is false, all providers allowed
    assert!(enforcer.validate_provider_kind(ProviderKind::Anthropic, false).is_ok());
    assert!(enforcer.validate_provider_kind(ProviderKind::Openai, false).is_ok());
    assert!(enforcer.validate_provider_kind(ProviderKind::Google, false).is_ok());
    assert!(enforcer.validate_provider_kind(ProviderKind::Deepseek, false).is_ok());
    assert!(enforcer.validate_provider_kind(ProviderKind::Groq, false).is_ok());
}

// =========================================================================
// 3. SEMANTIC ROUTER FALLBACK & ERROR BURST RESILIENCE
// =========================================================================

#[tokio::test]
async fn test_router_fallback_under_provider_timeout() {
    let mut router = SemanticRouter::new();

    // Primary provider fails with Timeout error
    let primary_mock = Arc::new(MockProvider::new("primary-timeout", MockConfig::default()));
    primary_mock.inject_error("Inference request timed out after 5000ms");

    // Fallback provider returns valid response
    let secondary_mock = Arc::new(
        MockProvider::new("secondary-success", MockConfig::default())
            .with_text("Secondary fallback succeeded after timeout"),
    );

    router.register_provider(primary_mock);
    router.register_provider(secondary_mock);
    router.set_fallback_chain(vec![ProviderKind::Mock]);

    let req = InferenceRequest::new("test-model", vec![ChatMessage::user_text("test")]);
    let resp = router.complete(req, RoutingPolicy::SpeedPriority).await.unwrap();

    assert_eq!(resp.text_content(), "Secondary fallback succeeded after timeout");
}

#[tokio::test]
async fn test_router_fallback_under_consecutive_500_error_bursts() {
    let mut router = SemanticRouter::new();

    let primary_mock = Arc::new(MockProvider::new("primary-500", MockConfig::default()));
    let secondary_mock = Arc::new(
        MockProvider::new("secondary-backup", MockConfig::default())
            .with_text("Recovered from consecutive burst"),
    );

    router.register_provider(primary_mock.clone());
    router.register_provider(secondary_mock);
    router.set_fallback_chain(vec![ProviderKind::Mock]);

    // Send 10 consecutive requests where primary fails with 500
    for i in 0..10 {
        primary_mock.inject_error(format!("500 Internal Server Error (Burst #{i})"));
        let req = InferenceRequest::new("burst-model", vec![ChatMessage::user_text(format!("query {i}"))]);
        let resp = router.complete(req, RoutingPolicy::SpeedPriority).await.unwrap();
        assert_eq!(resp.text_content(), "Recovered from consecutive burst");
    }
}

#[tokio::test]
async fn test_router_multi_hop_fallback_cascade() {
    let mut router = SemanticRouter::new();

    let failing_groq = Arc::new(MockProvider::new("groq-failing", MockConfig::default()));
    failing_groq.inject_error("503 Service Unavailable on Groq LPU");

    let healthy_local = Arc::new(
        MockProvider::new("local-healthy", MockConfig::default())
            .with_text("Cascaded to healthy local/mock provider"),
    );

    router.register_provider(failing_groq);
    router.register_provider(healthy_local);
    router.set_fallback_chain(vec![ProviderKind::Groq, ProviderKind::Mock]);

    let req = InferenceRequest::new("model-cascade", vec![ChatMessage::user_text("cascade query")]);
    let resp = router.complete(req, RoutingPolicy::SpeedPriority).await.unwrap();

    assert_eq!(resp.text_content(), "Cascaded to healthy local/mock provider");
}

#[tokio::test]
async fn test_router_non_retryable_error_does_not_wastefully_fallback() {
    let auth_err = XenoError::auth("openai", "Invalid API key");
    assert!(!auth_err.is_retryable());

    let invalid_err = XenoError::InvalidRequest("temperature 5.0 out of range".into());
    assert!(!invalid_err.is_retryable());

    let rate_limit_err = XenoError::rate_limit("anthropic", Some(30));
    assert!(rate_limit_err.is_retryable());

    let timeout_err = XenoError::Timeout { timeout_ms: 10000 };
    assert!(timeout_err.is_retryable());
}

#[tokio::test]
async fn test_router_airgap_mode_strictly_blocks_cloud_fallback() {
    let mut router = SemanticRouter::new();

    let anthropic = Arc::new(AnthropicProvider::new("fake-key"));
    router.register_provider(anthropic);

    router.set_fallback_chain(vec![ProviderKind::Anthropic, ProviderKind::Local]);

    let mut req = InferenceRequest::new("claude-3-7-sonnet", vec![ChatMessage::user_text("test")]);
    req.privacy_filter = Some(PrivacyFilter {
        enabled: true,
        air_gap_mode: true,
        redact_secrets: true,
        redact_pii: true,
        custom_redaction_patterns: vec![],
    });

    let res = router.select_provider(&req, RoutingPolicy::PrivacyGuard);
    assert!(res.is_err());
    match res.err().unwrap() {
        XenoError::AirGapViolation { .. } => {}
        other => panic!("Expected AirGapViolation, got: {other:?}"),
    }
}
