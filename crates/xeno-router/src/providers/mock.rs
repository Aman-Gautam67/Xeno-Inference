//! Mock inference provider for deterministic testing, stream simulation, and error injection.

use crate::provider::{BoxStream, HealthStatus, InferenceProvider, ModelInfo};
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use xeno_core::{
    contracts::{
        ContentBlock, InferenceRequest, InferenceResponse, StopReason, StreamChunk,
        StreamChunkDelta,
    },
    errors::XenoError,
    metrics::TokenMetrics,
    types::ProviderKind,
};

/// Configuration for controlling mock provider responses, streaming behavior, and errors.
#[derive(Debug, Clone)]
pub struct MockConfig {
    /// Default text response if queue is empty.
    pub default_text: String,
    /// Optional default reasoning / thinking response.
    pub default_thinking: Option<String>,
    /// Queue of canned text responses.
    pub text_responses: VecDeque<String>,
    /// Queue of canned thinking responses.
    pub thinking_responses: VecDeque<String>,
    /// Queue of canned tool calls: `(tool_name, arguments_json)`.
    pub tool_calls: VecDeque<(String, serde_json::Value)>,
    /// Simulated latency per stream chunk.
    pub chunk_delay: Duration,
    /// Number of characters per streamed chunk.
    pub chunk_size: usize,
    /// Injected error to trigger on the next request.
    pub injected_error: Option<String>,
    /// Health status to report.
    pub health_status: HealthStatus,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            default_text: "Mock response from XENO inference engine.".into(),
            default_thinking: None,
            text_responses: VecDeque::new(),
            thinking_responses: VecDeque::new(),
            tool_calls: VecDeque::new(),
            chunk_delay: Duration::from_millis(0),
            chunk_size: 16,
            injected_error: None,
            health_status: HealthStatus::Healthy,
        }
    }
}

/// Deterministic mock inference provider.
#[derive(Debug, Clone)]
pub struct MockProvider {
    name: String,
    config: Arc<Mutex<MockConfig>>,
    recorded_requests: Arc<Mutex<Vec<InferenceRequest>>>,
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new("mock-provider", MockConfig::default())
    }
}

impl MockProvider {
    /// Constructs a new [`MockProvider`] with custom name and configuration.
    pub fn new(name: impl Into<String>, config: MockConfig) -> Self {
        Self {
            name: name.into(),
            config: Arc::new(Mutex::new(config)),
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Sets canned text response.
    pub fn with_text(self, text: impl Into<String>) -> Self {
        {
            let mut conf = self.config.lock().unwrap();
            conf.default_text = text.into();
        }
        self
    }

    /// Sets canned thinking/reasoning response.
    pub fn with_thinking(self, thinking: impl Into<String>) -> Self {
        {
            let mut conf = self.config.lock().unwrap();
            conf.default_thinking = Some(thinking.into());
        }
        self
    }

    /// Adds a canned text response to the queue.
    pub fn queue_text(&self, text: impl Into<String>) {
        let mut conf = self.config.lock().unwrap();
        conf.text_responses.push_back(text.into());
    }

    /// Adds a canned tool call to the queue.
    pub fn queue_tool_call(&self, tool_name: impl Into<String>, arguments: serde_json::Value) {
        let mut conf = self.config.lock().unwrap();
        conf.tool_calls.push_back((tool_name.into(), arguments));
    }

    /// Injects an error to be returned by the next invocation.
    pub fn inject_error(&self, error_desc: impl Into<String>) {
        let mut conf = self.config.lock().unwrap();
        conf.injected_error = Some(error_desc.into());
    }

    /// Sets simulated chunk streaming delay.
    pub fn set_chunk_delay(&self, delay: Duration) {
        let mut conf = self.config.lock().unwrap();
        conf.chunk_delay = delay;
    }

    /// Retrieves all recorded requests sent to this provider.
    pub fn recorded_requests(&self) -> Vec<InferenceRequest> {
        self.recorded_requests.lock().unwrap().clone()
    }

    /// Clears recorded request history.
    pub fn clear_recorded_requests(&self) {
        self.recorded_requests.lock().unwrap().clear();
    }
}

#[async_trait]
impl InferenceProvider for MockProvider {
    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Mock
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn model_info(&self, model: &str) -> Result<ModelInfo, XenoError> {
        Ok(ModelInfo {
            id: model.to_string(),
            provider: ProviderKind::Mock,
            context_window: 128_000,
            max_output_tokens: 8_192,
            supports_streaming: true,
            supports_tools: true,
            supports_reasoning: true,
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
        })
    }

    async fn health_check(&self) -> Result<HealthStatus, XenoError> {
        let conf = self.config.lock().unwrap();
        Ok(conf.health_status.clone())
    }

    async fn complete(&self, req: &InferenceRequest) -> Result<InferenceResponse, XenoError> {
        // Record request
        self.recorded_requests.lock().unwrap().push(req.clone());

        // Check for injected error
        let mut conf = self.config.lock().unwrap();
        if let Some(err) = conf.injected_error.take() {
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: 500,
                message: err,
            });
        }

        let mut content = Vec::new();
        let mut reasoning_tokens = 0;

        // Add thinking content if configured
        let thinking_text = conf
            .thinking_responses
            .pop_front()
            .or_else(|| conf.default_thinking.clone());

        if let Some(thinking) = thinking_text {
            if !thinking.is_empty() {
                reasoning_tokens = (thinking.len() / 4).max(1) as u32;
                content.push(ContentBlock::thinking(thinking));
            }
        }

        // Add tool call if queued
        let mut stop_reason = StopReason::EndTurn;
        if let Some((tool_name, args)) = conf.tool_calls.pop_front() {
            let tool_id = format!("call_{}", uuid::Uuid::new_v4().simple());
            content.push(ContentBlock::tool_use(tool_id, tool_name, args));
            stop_reason = StopReason::ToolUse;
        } else {
            // Otherwise add text response
            let text = conf
                .text_responses
                .pop_front()
                .unwrap_or_else(|| conf.default_text.clone());
            content.push(ContentBlock::text(text));
        }

        let prompt_tokens = req
            .messages
            .iter()
            .map(|m| (m.text_content().len() / 4).max(1) as u32)
            .sum::<u32>()
            .max(10);

        let completion_text_len = content
            .iter()
            .filter_map(|c| c.as_text())
            .map(|s| s.len())
            .sum::<usize>();
        let completion_tokens = (completion_text_len / 4).max(1) as u32;

        let metrics = TokenMetrics::new(
            prompt_tokens,
            completion_tokens,
            reasoning_tokens,
            5,
            20,
            120.0,
            0.0,
        );

        Ok(InferenceResponse {
            id: format!("mock-resp-{}", uuid::Uuid::new_v4()),
            model: req.model.clone(),
            content,
            stop_reason,
            metrics,
        })
    }

    async fn stream(&self, req: &InferenceRequest) -> Result<BoxStream<StreamChunk>, XenoError> {
        // Record request
        self.recorded_requests.lock().unwrap().push(req.clone());

        let (thinking_text, text_resp, tool_call, chunk_delay, chunk_size, injected_err) = {
            let mut conf = self.config.lock().unwrap();
            let injected = conf.injected_error.take();
            let thinking = conf
                .thinking_responses
                .pop_front()
                .or_else(|| conf.default_thinking.clone());
            let tool = conf.tool_calls.pop_front();
            let text = if tool.is_none() {
                Some(
                    conf.text_responses
                        .pop_front()
                        .unwrap_or_else(|| conf.default_text.clone()),
                )
            } else {
                None
            };
            (
                thinking,
                text,
                tool,
                conf.chunk_delay,
                conf.chunk_size,
                injected,
            )
        };

        if let Some(err) = injected_err {
            return Err(XenoError::UpstreamError {
                provider: self.name.clone(),
                status_code: 500,
                message: err,
            });
        }

        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn(async move {
            let mut chunk_idx = 0;

            // Stream thinking deltas first if present
            if let Some(thinking) = thinking_text {
                let chars: Vec<char> = thinking.chars().collect();
                for chunk in chars.chunks(chunk_size) {
                    if chunk_delay > Duration::ZERO {
                        sleep(chunk_delay).await;
                    }
                    let chunk_str: String = chunk.iter().collect();
                    let msg = StreamChunk {
                        chunk_index: chunk_idx,
                        delta: StreamChunkDelta::ThinkingDelta {
                            reasoning: chunk_str,
                        },
                        stop_reason: None,
                        partial_metrics: None,
                    };
                    chunk_idx += 1;
                    if tx.send(Ok(msg)).await.is_err() {
                        return;
                    }
                }
            }

            // Stream tool call delta if present
            if let Some((tool_name, args)) = tool_call {
                let call_id = format!("call_{}", uuid::Uuid::new_v4().simple());
                let args_str = serde_json::to_string(&args).unwrap_or_default();
                let chars: Vec<char> = args_str.chars().collect();

                let mut is_first = true;
                for chunk in chars.chunks(chunk_size) {
                    if chunk_delay > Duration::ZERO {
                        sleep(chunk_delay).await;
                    }
                    let chunk_str: String = chunk.iter().collect();
                    let msg = StreamChunk {
                        chunk_index: chunk_idx,
                        delta: StreamChunkDelta::ToolCallDelta {
                            index: 0,
                            id: if is_first {
                                Some(call_id.clone())
                            } else {
                                None
                            },
                            name: if is_first {
                                Some(tool_name.clone())
                            } else {
                                None
                            },
                            arguments_delta: chunk_str,
                        },
                        stop_reason: None,
                        partial_metrics: None,
                    };
                    is_first = false;
                    chunk_idx += 1;
                    if tx.send(Ok(msg)).await.is_err() {
                        return;
                    }
                }

                // Final tool chunk
                let final_chunk = StreamChunk {
                    chunk_index: chunk_idx,
                    delta: StreamChunkDelta::ToolCallDelta {
                        index: 0,
                        id: None,
                        name: None,
                        arguments_delta: String::new(),
                    },
                    stop_reason: Some(StopReason::ToolUse),
                    partial_metrics: None,
                };
                let _ = tx.send(Ok(final_chunk)).await;
                return;
            }

            // Stream text deltas
            if let Some(text) = text_resp {
                let chars: Vec<char> = text.chars().collect();
                for chunk in chars.chunks(chunk_size) {
                    if chunk_delay > Duration::ZERO {
                        sleep(chunk_delay).await;
                    }
                    let chunk_str: String = chunk.iter().collect();
                    let msg = StreamChunk {
                        chunk_index: chunk_idx,
                        delta: StreamChunkDelta::TextDelta { text: chunk_str },
                        stop_reason: None,
                        partial_metrics: None,
                    };
                    chunk_idx += 1;
                    if tx.send(Ok(msg)).await.is_err() {
                        return;
                    }
                }
            }

            // Terminating chunk
            let final_chunk = StreamChunk {
                chunk_index: chunk_idx,
                delta: StreamChunkDelta::TextDelta {
                    text: String::new(),
                },
                stop_reason: Some(StopReason::EndTurn),
                partial_metrics: None,
            };
            let _ = tx.send(Ok(final_chunk)).await;
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use xeno_core::contracts::ChatMessage;

    #[tokio::test]
    async fn test_mock_provider_complete() {
        let provider = MockProvider::default().with_text("Hello from mock!");
        let req = InferenceRequest::new("mock-model", vec![ChatMessage::user_text("hi")]);

        let resp = provider.complete(&req).await.unwrap();
        assert_eq!(resp.text_content(), "Hello from mock!");
        assert_eq!(resp.stop_reason, StopReason::EndTurn);
        assert_eq!(provider.recorded_requests().len(), 1);
    }

    #[tokio::test]
    async fn test_mock_provider_thinking_stream() {
        let provider = MockProvider::default()
            .with_thinking("Thinking deeply...")
            .with_text("Here is the solution.");

        let req = InferenceRequest::new("mock-model", vec![ChatMessage::user_text("test")]);
        let mut stream = provider.stream(&req).await.unwrap();

        let mut collected_thinking = String::new();
        let mut collected_text = String::new();

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.unwrap();
            match chunk.delta {
                StreamChunkDelta::ThinkingDelta { reasoning } => {
                    collected_thinking.push_str(&reasoning);
                }
                StreamChunkDelta::TextDelta { text } => {
                    collected_text.push_str(&text);
                }
                _ => {}
            }
        }

        assert_eq!(collected_thinking, "Thinking deeply...");
        assert_eq!(collected_text, "Here is the solution.");
    }

    #[tokio::test]
    async fn test_mock_provider_error_injection() {
        let provider = MockProvider::default();
        provider.inject_error("Simulated rate limit error");

        let req = InferenceRequest::new("mock-model", vec![ChatMessage::user_text("test")]);
        let err = provider.complete(&req).await.unwrap_err();
        match err {
            XenoError::UpstreamError { message, .. } => {
                assert!(message.contains("Simulated rate limit"));
            }
            _ => panic!("Expected UpstreamError"),
        }
    }
}
