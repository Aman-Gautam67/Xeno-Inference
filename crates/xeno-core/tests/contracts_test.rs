//! Integration tests for xeno-core contracts and data models.

use serde_json::json;
use xeno_core::prelude::*;

#[test]
fn test_chat_message_constructors_and_helpers() {
    let user_msg = ChatMessage::user_text("Hello XENO").with_name("Developer");
    assert_eq!(user_msg.role, Role::User);
    assert_eq!(user_msg.name.as_deref(), Some("Developer"));
    assert_eq!(user_msg.text_content(), "Hello XENO");

    let sys_msg = ChatMessage::system("You are a helpful assistant.");
    assert_eq!(sys_msg.role, Role::System);
    assert_eq!(sys_msg.text_content(), "You are a helpful assistant.");

    let tool_msg = ChatMessage::tool_result("call_123", "{\"status\": \"ok\"}", false);
    assert_eq!(tool_msg.role, Role::Tool);
    assert_eq!(tool_msg.content.len(), 1);
    match &tool_msg.content[0] {
        ContentBlock::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => {
            assert_eq!(tool_use_id, "call_123");
            assert_eq!(content, "{\"status\": \"ok\"}");
            assert!(!is_error);
        }
        _ => panic!("Expected ToolResult content block"),
    }
}

#[test]
fn test_content_block_polymorphic_serde() {
    let text_block = ContentBlock::text("sample text");
    let json_text = serde_json::to_string(&text_block).unwrap();
    assert!(json_text.contains("\"type\":\"text\""));
    let de_text: ContentBlock = serde_json::from_str(&json_text).unwrap();
    assert_eq!(de_text, text_block);

    let thinking_block = ContentBlock::thinking("Analyzing problem space...");
    let json_think = serde_json::to_string(&thinking_block).unwrap();
    assert!(json_think.contains("\"type\":\"thinking\""));
    let de_think: ContentBlock = serde_json::from_str(&json_think).unwrap();
    assert_eq!(de_think, thinking_block);

    let tool_use = ContentBlock::tool_use(
        "call_abc",
        "terminal_exec",
        json!({ "command": "cargo test" }),
    );
    let json_tool = serde_json::to_string(&tool_use).unwrap();
    assert!(json_tool.contains("\"type\":\"tool_use\""));
    let de_tool: ContentBlock = serde_json::from_str(&json_tool).unwrap();
    assert_eq!(de_tool, tool_use);
}

#[test]
fn test_inference_request_builder_and_serde() {
    let tool = ToolDefinition::new(
        "file_read",
        "Reads slice of file",
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" }
            },
            "required": ["path"]
        }),
    );

    let request = InferenceRequest::new(
        "claude-3-7-sonnet-20250219",
        vec![ChatMessage::user_text("Implement feature X")],
    )
    .with_system_prompt("System rules")
    .with_temperature(0.2)
    .with_max_tokens(8192)
    .with_stream(true)
    .with_tools(vec![tool])
    .with_reasoning_effort("high");

    assert_eq!(request.model, "claude-3-7-sonnet-20250219");
    assert_eq!(request.temperature, 0.2);
    assert_eq!(request.max_tokens, Some(8192));
    assert!(request.stream);
    assert_eq!(request.tools.len(), 1);
    assert_eq!(request.reasoning_effort.as_deref(), Some("high"));

    let json_req = serde_json::to_string_pretty(&request).unwrap();
    assert!(json_req.contains("\"systemPrompt\": \"System rules\""));
    assert!(json_req.contains("\"maxTokens\": 8192"));
    assert!(json_req.contains("\"reasoningEffort\": \"high\""));

    let de_req: InferenceRequest = serde_json::from_str(&json_req).unwrap();
    assert_eq!(de_req.model, request.model);
    assert_eq!(de_req.messages.len(), 1);
    assert_eq!(de_req.tools.len(), 1);
}

#[test]
fn test_inference_response_accessors_and_serde() {
    let metrics = TokenMetrics::new(120, 80, 250, 42, 1200, 275.0, 0.0045);
    let response = InferenceResponse {
        id: "resp_test_001".into(),
        model: "deepseek-reasoner".into(),
        content: vec![
            ContentBlock::thinking("I will execute the command first."),
            ContentBlock::tool_use("call_1", "terminal_exec", json!({"cmd": "ls"})),
            ContentBlock::text("Command queued."),
        ],
        stop_reason: StopReason::ToolUse,
        metrics,
    };

    assert_eq!(response.text_content(), "Command queued.");
    assert_eq!(
        response.thinking_content().as_deref(),
        Some("I will execute the command first.")
    );
    let tools = response.tool_uses();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].0, "call_1");
    assert_eq!(tools[0].1, "terminal_exec");
    assert!(response.is_success());

    let json_resp = serde_json::to_string(&response).unwrap();
    let de_resp: InferenceResponse = serde_json::from_str(&json_resp).unwrap();
    assert_eq!(de_resp.id, response.id);
    assert_eq!(de_resp.stop_reason, StopReason::ToolUse);
    assert_eq!(de_resp.metrics.ttft_ms, 42);
}

#[test]
fn test_stream_chunk_variants_and_serde() {
    let text_chunk = StreamChunk::text(0, "Hello");
    assert_eq!(text_chunk.chunk_index, 0);
    let json_text = serde_json::to_string(&text_chunk).unwrap();
    assert!(json_text.contains("\"type\":\"text_delta\""));

    let think_chunk = StreamChunk::thinking(1, "Step 1 reasoning");
    let json_think = serde_json::to_string(&think_chunk).unwrap();
    assert!(json_think.contains("\"type\":\"thinking_delta\""));

    let tool_chunk = StreamChunk::tool_call(
        2,
        0,
        Some("call_123".into()),
        Some("terminal_exec".into()),
        "{\"arg\":",
    );
    let json_tool = serde_json::to_string(&tool_chunk).unwrap();
    assert!(json_tool.contains("\"type\":\"tool_call_delta\""));

    let de_tool: StreamChunk = serde_json::from_str(&json_tool).unwrap();
    assert_eq!(de_tool.chunk_index, 2);
}

#[test]
fn test_provider_config_and_privacy_filter() {
    let mut config = ProviderConfig::new(ProviderKind::Openai)
        .with_api_key("sk-test-key")
        .with_base_url("https://api.openai.com/v1")
        .with_timeout_ms(15_000);
    config
        .extra_headers
        .insert("X-Custom-Header".into(), "value123".into());

    assert_eq!(config.provider, ProviderKind::Openai);
    assert_eq!(config.api_key.as_deref(), Some("sk-test-key"));
    assert_eq!(config.timeout_ms, Some(15_000));

    let filter_strict = PrivacyFilter::air_gapped();
    assert!(filter_strict.air_gap_mode);
    assert!(filter_strict.redact_secrets);

    let filter_off = PrivacyFilter::disabled();
    assert!(!filter_off.enabled);
}
