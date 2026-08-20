//! DeepSeek API provider adapter (DeepSeek V3 & DeepSeek R1 reasoning models).

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

/// DeepSeek provider adapter supporting DeepSeek V3 and DeepSeek R1 thinking/reasoning.
#[derive(Debug, Clone)]
pub struct DeepSeekProvider {
    name: String,
    base_url: String,
    api_key: String,
    client: Client,
    timeout: Duration,
}

impl DeepSeekProvider {
    /// Constructs a new [`DeepSeekProvider`] with an API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            name: "deepseek".into(),
            base_url: "https://api.deepseek.com/v1".into(),
            api_key: api_key.into(),
            client: Client::builder().build().unwrap_or_default(),
            timeout: Duration::from_secs(180),
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
            Some("content_filter") => StopReason::ContentFilter,
            _ => StopReason::EndTurn,
        }
    }

    fn build_wire_messages(&self, req: &InferenceRequest) -> Vec<DeepSeekWireMessage> {
        let mut wire_messages = Vec::new();

        if let Some(sys) = &req.system_prompt {
            wire_messages.push(DeepSeekWireMessage {
                role: "system".into(),
                content: sys.clone(),
            });
        }

        for msg in &req.messages {
            let role_str = match msg.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "user",
            };

            wire_messages.push(DeepSeekWireMessage {
                role: role_str.into(),
                content: msg.text_content(),
            });
        }

        wire_messages
    }

    /// Helper to parse inline `<think>...</think>` tags if present in plain text responses.
    pub fn parse_inline_thinking(raw_text: &str) -> (Option<String>, String) {
        if let Some(start_idx) = raw_text.find("<think>") {
            if let Some(end_idx) = raw_text.find("</think>") {
                let thinking = raw_text[start_idx + 7..end_idx].trim().to_string();
                let mut remainder = String::new();
                remainder.push_str(&raw_text[..start_idx]);
                remainder.push_str(&raw_text[end_idx + 8..]);
                return (Some(thinking), remainder.trim().to_string());
            }
        }
        (None, raw_text.to_string())
    }
}

#[derive(Serialize)]
struct DeepSeekWireRequest {
    model: String,
    messages: Vec<DeepSeekWireMessage>,
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
struct DeepSeekWireMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct DeepSeekWireResponse {
    id: String,
    model: String,
    choices: Vec<DeepSeekWireChoice>,
    usage: Option<DeepSeekWireUsage>,
}

#[derive(Deserialize)]
struct DeepSeekWireChoice {
    message: DeepSeekWireMessageChoice,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct DeepSeekWireMessageChoice {
    content: Option<String>,
    reasoning_content: Option<String>,
}

#[derive(Deserialize)]
struct DeepSeekWireUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    reasoning_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct DeepSeekStreamChunk {
    choices: Vec<DeepSeekStreamChoice>,
}

#[derive(Deserialize)]
struct DeepSeekStreamChoice {
    delta: Option<DeepSeekStreamDelta>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct DeepSeekStreamDelta {
    content: Option<String>,
    reasoning_content: Option<String>,
}

#[async_trait]
impl InferenceProvider for DeepSeekProvider {
    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Deepseek
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn model_info(&self, model: &str) -> Result<ModelInfo, XenoError> {
        let (in_cost, out_cost, reasoning) = match model {
            "deepseek-reasoner" | "deepseek-r1" => (0.55, 2.19, true),
            "deepseek-chat" | "deepseek-v3" => (0.14, 0.28, false),
            _ => (0.14, 0.28, false),
        };

        Ok(ModelInfo {
            id: model.to_string(),
            provider: ProviderKind::Deepseek,
            context_window: 64_000,
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
                reason: "DeepSeek API key is not set".into(),
            });
        }
        Ok(HealthStatus::Healthy)
    }

    async fn complete(&self, req: &InferenceRequest) -> Result<InferenceResponse, XenoError> {
        let endpoint = format!("{}/chat/completions", self.base_url);
        let wire_req = DeepSeekWireRequest {
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
                message: format!("DeepSeek request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: status.as_u16(),
                message: format!("DeepSeek upstream error: {err_body}"),
            });
        }

        let wire_resp: DeepSeekWireResponse = resp.json().await.map_err(|e| {
            XenoError::Inference(xeno_core::errors::InferenceError::MalformedResponse(
                format!("JSON error: {e}"),
            ))
        })?;

        let choice = wire_resp
            .choices
            .first()
            .ok_or_else(|| {
                XenoError::Inference(xeno_core::errors::InferenceError::MalformedResponse(
                    "Empty choices from DeepSeek".into(),
                ))
            })?;

        let mut content = Vec::new();
        let mut reasoning_tokens = 0;

        // Check if reasoning_content field is present
        if let Some(reasoning) = &choice.message.reasoning_content {
            if !reasoning.is_empty() {
                reasoning_tokens = (reasoning.len() / 4) as u32;
                content.push(ContentBlock::thinking(reasoning.clone()));
            }
        }

        // Process message content
        if let Some(raw_text) = &choice.message.content {
            if !raw_text.is_empty() {
                // If reasoning wasn't in explicit field, check for <think> tags
                if content.is_empty() {
                    let (parsed_think, clean_text) = Self::parse_inline_thinking(raw_text);
                    if let Some(think) = parsed_think {
                        reasoning_tokens = (think.len() / 4) as u32;
                        content.push(ContentBlock::thinking(think));
                    }
                    if !clean_text.is_empty() {
                        content.push(ContentBlock::text(clean_text));
                    }
                } else {
                    content.push(ContentBlock::text(raw_text.clone()));
                }
            }
        }

        let stop_reason = Self::map_finish_reason(choice.finish_reason.as_deref());
        let prompt_tokens = wire_resp.usage.as_ref().and_then(|u| u.prompt_tokens).unwrap_or(0);
        let completion_tokens = wire_resp.usage.as_ref().and_then(|u| u.completion_tokens).unwrap_or(0);
        let reasoning_tokens_usage = wire_resp
            .usage
            .as_ref()
            .and_then(|u| u.reasoning_tokens)
            .unwrap_or(reasoning_tokens);

        let metrics = TokenMetrics::new(
            prompt_tokens,
            completion_tokens,
            reasoning_tokens_usage,
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
        let wire_req = DeepSeekWireRequest {
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
                message: format!("DeepSeek streaming request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: status.as_u16(),
                message: format!("DeepSeek streaming error: {err_body}"),
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

                        if let Ok(parsed) = serde_json::from_str::<DeepSeekStreamChunk>(&event.data) {
                            if let Some(choice) = parsed.choices.first() {
                                if let Some(delta) = &choice.delta {
                                    // Handle reasoning_content delta
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

                                    // Handle content delta
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inline_thinking() {
        let raw = "<think>\nLet's analyze the problem step by step.\n</think>\nThe answer is 42.";
        let (thinking, text) = DeepSeekProvider::parse_inline_thinking(raw);
        assert_eq!(thinking.as_deref(), Some("Let's analyze the problem step by step."));
        assert_eq!(text, "The answer is 42.");
    }
}
