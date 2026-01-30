//! SSE parsing utilities for OpenAI-format streaming responses.
//!
//! OpenAI's streaming API uses Server-Sent Events (SSE) to deliver chunks of the
//! completion response. This module provides deserialization and parsing utilities.
//!
//! # SSE Format
//!
//! Each SSE event has the format:
//! ```text
//! data: {"choices":[{"delta":{"content":"Hello"}}]}
//! ```
//!
//! The stream ends with:
//! ```text
//! data: [DONE]
//! ```

use serde::Deserialize;

/// A chunk from the OpenAI streaming response.
///
/// The streaming API sends these as SSE events. Each chunk contains
/// partial content that should be appended to build the complete response.
#[derive(Debug, Deserialize)]
pub struct OpenAiChunk {
    /// The choices array (typically contains one element for streaming)
    pub choices: Vec<OpenAiChoice>,
}

/// A single choice in a streaming response.
#[derive(Debug, Deserialize)]
pub struct OpenAiChoice {
    /// The delta containing incremental content
    pub delta: OpenAiDelta,
}

/// The delta (incremental update) in a streaming chunk.
#[derive(Debug, Deserialize)]
pub struct OpenAiDelta {
    /// Partial content string, if present in this chunk.
    /// May be None for the initial chunk or role-only chunks.
    pub content: Option<String>,
}

/// Parse an SSE data payload into content text.
///
/// # Arguments
///
/// * `data` - The raw data string from the SSE event (after "data: " prefix)
///
/// # Returns
///
/// - `None` if this is the `[DONE]` signal or if no content is present
/// - `Some(content)` if content was successfully extracted
///
/// # Example
///
/// ```
/// use cherry2k_core::provider::sse::parse_sse_chunk;
///
/// // Normal content chunk
/// let content = parse_sse_chunk(r#"{"choices":[{"delta":{"content":"Hello"}}]}"#);
/// assert_eq!(content, Some("Hello".to_string()));
///
/// // End of stream signal
/// let done = parse_sse_chunk("[DONE]");
/// assert_eq!(done, None);
/// ```
pub fn parse_sse_chunk(data: &str) -> Option<String> {
    // Check for stream end signal
    if data == "[DONE]" {
        return None;
    }

    // Parse JSON
    match serde_json::from_str::<OpenAiChunk>(data) {
        Ok(chunk) => {
            // Extract content from first choice's delta
            chunk
                .choices
                .first()
                .and_then(|choice| choice.delta.content.clone())
        }
        Err(e) => {
            // Log parse errors but don't break the stream
            tracing::warn!("Failed to parse SSE chunk: {e} (data: {data})");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_done_signal() {
        assert_eq!(parse_sse_chunk("[DONE]"), None);
    }

    #[test]
    fn parse_content_chunk() {
        let data = r#"{"choices":[{"delta":{"content":"Hello"}}]}"#;
        assert_eq!(parse_sse_chunk(data), Some("Hello".to_string()));
    }

    #[test]
    fn parse_chunk_with_no_content() {
        // First chunk often has role but no content
        let data = r#"{"choices":[{"delta":{"role":"assistant"}}]}"#;
        assert_eq!(parse_sse_chunk(data), None);
    }

    #[test]
    fn parse_chunk_with_empty_choices() {
        let data = r#"{"choices":[]}"#;
        assert_eq!(parse_sse_chunk(data), None);
    }

    #[test]
    fn parse_invalid_json_returns_none() {
        assert_eq!(parse_sse_chunk("not json"), None);
    }

    #[test]
    fn parse_chunk_with_empty_content() {
        let data = r#"{"choices":[{"delta":{"content":""}}]}"#;
        assert_eq!(parse_sse_chunk(data), Some("".to_string()));
    }

    #[test]
    fn parse_chunk_with_multiline_content() {
        let data = r#"{"choices":[{"delta":{"content":"Hello\nWorld"}}]}"#;
        assert_eq!(parse_sse_chunk(data), Some("Hello\nWorld".to_string()));
    }
}
