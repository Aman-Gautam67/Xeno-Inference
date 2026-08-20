//! Integration tests for XenoAgentStepEvent and Blueprint §13 compliance.

use serde_json::json;
use xeno_core::prelude::*;

#[test]
fn test_agent_step_event_thinking_phase() {
    let telemetry = TelemetryPayload {
        model_used: "claude-3-7-sonnet".into(),
        backend_type: BackendType::CloudApi,
        vram_allocated_bytes: 0,
        prompt_tokens: 500,
        completion_tokens: 0,
        reasoning_tokens: 350,
        estimated_cost_usd: 0.00675,
        ttft_ms: 220,
        total_latency_ms: 1500,
        tokens_per_second: 233.3,
    };

    let thinking = ThinkingPayload::new(
        "Decomposing task into 3 sub-tasks: 1. read file, 2. edit AST, 3. verify.",
        1500,
        233.3,
        "branch_main_01",
        false,
    );

    let event = XenoAgentStepEvent::thinking_step(
        "session_xyz_789",
        AgentRole::Commander,
        thinking,
        telemetry,
    );

    assert_eq!(event.agent_role, AgentRole::Commander);
    assert_eq!(event.execution_phase, ExecutionPhase::Thinking);
    assert!(event.thinking.is_some());
    assert!(event.tool_call.is_none());
    assert!(event.observation.is_none());

    let json_str = serde_json::to_string_pretty(&event).unwrap();

    // Blueprint §13 field name verification
    assert!(json_str.contains("\"eventId\""));
    assert!(json_str.contains("\"timestamp\""));
    assert!(json_str.contains("\"sessionId\": \"session_xyz_789\""));
    assert!(json_str.contains("\"agentRole\": \"commander\""));
    assert!(json_str.contains("\"executionPhase\": \"thinking\""));
    assert!(json_str.contains("\"rawTokens\""));
    assert!(json_str.contains("\"elapsedMs\": 1500"));
    assert!(json_str.contains("\"tokensPerSecond\": 233.3"));
    assert!(json_str.contains("\"branchId\": \"branch_main_01\""));
    assert!(json_str.contains("\"isPruned\": false"));
    assert!(json_str.contains("\"modelUsed\": \"claude-3-7-sonnet\""));
    assert!(json_str.contains("\"backendType\": \"cloud_api\""));

    let de_event: XenoAgentStepEvent = serde_json::from_str(&json_str).unwrap();
    assert_eq!(de_event.session_id, event.session_id);
    assert_eq!(de_event.agent_role, AgentRole::Commander);
    assert_eq!(de_event.execution_phase, ExecutionPhase::Thinking);
    assert_eq!(
        de_event.thinking.as_ref().unwrap().branch_id,
        "branch_main_01"
    );
}

#[test]
fn test_agent_step_event_tool_call_phase() {
    let telemetry = TelemetryPayload {
        model_used: "deepseek-r1".into(),
        backend_type: BackendType::LocalVllm,
        vram_allocated_bytes: 16_000_000_000,
        prompt_tokens: 1200,
        completion_tokens: 150,
        reasoning_tokens: 800,
        estimated_cost_usd: 0.0,
        ttft_ms: 45,
        total_latency_ms: 850,
        tokens_per_second: 176.4,
    };

    let tool_call = ToolCallPayload::with_mcp(
        "call_456",
        "ast_replace",
        json!({ "file": "src/main.rs", "target": "fn old()", "replacement": "fn new()" }),
        "mcp-ast-tools",
    );

    let event =
        XenoAgentStepEvent::tool_call_step("session_abc_123", AgentRole::Coder, tool_call, telemetry);

    assert_eq!(event.agent_role, AgentRole::Coder);
    assert_eq!(event.execution_phase, ExecutionPhase::ToolCall);
    assert!(event.tool_call.is_some());

    let json_str = serde_json::to_string_pretty(&event).unwrap();
    assert!(json_str.contains("\"agentRole\": \"coder\""));
    assert!(json_str.contains("\"executionPhase\": \"tool_call\""));
    assert!(json_str.contains("\"callId\": \"call_456\""));
    assert!(json_str.contains("\"toolName\": \"ast_replace\""));
    assert!(json_str.contains("\"mcpServer\": \"mcp-ast-tools\""));

    let de_event: XenoAgentStepEvent = serde_json::from_str(&json_str).unwrap();
    assert_eq!(de_event.tool_call.unwrap().tool_name, "ast_replace");
}

#[test]
fn test_agent_step_event_observation_and_verified_phases() {
    let metrics = TokenMetrics::new(300, 50, 0, 10, 100, 500.0, 0.0);
    let telemetry = TelemetryPayload::from_metrics(
        "llama-3.3-70b",
        BackendType::LocalGguf,
        14_000_000_000,
        &metrics,
    );

    let mut obs = ObservationPayload::success("Build succeeded (0 errors).");
    obs.diff_snippet = Some("+ pub fn calculate() -> u32 { 42 }".into());

    let obs_event = XenoAgentStepEvent::observation_step(
        "session_test",
        AgentRole::Tester,
        obs,
        telemetry.clone(),
    );

    assert_eq!(obs_event.agent_role, AgentRole::Tester);
    assert_eq!(obs_event.execution_phase, ExecutionPhase::Observation);
    let obs_payload = obs_event.observation.as_ref().unwrap();
    assert_eq!(obs_payload.exit_code, 0);
    assert!(obs_payload.ast_validation_passed);
    assert!(obs_payload.diff_snippet.is_some());

    let verified_event =
        XenoAgentStepEvent::verified_step("session_test", AgentRole::Security, telemetry);
    assert_eq!(verified_event.execution_phase, ExecutionPhase::Verified);
    assert_eq!(verified_event.agent_role, AgentRole::Security);
}
