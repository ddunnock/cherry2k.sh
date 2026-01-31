//! Anthropic Claude API provider implementation.
//!
//! This module implements the [`AiProvider`] trait for Anthropic's Claude API,
//! including support for streaming responses via Server-Sent Events (SSE).
//!
//! # Configuration
//!
//! The provider is configured via [`AnthropicConfig`]:
//! - `api_key`: API key (required, from env var or config file)
//! - `model`: Model to use (default: `claude-sonnet-4-20250514`)
//!
//! # Example
//!
//! ```ignore
//! use cherry2k_core::{AnthropicConfig, AnthropicProvider, AiProvider, CompletionRequest, Message};
//!
//! let config = AnthropicConfig {
//!     api_key: Some("sk-ant-...".to_string()),
//!     ..Default::default()
//! };
//!
//! let provider = AnthropicProvider::new(config);
//! provider.validate_config()?;
//!
//! let request = CompletionRequest::new()
//!     .with_message(Message::user("Hello!"));
//!
//! let stream = provider.complete(request).await?;
//! ```

use async_stream::try_stream;
use futures::future::BoxFuture;
use futures::{Stream, StreamExt};
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt};
use serde::{Deserialize, Serialize};

use super::AiProvider;
use super::types::{CompletionRequest, Message, Role};
use crate::config::AnthropicConfig;
use crate::error::{ConfigError, ProviderError};

/// Anthropic API version header value.
/// Required by Anthropic API - requests will 400 without it.
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Base URL for Anthropic API.
const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1";

/// Default max tokens for Anthropic requests.
/// Anthropic requires explicit max_tokens in requests.
const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Anthropic Claude API provider.
///
/// Implements streaming completions using Anthropic's messages API.
pub struct AnthropicProvider {
    client: Client,
    config: AnthropicConfig,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with the given configuration.
    ///
    /// Note: This does not validate the configuration. Call [`validate_config()`]
    /// before using the provider to ensure the configuration is valid.
    #[must_use]
    pub fn new(config: AnthropicConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
}

/// Request body for Anthropic messages API.
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// A message in Anthropic's format.
/// Note: Anthropic uses separate system parameter, so we only send user/assistant here.
#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

impl AiProvider for AnthropicProvider {
    fn complete(
        &self,
        request: CompletionRequest,
    ) -> BoxFuture<'_, Result<super::CompletionStream, ProviderError>> {
        // Clone what we need for the async block
        let client = self.client.clone();
        let api_key = self.config.api_key.clone().unwrap_or_default();
        let model = request.model.unwrap_or_else(|| self.config.model.clone());

        Box::pin(async move {
            let url = format!("{}/messages", ANTHROPIC_API_BASE);

            // Separate system messages from conversation messages
            let (system, messages) = convert_messages(request.messages);

            let body = AnthropicRequest {
                model,
                max_tokens: request.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
                messages,
                stream: true,
                system,
                temperature: request.temperature,
            };

            // Build the request with Anthropic-specific headers
            let request_builder = client
                .post(&url)
                .header("x-api-key", &api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .header("Content-Type", "application/json")
                .json(&body);

            // Create event source for SSE streaming
            let event_source = request_builder.eventsource().map_err(|e| {
                ProviderError::RequestFailed(format!("Failed to create event source: {e}"))
            })?;

            // Return a stream that processes SSE events
            let stream = create_anthropic_stream(event_source);
            Ok(Box::pin(stream) as super::CompletionStream)
        })
    }

    fn provider_id(&self) -> &'static str {
        "anthropic"
    }

    fn validate_config(&self) -> Result<(), ConfigError> {
        match &self.config.api_key {
            Some(key) if !key.is_empty() => Ok(()),
            _ => Err(ConfigError::MissingField {
                field: "anthropic.api_key".to_string(),
            }),
        }
    }

    fn health_check(&self) -> BoxFuture<'_, Result<(), ProviderError>> {
        let client = self.client.clone();
        let api_key = self.config.api_key.clone().unwrap_or_default();

        Box::pin(async move {
            // Make a lightweight request to verify connectivity and auth
            // Using /models endpoint as a health check
            let url = format!("{}/models", ANTHROPIC_API_BASE);

            let response = client
                .get(&url)
                .header("x-api-key", &api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .send()
                .await
                .map_err(|e| ProviderError::Unavailable {
                    provider: "anthropic".to_string(),
                    reason: e.to_string(),
                })?;

            match response.status().as_u16() {
                200..=299 => Ok(()),
                401 => Err(ProviderError::InvalidApiKey {
                    provider: "anthropic".to_string(),
                }),
                429 => {
                    let retry_after = response
                        .headers()
                        .get("Retry-After")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(60);
                    Err(ProviderError::RateLimited {
                        provider: "anthropic".to_string(),
                        retry_after_secs: retry_after,
                    })
                }
                500..=599 => Err(ProviderError::Unavailable {
                    provider: "anthropic".to_string(),
                    reason: "Server error".to_string(),
                }),
                status => Err(ProviderError::RequestFailed(format!(
                    "Unexpected status code: {status}"
                ))),
            }
        })
    }
}

/// Convert our messages to Anthropic format.
/// Returns (system_message, conversation_messages).
/// Anthropic requires system messages to be passed separately.
fn convert_messages(messages: Vec<Message>) -> (Option<String>, Vec<AnthropicMessage>) {
    let mut system = None;
    let mut conversation = Vec::new();

    for msg in messages {
        match msg.role {
            Role::System => {
                // Anthropic takes a single system parameter
                // If multiple system messages, concatenate them
                if let Some(existing) = system.take() {
                    system = Some(format!("{}\n\n{}", existing, msg.content));
                } else {
                    system = Some(msg.content);
                }
            }
            Role::User => {
                conversation.push(AnthropicMessage {
                    role: "user".to_string(),
                    content: msg.content,
                });
            }
            Role::Assistant => {
                conversation.push(AnthropicMessage {
                    role: "assistant".to_string(),
                    content: msg.content,
                });
            }
        }
    }

    (system, conversation)
}

/// Anthropic SSE event types we care about.
#[derive(Debug, Deserialize)]
struct AnthropicSseEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<AnthropicDelta>,
}

/// Delta content in Anthropic streaming response.
#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: Option<String>,
    text: Option<String>,
}

/// Parse an Anthropic SSE chunk and extract text content.
///
/// Anthropic SSE events have a different structure than OpenAI:
/// - `content_block_delta` event type contains text in `delta.text`
/// - `message_stop` signals end of stream
/// - Other events (message_start, content_block_start, etc.) should be ignored
fn parse_anthropic_sse_chunk(data: &str) -> Option<String> {
    // Parse the JSON event
    let event: AnthropicSseEvent = match serde_json::from_str(data) {
        Ok(e) => e,
        Err(err) => {
            tracing::warn!("Failed to parse Anthropic SSE chunk: {err} (data: {data})");
            return None;
        }
    };

    // Only content_block_delta events contain text
    if event.event_type == "content_block_delta"
        && let Some(delta) = event.delta
        && delta.delta_type.as_deref() == Some("text_delta")
    {
        return delta.text;
    }

    None
}

/// Create a stream that processes Anthropic SSE events and yields text chunks.
fn create_anthropic_stream(
    mut event_source: EventSource,
) -> impl Stream<Item = Result<String, ProviderError>> {
    try_stream! {
        loop {
            match event_source.next().await {
                Some(Ok(Event::Open)) => {
                    // Connection opened, continue to receive messages
                    tracing::debug!("Anthropic SSE connection opened");
                }
                Some(Ok(Event::Message(message))) => {
                    // Parse the SSE data
                    if let Some(content) = parse_anthropic_sse_chunk(&message.data)
                        && !content.is_empty()
                    {
                        yield content;
                    }
                    // Check for message_stop event
                    if let Ok(event) = serde_json::from_str::<AnthropicSseEvent>(&message.data)
                        && event.event_type == "message_stop"
                    {
                        break;
                    }
                }
                Some(Err(reqwest_eventsource::Error::StreamEnded)) => {
                    // Normal end of stream
                    break;
                }
                Some(Err(reqwest_eventsource::Error::InvalidStatusCode(status, response))) => {
                    // Handle HTTP error status codes
                    let status_code = status.as_u16();
                    let body = response.text().await.unwrap_or_default();

                    match status_code {
                        401 => {
                            Err(ProviderError::InvalidApiKey {
                                provider: "anthropic".to_string(),
                            })?;
                        }
                        429 => {
                            Err(ProviderError::RateLimited {
                                provider: "anthropic".to_string(),
                                retry_after_secs: 60,
                            })?;
                        }
                        500..=599 => {
                            Err(ProviderError::Unavailable {
                                provider: "anthropic".to_string(),
                                reason: body,
                            })?;
                        }
                        _ => {
                            Err(ProviderError::RequestFailed(format!(
                                "HTTP {status_code}: {body}"
                            )))?;
                        }
                    }
                }
                Some(Err(e)) => {
                    Err(ProviderError::StreamInterrupted(e.to_string()))?;
                }
                None => {
                    // Stream ended
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod config_validation {
        use super::*;

        #[test]
        fn valid_config_passes() {
            let config = AnthropicConfig {
                api_key: Some("sk-ant-test123".to_string()),
                ..Default::default()
            };
            let provider = AnthropicProvider::new(config);
            assert!(provider.validate_config().is_ok());
        }

        #[test]
        fn missing_api_key_fails() {
            let config = AnthropicConfig {
                api_key: None,
                ..Default::default()
            };
            let provider = AnthropicProvider::new(config);
            let result = provider.validate_config();
            assert!(matches!(result, Err(ConfigError::MissingField { .. })));
        }

        #[test]
        fn empty_api_key_fails() {
            let config = AnthropicConfig {
                api_key: Some("".to_string()),
                ..Default::default()
            };
            let provider = AnthropicProvider::new(config);
            let result = provider.validate_config();
            assert!(matches!(result, Err(ConfigError::MissingField { .. })));
        }
    }

    mod provider_id {
        use super::*;

        #[test]
        fn returns_anthropic() {
            let provider = AnthropicProvider::new(AnthropicConfig::default());
            assert_eq!(provider.provider_id(), "anthropic");
        }
    }

    mod provider_traits {
        use super::*;

        fn assert_send_sync<T: Send + Sync>() {}

        #[test]
        fn provider_is_send_sync() {
            assert_send_sync::<AnthropicProvider>();
        }
    }

    mod message_conversion {
        use super::*;

        #[test]
        fn separates_system_messages() {
            let messages = vec![
                Message::system("You are helpful"),
                Message::user("Hello"),
            ];

            let (system, conversation) = convert_messages(messages);

            assert_eq!(system, Some("You are helpful".to_string()));
            assert_eq!(conversation.len(), 1);
            assert_eq!(conversation[0].role, "user");
            assert_eq!(conversation[0].content, "Hello");
        }

        #[test]
        fn concatenates_multiple_system_messages() {
            let messages = vec![
                Message::system("First instruction"),
                Message::system("Second instruction"),
                Message::user("Hello"),
            ];

            let (system, conversation) = convert_messages(messages);

            assert_eq!(
                system,
                Some("First instruction\n\nSecond instruction".to_string())
            );
            assert_eq!(conversation.len(), 1);
        }

        #[test]
        fn handles_no_system_message() {
            let messages = vec![
                Message::user("Hello"),
                Message::assistant("Hi there!"),
            ];

            let (system, conversation) = convert_messages(messages);

            assert_eq!(system, None);
            assert_eq!(conversation.len(), 2);
            assert_eq!(conversation[0].role, "user");
            assert_eq!(conversation[1].role, "assistant");
        }
    }

    mod sse_parsing {
        use super::*;

        #[test]
        fn parses_content_block_delta() {
            let data = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
            assert_eq!(parse_anthropic_sse_chunk(data), Some("Hello".to_string()));
        }

        #[test]
        fn ignores_message_start() {
            let data = r#"{"type":"message_start","message":{"id":"msg_01"}}"#;
            assert_eq!(parse_anthropic_sse_chunk(data), None);
        }

        #[test]
        fn ignores_message_stop() {
            let data = r#"{"type":"message_stop"}"#;
            assert_eq!(parse_anthropic_sse_chunk(data), None);
        }

        #[test]
        fn ignores_content_block_start() {
            let data = r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#;
            assert_eq!(parse_anthropic_sse_chunk(data), None);
        }

        #[test]
        fn handles_invalid_json() {
            assert_eq!(parse_anthropic_sse_chunk("not json"), None);
        }

        #[test]
        fn handles_empty_text() {
            let data = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":""}}"#;
            assert_eq!(parse_anthropic_sse_chunk(data), Some("".to_string()));
        }
    }
}
