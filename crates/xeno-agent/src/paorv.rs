//! Plan-Act-Observe-Reflect-Verify (PAORV) continuous agentic state machine.

use serde::{Deserialize, Serialize};

/// Discrete states in the PAORV continuous execution loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PAORVPhase {
    /// 1. Plan: Goal decomposition, context pinning, DAG node synthesis.
    Plan,
    /// 2. Act: Tool selection, schema validation, and dispatch.
    Act,
    /// 3. Observe: Capture execution outputs, AST diffs, exit codes.
    Observe,
    /// 4. Reflect: Anomaly detection, hypothesis refinement.
    Reflect,
    /// 5. Verify: AST syntax checks, dynamic test execution, assertions.
    Verify,
    /// 6. Self-Heal: Automated patch synthesis and retry on anomalies.
    SelfHeal,
    /// Terminal successful state.
    Completed,
    /// Terminal failed state.
    Failed,
}

impl Default for PAORVPhase {
    fn default() -> Self {
        PAORVPhase::Plan
    }
}

impl PAORVPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Act => "act",
            Self::Observe => "observe",
            Self::Reflect => "reflect",
            Self::Verify => "verify",
            Self::SelfHeal => "self_heal",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    /// Determines the natural next state in the happy path.
    pub fn next_phase(&self) -> Self {
        match self {
            Self::Plan => Self::Act,
            Self::Act => Self::Observe,
            Self::Observe => Self::Reflect,
            Self::Reflect => Self::Verify,
            Self::Verify => Self::Completed,
            Self::SelfHeal => Self::Plan,
            Self::Completed => Self::Completed,
            Self::Failed => Self::Failed,
        }
    }
}

/// Execution snapshot of a single PAORV iteration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PAORVStepRecord {
    pub iteration: u32,
    pub phase: PAORVPhase,
    pub agent_role: String,
    pub action_taken: String,
    pub observation: Option<String>,
    pub reflection: Option<String>,
    pub verified: bool,
    pub duration_ms: u64,
}

impl PAORVStepRecord {
    pub fn new(
        iteration: u32,
        phase: PAORVPhase,
        agent_role: impl Into<String>,
        action_taken: impl Into<String>,
    ) -> Self {
        Self {
            iteration,
            phase,
            agent_role: agent_role.into(),
            action_taken: action_taken.into(),
            observation: None,
            reflection: None,
            verified: false,
            duration_ms: 0,
        }
    }
}
