//! Recursive Self-Healing engine for automated patch synthesis and rollback.

use serde::{Deserialize, Serialize};
use xeno_tools::prelude::RollbackStack;

/// Diagnostics and error context captured for self-healing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnomalyContext {
    pub failed_phase: String,
    pub error_message: String,
    pub affected_file: Option<String>,
    pub compiler_stderr: Option<String>,
    pub retry_attempt: u32,
    pub max_retries: u32,
}

/// Automated patch plan produced by self-healing reflection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfHealingPatch {
    pub explanation: String,
    pub target_file: String,
    pub target_content: String,
    pub replacement_content: String,
    pub rollback_required: bool,
}

/// Self-healing recovery loop engine.
#[derive(Debug, Default)]
pub struct SelfHealingEngine {
    rollback_stack: RollbackStack,
}

impl SelfHealingEngine {
    pub fn new() -> Self {
        Self {
            rollback_stack: RollbackStack::new(),
        }
    }

    /// Evaluates if an anomaly can be self-healed within budget.
    pub fn can_self_heal(&self, anomaly: &AnomalyContext) -> bool {
        anomaly.retry_attempt < anomaly.max_retries
    }

    /// Synthesizes a patch recommendation based on error diagnostic patterns.
    pub fn synthesize_repair_strategy(
        &self,
        anomaly: &AnomalyContext,
    ) -> Option<SelfHealingPatch> {
        if !self.can_self_heal(anomaly) {
            return None;
        }

        // Detect common syntax or type errors
        let err = &anomaly.error_message;
        if err.contains("unclosed delimiter") || err.contains("syntax error") {
            if let Some(file) = &anomaly.affected_file {
                return Some(SelfHealingPatch {
                    explanation: "Repair unbalanced delimiters and syntax defect".into(),
                    target_file: file.clone(),
                    target_content: "".into(),
                    replacement_content: "".into(),
                    rollback_required: true,
                });
            }
        }

        None
    }

    /// Accesses the rollback stack.
    pub fn rollback_stack(&self) -> &RollbackStack {
        &self.rollback_stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_healing_retry_budget() {
        let engine = SelfHealingEngine::new();
        let anomaly = AnomalyContext {
            failed_phase: "verify".into(),
            error_message: "unit test failed".into(),
            affected_file: Some("src/math.rs".into()),
            compiler_stderr: None,
            retry_attempt: 1,
            max_retries: 3,
        };

        assert!(engine.can_self_heal(&anomaly));

        let exhausted = AnomalyContext {
            retry_attempt: 3,
            ..anomaly
        };
        assert!(!engine.can_self_heal(&exhausted));
    }
}
