//! Empirical challenger test suite for Milestone 1 (crates/xeno-core) by Challenger 2.
//! Stress-testing:
//! 1. Concurrent event creation & token metrics aggregation across threads.
//! 2. Exhaustive pricing calculations across all 13 models in PricingCatalog + custom edge cases.
//! 3. DAG node state transitions, lifecycle timestamps, duration metrics, and Serde contracts.
//! 4. Exhaustive error taxonomy, machine error codes, and HTTP/transient retryability matrix.

use serde_json::json;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use xeno_core::prelude::*;

// ============================================================================
// 1. CONCURRENCY & THREAD-SAFETY STRESS TESTS
// ============================================================================

#[test]
fn test_concurrent_event_creation_uniqueness_and_ordering() {
    const NUM_THREADS: usize = 16;
    const EVENTS_PER_THREAD: usize = 100;

    let events_collected = Arc::new(Mutex::new(Vec::with_capacity(NUM_THREADS * EVENTS_PER_THREAD)));
    let mut handles = Vec::with_capacity(NUM_THREADS);

    for thread_idx in 0..NUM_THREADS {
        let events_clone = Arc::clone(&events_collected);
        let handle = thread::spawn(move || {
            let mut local_events = Vec::with_capacity(EVENTS_PER_THREAD);
            for event_idx in 0..EVENTS_PER_THREAD {
                let session_id = format!("session_t{}_{}", thread_idx, event_idx);
                let role = match (thread_idx + event_idx) % 5 {
                    0 => AgentRole::Commander,
                    1 => AgentRole::Architect,
                    2 => AgentRole::Coder,
                    3 => AgentRole::Tester,
                    _ => AgentRole::Security,
                };

                let telemetry = TelemetryPayload {
                    model_used: "claude-3-7-sonnet-20250219".into(),
                    backend_type: BackendType::CloudApi,
                    vram_allocated_bytes: 0,
                    prompt_tokens: 100 + (event_idx as u32),
                    completion_tokens: 50 + (event_idx as u32),
                    reasoning_tokens: 20,
                    estimated_cost_usd: 0.0015,
                    ttft_ms: 45,
                    total_latency_ms: 250,
                    tokens_per_second: 280.0,
                };

                let event = match event_idx % 4 {
                    0 => {
                        let thinking = ThinkingPayload::new(
                            format!("Step reasoning {} from thread {}", event_idx, thread_idx),
                            250,
                            280.0,
                            format!("branch_{}", thread_idx),
                            false,
                        );
                        XenoAgentStepEvent::thinking_step(&session_id, role, thinking, telemetry)
                    }
                    1 => {
                        let tool_call = ToolCallPayload::with_mcp(
                            format!("call_{}_{}", thread_idx, event_idx),
                            "ast_grep",
                            json!({ "pattern": "fn main" }),
                            "mcp-ast-server",
                        );
                        XenoAgentStepEvent::tool_call_step(&session_id, role, tool_call, telemetry)
                    }
                    2 => {
                        let observation = ObservationPayload::success("1 match found in src/lib.rs");
                        XenoAgentStepEvent::observation_step(&session_id, role, observation, telemetry)
                    }
                    _ => XenoAgentStepEvent::verified_step(&session_id, role, telemetry),
                };

                // Verify Serde roundtrip for every event concurrently
                let json_str = serde_json::to_string(&event).expect("Event serialization failed");
                let deserialized: XenoAgentStepEvent =
                    serde_json::from_str(&json_str).expect("Event deserialization failed");
                assert_eq!(deserialized.event_id, event.event_id);
                assert_eq!(deserialized.session_id, event.session_id);
                assert_eq!(deserialized.agent_role, event.agent_role);
                assert_eq!(deserialized.execution_phase, event.execution_phase);

                local_events.push(event);
            }

            let mut guard = events_clone.lock().unwrap();
            guard.extend(local_events);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread execution failed");
    }

    let all_events = events_collected.lock().unwrap();
    assert_eq!(all_events.len(), NUM_THREADS * EVENTS_PER_THREAD);

    // Verify all UUIDs are globally unique
    let mut uuid_set = std::collections::HashSet::new();
    for ev in all_events.iter() {
        assert!(
            uuid_set.insert(ev.event_id),
            "Duplicate UUID detected: {}",
            ev.event_id
        );
    }
}

#[test]
fn test_concurrent_metrics_aggregation_and_saturation() {
    const NUM_THREADS: usize = 20;
    const ITERS_PER_THREAD: usize = 500;

    let global_metrics = Arc::new(Mutex::new(TokenMetrics::default()));
    let mut handles = Vec::with_capacity(NUM_THREADS);

    for _ in 0..NUM_THREADS {
        let metrics_ref = Arc::clone(&global_metrics);
        let handle = thread::spawn(move || {
            for _ in 0..ITERS_PER_THREAD {
                let local = TokenMetrics::new(10, 5, 2, 25, 50, 140.0, 0.0001);
                let mut guard = metrics_ref.lock().unwrap();
                guard.merge(&local);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = global_metrics.lock().unwrap();
    let total_operations = (NUM_THREADS * ITERS_PER_THREAD) as u32;

    assert_eq!(result.prompt_tokens, total_operations * 10);
    assert_eq!(result.completion_tokens, total_operations * 5);
    assert_eq!(result.reasoning_tokens, total_operations * 2);
    assert_eq!(
        result.total_tokens(),
        total_operations * 10 + total_operations * 5 + total_operations * 2
    );
    assert_eq!(result.total_duration_ms, (total_operations as u64) * 50);
    assert_eq!(result.ttft_ms, 25);
    assert!((result.estimated_cost_usd - ((total_operations as f64) * 0.0001)).abs() < 1e-4);

    // Saturation test on arithmetic overflow
    let mut sat_metrics = TokenMetrics::new(u32::MAX - 10, u32::MAX - 10, u32::MAX - 10, 10, u64::MAX - 10, 10.0, 1.0);
    let add_metrics = TokenMetrics::new(20, 20, 20, 10, 20, 10.0, 1.0);
    sat_metrics.merge(&add_metrics);

    assert_eq!(sat_metrics.prompt_tokens, u32::MAX);
    assert_eq!(sat_metrics.completion_tokens, u32::MAX);
    assert_eq!(sat_metrics.reasoning_tokens, u32::MAX);
    assert_eq!(sat_metrics.total_tokens(), u32::MAX);
    assert_eq!(sat_metrics.total_duration_ms, u64::MAX);
}

// ============================================================================
// 2. PRICING CATALOG EXHAUSTIVE MATRIX & BOUNDARY TESTS
// ============================================================================

#[test]
fn test_pricing_catalog_all_default_models_matrix() {
    let catalog = PricingCatalog::default();

    // Verification matrix for all 13 supported models:
    // (Model Name, Expected Input Rate/M, Expected Output Rate/M)
    let models = vec![
        ("claude-3-7-sonnet-20250219", 3.00, 15.00),
        ("claude-3-5-sonnet-20241022", 3.00, 15.00),
        ("claude-3-5-haiku-20241022", 0.80, 4.00),
        ("gpt-4o", 2.50, 10.00),
        ("gpt-4o-mini", 0.15, 0.60),
        ("o1", 15.00, 60.00),
        ("o3-mini", 1.10, 4.40),
        ("gemini-2.0-flash", 0.10, 0.40),
        ("gemini-2.0-pro", 1.25, 5.00),
        ("deepseek-chat", 0.14, 0.28),
        ("deepseek-reasoner", 0.55, 2.19),
        ("llama-3.3-70b-versatile", 0.59, 0.79),
        ("llama-3.1-8b-instant", 0.05, 0.08),
    ];

    for (model, expected_in, expected_out) in models {
        let pricing = catalog.get_pricing(model);
        assert_eq!(
            pricing.input_cost_per_million, expected_in,
            "Input cost mismatch for model {}",
            model
        );
        assert_eq!(
            pricing.output_cost_per_million, expected_out,
            "Output cost mismatch for model {}",
            model
        );

        // Test with 1M prompt, 1M completion, 0 reasoning
        let cost_1m = pricing.calculate_cost(1_000_000, 1_000_000, 0);
        let expected_1m = expected_in + expected_out;
        assert!(
            (cost_1m - expected_1m).abs() < 1e-6,
            "1M token cost failed for {}: got {}, expected {}",
            model,
            cost_1m,
            expected_1m
        );

        // Test with 500k prompt, 200k completion, 300k reasoning (which falls back to output rate when not overridden)
        let cost_complex = pricing.calculate_cost(500_000, 200_000, 300_000);
        let expected_complex = (0.5 * expected_in) + (0.2 * expected_out) + (0.3 * expected_out);
        assert!(
            (cost_complex - expected_complex).abs() < 1e-6,
            "Complex token cost failed for {}: got {}, expected {}",
            model,
            cost_complex,
            expected_complex
        );
    }
}

#[test]
fn test_pricing_catalog_unregistered_and_custom_models() {
    let mut catalog = PricingCatalog::default();

    // 1. Unregistered model should return free pricing ($0.0)
    let unknown_pricing = catalog.get_pricing("random-unknown-model-xyz");
    assert_eq!(unknown_pricing.input_cost_per_million, 0.0);
    assert_eq!(unknown_pricing.output_cost_per_million, 0.0);
    assert_eq!(unknown_pricing.calculate_cost(100_000, 100_000, 100_000), 0.0);

    // 2. Custom model with distinct reasoning cost
    let custom_pricing = ModelPricing::with_reasoning(2.00, 8.00, 16.00);
    catalog.register("deepseek-r1-custom", custom_pricing);

    let retrieved = catalog.get_pricing("deepseek-r1-custom");
    assert_eq!(retrieved.input_cost_per_million, 2.00);
    assert_eq!(retrieved.output_cost_per_million, 8.00);
    assert_eq!(retrieved.reasoning_cost_per_million, Some(16.00));

    // Calculate: 1M input ($2) + 500k output ($4) + 250k reasoning ($4) = $10.00
    let cost = retrieved.calculate_cost(1_000_000, 500_000, 250_000);
    assert!((cost - 10.00).abs() < 1e-6);

    // 3. Zero token edge cases
    assert_eq!(retrieved.calculate_cost(0, 0, 0), 0.0);

    // 4. Fractional token loads
    let micro_cost = retrieved.calculate_cost(1, 1, 1);
    let expected_micro = (2.0 + 8.0 + 16.0) / 1_000_000.0;
    assert!((micro_cost - expected_micro).abs() < 1e-12);
}

// ============================================================================
// 3. DAG NODE LIFECYCLE & STATE TRANSITIONS
// ============================================================================

#[test]
fn test_dag_node_lifecycle_complete_state_transitions() {
    let model = ModelAssignment::new(ProviderKind::Openai, "o3-mini", 0.0);
    let mut node = XenoDAGNode::new("dag_step_01", "Validate Contracts", DAGNodeType::VerificationGate, model)
        .with_dependencies(vec!["dag_step_00_dep1", "dag_step_00_dep2"]);

    // Initial state check
    assert_eq!(node.status, DAGNodeStatus::Pending);
    assert_eq!(node.dependencies.len(), 2);
    assert_eq!(node.dependencies[0], "dag_step_00_dep1");
    assert_eq!(node.dependencies[1], "dag_step_00_dep2");
    assert!(!node.is_terminal());
    assert!(!node.status.is_active());
    assert!(node.completed_at.is_none());
    assert!(node.output_payload.is_none());

    // Transition to Running
    node.transition_to(DAGNodeStatus::Running);
    assert_eq!(node.status, DAGNodeStatus::Running);
    assert!(node.status.is_active());
    assert!(!node.is_terminal());
    assert!(node.completed_at.is_none());

    // Transition to SelfHealing (active retry)
    node.transition_to(DAGNodeStatus::SelfHealing);
    assert_eq!(node.status, DAGNodeStatus::SelfHealing);
    assert!(node.status.is_active());
    assert!(!node.is_terminal());
    assert!(node.completed_at.is_none());

    // Small sleep to ensure non-zero duration
    thread::sleep(Duration::from_millis(5));

    // Transition to Failed (terminal)
    node.transition_to(DAGNodeStatus::Failed);
    assert_eq!(node.status, DAGNodeStatus::Failed);
    assert!(!node.status.is_active());
    assert!(node.is_terminal());
    assert!(node.completed_at.is_some());
    assert!(node.duration_ms().unwrap() >= 0);

    let first_completed_at = node.completed_at;

    // Transition to Success via set_output
    node.set_output(json!({ "verification": "passed", "coverage": 99.4 }));
    assert_eq!(node.status, DAGNodeStatus::Success);
    assert!(node.is_terminal());
    assert!(node.output_payload.is_some());
    // completed_at should not be overwritten if already completed
    assert_eq!(node.completed_at, first_completed_at);

    // Verify Serde roundtrip for DAG Node
    let json_node = serde_json::to_string_pretty(&node).unwrap();
    let de_node: XenoDAGNode = serde_json::from_str(&json_node).unwrap();
    assert_eq!(de_node.node_id, node.node_id);
    assert_eq!(de_node.label, node.label);
    assert_eq!(de_node.node_type, DAGNodeType::VerificationGate);
    assert_eq!(de_node.status, DAGNodeStatus::Success);
    assert_eq!(de_node.dependencies, node.dependencies);
    assert_eq!(de_node.assigned_model.model_name, "o3-mini");
    assert_eq!(de_node.output_payload, node.output_payload);
}

// ============================================================================
// 4. ERROR TAXONOMY & RETRYABILITY MATRIX
// ============================================================================

#[test]
fn test_error_taxonomy_exhaustive_matrix() {
    struct ErrorCase {
        error: XenoError,
        expected_code: &'static str,
        expected_retryable: bool,
    }

    let test_cases = vec![
        // Direct XenoError variants
        ErrorCase {
            error: XenoError::auth("google", "OAuth token expired"),
            expected_code: "AUTH_FAILED",
            expected_retryable: false,
        },
        ErrorCase {
            error: XenoError::rate_limit("anthropic", Some(30)),
            expected_code: "RATE_LIMIT_EXCEEDED",
            expected_retryable: true,
        },
        ErrorCase {
            error: XenoError::ContextLengthExceeded {
                requested_tokens: 200_000,
                max_tokens: 128_000,
            },
            expected_code: "CONTEXT_OVERFLOW",
            expected_retryable: false,
        },
        ErrorCase {
            error: XenoError::ModelNotFound {
                provider: "local".into(),
                model: "non-existent-gguf".into(),
            },
            expected_code: "MODEL_NOT_FOUND",
            expected_retryable: false,
        },
        ErrorCase {
            error: XenoError::upstream("openai", 500, "Internal Server Error"),
            expected_code: "UPSTREAM_ERROR",
            expected_retryable: true,
        },
        ErrorCase {
            error: XenoError::upstream("openai", 502, "Bad Gateway"),
            expected_code: "UPSTREAM_ERROR",
            expected_retryable: true,
        },
        ErrorCase {
            error: XenoError::upstream("openai", 503, "Service Unavailable"),
            expected_code: "UPSTREAM_ERROR",
            expected_retryable: true,
        },
        ErrorCase {
            error: XenoError::upstream("openai", 504, "Gateway Timeout"),
            expected_code: "UPSTREAM_ERROR",
            expected_retryable: true,
        },
        ErrorCase {
            error: XenoError::upstream("openai", 429, "Too Many Requests"),
            expected_code: "UPSTREAM_ERROR",
            expected_retryable: true,
        },
        ErrorCase {
            error: XenoError::upstream("openai", 400, "Bad Request"),
            expected_code: "UPSTREAM_ERROR",
            expected_retryable: false,
        },
        ErrorCase {
            error: XenoError::upstream("openai", 401, "Unauthorized"),
            expected_code: "UPSTREAM_ERROR",
            expected_retryable: false,
        },
        ErrorCase {
            error: XenoError::upstream("openai", 404, "Not Found"),
            expected_code: "UPSTREAM_ERROR",
            expected_retryable: false,
        },
        ErrorCase {
            error: XenoError::Timeout { timeout_ms: 5000 },
            expected_code: "REQUEST_TIMEOUT",
            expected_retryable: true,
        },
        ErrorCase {
            error: XenoError::NetworkError {
                message: "DNS lookup failed".into(),
            },
            expected_code: "NETWORK_ERROR",
            expected_retryable: true,
        },
        ErrorCase {
            error: XenoError::AirGapViolation {
                mode: "AirGapped".into(),
                target: "http://external.com".into(),
            },
            expected_code: "AIRGAP_VIOLATION",
            expected_retryable: false,
        },
        ErrorCase {
            error: XenoError::PrivacyViolation {
                rule_name: "openai_api_key".into(),
            },
            expected_code: "PRIVACY_VIOLATION",
            expected_retryable: false,
        },
        ErrorCase {
            error: XenoError::StreamInterrupted {
                reason: "TCP RST".into(),
            },
            expected_code: "STREAM_INTERRUPTED",
            expected_retryable: true,
        },
        ErrorCase {
            error: XenoError::InvalidRequest("temperature cannot be negative".into()),
            expected_code: "INVALID_REQUEST",
            expected_retryable: false,
        },
        ErrorCase {
            error: XenoError::Internal("Kernel panic in runtime".into()),
            expected_code: "INTERNAL_ERROR",
            expected_retryable: false,
        },
        // Subsystem: InferenceError
        ErrorCase {
            error: InferenceError::StreamingFailed("chunk corrupted".into()).into(),
            expected_code: "INFERENCE_ERROR",
            expected_retryable: true,
        },
        ErrorCase {
            error: InferenceError::RequestFailed {
                status: 503,
                message: "Overloaded".into(),
            }
            .into(),
            expected_code: "INFERENCE_ERROR",
            expected_retryable: true,
        },
        ErrorCase {
            error: InferenceError::RequestFailed {
                status: 429,
                message: "Rate limit".into(),
            }
            .into(),
            expected_code: "INFERENCE_ERROR",
            expected_retryable: true,
        },
        ErrorCase {
            error: InferenceError::RequestFailed {
                status: 400,
                message: "Bad request".into(),
            }
            .into(),
            expected_code: "INFERENCE_ERROR",
            expected_retryable: false,
        },
        ErrorCase {
            error: InferenceError::ProviderNotFound("unknown_backend".into()).into(),
            expected_code: "INFERENCE_ERROR",
            expected_retryable: false,
        },
        // Subsystem: ToolError
        ErrorCase {
            error: ToolError::Timeout {
                tool: "conpty_exec".into(),
                timeout_ms: 10_000,
            }
            .into(),
            expected_code: "TOOL_ERROR",
            expected_retryable: true,
        },
        ErrorCase {
            error: ToolError::PermissionDenied {
                tool: "rm_rf".into(),
                tier: "Tier3Destructive".into(),
                reason: "User denied confirmation".into(),
            }
            .into(),
            expected_code: "TOOL_ERROR",
            expected_retryable: false,
        },
        ErrorCase {
            error: ToolError::AstValidationFailed {
                file: "src/lib.rs".into(),
                reason: "Unclosed delimiter".into(),
            }
            .into(),
            expected_code: "TOOL_ERROR",
            expected_retryable: false,
        },
        // Subsystem: AgentError
        ErrorCase {
            error: AgentError::ConsensusFailed {
                agreement_pct: 33.3,
                required_pct: 66.7,
                reason: "Coder and QA disagreed on patch".into(),
            }
            .into(),
            expected_code: "AGENT_ERROR",
            expected_retryable: false,
        },
        ErrorCase {
            error: AgentError::SelfHealingExhausted {
                attempts: 5,
                error: "Compilation failed repeatedly".into(),
            }
            .into(),
            expected_code: "AGENT_ERROR",
            expected_retryable: false,
        },
        // Subsystem: DAGError
        ErrorCase {
            error: DAGError::DependencyUnmet {
                node_id: "node_b".into(),
                unmet_dependency: "node_a".into(),
            }
            .into(),
            expected_code: "DAG_ERROR",
            expected_retryable: false,
        },
        ErrorCase {
            error: DAGError::CycleDetected("A -> B -> C -> A".into()).into(),
            expected_code: "DAG_ERROR",
            expected_retryable: false,
        },
    ];

    for case in test_cases {
        assert_eq!(
            case.error.error_code(),
            case.expected_code,
            "Error code mismatch for {:?}",
            case.error
        );
        assert_eq!(
            case.error.is_retryable(),
            case.expected_retryable,
            "Retryable mismatch for {:?}",
            case.error
        );
        // Verify Display string is non-empty
        let display_str = case.error.to_string();
        assert!(!display_str.is_empty(), "Display string empty for {:?}", case.error);
    }
}
