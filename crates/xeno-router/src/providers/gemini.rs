//! Google Gemini REST & SSE streaming provider adapter (`generateContent` & `streamGenerateContent`).

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

/// Google Gemini provider adapter.
#[derive(Debug, Clone)]
pub struct GeminiProvider {
    name: String,
    base_url: String,
    api_key: String,
    client: Client,
    timeout: Duration,
}

impl GeminiProvider {
    /// Constructs a new [`GeminiProvider`] with an API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            name: "gemini".into(),
            base_url: "https://generativelanguage.googleapis.com/v1beta/models".into(),
            api_key: api_key.into(),
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

    /// Sets custom request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn map_finish_reason(reason: Option<&str>) -> StopReason {
        match reason {
            Some("STOP") => StopReason::EndTurn,
            Some("MAX_TOKENS") => StopReason::MaxTokens,
            Some("SAFETY") => StopReason::ContentFilter,
            Some("RECITATION") => StopReason::ContentFilter,
            _ => StopReason::EndTurn,
        }
    }

    fn build_wire_payload(&self, req: &InferenceRequest) -> GeminiWireRequest {
        let mut contents = Vec::new();

        for msg in &req.messages {
            let role_str = match msg.role {
                Role::User | Role::Tool => "user",
                Role::Assistant => "model",
                Role::System => continue,
            };

            let text = msg.text_content();
            contents.push(GeminiContent {
                role: role_str.into(),
                parts: vec![GeminiPart { text }],
            });
        }

        let system_instruction = req.system_prompt.as_ref().map(|sys| GeminiContent {
            role: "user".into(),
            parts: vec![GeminiPart { text: sys.clone() }],
        });

        let generation_config = Some(GeminiGenerationConfig {
            temperature: Some(req.temperature),
            max_output_tokens: req.max_tokens,
            top_p: req.top_p,
            stop_sequences: if req.stop_sequences.is_empty() {
                None
            } else {
                Some(req.stop_sequences.clone())
            },
        });

        GeminiWireRequest {
            contents,
            system_instruction,
            generation_config,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiWireRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiWireResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiCandidate {
    content: Option<GeminiContent>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiUsageMetadata {
    prompt_token_count: Option<u32>,
    candidates_token_count: Option<u32>,
}

#[async_trait]
impl InferenceProvider for GeminiProvider {
    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Google
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn model_info(&self, model: &str) -> Result<ModelInfo, XenoError> {
        let (in_cost, out_cost) = match model {
            "gemini-2.0-flash" => (0.10, 0.40),
            "gemini-2.0-pro" => (1.25, 5.00),
            _ => (0.10, 0.40),
        };

        Ok(ModelInfo {
            id: model.to_string(),
            provider: ProviderKind::Google,
            context_window: 1_000_000,
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
                reason: "Google Gemini API key is not set".into(),
            });
        }
        Ok(HealthStatus::Healthy)
    }

    async fn complete(&self, req: &InferenceRequest) -> Result<InferenceResponse, XenoError> {
        let endpoint = format!("{}/{}:generateContent", self.base_url, req.model);
        let wire_req = self.build_wire_payload(req);

        let resp = self
            .client
            .post(&endpoint)
            .header("x-goog-api-key", &self.api_key)
            .json(&wire_req)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| XenoError::NetworkError {
                message: format!("Gemini request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: status.as_u16(),
                message: format!("Gemini upstream error: {err_body}"),
            });
        }

        let wire_resp: GeminiWireResponse = resp.json().await.map_err(|e| {
            XenoError::Inference(xeno_core::errors::InferenceError::MalformedResponse(
                format!("JSON error: {e}"),
            ))
        })?;

        let candidate = wire_resp
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .ok_or_else(|| {
                XenoError::Inference(xeno_core::errors::InferenceError::MalformedResponse(
                    "Empty candidates in Gemini response".into(),
                ))
            })?;

        let text = candidate
            .content
            .as_ref()
            .and_then(|c| c.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_default();

        let stop_reason = Self::map_finish_reason(candidate.finish_reason.as_deref());

        let prompt_tokens = wire_resp
            .usage_metadata
            .as_ref()
            .and_then(|u| u.prompt_token_count)
            .unwrap_or(0);
        let completion_tokens = wire_resp
            .usage_metadata
            .as_ref()
            .and_then(|u| u.candidates_token_count)
            .unwrap_or(0);

        let metrics = TokenMetrics::new(prompt_tokens, completion_tokens, 0, 0, 0, 0.0, 0.0);

        Ok(InferenceResponse {
            id: format!("gemini-{}", uuid::Uuid::new_v4()),
            model: req.model.clone(),
            content: vec![ContentBlock::text(text)],
            stop_reason,
            metrics,
        })
    }

    async fn stream(&self, req: &InferenceRequest) -> Result<BoxStream<StreamChunk>, XenoError> {
        let endpoint = format!(
            "{}/{}:streamGenerateContent?alt=sse",
            self.base_url, req.model
        );
        let wire_req = self.build_wire_payload(req);

        let resp = self
            .client
            .post(&endpoint)
            .header("x-goog-api-key", &self.api_key)
            .json(&wire_req)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| XenoError::NetworkError {
                message: format!("Gemini streaming request failed: {e}"),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: status.as_u16(),
                message: format!("Gemini streaming error: {err_body}"),
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

                        if let Ok(parsed) = serde_json::from_str::<GeminiWireResponse>(&event.data) {
                            if let Some(candidate) = parsed.candidates.as_ref().and_then(|c| c.first()) {
                                if let Some(parts) = candidate.content.as_ref().map(|c| &c.parts) {
                                    for part in parts {
                                        if !part.text.is_empty() {
                                            let chunk = StreamChunk {
                                                chunk_index: chunk_idx,
                                                delta: StreamChunkDelta::TextDelta {
                                                    text: part.text.clone(),
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

                                if let Some(finish) = &candidate.finish_reason {
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
