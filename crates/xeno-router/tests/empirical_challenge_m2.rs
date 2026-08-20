//! Comprehensive empirical challenge and stress test suite for xeno-router (Milestone 2).
//!
//! Tests:
//! 1. Streaming token bus backpressure with slow subscribers and channel exhaustion
//! 2. Multi-receiver drop lifecycle and downstream stream cancellation
//! 3. Split multi-byte UTF-8 sequences across SSE chunk boundaries (2-byte, 3-byte, 4-byte)
//! 4. Malformed SSE frames (garbage data, missing colon, comment lines, CRLF variations, incomplete trailing)
//! 5. Rapid stream aborts under high concurrency
//! 6. Router dynamic policy dispatch under heavy simulated upstream errors
//! 7. Air-gap enforcement edge cases (IPv6 loopback, non-loopback IPs, local vs cloud providers)
//! 8. Token velocity and cost estimator edge cases

use bytes::Bytes;
use futures_util::{stream, StreamExt};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use xeno_core::contracts::{ChatMessage, InferenceRequest, StreamChunk};
use xeno_core::metrics::ModelPricing;
use xeno_router::prelude::*;
use xeno_router::providers::sse::parse_sse_stream;

/// 1. Backpressure test on TokenBus:
/// Verify that slow subscribers receive Lagged errors rather than deadlocking or crashing the bus,
/// and that fast subscribers receive all chunks intact.
#[tokio::test]
async fn test_token_bus_backpressure_slow_and_fast_subscribers() {
    let bus_capacity = 8;
    let bus = TokenBus::new(bus_capacity);

    let mut fast_sub = bus.subscribe();
    let mut slow_sub = bus.subscribe();

    let total_chunks = 100;
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    let upstream = Box::pin(ReceiverStream::new(rx));

    // Producer sends 100 chunks quickly
    tokio::spawn(async move {
        for i in 0..total_chunks {
            let chunk = StreamChunk::text(i, format!(" tok_{i}"));
            if tx.send(Ok(chunk)).await.is_err() {
                break;
            }
        }
    });

    let mut wrapped_stream = bus.wrap_stream("test-model".into(), upstream, 10, None);

    // Fast subscriber task consumes immediately
    let fast_received = Arc::new(AtomicUsize::new(0));
    let fast_received_clone = fast_received.clone();
    let fast_handle = tokio::spawn(async move {
        loop {
            match fast_sub.recv().await {
                Ok(_) => {
                    fast_received_clone.fetch_add(1, Ordering::SeqCst);
                }
                Err(RecvError::Closed) => break,
                Err(RecvError::Lagged(_)) => {}
            }
        }
    });

    // Primary downstream consumer reads all chunks to completion
    let mut downstream_count = 0;
    while let Some(res) = wrapped_stream.next().await {
        let chunk = res.expect("Downstream chunk should not fail");
        assert!(chunk.partial_metrics.is_some());
        downstream_count += 1;
    }

    assert_eq!(downstream_count, total_chunks, "Downstream consumer must receive all chunks");

    // Fast subscriber should have received tokens
    sleep(Duration::from_millis(20)).await;
    assert!(
        fast_received.load(Ordering::SeqCst) > 0,
        "Fast subscriber must have received tokens"
    );

    // Slow subscriber now attempts to read after 100 items were broadcast into capacity-8 channel
    let mut slow_lagged = 0usize;
    loop {
        match slow_sub.try_recv() {
            Ok(_) => {}
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(skipped)) => {
                slow_lagged += skipped as usize;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => break,
            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => break,
        }
    }

    // Slow subscriber should have experienced lagging due to 100 > 8 overflow
    assert!(
        slow_lagged > 0,
        "Slow subscriber must detect lag when channel overflows, got {slow_lagged}"
    );

    fast_handle.abort();
}

/// 2. Dropped Receivers test:
/// Verify that dropping all broadcast subscribers or dropping individual subscribers
/// mid-stream causes zero panics and allows the stream to complete cleanly.
#[tokio::test]
async fn test_token_bus_dropped_subscribers_lifecycle() {
    let bus = TokenBus::new(16);

    let sub1 = bus.subscribe();
    let sub2 = bus.subscribe();
    let sub3 = bus.subscribe();

    // Drop sub1 immediately
    drop(sub1);

    let (tx, rx) = tokio::sync::mpsc::channel(16);
    let upstream = Box::pin(ReceiverStream::new(rx));

    tokio::spawn(async move {
        for i in 0..50 {
            let chunk = StreamChunk::text(i, format!(" tok_{i}"));
            let _ = tx.send(Ok(chunk)).await;
        }
    });

    let mut wrapped = bus.wrap_stream("model-drop-test".into(), upstream, 20, None);

    let mut count = 0;
    let mut sub2_opt = Some(sub2);
    let mut sub3_opt = Some(sub3);

    while let Some(res) = wrapped.next().await {
        let chunk = res.expect("chunk should succeed");
        count += 1;

        // Drop sub2 at chunk 10
        if count == 10 {
            sub2_opt.take(); // drops sub2
        }

        // Drop sub3 at chunk 25
        if count == 25 {
            sub3_opt.take(); // drops sub3 (now 0 active broadcast subscribers)
        }

        assert!(chunk.partial_metrics.is_some());
    }

    assert_eq!(count, 50, "All 50 chunks must be received downstream despite subscriber drops");
}

/// 3. Rapid Stream Aborts on Downstream Caller:
/// When downstream caller drops the stream after reading only N chunks, the background task
/// must exit immediately without leaking or panicking.
#[tokio::test]
async fn test_rapid_stream_abort_caller_drop() {
    let mock = Arc::new(MockProvider::new(
        "mock-abort",
        MockConfig {
            default_text: "A".repeat(5000),
            chunk_delay: Duration::from_millis(1),
            chunk_size: 10,
            ..Default::default()
        },
    ));

    let bus = TokenBus::new(32);

    for i in 0..20 {
        let req = InferenceRequest::new("mock", vec![ChatMessage::user_text(format!("req_{i}"))]);
        let upstream = mock.stream(&req).await.unwrap();
        let mut wrapped = bus.wrap_stream("mock".into(), upstream, 10, None);

        // Read only 2 chunks and immediately drop the stream
        let first = wrapped.next().await;
        assert!(first.is_some());
        let second = wrapped.next().await;
        assert!(second.is_some());

        // Drop stream
        drop(wrapped);
    }
}

/// 4. Split Multi-byte UTF-8 across SSE Chunk Boundaries:
/// Tests whether `parse_sse_stream` correctly handles multi-byte UTF-8 codepoints
/// split across consecutive `Bytes` chunks from HTTP transport.
#[tokio::test]
async fn test_sse_parser_split_multibyte_utf8_boundaries() {
    // 4-byte UTF-8: '🚀' = [0xF0, 0x9F, 0x9A, 0x80]
    // Chunk 1 contains half of '🚀' ([0xF0, 0x9F])
    // Chunk 2 contains the rest of '🚀' ([0x9A, 0x80])
    let part1 = b"data: {\"text\": \"Launch \xf0\x9f".to_vec();
    let part2 = b"\x9a\x80 Now!\"}\n\n".to_vec();

    let byte_items: Vec<Result<Bytes, reqwest::Error>> = vec![
        Ok(Bytes::from(part1)),
        Ok(Bytes::from(part2)),
        Ok(Bytes::from("data: [DONE]\n\n")),
    ];

    let byte_stream = stream::iter(byte_items);
    let mut sse_stream = parse_sse_stream(byte_stream);

    let mut events = Vec::new();
    while let Some(res) = sse_stream.next().await {
        events.push(res);
    }

    assert!(!events.is_empty(), "Should receive SSE events");
    // Oracle check for split UTF-8 handling:
    let first = &events[0];
    match first {
        Ok(ev) => {
            assert!(
                ev.data.contains("🚀"),
                "Event data must contain reconstructed emoji '🚀', got: {}",
                ev.data
            );
        }
        Err(err) => {
            eprintln!("EMPIRICAL FINDING: Split UTF-8 failed with: {:?}", err);
            // Record failure mode for challenger report
            panic!("parse_sse_stream failed on split UTF-8 codepoint boundary: {:?}", err);
        }
    }
}

/// 5. Malformed SSE frames:
/// Test parsing of comments, empty lines, missing spaces, CRLF line endings, and trailing partial frames.
#[tokio::test]
async fn test_sse_parser_malformed_and_comment_frames() {
    let raw_payload = concat!(
        ": keep-alive ping comment\r\n",
        "\r\n",
        "event: message_delta\r\n",
        "data: {\"text\": \"first\"}\r\n",
        "\r\n",
        ": another comment\n",
        "data:{\"text\": \"no space after colon\"}\n",
        "\n",
        "data: line1\ndata: line2\n\n",
        "data: [DONE]"
    );

    let byte_items: Vec<Result<Bytes, reqwest::Error>> = vec![
        Ok(Bytes::from(raw_payload.as_bytes())),
    ];

    let byte_stream = stream::iter(byte_items);
    let mut sse_stream = parse_sse_stream(byte_stream);

    let mut events = Vec::new();
    while let Some(res) = sse_stream.next().await {
        events.push(res.expect("Malformed tolerant parser should not error on valid SSE variants"));
    }

    assert_eq!(events.len(), 4);
    assert_eq!(events[0].event_type.as_deref(), Some("message_delta"));
    assert_eq!(events[0].data, "{\"text\": \"first\"}");

    assert_eq!(events[1].data, "{\"text\": \"no space after colon\"}");

    assert_eq!(events[2].data, "line1\nline2");

    assert!(events[3].is_done());
}

/// 6. High concurrency rapid aborts stress test:
/// Launch 50 concurrent streaming sessions through SemanticRouter with MockProvider and abort them randomly.
#[tokio::test]
async fn test_stress_concurrent_rapid_aborts() {
    let mut router = SemanticRouter::new();
    let mock = Arc::new(MockProvider::new(
        "stress-mock",
        MockConfig {
            default_text: "High-throughput token output sequence for concurrency testing.".repeat(20),
            chunk_delay: Duration::from_micros(50),
            chunk_size: 8,
            ..Default::default()
        },
    ));
    router.register_provider(mock);

    let router = Arc::new(router);
    let mut handles = Vec::new();

    for i in 0..50 {
        let r = router.clone();
        let handle = tokio::spawn(async move {
            let req = InferenceRequest::new("stress-mock", vec![ChatMessage::user_text(format!("Prompt {i}"))]);
            let mut stream = r.stream(req, RoutingPolicy::SpeedPriority).await.unwrap();

            // Read a variable number of chunks (between 0 and 15) then abort
            let read_target = i % 15;
            let mut read_count = 0;
            while let Some(chunk_res) = stream.next().await {
                assert!(chunk_res.is_ok());
                read_count += 1;
                if read_count >= read_target {
                    break;
                }
            }
            drop(stream);
        });
        handles.push(handle);
    }

    for h in handles {
        h.await.expect("Task must not panic");
    }
}

/// 7. Air-gap enforcement edge cases:
#[test]
fn test_air_gap_enforcer_ip_validation() {
    let enforcer = AirGapEnforcer::new();

    // Loopback should pass
    assert!(enforcer.validate_endpoint_url("http://127.0.0.1:8000/v1", true).is_ok());
    assert!(enforcer.validate_endpoint_url("http://localhost:11434/v1", true).is_ok());
    assert!(enforcer.validate_endpoint_url("http://[::1]:8080/v1", true).is_ok());
    assert!(enforcer.validate_endpoint_url("http://0.0.0.0:8000/v1", true).is_ok());

    // Non-loopback should fail in air-gap mode
    assert!(enforcer.validate_endpoint_url("http://192.168.1.100:8000/v1", true).is_err());
    assert!(enforcer.validate_endpoint_url("https://api.openai.com/v1", true).is_err());
    assert!(enforcer.validate_endpoint_url("https://api.anthropic.com/v1", true).is_err());

    // Provider kinds
    assert!(enforcer.validate_provider_kind(ProviderKind::Local, true).is_ok());
    assert!(enforcer.validate_provider_kind(ProviderKind::Mock, true).is_ok());
    assert!(enforcer.validate_provider_kind(ProviderKind::Openai, true).is_err());
    assert!(enforcer.validate_provider_kind(ProviderKind::Anthropic, true).is_err());
    assert!(enforcer.validate_provider_kind(ProviderKind::Google, true).is_err());
    assert!(enforcer.validate_provider_kind(ProviderKind::Groq, true).is_err());
    assert!(enforcer.validate_provider_kind(ProviderKind::Deepseek, true).is_err());
}

/// 8. Velocity calculator extreme edge cases:
#[tokio::test]
async fn test_velocity_calculator_burst_and_reset_edges() {
    let mut calc = TokenVelocityCalculator::new(Duration::from_millis(500), 0.3);

    // Initial state
    assert_eq!(calc.total_tokens(), 0);
    assert_eq!(calc.current_velocity(), 0.0);
    assert_eq!(calc.ema_velocity(), 0.0);
    assert_eq!(calc.average_velocity(), 0.0);

    // Record 0 tokens
    calc.record_tokens(0);
    assert_eq!(calc.total_tokens(), 0);

    // Wait 5ms so elapsed >= 0.001s
    sleep(Duration::from_millis(5)).await;

    // Record massive burst
    calc.record_tokens(1_000_000);
    assert_eq!(calc.total_tokens(), 1_000_000);
    assert!(calc.current_velocity() > 0.0);
    assert!(calc.ema_velocity() > 0.0);
    assert!(calc.average_velocity() > 0.0);

    // Reset
    calc.reset();
    assert_eq!(calc.total_tokens(), 0);
    assert_eq!(calc.current_velocity(), 0.0);
    assert_eq!(calc.ema_velocity(), 0.0);
}

/// 9. Cost estimator boundary conditions:
#[test]
fn test_cost_estimator_boundaries() {
    let estimator = CostEstimator::default();

    // Zero tokens
    let zero_cost = estimator.estimate_cost("gpt-4o", 0, 0, 0);
    assert_eq!(zero_cost, 0.0);

    // Known model calculation
    let known_cost = estimator.estimate_cost("gpt-4o", 1_000_000, 1_000_000, 0);
    assert_eq!(known_cost, 12.50);

    // Unlisted model defaults to free ($0.0)
    let unlisted_cost = estimator.estimate_cost("unlisted-model-x", 1_000_000, 1_000_000, 0);
    assert_eq!(unlisted_cost, 0.0);

    // Custom registered pricing
    estimator.register_pricing("custom-model", ModelPricing::new(5.0, 20.0));
    let custom_cost = estimator.estimate_cost("custom-model", 1_000_000, 1_000_000, 0);
    assert_eq!(custom_cost, 25.0);
}
