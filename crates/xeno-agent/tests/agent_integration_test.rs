//! Agent Harness, Swarm, and PAORV loop integration tests.

use xeno_agent::prelude::*;

#[tokio::test]
async fn test_agent_harness_full_paorv_cycle() {
    let mut harness = XenoAgentHarness::new("sess-test-042", "Refactor module and verify");
    let result = harness.execute_goal("Refactor module and verify").await.unwrap();

    assert!(result.success);
    assert_eq!(result.session_id, "sess-test-042");
    assert_eq!(result.total_steps, 3);
    assert_eq!(result.telemetry_summary.step_count, 3);

    // Verify L2 store record
    let l2_rec = harness.l2_store.get_session("sess-test-042").unwrap();
    assert!(l2_rec.success);
    assert_eq!(l2_rec.total_steps, 3);
}

#[tokio::test]
async fn test_swarm_council_three_way_consensus() {
    let council = SwarmCouncil::new();

    let unanimous_votes = vec![
        ConsensusVote {
            voter_id: "commander".into(),
            role: SwarmRole::Commander,
            model_name: "claude-3-7-sonnet".into(),
            approved: true,
            confidence: 0.98,
            findings: vec![],
        },
        ConsensusVote {
            voter_id: "coder".into(),
            role: SwarmRole::Coder,
            model_name: "deepseek-reasoner".into(),
            approved: true,
            confidence: 0.95,
            findings: vec![],
        },
        ConsensusVote {
            voter_id: "qa_tester".into(),
            role: SwarmRole::QATester,
            model_name: "qwen-2.5-72b-local".into(),
            approved: true,
            confidence: 0.92,
            findings: vec![],
        },
    ];

    let decision = council.verify_critical_decision(unanimous_votes);
    assert!(decision.approved);
    assert_eq!(decision.positive_votes, 3);
    assert_eq!(decision.consensus_ratio, 1.0);
}
