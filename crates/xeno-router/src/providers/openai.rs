//! OpenAI Chat Completions and reasoning models (GPT-4o, o1, o3-mini) provider adapter.

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

/// OpenAI chat completions and reasoning model provider adapter.
#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    name: String,
    base_url: String,
    api_key: String,
    organization: Option<String>,
    client: Client,
    timeout: Duration,
}

impl OpenAIProvider {
    /// Constructs a new [`OpenAIProvider`] with an API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            name: "openai".into(),
            base_url: "https://api.openai.com/v1".into(),
            api_key: api_key.into(),
            organization: None,
            client: Client::builder().build().unwrap_or_default(),
            timeout: Duration::from_secs(120),
        }
    }

    /// Sets custom base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        let mut url = base_url.into();
        if url.ends_with('/') {
            url.pop();
        }
        self.base_url = url;
        self
    }

    /// Sets organization identifier header.
    pub fn with_organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    /// Sets custom timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn map_finish_reason(reason: Option<&str>) -> StopReason {
        match reason {
            Some("stop") => StopReason::EndTurn,
            Some("length") => StopReason::MaxTokens,
            Some("tool_calls") | Some("function_call") => StopReason::ToolUse,
            Some("content_filter") => StopReason::ContentFilter,
            _ => StopReason::EndTurn,
        }
    }

    fn build_wire_payload(&self, req: &InferenceRequest, stream: bool) -> OpenAiWireRequest {
        let mut wire_messages = Vec::new();

        if let Some(sys) = &req.system_prompt {
            wire_messages.push(OpenAiWireMessage {
                role: "system".into(),
                content: Some(sys.clone()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });
        }

        for msg in &req.messages {
            let role_str = match msg.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "tool",
            };

            let mut tool_calls = Vec::new();
            let mut text_parts = Vec::new();
            let mut tool_call_id = None;

            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        text_parts.push(text.clone());
                    }
                    ContentBlock::ToolUse { id, name, input } => {
                        tool_calls.push(OpenAiWireToolCall {
                            id: id.clone(),
                            tool_type: "function".into(),
                            function: OpenAiWireFunctionCall {
                                name: name.clone(),
                                arguments: serde_json::to_string(input).unwrap_or_default(),
                            },
                        });
                    }
                    ContentBlock::ToolResult {
                        tool_use_id,
                        content,
                        ..
                    } => {
                        tool_call_id = Some(tool_use_id.clone());
                        text_parts.push(content.clone());
                    }
                    _ => {}
                }
            }

            wire_messages.push(OpenAiWireMessage {
                role: role_str.into(),
                content: if text_parts.is_empty() {
                    None
                } else {
                    Some(text_parts.join("\n"))
                },
                tool_calls: if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls)
                },
                tool_call_id,
                name: msg.name.clone(),
            });
        }

        let tools = if req.tools.is_empty() {
            None
        } else {
            Some(
                req.tools
                    .iter()
                    .map(|t| OpenAiWireToolDef {
                        tool_type: "function".into(),
                        function: OpenAiWireFunctionDef {
                            name: t.name.clone(),
                            description: t.description.clone(),
                            parameters: t.input_schema.clone(),
                        },
                    })
                    .collect(),
            )
        };

        // If reasoning model (o1, o3-mini), temperature is often fixed or omitted
        let is_o_series = req.model.starts_with("o1") || req.model.starts_with("o3");

        OpenAiWireRequest {
            model: req.model.clone(),
            messages: wire_messages,
            temperature: if is_o_series { None } else { Some(req.temperature) },
            max_tokens: req.max_tokens,
            top_p: req.top_p,
            stop: if req.stop_sequences.is_empty() {
                None
            } else {
                Some(req.stop_sequences.clone())
            },
            tools,
            reasoning_effort: req.reasoning_effort.clone(),
            stream,
        }
    }
}

#[derive(Serialize)]
struct OpenAiWireRequest {
    model: String,
    messages: Vec<OpenAiWireMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAiWireToolDef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct OpenAiWireMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAiWireToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct OpenAiWireToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAiWireFunctionCall,
}

#[derive(Serialize, Deserialize)]
struct OpenAiWireFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Serialize, Deserialize)]
struct OpenAiWireToolDef {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAiWireFunctionDef,
}

#[derive(Serialize, Deserialize)]
struct OpenAiWireFunctionDef {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Deserialize)]
struct OpenAiWireResponse {
    id: String,
    model: String,
    choices: Vec<OpenAiWireChoice>,
    usage: Option<OpenAiWireUsage>,
}

#[derive(Deserialize)]
struct OpenAiWireChoice {
    message: OpenAiWireMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiWireUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    completion_tokens_details: Option<OpenAiCompletionDetails>,
}

#[derive(Deserialize)]
struct OpenAiCompletionDetails {
    reasoning_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Deserialize)]
struct OpenAiStreamChoice {
    delta: Option<OpenAiStreamDelta>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiStreamDelta {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAiStreamToolCall>>,
}

#[derive(Deserialize)]
struct OpenAiStreamToolCall {
    index: u32,
    id: Option<String>,
    function: Option<OpenAiStreamFunction>,
}

#[derive(Deserialize)]
struct OpenAiStreamFunction {
    name: Option<String>,
    arguments: Option<String>,
}

#[async_trait]
impl InferenceProvider for OpenAIProvider {
    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Openai
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn model_info(&self, model: &str) -> Result<ModelInfo, XenoError> {
        let (in_cost, out_cost, reasoning) = match model {
            "gpt-4o" => (2.50, 10.00, false),
            "gpt-4o-mini" => (0.15, 0.60, false),
            "o1" => (15.00, 60.00, true),
            "o3-mini" => (1.10, 4.40, true),
            _ => (2.50, 10.00, false),
        };

        Ok(ModelInfo {
            id: model.to_string(),
            provider: ProviderKind::Openai,
            context_window: 128_000,
            max_output_tokens: 16_384,
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
                reason: "OpenAI API key is not set".into(),
            });
        }
        Ok(HealthStatus::Healthy)
    }

    async fn complete(&self, req: &InferenceRequest) -> Result<InferenceResponse, XenoError> {
        let endpoint = format!("{}/chat/completions", self.base_url);
        let wire_req = self.build_wire_payload(req, false);

        let mut req_builder = self
            .client
            .post(&endpoint)
            .bearer_auth(&self.api_key)
            .json(&wire_req)
            .timeout(self.timeout);

        if let Some(org) = &self.organization {
            req_builder = req_builder.header("OpenAI-Organization", org);
        }

        let resp = req_builder.send().await.map_err(|e| XenoError::NetworkError {
            message: format!("OpenAI request failed: {e}"),
        })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: status.as_u16(),
                message: format!("OpenAI upstream error: {err_body}"),
            });
        }

        let wire_resp: OpenAiWireResponse = resp.json().await.map_err(|e| {
            XenoError::Inference(xeno_core::errors::InferenceError::MalformedResponse(
                format!("JSON error: {e}"),
            ))
        })?;

        let choice = wire_resp
            .choices
            .first()
            .ok_or_else(|| XenoError::Inference(
                xeno_core::errors::InferenceError::MalformedResponse("Empty choices from OpenAI".into()),
            ))?;

        let mut content = Vec::new();
        if let Some(text) = &choice.message.content {
            if !text.is_empty() {
                content.push(ContentBlock::text(text.clone()));
            }
        }

        if let Some(tool_calls) = &choice.message.tool_calls {
            for tc in tool_calls {
                let parsed_args: serde_json::Value =
                    serde_json::from_str(&tc.function.arguments).unwrap_or(serde_json::json!({}));
                content.push(ContentBlock::tool_use(
                    tc.id.clone(),
                    tc.function.name.clone(),
                    parsed_args,
                ));
            }
        }

        let stop_reason = Self::map_finish_reason(choice.finish_reason.as_deref());

        let prompt_tokens = wire_resp.usage.as_ref().and_then(|u| u.prompt_tokens).unwrap_or(0);
        let completion_tokens = wire_resp.usage.as_ref().and_then(|u| u.completion_tokens).unwrap_or(0);
        let reasoning_tokens = wire_resp
            .usage
            .as_ref()
            .and_then(|u| u.completion_tokens_details.as_ref())
            .and_then(|d| d.reasoning_tokens)
            .unwrap_or(0);

        let metrics = TokenMetrics::new(
            prompt_tokens,
            completion_tokens,
            reasoning_tokens,
            0,
            0,
            0.0,
            0.0,
        );

        Ok(InferenceResponse {
            id: wire_resp.id,
            model: wire_resp.model,
            content,
            stop_reason,
            metrics,
        })
    }

    async fn stream(&self, req: &InferenceRequest) -> Result<BoxStream<StreamChunk>, XenoError> {
        let endpoint = format!("{}/chat/completions", self.base_url);
        let wire_req = self.build_wire_payload(req, true);

        let mut req_builder = self
            .client
            .post(&endpoint)
            .bearer_auth(&self.api_key)
            .json(&wire_req)
            .timeout(self.timeout);

        if let Some(org) = &self.organization {
            req_builder = req_builder.header("OpenAI-Organization", org);
        }

        let resp = req_builder.send().await.map_err(|e| XenoError::NetworkError {
            message: format!("OpenAI streaming request failed: {e}"),
        })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: status.as_u16(),
                message: format!("OpenAI streaming error: {err_body}"),
            });
        }

        let mut sse_stream = parse_sse_stream(resp.bytes_stream());
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        tokio::spawn(async move {
            let mut chunk_idx = 0;

            while let Some(sse_res) = sse_stream.next().await {
                match sse_res {
                    Ok(event) => {
                        if event.is_done() {
                            let end_chunk = StreamChunk {
                                chunk_index: chunk_idx,
                                delta: StreamChunkDelta::TextDelta {
                                    text: String::new(),
                                },
                                stop_reason: Some(StopReason::EndTurn),
                                partial_metrics: None,
                            };
                            let _ = tx.send(Ok(end_chunk)).await;
                            return;
                        }

                        if let Ok(parsed) = serde_json::from_str::<OpenAiStreamChunk>(&event.data) {
                            if let Some(choice) = parsed.choices.first() {
                                if let Some(delta) = &choice.delta {
                                    if let Some(text) = &delta.content {
                                        if !text.is_empty() {
                                            let chunk = StreamChunk {
                                                chunk_index: chunk_idx,
                                                delta: StreamChunkDelta::TextDelta {
                                                    text: text.clone(),
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

                                    if let Some(tool_calls) = &delta.tool_calls {
                                        for tc in tool_calls {
                                            let chunk = StreamChunk {
                                                chunk_index: chunk_idx,
                                                delta: StreamChunkDelta::ToolCallDelta {
                                                    index: tc.index,
                                                    id: tc.id.clone(),
                                                    name: tc.function.as_ref().and_then(|f| f.name.clone()),
                                                    arguments_delta: tc
                                                        .function
                                                        .as_ref()
                                                        .and_then(|f| f.arguments.clone())
                                                        .unwrap_or_default(),
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

                                if let Some(finish) = &choice.finish_reason {
                                    let stop_reason = Self::map_finish_reason(Some(finish));
                                    let chunk = StreamChunk {
                                        chunk_index: chunk_idx,
                                        delta: StreamChunkDelta::TextDelta {
                                            text: String::new(),
                                        },
                                        stop_reason: Some(stop_reason),
                                        partial_metrics: None,
                                    };
                                    let _ = tx.send(Ok(chunk)).await;
                                    return;
                                }
                            }
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
