//! Server-Sent Events (SSE) stream parser for HTTP response streaming.

use futures_util::{Stream, StreamExt};
use std::pin::Pin;
use xeno_core::errors::XenoError;

/// Discrete Server-Sent Event frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseEvent {
    /// Optional SSE event name (e.g. `message_start`, `content_block_delta`, `ping`).
    pub event_type: Option<String>,
    /// Accumulated event payload content from `data:` lines.
    pub data: String,
    /// Optional event identifier from `id:` line.
    pub id: Option<String>,
}

impl SseEvent {
    /// Returns true if data indicates end-of-stream `[DONE]`.
    pub fn is_done(&self) -> bool {
        self.data.trim() == "[DONE]"
    }
}

/// Converts a byte stream (from `reqwest::Response::bytes_stream()`) into an SSE event stream.
pub fn parse_sse_stream<S>(
    mut byte_stream: S,
) -> Pin<Box<dyn Stream<Item = Result<SseEvent, XenoError>> + Send>>
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + Unpin + 'static,
{
    let (tx, rx) = tokio::sync::mpsc::channel(128);

    tokio::spawn(async move {
        let mut raw_buffer: Vec<u8> = Vec::new();
        let mut buffer = String::new();

        while let Some(chunk_res) = byte_stream.next().await {
            match chunk_res {
                Ok(bytes) => {
                    raw_buffer.extend_from_slice(&bytes);

                    let valid_len = match std::str::from_utf8(&raw_buffer) {
                        Ok(s) => s.len(),
                        Err(e) => {
                            if e.error_len().is_none() {
                                // Incomplete multi-byte codepoint at end of buffer: decode up to valid boundary
                                e.valid_up_to()
                            } else {
                                // Genuinely invalid UTF-8 byte sequence
                                let _ = tx
                                    .send(Err(XenoError::StreamInterrupted {
                                        reason: format!("UTF-8 decode error: {e}"),
                                    }))
                                    .await;
                                return;
                            }
                        }
                    };

                    if valid_len > 0 {
                        let valid_str = match std::str::from_utf8(&raw_buffer[..valid_len]) {
                            Ok(s) => s,
                            Err(e) => {
                                let _ = tx
                                    .send(Err(XenoError::StreamInterrupted {
                                        reason: format!("UTF-8 decode error: {e}"),
                                    }))
                                    .await;
                                return;
                            }
                        };
                        buffer.push_str(valid_str);
                        raw_buffer.drain(..valid_len);
                    }

                    // Normalize carriage returns
                    buffer = buffer.replace("\r\n", "\n");

                    // Process full SSE event blocks delimited by double newlines
                    while let Some(idx) = buffer.find("\n\n") {
                        let event_block = buffer[..idx].to_string();
                        buffer = buffer[idx + 2..].to_string();

                        if let Some(event) = parse_single_event_block(&event_block) {
                            if tx.send(Ok(event)).await.is_err() {
                                return;
                            }
                        }
                    }
                }
                Err(err) => {
                    let _ = tx
                        .send(Err(XenoError::NetworkError {
                            message: format!("HTTP streaming error: {err}"),
                        }))
                        .await;
                    return;
                }
            }
        }

        // Flush remaining valid UTF-8 in raw_buffer if any
        if !raw_buffer.is_empty() {
            if let Ok(valid_str) = std::str::from_utf8(&raw_buffer) {
                buffer.push_str(valid_str);
                raw_buffer.clear();
            }
        }

        // Flush any trailing event in the buffer
        if !buffer.trim().is_empty() {
            if let Some(event) = parse_single_event_block(&buffer) {
                let _ = tx.send(Ok(event)).await;
            }
        }
    });

    Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))
}

fn parse_single_event_block(block: &str) -> Option<SseEvent> {
    let mut event_type = None;
    let mut data_lines = Vec::new();
    let mut id = None;

    for line in block.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with(':') {
            // Ignore empty lines and SSE comments / keepalives
            continue;
        }

        if let Some(rest) = line.strip_prefix("event:") {
            event_type = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("data:") {
            // Remove single leading space after 'data:' if present according to SSE spec
            let data_content = rest.strip_prefix(' ').unwrap_or(rest);
            data_lines.push(data_content.to_string());
        } else if let Some(rest) = line.strip_prefix("id:") {
            id = Some(rest.trim().to_string());
        }
    }

    if data_lines.is_empty() && event_type.is_none() {
        None
    } else {
        Some(SseEvent {
            event_type,
            data: data_lines.join("\n"),
            id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_event_block() {
        let block = "event: message_delta\ndata: {\"key\": \"value\"}\nid: 123";
        let ev = parse_single_event_block(block).unwrap();
        assert_eq!(ev.event_type.as_deref(), Some("message_delta"));
        assert_eq!(ev.data, "{\"key\": \"value\"}");
        assert_eq!(ev.id.as_deref(), Some("123"));
        assert!(!ev.is_done());
    }

    #[test]
    fn test_parse_done_event() {
        let block = "data: [DONE]";
        let ev = parse_single_event_block(block).unwrap();
        assert!(ev.is_done());
    }
}
