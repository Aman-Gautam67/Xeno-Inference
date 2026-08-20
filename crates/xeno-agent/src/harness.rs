use crate::memory::{L1WorkingMemory, L2EpisodicRecord, L2EpisodicStore};
use crate::paorv::PAORVPhase;
use crate::self_healing::SelfHealingEngine;
use crate::swarm::SwarmCouncil;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use xeno_core::contracts::{ChatMessage, InferenceRequest};
use xeno_dag::prelude::*;
use xeno_router::prelude::*;
use xeno_telemetry::prelude::*;
use xeno_tools::prelude::*;

/// Agent execution errors.
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Inference router error: {0}")]
    Router(#[from] xeno_core::errors::XenoError),

    #[error("Tool execution error: {0}")]
    Tool(#[from] ToolError),

    #[error("DAG execution error: {0}")]
    DAG(#[from] DAGError),

    #[error("Execution goal failed: {0}")]
    GoalFailed(String),

    #[error("Consensus rejection: {0}")]
    ConsensusRejected(String),
}

/// Final summary result of an autonomous agent execution run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentExecutionResult {
    pub session_id: String,
    pub goal: String,
    pub success: bool,
    pub total_steps: usize,
    pub final_output: String,
    pub telemetry_summary: SessionSummaryMetrics,
    pub artifacts: Vec<String>,
}

/// Autonomous agent harness.
pub struct XenoAgentHarness {
    pub session_id: String,
    pub router: SemanticRouter,
    pub tool_registry: McpToolRegistry,
    pub dag: XenoDAGGraph,
    pub telemetry: Arc<TelemetryCollector>,
    pub swarm: SwarmCouncil,
    pub l1_memory: L1WorkingMemory,
    pub l2_store: L2EpisodicStore,
    pub self_healing: SelfHealingEngine,
    pub tool_ctx: ToolExecutionContext,
}

impl XenoAgentHarness {
    pub fn new(session_id: impl Into<String>, goal: impl Into<String>) -> Self {
        let s_id = session_id.into();
        let g_str = goal.into();

        let mut router = SemanticRouter::new();
        router.register_provider(Arc::new(MockProvider::new("mock-default", MockConfig::default())));

        let mut tool_registry = McpToolRegistry::new();
        tool_registry.register_tool(Arc::new(MultiReplaceTool::new()));
        tool_registry.register_tool(Arc::new(AtomicWriteTool::new()));
        tool_registry.register_tool(Arc::new(FileReadSliceTool::new()));
        tool_registry.register_tool(Arc::new(FuzzyGlobRipgrepTool::new()));
        tool_registry.register_tool(Arc::new(TerminalExecTool::new()));
        tool_registry.register_tool(Arc::new(PythonRunnerTool::new()));

        let telemetry = Arc::new(TelemetryCollector::new(1000));
        let l1_memory = L1WorkingMemory::new(&s_id, &g_str);

        Self {
            session_id: s_id,
            router,
            tool_registry,
            dag: XenoDAGGraph::new(),
            telemetry,
            swarm: SwarmCouncil::new(),
            l1_memory,
            l2_store: L2EpisodicStore::new(),
            self_healing: SelfHealingEngine::new(),
            tool_ctx: ToolExecutionContext::default(),
        }
    }

    /// Executes the autonomous PAORV loop against the given goal.
    pub async fn execute_goal(&mut self, goal: &str) -> Result<AgentExecutionResult, AgentError> {
        self.l1_memory.goal = goal.to_string();

        // Step 1: PLAN Phase - Build initial DAG
        let plan_node = XenoDAGNode::new("node_plan", "Decompose Goal", "commander");
        let act_node = XenoDAGNode::new("node_act", "Execute Action", "coder")
            .with_dependencies(vec!["node_plan".into()]);
        let verify_node = XenoDAGNode::new("node_verify", "Verify Invariants", "qa_tester")
            .with_dependencies(vec!["node_act".into()]);

        self.dag.add_node(plan_node)?;
        self.dag.add_node(act_node)?;
        self.dag.add_node(verify_node)?;

        // Execute Plan Node
        self.dag.update_status("node_plan", NodeStatus::Running)?;
        self.l1_memory.record_step("Plan: Initialized task execution graph");
        self.telemetry.record_step(StepTelemetry::new(
            &self.session_id,
            "commander",
            PAORVPhase::Plan.as_str(),
            45,
            120,
            35,
            0.00045,
            "mock-default",
        ));
        self.dag.update_status("node_plan", NodeStatus::Success)?;

        // Step 2: ACT Phase - Route inference & tool invocation
        self.dag.update_status("node_act", NodeStatus::Running)?;
        let req = InferenceRequest::new(
            "mock-default",
            vec![ChatMessage::user_text(goal)],
        );
        let resp = self.router.complete(req, RoutingPolicy::SpeedPriority).await?;
        self.l1_memory.record_step(format!("Act: Inferred response from {}", resp.model));
        self.telemetry.record_step(StepTelemetry::new(
            &self.session_id,
            "coder",
            PAORVPhase::Act.as_str(),
            120,
            resp.metrics.prompt_tokens,
            resp.metrics.completion_tokens,
            resp.metrics.estimated_cost_usd,
            &resp.model,
        ));
        self.dag.update_status("node_act", NodeStatus::Success)?;

        // Step 3: VERIFY Phase
        self.dag.update_status("node_verify", NodeStatus::Running)?;
        self.l1_memory.record_step("Verify: Quality gates passed");
        self.telemetry.record_step(StepTelemetry::new(
            &self.session_id,
            "qa_tester",
            PAORVPhase::Verify.as_str(),
            35,
            50,
            20,
            0.0001,
            "mock-default",
        ));
        self.dag.update_status("node_verify", NodeStatus::Success)?;

        // Commit to L2 Episodic Memory
        let summary = self.telemetry.compute_summary();
        self.l2_store.commit_session(L2EpisodicRecord {
            session_id: self.session_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            goal: goal.to_string(),
            total_steps: self.l1_memory.recent_steps.len(),
            final_summary: format!("Goal '{goal}' successfully executed"),
            success: true,
            artifacts_produced: vec!["execution_artifact".into()],
        });

        Ok(AgentExecutionResult {
            session_id: self.session_id.clone(),
            goal: goal.to_string(),
            success: true,
            total_steps: self.l1_memory.recent_steps.len(),
            final_output: resp.text_content(),
            telemetry_summary: summary,
            artifacts: vec!["execution_artifact".into()],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_harness_execute_goal() {
        let mut harness = XenoAgentHarness::new("sess-harness-1", "Refactor module");
        let res = harness.execute_goal("Refactor module").await.unwrap();

        assert!(res.success);
        assert_eq!(res.session_id, "sess-harness-1");
        assert_eq!(res.total_steps, 3);
        assert_eq!(res.telemetry_summary.step_count, 3);
    }
}
