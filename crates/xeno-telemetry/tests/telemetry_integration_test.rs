//! Telemetry integration test suite for `crates/xeno-telemetry`.

use xeno_telemetry::prelude::*;

#[tokio::test]
async fn test_telemetry_broadcast_and_aggregation() {
    let collector = TelemetryCollector::new(100);
    let mut rx = collector.subscribe();

    let step = StepTelemetry::new(
        "session-test-001",
        "commander",
        "thinking",
        500,
        1200,
        300,
        0.0045,
        "claude-3-7-sonnet",
    );

    collector.record_step(step.clone());

    let received = rx.recv().await.unwrap();
    assert_eq!(received.agent_role, "commander");
    assert_eq!(received.duration_ms, 500);
    assert_eq!(received.prompt_tokens, 1200);
    assert_eq!(received.completion_tokens, 300);

    let summary = collector.compute_summary();
    assert_eq!(summary.step_count, 1);
    assert_eq!(summary.total_prompt_tokens, 1200);
    assert_eq!(summary.total_completion_tokens, 300);
}
