//! Adversarial stress test suite and fuzz fixtures for xeno-core (Milestone 1).
//!
//! Tests extreme values, numeric boundaries, unicode control sequences,
//! massive nested JSON payloads, empty strings, and deserialization edge cases
//! to guarantee zero panics, robust error reporting, and schema integrity.

use chrono::Duration;
use serde_json::json;
use xeno_core::prelude::*;

#[test]
fn test_adversarial_token_metrics_and_numeric_extremes() {
    // 1. Extreme token counts with u32::MAX
    let max_metrics = TokenMetrics::new(
        u32::MAX,
        u32::MAX,
        u32::MAX,
        u64::MAX,
        u64::MAX,
        f64::MAX,
        f64::MAX,
    );

    // Total tokens should saturate at u32::MAX without panic
    assert_eq!(max_metrics.total_tokens(), u32::MAX);

    // 2. Merging u32::MAX metrics should saturate without overflowing
    let mut accumulator = max_metrics.clone();
    let other_max = max_metrics.clone();
    accumulator.merge(&other_max);
    assert_eq!(accumulator.prompt_tokens, u32::MAX);
    assert_eq!(accumulator.completion_tokens, u32::MAX);
    assert_eq!(accumulator.reasoning_tokens, u32::MAX);
    assert_eq!(accumulator.total_duration_ms, u64::MAX);

    // 3. Zero duration in velocity calculation should return 0.0 without division by zero panic
    let zero_vel = TokenMetrics::calculate_velocity(1_000_000, 0);
    assert_eq!(zero_vel, 0.0);
    let zero_tok_vel = TokenMetrics::calculate_velocity(0, 0);
    assert_eq!(zero_tok_vel, 0.0);

    // 4. Hardware stats with zero total memory / vram (prevent division by zero)
    let zero_hw = HardwareStats {
        vram_allocated_bytes: 1024,
        vram_total_bytes: 0,
        gpu_utilization_pct: 0.0,
        cpu_utilization_pct: 0.0,
        system_ram_allocated_bytes: 2048,
        system_ram_total_bytes: 0,
        temperature_celsius: None,
    };
    assert_eq!(zero_hw.vram_usage_pct(), 0.0);
    assert_eq!(zero_hw.ram_usage_pct(), 0.0);
    assert!(!zero_hw.is_vram_constrained(50.0));

    // 5. Hardware stats with allocated > total (e.g. overcommit/swap)
    let overcommitted_hw = HardwareStats {
        vram_allocated_bytes: 32_000_000_000,
        vram_total_bytes: 16_000_000_000,
        gpu_utilization_pct: 150.0,
        cpu_utilization_pct: 200.0,
        system_ram_allocated_bytes: 64_000_000_000,
        system_ram_total_bytes: 32_000_000_000,
        temperature_celsius: Some(-10.5),
    };
    assert!((overcommitted_hw.vram_usage_pct() - 200.0).abs() < 1e-3);
    assert!((overcommitted_hw.ram_usage_pct() - 200.0).abs() < 1e-3);
    assert!(overcommitted_hw.is_vram_constrained(100.0));

    // 6. Pricing calculations with extreme values (u32::MAX tokens)
    let catalog = PricingCatalog::default();
    let claude_p = catalog.get_pricing("claude-3-7-sonnet-20250219");
    let max_cost = claude_p.calculate_cost(u32::MAX, u32::MAX, u32::MAX);
    assert!(max_cost.is_finite());
    assert!(max_cost > 0.0);

    // Free tier with u32::MAX tokens
    let free_p = ModelPricing::free();
    assert_eq!(free_p.calculate_cost(u32::MAX, u32::MAX, u32::MAX), 0.0);

    // 7. DAG Node negative duration (clock skew scenario)
    let mut skewed_node = XenoDAGNode::new(
        "skewed_node",
        "Testing backwards time travel",
        DAGNodeType::ToolExec,
        ModelAssignment::new(ProviderKind::Mock, "mock-model", 0.0),
    );
    // Artificially set completed_at before created_at
    skewed_node.completed_at = Some(skewed_node.created_at - Duration::milliseconds(5000));
    assert_eq!(skewed_node.duration_ms(), Some(-5000));
}

#[test]
fn test_adversarial_unicode_and_control_characters() {
    // Strings containing Astral Plane Emojis, Zero-Width Joiners, RTL overrides, null bytes, and ANSI codes
    let adversarial_strings = vec![
        "🦀🚀👨‍👩‍👧‍👦💥",                                    // Complex emojis & ZWJ
        "\u{200B}\u{200C}\u{200D}\u{FEFF}",                   // Zero-width characters
        "\u{202E}gnirts desrever\u{202C}",                    // Right-to-left override
        "Line1\x00Embedded\x00Null\x00Bytes",                // Embedded null bytes
        "\x1b[31;1m\x1b[42mESC[2J\x1b[0m\t\r\n\x07\x08\x0C", // ANSI escapes & control characters
        "اللغة العربية / 简体中文 / עִבְרִית / Tiếng Việt",  // Multilingual complex scripts
        "{\"injection\": \"\\\"}' OR 1=1; DROP TABLE nodes;--\"}", // Code/SQL injection syntax
    ];

    for (idx, text) in adversarial_strings.iter().enumerate() {
        // Test in ChatMessage
        let msg = ChatMessage::user_text(*text).with_name(format!("user_{idx}_{text}"));
        let json_msg = serde_json::to_string(&msg).unwrap();
        let de_msg: ChatMessage = serde_json::from_str(&json_msg).unwrap();
        assert_eq!(de_msg.text_content(), *text);

        // Test in ToolCallPayload & ObservationPayload
        let tool_call = ToolCallPayload::with_mcp(
            format!("call_{idx}"),
            format!("tool_{text}"),
            json!({ "arg_unicode": text, "nested": { "raw": text } }),
            format!("server_{text}"),
        );
        let json_tc = serde_json::to_string(&tool_call).unwrap();
        let de_tc: ToolCallPayload = serde_json::from_str(&json_tc).unwrap();
        assert_eq!(de_tc.call_id, format!("call_{idx}"));
        assert_eq!(
            de_tc.arguments["arg_unicode"].as_str().unwrap(),
            *text
        );

        let obs = ObservationPayload {
            exit_code: -(idx as i32),
            stdout: text.to_string(),
            stderr: text.to_string(),
            diff_snippet: Some(text.to_string()),
            ast_validation_passed: idx % 2 == 0,
        };
        let json_obs = serde_json::to_string(&obs).unwrap();
        let de_obs: ObservationPayload = serde_json::from_str(&json_obs).unwrap();
        assert_eq!(de_obs.stdout, *text);
        assert_eq!(de_obs.stderr, *text);
        assert_eq!(de_obs.diff_snippet.as_deref(), Some(*text));

        // Test in ThinkingPayload
        let thinking = ThinkingPayload::new(*text, 100, 50.0, format!("branch_{text}"), false);
        let json_think = serde_json::to_string(&thinking).unwrap();
        let de_think: ThinkingPayload = serde_json::from_str(&json_think).unwrap();
        assert_eq!(de_think.raw_tokens, *text);

        // Test in full XenoAgentStepEvent
        let event = XenoAgentStepEvent::thinking_step(
            format!("session_{text}"),
            AgentRole::Security,
            thinking,
            TelemetryPayload {
                model_used: text.to_string(),
                backend_type: BackendType::CloudApi,
                vram_allocated_bytes: 0,
                prompt_tokens: 10,
                completion_tokens: 10,
                reasoning_tokens: 10,
                estimated_cost_usd: 0.001,
                ttft_ms: 5,
                total_latency_ms: 20,
                tokens_per_second: 500.0,
            },
        );
        let json_event = serde_json::to_string(&event).unwrap();
        let de_event: XenoAgentStepEvent = serde_json::from_str(&json_event).unwrap();
        assert_eq!(de_event.session_id, format!("session_{text}"));
        assert_eq!(de_event.telemetry.model_used, *text);
    }
}

#[test]
fn test_adversarial_massive_nested_json_payloads() {
    // 1. Construct 50 levels of deeply nested JSON in tool arguments
    let mut deep_value = json!({ "leaf": "deepest_value", "depth": 50 });
    for level in (1..50).rev() {
        deep_value = json!({
            "level": level,
            "child": deep_value
        });
    }

    let tool_call = ToolCallPayload::new("deep_call_001", "deep_parser", deep_value.clone());
    let serialized_tc = serde_json::to_string(&tool_call).unwrap();
    let deserialized_tc: ToolCallPayload = serde_json::from_str(&serialized_tc).unwrap();
    assert_eq!(deserialized_tc.call_id, "deep_call_001");
    assert_eq!(deserialized_tc.arguments, deep_value);

    // 2. Deeply nested JSON in XenoDAGNode output_payload
    let mut node = XenoDAGNode::new(
        "deep_node",
        "Deep JSON Node",
        DAGNodeType::Artifact,
        ModelAssignment::new(ProviderKind::Mock, "mock", 0.0),
    );
    node.set_output(deep_value.clone());
    let node_json = serde_json::to_string(&node).unwrap();
    let node_de: XenoDAGNode = serde_json::from_str(&node_json).unwrap();
    assert_eq!(node_de.output_payload, Some(deep_value));

    // 3. Multi-megabyte large string payload (2MB) in ContentBlock & Observation
    let large_string = "X".repeat(2 * 1024 * 1024); // 2 MB string
    let text_block = ContentBlock::text(large_string.clone());
    let block_json = serde_json::to_string(&text_block).unwrap();
    let block_de: ContentBlock = serde_json::from_str(&block_json).unwrap();
    assert_eq!(block_de.as_text().map(|s| s.len()), Some(2 * 1024 * 1024));

    let large_obs = ObservationPayload::success(large_string.clone());
    let obs_json = serde_json::to_string(&large_obs).unwrap();
    let obs_de: ObservationPayload = serde_json::from_str(&obs_json).unwrap();
    assert_eq!(obs_de.stdout.len(), 2 * 1024 * 1024);
}

#[test]
fn test_adversarial_empty_and_extreme_strings() {
    // 1. Empty strings everywhere
    let empty_msg = ChatMessage::user_text("").with_name("");
    assert_eq!(empty_msg.text_content(), "");
    assert_eq!(empty_msg.name.as_deref(), Some(""));

    let empty_req = InferenceRequest::new("", vec![empty_msg.clone()])
        .with_system_prompt("")
        .with_reasoning_effort("")
        .with_tools(vec![ToolDefinition::new("", "", json!({}))]);

    let req_json = serde_json::to_string(&empty_req).unwrap();
    let req_de: InferenceRequest = serde_json::from_str(&req_json).unwrap();
    assert_eq!(req_de.model, "");
    assert_eq!(req_de.system_prompt.as_deref(), Some(""));
    assert_eq!(req_de.reasoning_effort.as_deref(), Some(""));
    assert_eq!(req_de.tools[0].name, "");

    // 2. Massive 100,000 char identifier strings
    let massive_id = "A".repeat(100_000);
    let long_node = XenoDAGNode::new(
        massive_id.clone(),
        massive_id.clone(),
        DAGNodeType::Subagent,
        ModelAssignment::new(ProviderKind::Local, massive_id.clone(), 1.5),
    )
    .with_dependency(massive_id.clone());

    let long_json = serde_json::to_string(&long_node).unwrap();
    let long_de: XenoDAGNode = serde_json::from_str(&long_json).unwrap();
    assert_eq!(long_de.node_id.len(), 100_000);
    assert_eq!(long_de.assigned_model.model_name.len(), 100_000);
    assert_eq!(long_de.dependencies[0].len(), 100_000);
}

#[test]
fn test_adversarial_serde_fuzz_and_malformed_inputs() {
    // 1. Unknown fields in JSON payloads (forward compatibility)
    let json_with_extra = json!({
        "nodeId": "extra_fields_node",
        "label": "Extra Fields",
        "nodeType": "orchestrator",
        "status": "pending",
        "dependencies": [],
        "assignedModel": {
            "provider": "anthropic",
            "modelName": "claude-3-7-sonnet",
            "temperature": 0.5,
            "unexpectedFutureField": "future_data_123"
        },
        "createdAt": "2026-08-20T12:00:00Z",
        "unknownTopLevelField": [1, 2, 3]
    });
    let node_de = serde_json::from_value::<XenoDAGNode>(json_with_extra);
    assert!(node_de.is_ok());
    assert_eq!(node_de.unwrap().node_id, "extra_fields_node");

    // 2. Invalid enum variant deserialization should cleanly fail without panicking
    let invalid_role_json = json!({ "role": "super_admin", "content": [] });
    let bad_msg = serde_json::from_value::<ChatMessage>(invalid_role_json);
    assert!(bad_msg.is_err());

    let invalid_phase_json = json!("quantum_teleport");
    let bad_phase = serde_json::from_value::<ExecutionPhase>(invalid_phase_json);
    assert!(bad_phase.is_err());

    let invalid_node_status = json!("hibernating");
    let bad_status = serde_json::from_value::<DAGNodeStatus>(invalid_node_status);
    assert!(bad_status.is_err());

    let invalid_provider = json!("alien_quantum_cloud");
    let bad_prov = serde_json::from_value::<ProviderKind>(invalid_provider);
    assert!(bad_prov.is_err());

    // 3. Corrupted ContentBlock type tags
    let bad_content_block = json!({
        "type": "unsupported_media_block",
        "data": "some_data"
    });
    let bad_block_res = serde_json::from_value::<ContentBlock>(bad_content_block);
    assert!(bad_block_res.is_err());

    // 4. Corrupted StreamChunkDelta type tags
    let bad_chunk_delta = json!({
        "type": "telepathic_delta",
        "tokens": "foo"
    });
    let bad_delta_res = serde_json::from_value::<StreamChunkDelta>(bad_chunk_delta);
    assert!(bad_delta_res.is_err());

    // 5. Bare scalars and empty JSON strings
    assert!(serde_json::from_str::<InferenceRequest>("").is_err());
    assert!(serde_json::from_str::<InferenceRequest>("   \n\t  ").is_err());
    assert!(serde_json::from_str::<InferenceRequest>("12345").is_err());
    assert!(serde_json::from_str::<InferenceRequest>("\"just_a_string\"").is_err());
    assert!(serde_json::from_str::<InferenceRequest>("[]").is_err());
}

#[test]
fn test_adversarial_error_taxonomy_and_chaining() {
    // 1. Extreme error message lengths (100KB error string)
    let giant_error_msg = "FATAL_ERROR_".repeat(10_000);
    let giant_err = XenoError::Internal(giant_error_msg.clone());
    assert_eq!(giant_err.error_code(), "INTERNAL_ERROR");
    assert!(giant_err.to_string().contains("FATAL_ERROR_"));

    // 2. Verify all error codes and retryable rules across status codes
    let status_codes = [0, 200, 400, 401, 403, 404, 408, 429, 500, 502, 503, 504, 999];
    for &status in &status_codes {
        let upstream = XenoError::upstream("test_provider", status, "Message");
        if status >= 500 || status == 429 {
            assert!(upstream.is_retryable(), "Status {status} should be retryable");
        } else {
            assert!(!upstream.is_retryable(), "Status {status} should not be retryable");
        }

        let inf_req_fail: XenoError = InferenceError::RequestFailed {
            status,
            message: "Failed".into(),
        }
        .into();
        if status >= 500 || status == 429 {
            assert!(inf_req_fail.is_retryable(), "Inference status {status} should be retryable");
        } else {
            assert!(!inf_req_fail.is_retryable(), "Inference status {status} should not be retryable");
        }
    }

    // 3. From<serde_json::Error> conversion
    let bad_json = serde_json::from_str::<ChatMessage>("INVALID_JSON_HERE");
    let json_err: XenoError = bad_json.unwrap_err().into();
    assert_eq!(json_err.error_code(), "JSON_PARSE_ERROR");
    assert!(!json_err.is_retryable());

    // 4. Exhaustive error_code check for all variants
    let errors: Vec<XenoError> = vec![
        InferenceError::ProviderNotFound("p".into()).into(),
        InferenceError::ModelUnavailable { model: "m".into(), reason: "r".into() }.into(),
        InferenceError::StreamingFailed("s".into()).into(),
        InferenceError::TokenLimitExceeded { current: 100, limit: 50 }.into(),
        InferenceError::UnsupportedFeature { provider: "p".into(), feature: "f".into() }.into(),
        InferenceError::MalformedResponse("m".into()).into(),
        InferenceError::RequestFailed { status: 500, message: "m".into() }.into(),
        ToolError::NotFound("t".into()).into(),
        ToolError::PermissionDenied { tool: "t".into(), tier: "1".into(), reason: "r".into() }.into(),
        ToolError::ExecutionFailed { tool: "t".into(), error: "e".into() }.into(),
        ToolError::Timeout { tool: "t".into(), timeout_ms: 100 }.into(),
        ToolError::InvalidArguments { tool: "t".into(), details: "d".into() }.into(),
        ToolError::AstValidationFailed { file: "f".into(), reason: "r".into() }.into(),
        ToolError::SandboxViolation { tool: "t".into(), reason: "r".into() }.into(),
        ToolError::ProcessKilled { tool: "t".into(), signal_or_code: "SIGKILL".into() }.into(),
        AgentError::GoalPlanningFailed("g".into()).into(),
        AgentError::MaxIterationsReached { iterations: 10, max: 10 }.into(),
        AgentError::ConsensusFailed { agreement_pct: 50.0, required_pct: 100.0, reason: "r".into() }.into(),
        AgentError::MemoryError("m".into()).into(),
        AgentError::RoleTransitionFailed { from: "a".into(), to: "b".into(), reason: "r".into() }.into(),
        AgentError::VerificationFailed("v".into()).into(),
        AgentError::SelfHealingExhausted { attempts: 3, error: "e".into() }.into(),
        DAGError::CycleDetected("c".into()).into(),
        DAGError::NodeNotFound("n".into()).into(),
        DAGError::DependencyUnmet { node_id: "a".into(), unmet_dependency: "b".into() }.into(),
        DAGError::InvalidTransition { node_id: "a".into(), from: "p".into(), to: "s".into() }.into(),
        DAGError::ExecutionFailed { node_id: "a".into(), reason: "r".into() }.into(),
        DAGError::GraftingFailed("g".into()).into(),
        XenoError::auth("p", "m"),
        XenoError::rate_limit("p", None),
        XenoError::ContextLengthExceeded { requested_tokens: 100, max_tokens: 50 },
        XenoError::ModelNotFound { provider: "p".into(), model: "m".into() },
        XenoError::upstream("p", 500, "m"),
        XenoError::Timeout { timeout_ms: 1000 },
        XenoError::NetworkError { message: "n".into() },
        XenoError::AirGapViolation { mode: "m".into(), target: "t".into() },
        XenoError::PrivacyViolation { rule_name: "r".into() },
        XenoError::StreamInterrupted { reason: "r".into() },
        XenoError::InvalidRequest("r".into()),
        XenoError::Internal("i".into()),
    ];

    for err in errors {
        assert!(!err.error_code().is_empty());
        assert!(!err.to_string().is_empty());
    }
}

#[test]
fn test_adversarial_dag_lifecycle_and_high_concurrency_dependencies() {
    let model = ModelAssignment::new(ProviderKind::Deepseek, "deepseek-r1", 0.0);
    let mut node = XenoDAGNode::new("stress_node", "Lifecycle Stress", DAGNodeType::Subagent, model);

    // 1. Rapid state transitions
    let transitions = [
        DAGNodeStatus::Running,
        DAGNodeStatus::SelfHealing,
        DAGNodeStatus::Running,
        DAGNodeStatus::SelfHealing,
        DAGNodeStatus::Running,
        DAGNodeStatus::Failed,
    ];

    for state in transitions {
        node.transition_to(state);
        assert_eq!(node.status, state);
    }
    assert!(node.is_terminal());
    assert!(node.completed_at.is_some());
    let initial_completed = node.completed_at;

    // Transitioning after terminal state should retain completion timestamp
    node.transition_to(DAGNodeStatus::Success);
    assert_eq!(node.status, DAGNodeStatus::Success);
    assert_eq!(node.completed_at, initial_completed);

    // 2. High dependency count (5,000 dependencies)
    let deps: Vec<String> = (0..5000).map(|i| format!("dep_node_{i:04}")).collect();
    let large_dep_node = XenoDAGNode::new(
        "hub_node",
        "Consolidator",
        DAGNodeType::VerificationGate,
        ModelAssignment::new(ProviderKind::Groq, "llama-3.3-70b-versatile", 0.1),
    )
    .with_dependencies(deps.clone());

    assert_eq!(large_dep_node.dependencies.len(), 5000);
    let serialized_hub = serde_json::to_string(&large_dep_node).unwrap();
    let deserialized_hub: XenoDAGNode = serde_json::from_str(&serialized_hub).unwrap();
    assert_eq!(deserialized_hub.dependencies.len(), 5000);
    assert_eq!(deserialized_hub.dependencies[4999], "dep_node_4999");
}

#[test]
fn test_adversarial_multimodal_and_stream_chunk_extremes() {
    // 1. Massive chunk index (u64::MAX)
    let max_chunk = StreamChunk {
        chunk_index: u64::MAX,
        delta: StreamChunkDelta::ToolCallDelta {
            index: u32::MAX,
            id: Some("call_extreme".into()),
            name: Some("tool_extreme".into()),
            arguments_delta: "{\"massive\": 999999999}".into(),
        },
        stop_reason: Some(StopReason::ToolUse),
        partial_metrics: Some(TokenMetrics::new(1000, 500, 200, 10, 50, 1000.0, 0.05)),
    };
    let json_chunk = serde_json::to_string(&max_chunk).unwrap();
    let de_chunk: StreamChunk = serde_json::from_str(&json_chunk).unwrap();
    assert_eq!(de_chunk.chunk_index, u64::MAX);
    assert_eq!(de_chunk.stop_reason, Some(StopReason::ToolUse));

    // 2. Multimodal message with thousands of mixed content blocks
    let mut large_msg = ChatMessage::user_text("Initial prompt");
    for i in 0..1000 {
        if i % 3 == 0 {
            large_msg.add_content(ContentBlock::text(format!("Block text {i}")));
        } else if i % 3 == 1 {
            large_msg.add_content(ContentBlock::thinking(format!("Thought step {i}")));
        } else {
            large_msg.add_content(ContentBlock::tool_result(format!("call_{i}"), format!("Result {i}"), i % 2 == 0));
        }
    }
    assert_eq!(large_msg.content.len(), 1001);
    let json_large_msg = serde_json::to_string(&large_msg).unwrap();
    let de_large_msg: ChatMessage = serde_json::from_str(&json_large_msg).unwrap();
    assert_eq!(de_large_msg.content.len(), 1001);
    assert!(!de_large_msg.text_content().is_empty());
}
