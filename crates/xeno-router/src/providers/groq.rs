//! Groq LPU ultra-low latency inference provider adapter.

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

/// Groq LPU provider adapter.
#[derive(Debug, Clone)]
pub struct GroqProvider {
    name: String,
    base_url: String,
    api_key: String,
    client: Client,
    timeout: Duration,
}

impl GroqProvider {
    /// Constructs a new [`GroqProvider`] with an API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            name: "groq".into(),
            base_url: "https://api.groq.com/openai/v1".into(),
            api_key: api_key.into(),
            client: Client::builder().build().unwrap_or_default(),
            timeout: Duration::from_secs(60),
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

    /// Sets custom request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn map_finish_reason(reason: Option<&str>) -> StopReason {
        match reason {
            Some("stop") => StopReason::EndTurn,
            Some("length") => StopReason::MaxTokens,
            Some("tool_calls") => StopReason::ToolUse,
            _ => StopReason::EndTurn,
        }
    }

    fn build_wire_messages(&self, req: &InferenceRequest) -> Vec<GroqWireMessage> {
        let mut wire_messages = Vec::new();

        if let Some(sys) = &req.system_prompt {
            wire_messages.push(GroqWireMessage {
                role: "system".into(),
                content: sys.clone(),
            });
        }

        for msg in &req.messages {
            let role_str = match msg.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "tool",
            };

            wire_messages.push(GroqWireMessage {
                role: role_str.into(),
                content: msg.text_content(),
            });
        }

        wire_messages
    }
}

#[derive(Serialize)]
struct GroqWireRequest {
    model: String,
    messages: Vec<GroqWireMessage>,
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
struct GroqWireMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct GroqWireResponse {
    id: String,
    model: String,
    choices: Vec<GroqWireChoice>,
    usage: Option<GroqWireUsage>,
}

#[derive(Deserialize)]
struct GroqWireChoice {
    message: GroqWireMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct GroqWireUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct GroqStreamChunk {
    choices: Vec<GroqStreamChoice>,
}

#[derive(Deserialize)]
struct GroqStreamChoice {
    delta: Option<GroqStreamDelta>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct GroqStreamDelta {
    content: Option<String>,
}

#[async_trait]
impl InferenceProvider for GroqProvider {
    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Groq
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn model_info(&self, model: &str) -> Result<ModelInfo, XenoError> {
        let (in_cost, out_cost) = match model {
            "llama-3.3-70b-versatile" => (0.59, 0.79),
            "llama-3.1-8b-instant" => (0.05, 0.08),
            _ => (0.59, 0.79),
        };

        Ok(ModelInfo {
            id: model.to_string(),
            provider: ProviderKind::Groq,
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_streaming: true,
            supports_tools: true,
            supports_reasoning: false,
            input_cost_per_million: in_cost,
            output_cost_per_million: out_cost,
        })
    }

    async fn health_check(&self) -> Result<HealthStatus, XenoError> {
        if self.api_key.trim().is_empty() {
            return Ok(HealthStatus::Unhealthy {
                reason: "Groq API key is not set".into(),
            });
        }
        Ok(HealthStatus::Healthy)
    }

    async fn complete(&self, req: &InferenceRequest) -> Result<InferenceResponse, XenoError> {
        let endpoint = format!("{}/chat/completions", self.base_url);
        let wire_req = GroqWireRequest {
            model: req.model.clone(),
            messages: self.build_wire_messages(req),
            temperature: Some(req.temperature),
            max_tokens: req.max_tokens,
            top_p: req.top_p,
            stop: req.stop_sequences.clone(),
            stream: false,
        };

        let resp = self
            .client
            .post(&endpoint)
            .bearer_auth(&self.api_key)
            .json(&wire_req)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| XenoError::NetworkError {
                message: format!("Groq request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: status.as_u16(),
                message: format!("Groq upstream error: {err_body}"),
            });
        }

        let wire_resp: GroqWireResponse = resp.json().await.map_err(|e| {
            XenoError::Inference(xeno_core::errors::InferenceError::MalformedResponse(
                format!("JSON error: {e}"),
            ))
        })?;

        let choice = wire_resp
            .choices
            .first()
            .ok_or_else(|| {
                XenoError::Inference(xeno_core::errors::InferenceError::MalformedResponse(
                    "Empty choices from Groq".into(),
                ))
            })?;

        let stop_reason = Self::map_finish_reason(choice.finish_reason.as_deref());
        let prompt_tokens = wire_resp.usage.as_ref().and_then(|u| u.prompt_tokens).unwrap_or(0);
        let completion_tokens = wire_resp.usage.as_ref().and_then(|u| u.completion_tokens).unwrap_or(0);

        let metrics = TokenMetrics::new(prompt_tokens, completion_tokens, 0, 20, 50, 450.0, 0.0);

        Ok(InferenceResponse {
            id: wire_resp.id,
            model: wire_resp.model,
            content: vec![ContentBlock::text(choice.message.content.clone())],
            stop_reason,
            metrics,
        })
    }

    async fn stream(&self, req: &InferenceRequest) -> Result<BoxStream<StreamChunk>, XenoError> {
        let endpoint = format!("{}/chat/completions", self.base_url);
        let wire_req = GroqWireRequest {
            model: req.model.clone(),
            messages: self.build_wire_messages(req),
            temperature: Some(req.temperature),
            max_tokens: req.max_tokens,
            top_p: req.top_p,
            stop: req.stop_sequences.clone(),
            stream: true,
        };

        let resp = self
            .client
            .post(&endpoint)
            .bearer_auth(&self.api_key)
            .json(&wire_req)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| XenoError::NetworkError {
                message: format!("Groq streaming request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: status.as_u16(),
                message: format!("Groq streaming error: {err_body}"),
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

                        if let Ok(parsed) = serde_json::from_str::<GroqStreamChunk>(&event.data) {
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
