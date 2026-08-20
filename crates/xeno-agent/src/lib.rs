//! XENO INFERENCE — Autonomous Agent Harness & Swarm Orchestrator (`xeno-agent`).
//!
//! Implements the Plan-Act-Observe-Reflect-Verify (PAORV) continuous state machine,
//! hierarchical swarm council (Commander, Architect, Coder, QA Tester, Red-Team Auditor),
//! multi-tier memory (L1 working context and L2 episodic session store), 3-way consensus checking,
//! and native Model Context Protocol (MCP) tool hosting.

pub mod harness;
pub mod memory;
pub mod paorv;
pub mod self_healing;
pub mod swarm;

pub use harness::{AgentError, AgentExecutionResult, XenoAgentHarness};
pub use memory::{L1WorkingMemory, L2EpisodicRecord, L2EpisodicStore};
pub use paorv::{PAORVPhase, PAORVStepRecord};
pub use self_healing::{AnomalyContext, SelfHealingEngine, SelfHealingPatch};
pub use swarm::{ConsensusChecker, ConsensusEvaluation, ConsensusVote, SwarmCouncil, SwarmRole};

/// Prelude exporting all agent harness primitives.
pub mod prelude {
    pub use super::harness::{AgentError, AgentExecutionResult, XenoAgentHarness};
    pub use super::memory::{L1WorkingMemory, L2EpisodicRecord, L2EpisodicStore};
    pub use super::paorv::{PAORVPhase, PAORVStepRecord};
    pub use super::self_healing::{AnomalyContext, SelfHealingEngine, SelfHealingPatch};
    pub use super::swarm::{
        ConsensusChecker, ConsensusEvaluation, ConsensusVote, SwarmCouncil, SwarmRole,
    };
}
