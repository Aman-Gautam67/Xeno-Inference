//! WireMock HTTP integration tests for all real provider adapters.

use futures_util::StreamExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xeno_core::contracts::{ChatMessage, InferenceRequest, StopReason, StreamChunkDelta};
use xeno_router::prelude::*;

#[tokio::test]
async fn test_local_openai_provider_wiremock() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "id": "chatcmpl-123",
        "model": "llama-3-8b",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello from local llama.cpp!"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 15,
            "completion_tokens": 8
        }
    });

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let provider = LocalOpenAIProvider::new("local-test", format!("{}/v1", mock_server.uri()), None);
    let req = InferenceRequest::new("llama-3-8b", vec![ChatMessage::user_text("hi")]);

    let resp = provider.complete(&req).await.unwrap();
    assert_eq!(resp.text_content(), "Hello from local llama.cpp!");
    assert_eq!(resp.stop_reason, StopReason::EndTurn);
    assert_eq!(resp.metrics.prompt_tokens, 15);
    assert_eq!(resp.metrics.completion_tokens, 8);
}

#[tokio::test]
async fn test_anthropic_provider_wiremock_thinking_stream() {
    let mock_server = MockServer::start().await;

    let sse_body = "event: message_start\ndata: {\"type\": \"message_start\", \"message\": {\"id\": \"msg_1\", \"type\": \"message\", \"role\": \"assistant\", \"model\": \"claude-3-7-sonnet-20250219\", \"usage\": {\"input_tokens\": 25, \"output_tokens\": 1}}}\n\n\
event: content_block_start\ndata: {\"type\": \"content_block_start\", \"index\": 0, \"content_block\": {\"type\": \"thinking\", \"thinking\": \"\"}}\n\n\
event: content_block_delta\ndata: {\"type\": \"content_block_delta\", \"index\": 0, \"delta\": {\"type\": \"thinking_delta\", \"thinking\": \"Decomposing problem into AST components...\"}}\n\n\
event: content_block_stop\ndata: {\"type\": \"content_block_stop\", \"index\": 0}\n\n\
event: content_block_start\ndata: {\"type\": \"content_block_start\", \"index\": 1, \"content_block\": {\"type\": \"text\", \"text\": \"\"}}\n\n\
event: content_block_delta\ndata: {\"type\": \"content_block_delta\", \"index\": 1, \"delta\": {\"type\": \"text_delta\", \"text\": \"Here is the generated patch.\"}}\n\n\
event: content_block_stop\ndata: {\"type\": \"content_block_stop\", \"index\": 1}\n\n\
event: message_delta\ndata: {\"type\": \"message_delta\", \"delta\": {\"stop_reason\": \"end_turn\"}, \"usage\": {\"output_tokens\": 45}}\n\n\
event: message_stop\ndata: {\"type\": \"message_stop\"}\n\n";

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "test-anthropic-key"))
        .and(header("anthropic-version", "2023-06-01"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_body),
        )
        .mount(&mock_server)
        .await;

    let provider = AnthropicProvider::new("test-anthropic-key")
        .with_base_url(format!("{}/v1", mock_server.uri()));

    let req = InferenceRequest::new("claude-3-7-sonnet-20250219", vec![ChatMessage::user_text("Solve task")])
        .with_reasoning_effort("high");

    let mut stream = provider.stream(&req).await.unwrap();

    let mut thinking_accum = String::new();
    let mut text_accum = String::new();

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.unwrap();
        match chunk.delta {
            StreamChunkDelta::ThinkingDelta { reasoning } => {
                thinking_accum.push_str(&reasoning);
            }
            StreamChunkDelta::TextDelta { text } => {
                text_accum.push_str(&text);
            }
            _ => {}
        }
    }

    assert_eq!(thinking_accum, "Decomposing problem into AST components...");
    assert_eq!(text_accum, "Here is the generated patch.");
}

#[tokio::test]
async fn test_openai_provider_wiremock_reasoning_tokens() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "id": "chatcmpl-o1-999",
        "model": "o1",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Mathematical proof complete."
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 120,
            "completion_tokens": 350,
            "completion_tokens_details": {
                "reasoning_tokens": 300
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer test-openai-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let provider = OpenAIProvider::new("test-openai-key")
        .with_base_url(format!("{}/v1", mock_server.uri()));

    let req = InferenceRequest::new("o1", vec![ChatMessage::user_text("Prove theorem")])
        .with_reasoning_effort("high");

    let resp = provider.complete(&req).await.unwrap();
    assert_eq!(resp.text_content(), "Mathematical proof complete.");
    assert_eq!(resp.metrics.prompt_tokens, 120);
    assert_eq!(resp.metrics.completion_tokens, 350);
    assert_eq!(resp.metrics.reasoning_tokens, 300);
}

#[tokio::test]
async fn test_gemini_provider_wiremock_stream() {
    let mock_server = MockServer::start().await;

    let sse_body = "data: {\"candidates\": [{\"content\": {\"parts\": [{\"text\": \"Gemini \"}], \"role\": \"model\"}}]}\n\n\
data: {\"candidates\": [{\"content\": {\"parts\": [{\"text\": \"2.0 Flash response.\"}], \"role\": \"model\"}, \"finishReason\": \"STOP\"}], \"usageMetadata\": {\"promptTokenCount\": 10, \"candidatesTokenCount\": 6}}\n\n";

    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-2.0-flash:streamGenerateContent"))
        .and(header("x-goog-api-key", "test-gemini-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_body),
        )
        .mount(&mock_server)
        .await;

    let provider = GeminiProvider::new("test-gemini-key")
        .with_base_url(format!("{}/v1beta/models", mock_server.uri()));

    let req = InferenceRequest::new("gemini-2.0-flash", vec![ChatMessage::user_text("test")]);
    let mut stream = provider.stream(&req).await.unwrap();

    let mut text_accum = String::new();
    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.unwrap();
        if let StreamChunkDelta::TextDelta { text } = chunk.delta {
            text_accum.push_str(&text);
        }
    }

    assert_eq!(text_accum, "Gemini 2.0 Flash response.");
}

#[tokio::test]
async fn test_deepseek_provider_wiremock_r1_reasoning() {
    let mock_server = MockServer::start().await;

    let sse_body = "data: {\"choices\": [{\"delta\": {\"reasoning_content\": \"Thinking step 1...\"}}]}\n\n\
data: {\"choices\": [{\"delta\": {\"reasoning_content\": \"Thinking step 2...\"}}]}\n\n\
data: {\"choices\": [{\"delta\": {\"content\": \"DeepSeek R1 answer.\"}}]}\n\n\
data: {\"choices\": [{\"delta\": {}, \"finish_reason\": \"stop\"}]}\n\n\
data: [DONE]\n\n";

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer test-deepseek-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_body),
        )
        .mount(&mock_server)
        .await;

    let provider = DeepSeekProvider::new("test-deepseek-key")
        .with_base_url(format!("{}/v1", mock_server.uri()));

    let req = InferenceRequest::new("deepseek-reasoner", vec![ChatMessage::user_text("Analyze code")]);
    let mut stream = provider.stream(&req).await.unwrap();

    let mut thinking_accum = String::new();
    let mut text_accum = String::new();

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.unwrap();
        match chunk.delta {
            StreamChunkDelta::ThinkingDelta { reasoning } => {
                thinking_accum.push_str(&reasoning);
            }
            StreamChunkDelta::TextDelta { text } => {
                text_accum.push_str(&text);
            }
            _ => {}
        }
    }

    assert_eq!(thinking_accum, "Thinking step 1...Thinking step 2...");
    assert_eq!(text_accum, "DeepSeek R1 answer.");
}

#[tokio::test]
async fn test_groq_provider_wiremock_fast_completion() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "id": "chatcmpl-groq-111",
        "model": "llama-3.3-70b-versatile",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Ultra fast LPU output."
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_time": 0.015
        }
    });

    Mock::given(method("POST"))
        .and(path("/openai/v1/chat/completions"))
        .and(header("authorization", "Bearer test-groq-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let provider = GroqProvider::new("test-groq-key")
        .with_base_url(format!("{}/openai/v1", mock_server.uri()));

    let req = InferenceRequest::new("llama-3.3-70b-versatile", vec![ChatMessage::user_text("fast")]);
    let resp = provider.complete(&req).await.unwrap();

    assert_eq!(resp.text_content(), "Ultra fast LPU output.");
    assert_eq!(resp.metrics.prompt_tokens, 10);
    assert_eq!(resp.metrics.completion_tokens, 5);
}
