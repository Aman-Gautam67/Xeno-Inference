//! Telemetry metrics, token counters, hardware resource stats, and financial cost models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Detailed token usage, latency timings, and throughput metrics for an inference operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetrics {
    /// Number of tokens in the input prompt.
    pub prompt_tokens: u32,
    /// Number of output completion tokens generated.
    pub completion_tokens: u32,
    /// Number of internal reasoning/thinking tokens generated.
    pub reasoning_tokens: u32,
    /// Time-To-First-Token in milliseconds from request dispatch.
    pub ttft_ms: u64,
    /// Total wall-clock execution duration in milliseconds.
    pub total_duration_ms: u64,
    /// Generation velocity in tokens per second.
    pub tokens_per_second: f64,
    /// Estimated financial expense in USD for this inference invocation.
    pub estimated_cost_usd: f64,
}

impl Default for TokenMetrics {
    fn default() -> Self {
        Self {
            prompt_tokens: 0,
            completion_tokens: 0,
            reasoning_tokens: 0,
            ttft_ms: 0,
            total_duration_ms: 0,
            tokens_per_second: 0.0,
            estimated_cost_usd: 0.0,
        }
    }
}

impl TokenMetrics {
    /// Constructs a new [`TokenMetrics`] instance with explicit parameters.
    pub fn new(
        prompt_tokens: u32,
        completion_tokens: u32,
        reasoning_tokens: u32,
        ttft_ms: u64,
        total_duration_ms: u64,
        tokens_per_second: f64,
        estimated_cost_usd: f64,
    ) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            reasoning_tokens,
            ttft_ms,
            total_duration_ms,
            tokens_per_second,
            estimated_cost_usd,
        }
    }

    /// Returns the sum of all tokens (prompt + completion + reasoning).
    pub fn total_tokens(&self) -> u32 {
        self.prompt_tokens
            .saturating_add(self.completion_tokens)
            .saturating_add(self.reasoning_tokens)
    }

    /// Computes token velocity in tokens/second given count and duration.
    pub fn calculate_velocity(token_count: u32, duration_ms: u64) -> f64 {
        if duration_ms == 0 {
            0.0
        } else {
            (token_count as f64) / (duration_ms as f64 / 1000.0)
        }
    }

    /// Merges another metric accumulation into self.
    pub fn merge(&mut self, other: &TokenMetrics) {
        self.prompt_tokens = self.prompt_tokens.saturating_add(other.prompt_tokens);
        self.completion_tokens = self.completion_tokens.saturating_add(other.completion_tokens);
        self.reasoning_tokens = self.reasoning_tokens.saturating_add(other.reasoning_tokens);
        self.total_duration_ms = self.total_duration_ms.saturating_add(other.total_duration_ms);
        self.estimated_cost_usd += other.estimated_cost_usd;
        if self.ttft_ms == 0 {
            self.ttft_ms = other.ttft_ms;
        }
        let total_output = self.completion_tokens.saturating_add(self.reasoning_tokens);
        self.tokens_per_second = Self::calculate_velocity(total_output, self.total_duration_ms);
    }
}

/// Real-time hardware telemetry and GPU/CPU resource utilization statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareStats {
    /// Currently allocated GPU VRAM in bytes.
    pub vram_allocated_bytes: u64,
    /// Total available GPU VRAM in bytes.
    pub vram_total_bytes: u64,
    /// Percentage GPU core utilization (0.0 to 100.0).
    pub gpu_utilization_pct: f32,
    /// Percentage CPU core utilization (0.0 to 100.0).
    pub cpu_utilization_pct: f32,
    /// Allocated system RAM in bytes.
    pub system_ram_allocated_bytes: u64,
    /// Total system RAM in bytes.
    pub system_ram_total_bytes: u64,
    /// GPU temperature in degrees Celsius (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature_celsius: Option<f32>,
}

impl Default for HardwareStats {
    fn default() -> Self {
        Self {
            vram_allocated_bytes: 0,
            vram_total_bytes: 0,
            gpu_utilization_pct: 0.0,
            cpu_utilization_pct: 0.0,
            system_ram_allocated_bytes: 0,
            system_ram_total_bytes: 0,
            temperature_celsius: None,
        }
    }
}

impl HardwareStats {
    /// Computes percentage of VRAM in active use.
    pub fn vram_usage_pct(&self) -> f32 {
        if self.vram_total_bytes == 0 {
            0.0
        } else {
            ((self.vram_allocated_bytes as f64 / self.vram_total_bytes as f64) * 100.0) as f32
        }
    }

    /// Computes percentage of system RAM in active use.
    pub fn ram_usage_pct(&self) -> f32 {
        if self.system_ram_total_bytes == 0 {
            0.0
        } else {
            ((self.system_ram_allocated_bytes as f64 / self.system_ram_total_bytes as f64) * 100.0)
                as f32
        }
    }

    /// Checks if VRAM usage exceeds a given threshold percentage.
    pub fn is_vram_constrained(&self, threshold_pct: f32) -> bool {
        self.vram_usage_pct() >= threshold_pct
    }
}

/// Token pricing model rates per 1,000,000 tokens in USD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPricing {
    /// Cost per 1,000,000 prompt/input tokens in USD.
    pub input_cost_per_million: f64,
    /// Cost per 1,000,000 completion/output tokens in USD.
    pub output_cost_per_million: f64,
    /// Optional cost per 1,000,000 reasoning tokens in USD (defaults to output cost).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_cost_per_million: Option<f64>,
}

impl ModelPricing {
    /// Free pricing tier (e.g. for local or mock models).
    pub const fn free() -> Self {
        Self {
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
            reasoning_cost_per_million: None,
        }
    }

    /// Standard pricing definition.
    pub fn new(input_cost_per_million: f64, output_cost_per_million: f64) -> Self {
        Self {
            input_cost_per_million,
            output_cost_per_million,
            reasoning_cost_per_million: None,
        }
    }

    /// Extended pricing with explicit reasoning token costs.
    pub fn with_reasoning(
        input_cost_per_million: f64,
        output_cost_per_million: f64,
        reasoning_cost_per_million: f64,
    ) -> Self {
        Self {
            input_cost_per_million,
            output_cost_per_million,
            reasoning_cost_per_million: Some(reasoning_cost_per_million),
        }
    }

    /// Calculates total financial cost in USD given prompt, completion, and reasoning token counts.
    pub fn calculate_cost(
        &self,
        prompt_tokens: u32,
        completion_tokens: u32,
        reasoning_tokens: u32,
    ) -> f64 {
        let prompt_cost = (prompt_tokens as f64 / 1_000_000.0) * self.input_cost_per_million;
        let completion_cost =
            (completion_tokens as f64 / 1_000_000.0) * self.output_cost_per_million;
        let reasoning_rate = self
            .reasoning_cost_per_million
            .unwrap_or(self.output_cost_per_million);
        let reasoning_cost = (reasoning_tokens as f64 / 1_000_000.0) * reasoning_rate;

        prompt_cost + completion_cost + reasoning_cost
    }
}

/// Catalog of default model pricing rates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PricingCatalog {
    pricing_map: HashMap<String, ModelPricing>,
}

impl Default for PricingCatalog {
    fn default() -> Self {
        let mut catalog = Self {
            pricing_map: HashMap::new(),
        };

        // Claude Models (Anthropic)
        catalog.register("claude-3-7-sonnet-20250219", ModelPricing::new(3.00, 15.00));
        catalog.register("claude-3-5-sonnet-20241022", ModelPricing::new(3.00, 15.00));
        catalog.register("claude-3-5-haiku-20241022", ModelPricing::new(0.80, 4.00));

        // OpenAI Models
        catalog.register("gpt-4o", ModelPricing::new(2.50, 10.00));
        catalog.register("gpt-4o-mini", ModelPricing::new(0.15, 0.60));
        catalog.register("o1", ModelPricing::new(15.00, 60.00));
        catalog.register("o3-mini", ModelPricing::new(1.10, 4.40));

        // Google Gemini Models
        catalog.register("gemini-2.0-flash", ModelPricing::new(0.10, 0.40));
        catalog.register("gemini-2.0-pro", ModelPricing::new(1.25, 5.00));

        // DeepSeek Models
        catalog.register("deepseek-chat", ModelPricing::new(0.14, 0.28));
        catalog.register("deepseek-reasoner", ModelPricing::new(0.55, 2.19));

        // Groq Models
        catalog.register("llama-3.3-70b-versatile", ModelPricing::new(0.59, 0.79));
        catalog.register("llama-3.1-8b-instant", ModelPricing::new(0.05, 0.08));

        catalog
    }
}

impl PricingCatalog {
    /// Creates a new empty pricing catalog.
    pub fn new() -> Self {
        Self {
            pricing_map: HashMap::new(),
        }
    }

    /// Registers a custom model with its pricing rates.
    pub fn register(&mut self, model: impl Into<String>, pricing: ModelPricing) {
        self.pricing_map.insert(model.into(), pricing);
    }

    /// Retrieves pricing for a given model, defaulting to free ($0.0) if unlisted.
    pub fn get_pricing(&self, model: &str) -> ModelPricing {
        self.pricing_map
            .get(model)
            .cloned()
            .unwrap_or(ModelPricing::free())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_pricing_reasoning_rate() {
        let p = ModelPricing::with_reasoning(2.0, 10.0, 12.0);
        let cost = p.calculate_cost(1_000_000, 1_000_000, 1_000_000);
        assert!((cost - 24.0).abs() < 1e-6);
    }

    #[test]
    fn test_pricing_catalog_custom_register() {
        let mut cat = PricingCatalog::new();
        cat.register("custom-fine-tune", ModelPricing::new(0.5, 1.5));
        let p = cat.get_pricing("custom-fine-tune");
        assert_eq!(p.input_cost_per_million, 0.5);
        assert_eq!(p.output_cost_per_million, 1.5);
    }
}

