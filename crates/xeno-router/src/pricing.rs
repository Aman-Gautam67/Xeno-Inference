//! Real-time inference cost estimation and budget tracking engine.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use xeno_core::metrics::{ModelPricing, PricingCatalog, TokenMetrics};

/// Real-time USD cost estimation and session budget tracking engine.
#[derive(Debug, Clone)]
pub struct CostEstimator {
    catalog: Arc<Mutex<PricingCatalog>>,
    session_totals: Arc<Mutex<HashMap<String, f64>>>,
}

impl Default for CostEstimator {
    fn default() -> Self {
        Self::new(PricingCatalog::default())
    }
}

impl CostEstimator {
    /// Constructs a new [`CostEstimator`] with a custom pricing catalog.
    pub fn new(catalog: PricingCatalog) -> Self {
        Self {
            catalog: Arc::new(Mutex::new(catalog)),
            session_totals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers or updates pricing rates for a model.
    pub fn register_pricing(&self, model: impl Into<String>, pricing: ModelPricing) {
        let mut cat = self.catalog.lock().unwrap();
        cat.register(model, pricing);
    }

    /// Retrieves the pricing rates for a given model.
    pub fn get_pricing(&self, model: &str) -> ModelPricing {
        let cat = self.catalog.lock().unwrap();
        cat.get_pricing(model)
    }

    /// Estimates the USD cost for the specified token quantities.
    pub fn estimate_cost(
        &self,
        model: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        reasoning_tokens: u32,
    ) -> f64 {
        let pricing = self.get_pricing(model);
        pricing.calculate_cost(prompt_tokens, completion_tokens, reasoning_tokens)
    }

    /// Updates metrics in-place with the computed estimated cost.
    pub fn enrich_metrics(&self, model: &str, metrics: &mut TokenMetrics) {
        metrics.estimated_cost_usd = self.estimate_cost(
            model,
            metrics.prompt_tokens,
            metrics.completion_tokens,
            metrics.reasoning_tokens,
        );
    }

    /// Records an inference cost into session totals.
    pub fn record_session_cost(&self, session_id: &str, model: &str, cost_usd: f64) {
        let mut totals = self.session_totals.lock().unwrap();
        let current = totals.entry(session_id.to_string()).or_insert(0.0);
        *current += cost_usd;

        let model_key = format!("{session_id}:{model}");
        let model_entry = totals.entry(model_key).or_insert(0.0);
        *model_entry += cost_usd;
    }

    /// Gets the total accumulated cost in USD for a session.
    pub fn get_session_cost(&self, session_id: &str) -> f64 {
        let totals = self.session_totals.lock().unwrap();
        totals.get(session_id).copied().unwrap_or(0.0)
    }

    /// Checks if a session has exceeded a financial budget limit in USD.
    pub fn is_budget_exceeded(&self, session_id: &str, budget_limit_usd: f64) -> bool {
        self.get_session_cost(session_id) >= budget_limit_usd
    }

    /// Clears recorded session costs.
    pub fn clear_sessions(&self) {
        self.session_totals.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimator_calculation() {
        let estimator = CostEstimator::default();
        // claude-3-7-sonnet is $3 input, $15 output per million
        let cost = estimator.estimate_cost("claude-3-7-sonnet-20250219", 10_000, 2_000, 0);
        // Prompt: (10,000 / 1,000,000) * 3 = 0.03
        // Completion: (2,000 / 1,000,000) * 15 = 0.03
        // Total: 0.06
        assert!((cost - 0.06).abs() < 1e-5);
    }

    #[test]
    fn test_cost_estimator_session_tracking() {
        let estimator = CostEstimator::default();
        estimator.record_session_cost("sess_1", "gpt-4o", 0.025);
        estimator.record_session_cost("sess_1", "gpt-4o", 0.015);

        assert!((estimator.get_session_cost("sess_1") - 0.040).abs() < 1e-5);
        assert!(!estimator.is_budget_exceeded("sess_1", 0.05));
        assert!(estimator.is_budget_exceeded("sess_1", 0.03));
    }
}
