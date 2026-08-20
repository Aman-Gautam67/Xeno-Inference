//! Asynchronous Streaming Token Bus with monotonic TTFT calculation and multi-subscriber broadcasting.

use crate::pricing::CostEstimator;
use crate::provider::BoxStream;
use crate::velocity::TokenVelocityCalculator;
use futures_util::StreamExt;
use std::time::Instant;
use tokio::sync::broadcast;
use tokio_stream::wrappers::ReceiverStream;
use xeno_core::{
    contracts::{StreamChunk, StreamChunkDelta},
    metrics::TokenMetrics,
};

/// Asynchronous streaming token bus providing real-time telemetry metrics and multi-receiver broadcasting.
#[derive(Debug, Clone)]
pub struct TokenBus {
    sender: broadcast::Sender<StreamChunk>,
}

impl Default for TokenBus {
    fn default() -> Self {
        Self::new(256)
    }
}

impl TokenBus {
    /// Constructs a new [`TokenBus`] with a given broadcast channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribes a new receiver to listen to live token chunks broadcasted across the bus.
    pub fn subscribe(&self) -> broadcast::Receiver<StreamChunk> {
        self.sender.subscribe()
    }

    /// Wraps an upstream provider stream, tracking TTFT, calculating rolling velocity,
    /// enriching chunks with live token metrics, and broadcasting to all subscribers.
    pub fn wrap_stream(
        &self,
        model: String,
        mut upstream: BoxStream<StreamChunk>,
        prompt_tokens: u32,
        cost_estimator: Option<CostEstimator>,
    ) -> BoxStream<StreamChunk> {
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let bus_sender = self.sender.clone();

        tokio::spawn(async move {
            let start_time = Instant::now();
            let mut ttft_recorded = false;
            let mut ttft_ms = 0;
            let mut completion_tokens = 0;
            let mut reasoning_tokens = 0;
            let mut velocity_calc = TokenVelocityCalculator::default();

            while let Some(chunk_res) = upstream.next().await {
                match chunk_res {
                    Ok(mut chunk) => {
                        // Check if this chunk has token content to record TTFT
                        let mut delta_toks = 0;
                        match &chunk.delta {
                            StreamChunkDelta::ThinkingDelta { reasoning } => {
                                if !reasoning.is_empty() {
                                    let tok_count = (reasoning.len() / 4).max(1);
                                    delta_toks += tok_count;
                                    reasoning_tokens += tok_count as u32;
                                    if !ttft_recorded {
                                        ttft_ms = start_time.elapsed().as_millis() as u64;
                                        ttft_recorded = true;
                                    }
                                }
                            }
                            StreamChunkDelta::TextDelta { text } => {
                                if !text.is_empty() {
                                    let tok_count = (text.len() / 4).max(1);
                                    delta_toks += tok_count;
                                    completion_tokens += tok_count as u32;
                                    if !ttft_recorded {
                                        ttft_ms = start_time.elapsed().as_millis() as u64;
                                        ttft_recorded = true;
                                    }
                                }
                            }
                            StreamChunkDelta::ToolCallDelta {
                                arguments_delta, ..
                            } => {
                                if !arguments_delta.is_empty() {
                                    let tok_count = (arguments_delta.len() / 4).max(1);
                                    delta_toks += tok_count;
                                    completion_tokens += tok_count as u32;
                                    if !ttft_recorded {
                                        ttft_ms = start_time.elapsed().as_millis() as u64;
                                        ttft_recorded = true;
                                    }
                                }
                            }
                        }

                        if delta_toks > 0 {
                            velocity_calc.record_tokens(delta_toks);
                        }

                        // Compute live metrics
                        let total_elapsed_ms = start_time.elapsed().as_millis() as u64;
                        let current_velocity = velocity_calc.current_velocity();
                        let estimated_cost = if let Some(ref estimator) = cost_estimator {
                            estimator.estimate_cost(
                                &model,
                                prompt_tokens,
                                completion_tokens,
                                reasoning_tokens,
                            )
                        } else {
                            0.0
                        };

                        let metrics = TokenMetrics::new(
                            prompt_tokens,
                            completion_tokens,
                            reasoning_tokens,
                            ttft_ms,
                            total_elapsed_ms,
                            current_velocity,
                            estimated_cost,
                        );

                        chunk.partial_metrics = Some(metrics);

                        // Broadcast to bus subscribers (ignore error if no active receivers)
                        let _ = bus_sender.send(chunk.clone());

                        // Send downstream to caller
                        if tx.send(Ok(chunk)).await.is_err() {
                            return;
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(Err(err)).await;
                        return;
                    }
                }
            }
        });

        Box::pin(ReceiverStream::new(rx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xeno_core::contracts::StopReason;

    #[tokio::test]
    async fn test_token_bus_wrapping_and_broadcast() {
        let bus = TokenBus::new(32);
        let mut subscriber = bus.subscribe();

        // Create a simulated upstream channel
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        let upstream: BoxStream<StreamChunk> = Box::pin(ReceiverStream::new(rx));

        tokio::spawn(async move {
            tx.send(Ok(StreamChunk::thinking(0, "Deep reasoning step")))
                .await
                .unwrap();
            tx.send(Ok(StreamChunk::text(1, "Final answer text")))
                .await
                .unwrap();
            tx.send(Ok(StreamChunk {
                chunk_index: 2,
                delta: StreamChunkDelta::TextDelta {
                    text: String::new(),
                },
                stop_reason: Some(StopReason::EndTurn),
                partial_metrics: None,
            }))
            .await
            .unwrap();
        });

        let mut wrapped = bus.wrap_stream("claude-3-7-sonnet-20250219".into(), upstream, 100, None);

        // Receive from wrapped stream
        let chunk1 = wrapped.next().await.unwrap().unwrap();
        assert!(matches!(chunk1.delta, StreamChunkDelta::ThinkingDelta { .. }));
        assert!(chunk1.partial_metrics.is_some());

        // Also check subscriber received the broadcast
        let sub_chunk1 = subscriber.recv().await.unwrap();
        assert_eq!(sub_chunk1.chunk_index, chunk1.chunk_index);

        let chunk2 = wrapped.next().await.unwrap().unwrap();
        assert!(matches!(chunk2.delta, StreamChunkDelta::TextDelta { .. }));

        let sub_chunk2 = subscriber.recv().await.unwrap();
        assert_eq!(sub_chunk2.chunk_index, chunk2.chunk_index);
    }
}
