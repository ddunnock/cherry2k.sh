//! Ollama local inference provider implementation.
//!
//! This module implements the [`AiProvider`] trait for Ollama, a local LLM runner.
//! Ollama uses newline-delimited JSON (NDJSON) for streaming, not Server-Sent Events.
//!
//! # Configuration
//!
//! The provider is configured via [`OllamaConfig`]:
//! - `host`: Ollama server URL (default: `http://localhost:11434`)
//! - `model`: Model to use (default: `llama3.2`)
//!
//! # No Authentication
//!
//! Ollama runs locally and doesn't require API keys.
//!
//! # Example
//!
//! ```ignore
//! use cherry2k_core::{OllamaConfig, OllamaProvider, AiProvider, CompletionRequest, Message};
//!
//! let config = OllamaConfig::default(); // localhost:11434
//!
//! let provider = OllamaProvider::new(config);
//! provider.validate_config()?;
//!
//! // Ensure Ollama is running: `ollama serve`
//! provider.health_check().await?;
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
use serde::Serialize;

use super::AiProvider;
use super::types::{CompletionRequest, Message};
use crate::config::OllamaConfig;
use crate::error::{ConfigError, ProviderError};

/// Ollama local inference provider.
///
/// Implements streaming completions using Ollama's chat API with NDJSON streaming.
/// Ollama runs locally, so no API key is required.
pub struct OllamaProvider {
    client: Client,
    config: OllamaConfig,
}

impl OllamaProvider {
    /// Create a new Ollama provider with the given configuration.
    ///
    /// Note: This does not validate the configuration or check if Ollama is running.
    /// Call [`validate_config()`] to check configuration and [`health_check()`]
    /// to verify Ollama is available.
    #[must_use]
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
}

/// Request body for Ollama chat API.
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

impl AiProvider for OllamaProvider {
    fn complete(
        &self,
        request: CompletionRequest,
    ) -> impl Future<Output = Result<super::CompletionStream, ProviderError>> + Send {
        // Clone what we need for the async block
        let client = self.client.clone();
        let host = self.config.host.clone();
        let model = request.model.unwrap_or_else(|| self.config.model.clone());

        async move {
            let url = format!("{}/api/chat", host);

            let body = OllamaChatRequest {
                model,
                messages: request.messages,
                stream: true,
            };

            // Make the request
            let response = client
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| {
                    if e.is_connect() {
                        ProviderError::Unavailable {
                            provider: "ollama".to_string(),
                            reason: "Ollama not running. Start with: ollama serve".to_string(),
                        }
                    } else {
                        ProviderError::RequestFailed(e.to_string())
                    }
                })?;

            // Check response status
            let status = response.status();
            if !status.is_success() {
                let status_code = status.as_u16();
                let body_text = response.text().await.unwrap_or_default();

                return match status_code {
                    404 => Err(ProviderError::RequestFailed(
                        "Model not found. Run: ollama pull <model>".to_string(),
                    )),
                    500..=599 => Err(ProviderError::Unavailable {
                        provider: "ollama".to_string(),
                        reason: body_text,
                    }),
                    _ => Err(ProviderError::RequestFailed(format!(
                        "HTTP {status_code}: {body_text}"
                    ))),
                };
            }

            // Return a stream that parses NDJSON
            let stream = parse_ollama_ndjson_stream(response);
            Ok(Box::pin(stream) as super::CompletionStream)
        }
    }

    fn provider_id(&self) -> &'static str {
        "ollama"
    }

    fn validate_config(&self) -> Result<(), ConfigError> {
        // Ollama doesn't need API key, but host must be non-empty
        if self.config.host.is_empty() {
            return Err(ConfigError::MissingField {
                field: "ollama.host".to_string(),
            });
        }
        Ok(())
    }

    fn health_check(&self) -> impl Future<Output = Result<(), ProviderError>> + Send {
        let client = self.client.clone();
        let host = self.config.host.clone();

        async move {
            // Use /api/version as a lightweight health check
            let url = format!("{}/api/version", host);

            let response = client.get(&url).send().await.map_err(|e| {
                if e.is_connect() {
                    ProviderError::Unavailable {
                        provider: "ollama".to_string(),
                        reason: "Ollama not running. Start with: ollama serve".to_string(),
                    }
                } else {
                    ProviderError::Unavailable {
                        provider: "ollama".to_string(),
                        reason: e.to_string(),
                    }
                }
            })?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(ProviderError::Unavailable {
                    provider: "ollama".to_string(),
                    reason: format!("Unexpected status: {}", response.status()),
                })
            }
        }
    }
}

/// Parse Ollama's NDJSON streaming response.
///
/// Ollama streams responses as newline-delimited JSON objects:
/// ```json
/// {"model":"llama3.2","message":{"role":"assistant","content":"Hello"},"done":false}
/// {"model":"llama3.2","message":{"role":"assistant","content":" there"},"done":false}
/// {"model":"llama3.2","message":{"role":"assistant","content":"!"},"done":true}
/// ```
///
/// Network chunks don't align with JSON line boundaries, so we buffer bytes
/// and parse complete lines as they arrive.
fn parse_ollama_ndjson_stream(
    response: reqwest::Response,
) -> impl Stream<Item = Result<String, ProviderError>> {
    try_stream! {
        let mut buffer = Vec::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| {
                ProviderError::StreamInterrupted(e.to_string())
            })?;
            buffer.extend_from_slice(&chunk);

            // Process complete lines (newline-delimited)
            while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                // Drain the line including newline
                let line_bytes: Vec<u8> = buffer.drain(..=newline_pos).collect();
                // Trim the newline for parsing
                let line_str = String::from_utf8_lossy(&line_bytes[..line_bytes.len() - 1]);

                if line_str.trim().is_empty() {
                    continue;
                }

                // Parse as JSON
                let json: serde_json::Value = serde_json::from_str(&line_str)
                    .map_err(|e| ProviderError::ParseError(format!(
                        "Invalid JSON from Ollama: {e}"
                    )))?;

                // Extract content from message.content
                if let Some(content) = json["message"]["content"].as_str()
                    && !content.is_empty()
                {
                    yield content.to_string();
                }

                // Check if stream is done
                if json["done"].as_bool() == Some(true) {
                    return;
                }
            }
        }

        // Process any remaining data in buffer (no trailing newline)
        if !buffer.is_empty() {
            let line_str = String::from_utf8_lossy(&buffer);
            if !line_str.trim().is_empty() {
                let json: serde_json::Value = serde_json::from_str(&line_str)
                    .map_err(|e| ProviderError::ParseError(format!(
                        "Invalid JSON from Ollama: {e}"
                    )))?;

                if let Some(content) = json["message"]["content"].as_str()
                    && !content.is_empty()
                {
                    yield content.to_string();
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
            let config = OllamaConfig::default();
            let provider = OllamaProvider::new(config);
            assert!(provider.validate_config().is_ok());
        }

        #[test]
        fn empty_host_fails() {
            let config = OllamaConfig {
                host: "".to_string(),
                model: "llama3.2".to_string(),
            };
            let provider = OllamaProvider::new(config);
            let result = provider.validate_config();
            assert!(matches!(result, Err(ConfigError::MissingField { .. })));
        }
    }

    mod provider_id {
        use super::*;

        #[test]
        fn returns_ollama() {
            let provider = OllamaProvider::new(OllamaConfig::default());
            assert_eq!(provider.provider_id(), "ollama");
        }
    }

    mod provider_traits {
        use super::*;

        fn assert_send_sync<T: Send + Sync>() {}

        #[test]
        fn ollama_provider_is_send_sync() {
            assert_send_sync::<OllamaProvider>();
        }
    }
}
