//! Strongly-typed telemetry metrics, step telemetry models, and hardware counters.

use serde::{Deserialize, Serialize};

/// Hardware metrics captured from runtime environment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareMetrics {
    pub vram_allocated_bytes: u64,
    pub vram_total_bytes: u64,
    pub gpu_core_utilization_pct: f32,
    pub host_ram_used_bytes: u64,
}

impl Default for HardwareMetrics {
    fn default() -> Self {
        Self {
            vram_allocated_bytes: 8 * 1024 * 1024 * 1024, // 8 GB default
            vram_total_bytes: 24 * 1024 * 1024 * 1024,    // 24 GB default
            gpu_core_utilization_pct: 0.0,
            host_ram_used_bytes: 4 * 1024 * 1024 * 1024,
        }
    }
}

/// Discrete telemetry record for an agentic or tool execution step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepTelemetry {
    pub step_id: String,
    pub session_id: String,
    pub timestamp: u64,
    pub agent_role: String,
    pub phase: String,
    pub duration_ms: u64,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub tokens_per_second: f64,
    pub estimated_cost_usd: f64,
    pub model_used: String,
    pub backend_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub success: bool,
}

impl StepTelemetry {
    pub fn new(
        session_id: impl Into<String>,
        agent_role: impl Into<String>,
        phase: impl Into<String>,
        duration_ms: u64,
        prompt_tokens: u32,
        completion_tokens: u32,
        cost_usd: f64,
        model_used: impl Into<String>,
    ) -> Self {
        let ts = chrono::Utc::now().timestamp_millis() as u64;
        let duration_secs = (duration_ms as f64) / 1000.0;
        let velocity = if duration_secs > 0.0 {
            (completion_tokens as f64) / duration_secs
        } else {
            0.0
        };

        Self {
            step_id: format!("step-{}", uuid::Uuid::new_v4()),
            session_id: session_id.into(),
            timestamp: ts,
            agent_role: agent_role.into(),
            phase: phase.into(),
            duration_ms,
            prompt_tokens,
            completion_tokens,
            tokens_per_second: velocity,
            estimated_cost_usd: cost_usd,
            model_used: model_used.into(),
            backend_type: "inference_bus".into(),
            tool_name: None,
            exit_code: None,
            success: true,
        }
    }
}

/// Cumulative aggregate session metrics summary.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummaryMetrics {
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_duration_ms: u64,
    pub average_velocity_tokens_per_sec: f64,
    pub total_cost_usd: f64,
    pub step_count: usize,
}
