//! OpenAI API provider implementation.
//!
//! This module implements the [`AiProvider`] trait for OpenAI's API, including
//! support for streaming responses via Server-Sent Events (SSE).
//!
//! # Configuration
//!
//! The provider is configured via [`OpenAiConfig`]:
//! - `api_key`: API key (required, from env var or config file)
//! - `base_url`: API base URL (default: `https://api.openai.com/v1`)
//! - `model`: Model to use (default: `gpt-4o`)
//!
//! # Example
//!
//! ```ignore
//! use cherry2k_core::{OpenAiConfig, OpenAiProvider, AiProvider, CompletionRequest, Message};
//!
//! let config = OpenAiConfig {
//!     api_key: Some("sk-...".to_string()),
//!     ..Default::default()
//! };
//!
//! let provider = OpenAiProvider::new(config);
//! provider.validate_config()?;
//!
//! let request = CompletionRequest::new()
//!     .with_message(Message::user("Hello!"));
//!
//! let stream = provider.complete(request).await?;
//! ```

use std::future::Future;

use async_stream::try_stream;
use futures::{Stream, StreamExt};
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt};
use serde::Serialize;

use super::sse::parse_sse_chunk;
use super::types::{CompletionRequest, Message};
use super::AiProvider;
use crate::config::OpenAiConfig;
use crate::error::{ConfigError, ProviderError};

/// OpenAI API provider.
///
/// Implements streaming completions using OpenAI's chat completions API.
/// Compatible with OpenAI API and OpenAI-compatible APIs (like Azure OpenAI).
pub struct OpenAiProvider {
    client: Client,
    config: OpenAiConfig,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider with the given configuration.
    ///
    /// Note: This does not validate the configuration. Call [`validate_config()`]
    /// before using the provider to ensure the configuration is valid.
    #[must_use]
    pub fn new(config: OpenAiConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
}

/// Request body for OpenAI chat completions API.
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

impl AiProvider for OpenAiProvider {
    fn complete(
        &self,
        request: CompletionRequest,
    ) -> impl Future<Output = Result<super::CompletionStream, ProviderError>> + Send {
        // Clone what we need for the async block
        let client = self.client.clone();
        let api_key = self.config.api_key.clone().unwrap_or_default();
        let base_url = self.config.base_url.clone();
        let model = request.model.unwrap_or_else(|| self.config.model.clone());

        async move {
            let url = format!("{}/chat/completions", base_url);

            let body = ChatCompletionRequest {
                model,
                messages: request.messages,
                stream: true,
                temperature: request.temperature,
                max_tokens: request.max_tokens,
            };

            // Build the request
            let request_builder = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body);

            // Create event source for SSE streaming
            let event_source = request_builder.eventsource().map_err(|e| {
                ProviderError::RequestFailed(format!("Failed to create event source: {e}"))
            })?;

            // Return a stream that processes SSE events
            let stream = create_completion_stream(event_source);
            Ok(Box::pin(stream) as super::CompletionStream)
        }
    }

    fn provider_id(&self) -> &'static str {
        "openai"
    }

    fn validate_config(&self) -> Result<(), ConfigError> {
        match &self.config.api_key {
            Some(key) if !key.is_empty() => Ok(()),
            _ => Err(ConfigError::MissingField {
                field: "openai.api_key".to_string(),
            }),
        }
    }

    fn health_check(&self) -> impl Future<Output = Result<(), ProviderError>> + Send {
        let client = self.client.clone();
        let base_url = self.config.base_url.clone();
        let api_key = self.config.api_key.clone().unwrap_or_default();

        async move {
            // Make a lightweight request to verify connectivity and auth
            // Using /models endpoint as a health check
            let url = format!("{}/models", base_url);

            let response = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|e| ProviderError::Unavailable {
                    provider: "openai".to_string(),
                    reason: e.to_string(),
                })?;

            match response.status().as_u16() {
                200..=299 => Ok(()),
                401 => Err(ProviderError::InvalidApiKey {
                    provider: "openai".to_string(),
                }),
                429 => {
                    let retry_after = response
                        .headers()
                        .get("Retry-After")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(60);
                    Err(ProviderError::RateLimited {
                        provider: "openai".to_string(),
                        retry_after_secs: retry_after,
                    })
                }
                500..=599 => Err(ProviderError::Unavailable {
                    provider: "openai".to_string(),
                    reason: "Server error".to_string(),
                }),
                status => Err(ProviderError::RequestFailed(format!(
                    "Unexpected status code: {status}"
                ))),
            }
        }
    }
}

/// Create a stream that processes SSE events and yields text chunks.
fn create_completion_stream(
    mut event_source: EventSource,
) -> impl Stream<Item = Result<String, ProviderError>> {
    try_stream! {
        loop {
            match event_source.next().await {
                Some(Ok(Event::Open)) => {
                    // Connection opened, continue to receive messages
                    tracing::debug!("SSE connection opened");
                }
                Some(Ok(Event::Message(message))) => {
                    // Parse the SSE data
                    if let Some(content) = parse_sse_chunk(&message.data) {
                        if !content.is_empty() {
                            yield content;
                        }
                    } else if message.data == "[DONE]" {
                        // Stream complete
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
                                provider: "openai".to_string(),
                            })?;
                        }
                        429 => {
                            // Try to parse retry-after from body or default to 60
                            Err(ProviderError::RateLimited {
                                provider: "openai".to_string(),
                                retry_after_secs: 60,
                            })?;
                        }
                        500..=599 => {
                            Err(ProviderError::Unavailable {
                                provider: "openai".to_string(),
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

// Implement From for reqwest_eventsource::Error to ProviderError
impl From<reqwest_eventsource::Error> for ProviderError {
    fn from(e: reqwest_eventsource::Error) -> Self {
        ProviderError::RequestFailed(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod config_validation {
        use super::*;

        #[test]
        fn valid_config_passes() {
            let config = OpenAiConfig {
                api_key: Some("sk-test123".to_string()),
                ..Default::default()
            };
            let provider = OpenAiProvider::new(config);
            assert!(provider.validate_config().is_ok());
        }

        #[test]
        fn missing_api_key_fails() {
            let config = OpenAiConfig {
                api_key: None,
                ..Default::default()
            };
            let provider = OpenAiProvider::new(config);
            let result = provider.validate_config();
            assert!(matches!(result, Err(ConfigError::MissingField { .. })));
        }

        #[test]
        fn empty_api_key_fails() {
            let config = OpenAiConfig {
                api_key: Some("".to_string()),
                ..Default::default()
            };
            let provider = OpenAiProvider::new(config);
            let result = provider.validate_config();
            assert!(matches!(result, Err(ConfigError::MissingField { .. })));
        }
    }

    mod provider_id {
        use super::*;

        #[test]
        fn returns_openai() {
            let provider = OpenAiProvider::new(OpenAiConfig::default());
            assert_eq!(provider.provider_id(), "openai");
        }
    }

    mod provider_traits {
        use super::*;

        fn assert_send_sync<T: Send + Sync>() {}

        #[test]
        fn openai_provider_is_send_sync() {
            assert_send_sync::<OpenAiProvider>();
        }
    }
}
