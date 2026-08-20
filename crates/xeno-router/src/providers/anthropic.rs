//! Anthropic Messages API v1 provider client with Thinking / CoT and tool streaming.

use super::sse::parse_sse_stream;
use crate::provider::{BoxStream, HealthStatus, InferenceProvider, ModelInfo};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use xeno_core::{
    contracts::{
        ContentBlock, InferenceRequest, InferenceResponse, Role, StopReason, StreamChunk,
        StreamChunkDelta,
    },
    errors::XenoError,
    metrics::TokenMetrics,
    types::ProviderKind,
};

/// Anthropic Messages API v1 client adapter.
#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    name: String,
    base_url: String,
    api_key: String,
    anthropic_version: String,
    client: Client,
    timeout: Duration,
}

impl AnthropicProvider {
    /// Constructs a new [`AnthropicProvider`] with the specified API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            name: "anthropic".into(),
            base_url: "https://api.anthropic.com/v1".into(),
            api_key: api_key.into(),
            anthropic_version: "2023-06-01".into(),
            client: Client::builder().build().unwrap_or_default(),
            timeout: Duration::from_secs(120),
        }
    }

    /// Sets custom base URL (useful for testing or reverse proxies).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        let mut url = base_url.into();
        if url.ends_with('/') {
            url.pop();
        }
        self.base_url = url;
        self
    }

    /// Sets custom request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn map_stop_reason(reason: Option<&str>) -> StopReason {
        match reason {
            Some("end_turn") => StopReason::EndTurn,
            Some("max_tokens") => StopReason::MaxTokens,
            Some("stop_sequence") => StopReason::StopSequence,
            Some("tool_use") => StopReason::ToolUse,
            _ => StopReason::EndTurn,
        }
    }

    fn build_wire_payload(&self, req: &InferenceRequest, stream: bool) -> AnthropicWireRequest {
        let mut wire_messages = Vec::new();

        for msg in &req.messages {
            // Anthropic Messages API only accepts 'user' and 'assistant' roles in the messages list
            let role = match msg.role {
                Role::User | Role::Tool => "user",
                Role::Assistant => "assistant",
                Role::System => continue, // System prompt handled separately at top-level
            };

            let mut content_blocks = Vec::new();
            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        content_blocks.push(AnthropicContentBlock::Text { text: text.clone() });
                    }
                    ContentBlock::Thinking { reasoning } => {
                        content_blocks.push(AnthropicContentBlock::Thinking {
                            thinking: reasoning.clone(),
                        });
                    }
                    ContentBlock::Image {
                        media_type,
                        data_base64,
                    } => {
                        content_blocks.push(AnthropicContentBlock::Image {
                            source: AnthropicImageSource {
                                source_type: "base64".into(),
                                media_type: media_type.clone(),
                                data: data_base64.clone(),
                            },
                        });
                    }
                    ContentBlock::ToolUse { id, name, input } => {
                        content_blocks.push(AnthropicContentBlock::ToolUse {
                            id: id.clone(),
                            name: name.clone(),
                            input: input.clone(),
                        });
                    }
                    ContentBlock::ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                    } => {
                        content_blocks.push(AnthropicContentBlock::ToolResult {
                            tool_use_id: tool_use_id.clone(),
                            content: content.clone(),
                            is_error: if *is_error { Some(true) } else { None },
                        });
                    }
                }
            }

            if !content_blocks.is_empty() {
                wire_messages.push(AnthropicWireMessage {
                    role: role.into(),
                    content: content_blocks,
                });
            }
        }

        // Configure thinking if requested
        let thinking = if req.reasoning_effort.is_some() || req.model.contains("thinking") {
            let max_tok = req.max_tokens.unwrap_or(4096);
            let budget = (max_tok / 2).max(1024).min(max_tok.saturating_sub(1024));
            Some(AnthropicThinkingConfig {
                thinking_type: "enabled".into(),
                budget_tokens: budget,
            })
        } else {
            None
        };

        // Tools definition
        let tools = if req.tools.is_empty() {
            None
        } else {
            Some(
                req.tools
                    .iter()
                    .map(|t| AnthropicTool {
                        name: t.name.clone(),
                        description: t.description.clone(),
                        input_schema: t.input_schema.clone(),
                    })
                    .collect(),
            )
        };

        AnthropicWireRequest {
            model: req.model.clone(),
            messages: wire_messages,
            system: req.system_prompt.clone(),
            max_tokens: req.max_tokens.unwrap_or(4096),
            temperature: if thinking.is_some() { None } else { Some(req.temperature) },
            top_p: req.top_p,
            stop_sequences: if req.stop_sequences.is_empty() {
                None
            } else {
                Some(req.stop_sequences.clone())
            },
            tools,
            thinking,
            stream,
        }
    }
}

#[derive(Serialize)]
struct AnthropicWireRequest {
    model: String,
    messages: Vec<AnthropicWireMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<AnthropicThinkingConfig>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct AnthropicWireMessage {
    role: String,
    content: Vec<AnthropicContentBlock>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContentBlock {
    Text {
        text: String,
    },
    Thinking {
        thinking: String,
    },
    Image {
        source: AnthropicImageSource,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

#[derive(Serialize, Deserialize)]
struct AnthropicImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

#[derive(Serialize, Deserialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct AnthropicThinkingConfig {
    #[serde(rename = "type")]
    thinking_type: String,
    budget_tokens: u32,
}

#[derive(Deserialize)]
struct AnthropicWireResponse {
    id: String,
    model: String,
    content: Vec<AnthropicContentBlock>,
    stop_reason: Option<String>,
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

// SSE Event payload schemas
#[derive(Deserialize)]
struct SseContentBlockStart {
    index: u32,
    content_block: Option<AnthropicContentBlock>,
}

#[derive(Deserialize)]
struct SseContentBlockDelta {
    index: u32,
    delta: Option<SseDeltaPayload>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SseDeltaPayload {
    TextDelta { text: String },
    ThinkingDelta { thinking: String },
    InputJsonDelta { partial_json: String },
}

#[derive(Deserialize)]
struct SseMessageDelta {
    delta: Option<SseMessageDeltaInner>,
}

#[derive(Deserialize)]
struct SseMessageDeltaInner {
    stop_reason: Option<String>,
}

#[async_trait]
impl InferenceProvider for AnthropicProvider {
    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Anthropic
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn model_info(&self, model: &str) -> Result<ModelInfo, XenoError> {
        let (in_cost, out_cost, reasoning) = match model {
            "claude-3-7-sonnet-20250219" | "claude-3-7-sonnet" => (3.0, 15.0, true),
            "claude-3-5-sonnet-20241022" | "claude-3-5-sonnet" => (3.0, 15.0, false),
            "claude-3-5-haiku-20241022" | "claude-3-5-haiku" => (0.80, 4.0, false),
            _ => (3.0, 15.0, false),
        };

        Ok(ModelInfo {
            id: model.to_string(),
            provider: ProviderKind::Anthropic,
            context_window: 200_000,
            max_output_tokens: 8_192,
            supports_streaming: true,
            supports_tools: true,
            supports_reasoning: reasoning,
            input_cost_per_million: in_cost,
            output_cost_per_million: out_cost,
        })
    }

    async fn health_check(&self) -> Result<HealthStatus, XenoError> {
        if self.api_key.trim().is_empty() {
            return Ok(HealthStatus::Unhealthy {
                reason: "Anthropic API key is not set".into(),
            });
        }
        Ok(HealthStatus::Healthy)
    }

    async fn complete(&self, req: &InferenceRequest) -> Result<InferenceResponse, XenoError> {
        let endpoint = format!("{}/messages", self.base_url);
        let wire_req = self.build_wire_payload(req, false);

        let resp = self
            .client
            .post(&endpoint)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.anthropic_version)
            .header("content-type", "application/json")
            .json(&wire_req)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| XenoError::NetworkError {
                message: format!("Anthropic request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: status.as_u16(),
                message: format!("Anthropic upstream error: {err_body}"),
            });
        }

        let wire_resp: AnthropicWireResponse = resp.json().await.map_err(|e| {
            XenoError::Inference(xeno_core::errors::InferenceError::MalformedResponse(
                format!("JSON error: {e}"),
            ))
        })?;

        let mut content = Vec::new();
        let mut reasoning_tokens = 0;

        for block in wire_resp.content {
            match block {
                AnthropicContentBlock::Text { text } => {
                    content.push(ContentBlock::text(text));
                }
                AnthropicContentBlock::Thinking { thinking } => {
                    reasoning_tokens = (thinking.len() / 4) as u32;
                    content.push(ContentBlock::thinking(thinking));
                }
                AnthropicContentBlock::ToolUse { id, name, input } => {
                    content.push(ContentBlock::tool_use(id, name, input));
                }
                _ => {}
            }
        }

        let stop_reason = Self::map_stop_reason(wire_resp.stop_reason.as_deref());
        let prompt_tokens = wire_resp.usage.as_ref().and_then(|u| u.input_tokens).unwrap_or(0);
        let completion_tokens = wire_resp.usage.as_ref().and_then(|u| u.output_tokens).unwrap_or(0);

        let metrics = TokenMetrics::new(prompt_tokens, completion_tokens, reasoning_tokens, 0, 0, 0.0, 0.0);

        Ok(InferenceResponse {
            id: wire_resp.id,
            model: wire_resp.model,
            content,
            stop_reason,
            metrics,
        })
    }

    async fn stream(&self, req: &InferenceRequest) -> Result<BoxStream<StreamChunk>, XenoError> {
        let endpoint = format!("{}/messages", self.base_url);
        let wire_req = self.build_wire_payload(req, true);

        let resp = self
            .client
            .post(&endpoint)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.anthropic_version)
            .header("content-type", "application/json")
            .json(&wire_req)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| XenoError::NetworkError {
                message: format!("Anthropic streaming request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: status.as_u16(),
                message: format!("Anthropic streaming error: {err_body}"),
            });
        }

        let mut sse_stream = parse_sse_stream(resp.bytes_stream());
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        tokio::spawn(async move {
            let mut chunk_idx = 0;

            while let Some(sse_res) = sse_stream.next().await {
                match sse_res {
                    Ok(event) => {
                        let event_type = event.event_type.as_deref().unwrap_or("");

                        match event_type {
                            "content_block_start" => {
                                if let Ok(parsed) = serde_json::from_str::<SseContentBlockStart>(&event.data) {
                                    if let Some(AnthropicContentBlock::ToolUse { id, name, .. }) = parsed.content_block {
                                        let chunk = StreamChunk {
                                            chunk_index: chunk_idx,
                                            delta: StreamChunkDelta::ToolCallDelta {
                                                index: parsed.index,
                                                id: Some(id),
                                                name: Some(name),
                                                arguments_delta: String::new(),
                                            },
                                            stop_reason: None,
                                            partial_metrics: None,
                                        };
                                        chunk_idx += 1;
                                        if tx.send(Ok(chunk)).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                            }
                            "content_block_delta" => {
                                if let Ok(parsed) = serde_json::from_str::<SseContentBlockDelta>(&event.data) {
                                    if let Some(delta) = parsed.delta {
                                        let stream_delta = match delta {
                                            SseDeltaPayload::TextDelta { text } => {
                                                StreamChunkDelta::TextDelta { text }
                                            }
                                            SseDeltaPayload::ThinkingDelta { thinking } => {
                                                StreamChunkDelta::ThinkingDelta { reasoning: thinking }
                                            }
                                            SseDeltaPayload::InputJsonDelta { partial_json } => {
                                                StreamChunkDelta::ToolCallDelta {
                                                    index: parsed.index,
                                                    id: None,
                                                    name: None,
                                                    arguments_delta: partial_json,
                                                }
                                            }
                                        };

                                        let chunk = StreamChunk {
                                            chunk_index: chunk_idx,
                                            delta: stream_delta,
                                            stop_reason: None,
                                            partial_metrics: None,
                                        };
                                        chunk_idx += 1;
                                        if tx.send(Ok(chunk)).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                            }
                            "message_delta" => {
                                if let Ok(parsed) = serde_json::from_str::<SseMessageDelta>(&event.data) {
                                    let stop_reason = parsed
                                        .delta
                                        .and_then(|d| d.stop_reason)
                                        .map(|r| Self::map_stop_reason(Some(&r)));

                                    let chunk = StreamChunk {
                                        chunk_index: chunk_idx,
                                        delta: StreamChunkDelta::TextDelta { text: String::new() },
                                        stop_reason,
                                        partial_metrics: None,
                                    };
                                    chunk_idx += 1;
                                    let _ = tx.send(Ok(chunk)).await;
                                }
                            }
                            "message_stop" => {
                                return;
                            }
                            _ => {}
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(Err(err)).await;
                        return;
                    }
                }
            }
        });

        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
