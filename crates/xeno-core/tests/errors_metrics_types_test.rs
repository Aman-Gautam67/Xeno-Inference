//! Integration tests for xeno-core error hierarchy, metrics, and DAG types.

use xeno_core::prelude::*;

#[test]
fn test_xeno_error_classification_and_retryability() {
    let rate_limit = XenoError::rate_limit("anthropic", Some(5));
    assert!(rate_limit.is_retryable());
    assert_eq!(rate_limit.error_code(), "RATE_LIMIT_EXCEEDED");

    let timeout = XenoError::Timeout { timeout_ms: 10_000 };
    assert!(timeout.is_retryable());
    assert_eq!(timeout.error_code(), "REQUEST_TIMEOUT");

    let network = XenoError::NetworkError {
        message: "Connection reset by peer".into(),
    };
    assert!(network.is_retryable());
    assert_eq!(network.error_code(), "NETWORK_ERROR");

    let auth = XenoError::auth("openai", "Invalid API key");
    assert!(!auth.is_retryable());
    assert_eq!(auth.error_code(), "AUTH_FAILED");

    let airgap = XenoError::AirGapViolation {
        mode: "AirGappedOnly".into(),
        target: "api.openai.com".into(),
    };
    assert!(!airgap.is_retryable());
    assert_eq!(airgap.error_code(), "AIRGAP_VIOLATION");

    let upstream_503 = XenoError::upstream("deepseek", 503, "Service Unavailable");
    assert!(upstream_503.is_retryable());

    let upstream_400 = XenoError::upstream("deepseek", 400, "Bad Request");
    assert!(!upstream_400.is_retryable());

    // From conversions
    let tool_err: XenoError = ToolError::NotFound("unknown_tool".into()).into();
    assert_eq!(tool_err.error_code(), "TOOL_ERROR");

    let inf_err: XenoError = InferenceError::StreamingFailed("buffer overflow".into()).into();
    assert!(inf_err.is_retryable());

    let agent_err: XenoError = AgentError::MaxIterationsReached {
        iterations: 50,
        max: 50,
    }
    .into();
    assert_eq!(agent_err.error_code(), "AGENT_ERROR");

    let dag_err: XenoError = DAGError::CycleDetected("A -> B -> A".into()).into();
    assert_eq!(dag_err.error_code(), "DAG_ERROR");
}

#[test]
fn test_token_metrics_and_velocity() {
    let mut m1 = TokenMetrics::new(100, 50, 20, 50, 1000, 70.0, 0.001);
    assert_eq!(m1.total_tokens(), 170);

    let m2 = TokenMetrics::new(100, 50, 30, 0, 1000, 80.0, 0.0015);
    m1.merge(&m2);

    assert_eq!(m1.prompt_tokens, 200);
    assert_eq!(m1.completion_tokens, 100);
    assert_eq!(m1.reasoning_tokens, 50);
    assert_eq!(m1.total_tokens(), 350);
    assert_eq!(m1.total_duration_ms, 2000);
    assert_eq!(m1.ttft_ms, 50); // Preserved from m1
    assert!((m1.estimated_cost_usd - 0.0025).abs() < 1e-6);

    let vel = TokenMetrics::calculate_velocity(300, 1500);
    assert!((vel - 200.0).abs() < 1e-6);
}

#[test]
fn test_hardware_stats_calculations() {
    let stats = HardwareStats {
        vram_allocated_bytes: 12_000_000_000,
        vram_total_bytes: 24_000_000_000,
        gpu_utilization_pct: 75.0,
        cpu_utilization_pct: 25.0,
        system_ram_allocated_bytes: 32_000_000_000,
        system_ram_total_bytes: 64_000_000_000,
        temperature_celsius: Some(58.5),
    };

    assert!((stats.vram_usage_pct() - 50.0).abs() < 1e-3);
    assert!((stats.ram_usage_pct() - 50.0).abs() < 1e-3);
    assert!(stats.is_vram_constrained(40.0));
    assert!(!stats.is_vram_constrained(60.0));
}

#[test]
fn test_pricing_catalog_calculations() {
    let catalog = PricingCatalog::default();

    // Claude 3.7 Sonnet: $3.00/M input, $15.00/M output
    let claude_pricing = catalog.get_pricing("claude-3-7-sonnet-20250219");
    let cost = claude_pricing.calculate_cost(10_000, 2_000, 5_000);
    // (10_000/1_000_000 * 3.00) = 0.030
    // (2_000/1_000_000 * 15.00) = 0.030
    // (5_000/1_000_000 * 15.00) = 0.075
    // Total = 0.135
    assert!((cost - 0.135).abs() < 1e-6);

    // DeepSeek Reasoner: $0.55/M input, $2.19/M output
    let deepseek_pricing = catalog.get_pricing("deepseek-reasoner");
    let ds_cost = deepseek_pricing.calculate_cost(1_000_000, 1_000_000, 0);
    assert!((ds_cost - 2.74).abs() < 1e-6);

    // Local / unlisted model: $0.00
    let local_pricing = catalog.get_pricing("unlisted-local-model");
    let local_cost = local_pricing.calculate_cost(50_000, 50_000, 50_000);
    assert_eq!(local_cost, 0.0);
}

#[test]
fn test_dag_node_lifecycle_and_transitions() {
    let model = ModelAssignment::new(ProviderKind::Anthropic, "claude-3-7-sonnet", 0.3);
    let mut node = XenoDAGNode::new(
        "node_coder_01",
        "Implement Auth Middleware",
        DAGNodeType::Subagent,
        model,
    )
    .with_dependency("node_architect_01");

    assert_eq!(node.status, DAGNodeStatus::Pending);
    assert_eq!(node.dependencies, vec!["node_architect_01"]);
    assert!(!node.is_terminal());

    node.transition_to(DAGNodeStatus::Running);
    assert_eq!(node.status, DAGNodeStatus::Running);
    assert!(node.status.is_active());
    assert!(!node.is_terminal());

    node.transition_to(DAGNodeStatus::SelfHealing);
    assert_eq!(node.status, DAGNodeStatus::SelfHealing);
    assert!(node.status.is_active());

    node.set_output(serde_json::json!({ "file": "src/auth.rs", "status": "compiled" }));
    assert_eq!(node.status, DAGNodeStatus::Success);
    assert!(node.is_terminal());
    assert!(node.output_payload.is_some());
    assert!(node.completed_at.is_some());
    assert!(node.duration_ms().is_some());
}

#[test]
fn test_provider_kind_and_security_tiers() {
    assert!(ProviderKind::Anthropic.is_cloud());
    assert!(ProviderKind::Openai.is_cloud());
    assert!(ProviderKind::Google.is_cloud());
    assert!(ProviderKind::Deepseek.is_cloud());
    assert!(ProviderKind::Groq.is_cloud());
    assert!(ProviderKind::Local.is_local());
    assert!(ProviderKind::Mock.is_local());

    assert_eq!(ToolSecurityTier::Tier1Safe.level(), 1);
    assert_eq!(ToolSecurityTier::Tier2Guarded.level(), 2);
    assert_eq!(ToolSecurityTier::Tier3Destructive.level(), 3);
}
