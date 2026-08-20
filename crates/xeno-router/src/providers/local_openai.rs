//! Local OpenAI-compatible provider adapter (llama.cpp, vLLM, Ollama, LM Studio).

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

/// Provider adapter connecting to local inference runtimes implementing the OpenAI chat completions API.
#[derive(Debug, Clone)]
pub struct LocalOpenAIProvider {
    name: String,
    base_url: String,
    api_key: Option<String>,
    client: Client,
    timeout: Duration,
}

impl Default for LocalOpenAIProvider {
    fn default() -> Self {
        Self::new("local-openai", "http://localhost:8080/v1", None)
    }
}

impl LocalOpenAIProvider {
    /// Constructs a new [`LocalOpenAIProvider`].
    pub fn new(
        name: impl Into<String>,
        base_url: impl Into<String>,
        api_key: Option<String>,
    ) -> Self {
        let mut url = base_url.into();
        if url.ends_with('/') {
            url.pop();
        }
        Self {
            name: name.into(),
            base_url: url,
            api_key,
            client: Client::builder().build().unwrap_or_default(),
            timeout: Duration::from_secs(60),
        }
    }

    /// Sets custom request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn build_wire_messages(&self, req: &InferenceRequest) -> Vec<WireMessage> {
        let mut wire_messages = Vec::new();

        // Include system prompt if explicitly provided
        if let Some(sys) = &req.system_prompt {
            wire_messages.push(WireMessage {
                role: "system".into(),
                content: WireContent::Text(sys.clone()),
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

            let text = msg.text_content();
            wire_messages.push(WireMessage {
                role: role_str.into(),
                content: WireContent::Text(text),
                name: msg.name.clone(),
            });
        }

        wire_messages
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
}

#[derive(Serialize)]
struct WireRequest {
    model: String,
    messages: Vec<WireMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop: Vec<String>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct WireMessage {
    role: String,
    content: WireContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum WireContent {
    Text(String),
}

#[derive(Deserialize)]
struct WireResponse {
    id: Option<String>,
    model: Option<String>,
    choices: Vec<WireChoice>,
    usage: Option<WireUsage>,
}

#[derive(Deserialize)]
struct WireChoice {
    message: Option<WireMessageChoice>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct WireMessageChoice {
    content: Option<String>,
}

#[derive(Deserialize)]
struct WireUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct WireStreamChunk {
    choices: Vec<WireStreamChoice>,
}

#[derive(Deserialize)]
struct WireStreamChoice {
    delta: Option<WireStreamDelta>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct WireStreamDelta {
    content: Option<String>,
    reasoning_content: Option<String>,
}

#[async_trait]
impl InferenceProvider for LocalOpenAIProvider {
    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Local
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn model_info(&self, model: &str) -> Result<ModelInfo, XenoError> {
        Ok(ModelInfo {
            id: model.to_string(),
            provider: ProviderKind::Local,
            context_window: 64_000,
            max_output_tokens: 4_096,
            supports_streaming: true,
            supports_tools: true,
            supports_reasoning: false,
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
        })
    }

    async fn health_check(&self) -> Result<HealthStatus, XenoError> {
        let endpoint = format!("{}/models", self.base_url);
        let mut req_builder = self.client.get(&endpoint).timeout(Duration::from_secs(5));
        if let Some(key) = &self.api_key {
            req_builder = req_builder.bearer_auth(key);
        }

        match req_builder.send().await {
            Ok(resp) if resp.status().is_success() => Ok(HealthStatus::Healthy),
            Ok(resp) => Ok(HealthStatus::Degraded {
                reason: format!("Server returned status {}", resp.status()),
            }),
            Err(e) => Ok(HealthStatus::Unhealthy {
                reason: format!("Cannot connect to local endpoint {}: {e}", self.base_url),
            }),
        }
    }

    async fn complete(&self, req: &InferenceRequest) -> Result<InferenceResponse, XenoError> {
        let endpoint = format!("{}/chat/completions", self.base_url);
        let wire_req = WireRequest {
            model: req.model.clone(),
            messages: self.build_wire_messages(req),
            temperature: Some(req.temperature),
            max_tokens: req.max_tokens,
            top_p: req.top_p,
            stop: req.stop_sequences.clone(),
            stream: false,
        };

        let mut req_builder = self
            .client
            .post(&endpoint)
            .json(&wire_req)
            .timeout(self.timeout);

        if let Some(key) = &self.api_key {
            req_builder = req_builder.bearer_auth(key);
        }

        let resp = req_builder.send().await.map_err(|e| XenoError::NetworkError {
            message: format!("Failed to connect to local OpenAI endpoint: {e}"),
        })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: status.as_u16(),
                message: format!("Local OpenAI error: {err_body}"),
            });
        }

        let wire_resp: WireResponse = resp.json().await.map_err(|e| XenoError::Inference(
            xeno_core::errors::InferenceError::MalformedResponse(format!("JSON error: {e}")),
        ))?;

        let choice = wire_resp
            .choices
            .first()
            .ok_or_else(|| XenoError::Inference(
                xeno_core::errors::InferenceError::MalformedResponse("Empty choices in response".into()),
            ))?;

        let text = choice
            .message
            .as_ref()
            .and_then(|m| m.content.clone())
            .unwrap_or_default();

        let stop_reason = Self::map_finish_reason(choice.finish_reason.as_deref());

        let prompt_tokens = wire_resp.usage.as_ref().and_then(|u| u.prompt_tokens).unwrap_or(0);
        let completion_tokens = wire_resp.usage.as_ref().and_then(|u| u.completion_tokens).unwrap_or(0);

        let metrics = TokenMetrics::new(prompt_tokens, completion_tokens, 0, 0, 0, 0.0, 0.0);

        Ok(InferenceResponse {
            id: wire_resp.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            model: wire_resp.model.unwrap_or_else(|| req.model.clone()),
            content: vec![ContentBlock::text(text)],
            stop_reason,
            metrics,
        })
    }

    async fn stream(&self, req: &InferenceRequest) -> Result<BoxStream<StreamChunk>, XenoError> {
        let endpoint = format!("{}/chat/completions", self.base_url);
        let wire_req = WireRequest {
            model: req.model.clone(),
            messages: self.build_wire_messages(req),
            temperature: Some(req.temperature),
            max_tokens: req.max_tokens,
            top_p: req.top_p,
            stop: req.stop_sequences.clone(),
            stream: true,
        };

        let mut req_builder = self
            .client
            .post(&endpoint)
            .json(&wire_req)
            .timeout(self.timeout);

        if let Some(key) = &self.api_key {
            req_builder = req_builder.bearer_auth(key);
        }

        let resp = req_builder.send().await.map_err(|e| XenoError::NetworkError {
            message: format!("Failed to connect to local OpenAI endpoint: {e}"),
        })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: status.as_u16(),
                message: format!("Local OpenAI streaming error: {err_body}"),
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

                        if let Ok(parsed) = serde_json::from_str::<WireStreamChunk>(&event.data) {
                            if let Some(choice) = parsed.choices.first() {
                                if let Some(delta) = &choice.delta {
                                    // Handle reasoning content if emitted
                                    if let Some(reasoning) = &delta.reasoning_content {
                                        if !reasoning.is_empty() {
                                            let chunk = StreamChunk {
                                                chunk_index: chunk_idx,
                                                delta: StreamChunkDelta::ThinkingDelta {
                                                    reasoning: reasoning.clone(),
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

                                    // Handle text content
                                    if let Some(text) = &delta.content {
                                        if !text.is_empty() {
                                            let stop_reason = choice
                                                .finish_reason
                                                .as_deref()
                                                .map(|r| Self::map_finish_reason(Some(r)));

                                            let chunk = StreamChunk {
                                                chunk_index: chunk_idx,
                                                delta: StreamChunkDelta::TextDelta {
                                                    text: text.clone(),
                                                },
                                                stop_reason,
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
