//! High-throughput ring buffer and session telemetry collector.

use crate::metrics::{HardwareMetrics, SessionSummaryMetrics, StepTelemetry};
use crate::privacy_guard::TelemetryPrivacyGuard;
use std::collections::VecDeque;
use std::sync::Mutex;
use tokio::sync::broadcast;

/// Telemetry metrics collector with ring-buffer storage and event broadcasting.
#[derive(Debug)]
pub struct TelemetryCollector {
    ring_buffer: Mutex<VecDeque<StepTelemetry>>,
    max_capacity: usize,
    privacy_guard: TelemetryPrivacyGuard,
    event_tx: broadcast::Sender<StepTelemetry>,
    hardware_metrics: Mutex<HardwareMetrics>,
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl TelemetryCollector {
    pub fn new(capacity: usize) -> Self {
        let (event_tx, _) = broadcast::channel(512);
        Self {
            ring_buffer: Mutex::new(VecDeque::with_capacity(capacity)),
            max_capacity: capacity,
            privacy_guard: TelemetryPrivacyGuard::new(),
            event_tx,
            hardware_metrics: Mutex::new(HardwareMetrics::default()),
        }
    }

    /// Records a new step telemetry entry, sanitizes it, and broadcasts it to UI subscribers.
    pub fn record_step(&self, mut step: StepTelemetry) {
        self.privacy_guard.sanitize_step_telemetry(&mut step);

        let _ = self.event_tx.send(step.clone());

        let mut buffer = self.ring_buffer.lock().unwrap();
        if buffer.len() >= self.max_capacity {
            buffer.pop_front();
        }
        buffer.push_back(step);
    }

    /// Computes summary metrics across all recorded steps in the buffer.
    pub fn compute_summary(&self) -> SessionSummaryMetrics {
        let buffer = self.ring_buffer.lock().unwrap();
        let mut total_prompt = 0u64;
        let mut total_completion = 0u64;
        let mut total_duration = 0u64;
        let mut total_cost = 0.0f64;

        for step in buffer.iter() {
            total_prompt += step.prompt_tokens as u64;
            total_completion += step.completion_tokens as u64;
            total_duration += step.duration_ms;
            total_cost += step.estimated_cost_usd;
        }

        let total_tokens = total_completion as f64;
        let total_secs = (total_duration as f64) / 1000.0;
        let avg_velocity = if total_secs > 0.0 {
            total_tokens / total_secs
        } else {
            0.0
        };

        SessionSummaryMetrics {
            total_prompt_tokens: total_prompt,
            total_completion_tokens: total_completion,
            total_duration_ms: total_duration,
            average_velocity_tokens_per_sec: avg_velocity,
            total_cost_usd: total_cost,
            step_count: buffer.len(),
        }
    }

    /// Subscribes to real-time step telemetry events.
    pub fn subscribe(&self) -> broadcast::Receiver<StepTelemetry> {
        self.event_tx.subscribe()
    }

    /// Retrieves current hardware metrics.
    pub fn get_hardware_metrics(&self) -> HardwareMetrics {
        self.hardware_metrics.lock().unwrap().clone()
    }

    /// Updates current hardware metrics snapshot.
    pub fn update_hardware_metrics(&self, metrics: HardwareMetrics) {
        let mut lock = self.hardware_metrics.lock().unwrap();
        *lock = metrics;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_recording_and_summary() {
        let collector = TelemetryCollector::new(50);
        let s1 = StepTelemetry::new("sess-1", "commander", "thinking", 1000, 500, 100, 0.001, "mock");
        let s2 = StepTelemetry::new("sess-1", "coder", "tool_call", 1000, 800, 200, 0.002, "mock");

        collector.record_step(s1);
        collector.record_step(s2);

        let summary = collector.compute_summary();
        assert_eq!(summary.step_count, 2);
        assert_eq!(summary.total_prompt_tokens, 1300);
        assert_eq!(summary.total_completion_tokens, 300);
        assert_eq!(summary.total_duration_ms, 2000);
        assert_eq!(summary.average_velocity_tokens_per_sec, 150.0);
        assert!((summary.total_cost_usd - 0.003).abs() < 1e-6);
    }
}
