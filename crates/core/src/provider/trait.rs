//! AI Provider trait definition.
//!
//! This module defines the core [`AiProvider`] trait that all AI backend implementations
//! must satisfy. The trait is designed to be:
//! - **Streaming-first**: All completions return a stream of chunks
//! - **Provider-agnostic**: Works with OpenAI, Anthropic, Ollama, etc.
//! - **Validation-explicit**: Constructors succeed, callers decide when to validate
//!
//! # Design Decisions
//!
//! - **Single `complete()` method**: Returns a stream; non-streaming callers collect it
//! - **Explicit `validate_config()`**: Separates construction from validation
//! - **`health_check()` for reachability**: Async ping to confirm provider is available
//! - **Native async**: Uses Rust 1.75+ async traits, no `async-trait` crate

use std::future::Future;
use std::pin::Pin;

use futures::Stream;

use super::types::CompletionRequest;
use crate::error::{ConfigError, ProviderError};

/// A stream of completion chunks from an AI provider.
///
/// Each item in the stream is either:
/// - `Ok(String)`: A text chunk (may be partial token/word)
/// - `Err(ProviderError)`: An error that terminated the stream
///
/// Consumers should collect all `Ok` chunks to build the complete response.
/// The stream ends when:
/// - Provider signals completion (clean end)
/// - An error occurs (error variant returned, stream ends)
///
/// # Example
///
/// ```ignore
/// use futures::StreamExt;
///
/// let mut stream = provider.complete(request).await?;
/// let mut response = String::new();
///
/// while let Some(chunk) = stream.next().await {
///     match chunk {
///         Ok(text) => response.push_str(&text),
///         Err(e) => return Err(e),
///     }
/// }
/// ```
pub type CompletionStream = Pin<Box<dyn Stream<Item = Result<String, ProviderError>> + Send>>;

/// Core trait for AI provider implementations.
///
/// All AI backends (OpenAI, Anthropic, Ollama, etc.) implement this trait to provide
/// a unified interface for the Cherry2K application.
///
/// # Implementation Notes
///
/// - Implementors MUST be `Send + Sync` for use across async tasks
/// - The `complete()` method should handle rate limiting internally when possible
/// - `validate_config()` should check API key format, endpoint URLs, etc.
/// - `health_check()` should make a lightweight API call to verify connectivity
///
/// # Example Implementation
///
/// ```ignore
/// struct MyProvider {
///     api_key: String,
///     base_url: String,
/// }
///
/// impl AiProvider for MyProvider {
///     fn complete(&self, request: CompletionRequest)
///         -> impl Future<Output = Result<CompletionStream, ProviderError>> + Send
///     {
///         async move {
///             // Make streaming API request
///             // Return stream of chunks
///         }
///     }
///
///     fn provider_id(&self) -> &'static str {
///         "my-provider"
///     }
///
///     fn validate_config(&self) -> Result<(), ConfigError> {
///         if self.api_key.is_empty() {
///             return Err(ConfigError::MissingField { field: "api_key".into() });
///         }
///         Ok(())
///     }
///
///     fn health_check(&self)
///         -> impl Future<Output = Result<(), ProviderError>> + Send
///     {
///         async move {
///             // Ping API to check connectivity
///             Ok(())
///         }
///     }
/// }
/// ```
pub trait AiProvider: Send + Sync {
    /// Sends a completion request and returns a stream of response chunks.
    ///
    /// This is the primary method for interacting with the AI provider.
    /// All responses are streamed, even if the underlying API doesn't support
    /// streaming natively (in which case a single-item stream is returned).
    ///
    /// # Arguments
    ///
    /// * `request` - The completion request containing messages and parameters
    ///
    /// # Returns
    ///
    /// A stream of text chunks on success, or an error if the request fails
    /// before streaming begins.
    ///
    /// # Errors
    ///
    /// - [`ProviderError::InvalidApiKey`]: API key is invalid or expired
    /// - [`ProviderError::RateLimited`]: Too many requests
    /// - [`ProviderError::RequestFailed`]: Network or API error
    /// - [`ProviderError::Unavailable`]: Provider is down or unreachable
    fn complete(
        &self,
        request: CompletionRequest,
    ) -> impl Future<Output = Result<CompletionStream, ProviderError>> + Send;

    /// Returns the unique identifier for this provider.
    ///
    /// Used for logging, configuration keys, and error messages.
    /// Should be a lowercase, hyphen-separated string (e.g., "openai", "anthropic", "ollama").
    fn provider_id(&self) -> &'static str;

    /// Validates the provider's configuration.
    ///
    /// This method checks that all required configuration is present and valid
    /// without making any network requests. Examples of what to validate:
    /// - API key format (not empty, correct prefix)
    /// - Endpoint URL format (valid URL)
    /// - Required fields are present
    ///
    /// # Design Note
    ///
    /// Constructors should succeed even with invalid config; this method
    /// allows callers to explicitly validate when appropriate. This supports:
    /// - Lazy validation (only when provider is actually used)
    /// - Configuration checking without side effects
    /// - Better error messages (can check all fields, not just first failure)
    ///
    /// # Errors
    ///
    /// - [`ConfigError::MissingField`]: Required configuration is missing
    /// - [`ConfigError::InvalidValue`]: Configuration value is malformed
    fn validate_config(&self) -> Result<(), ConfigError>;

    /// Performs a health check to verify the provider is reachable.
    ///
    /// Makes a lightweight API call to confirm:
    /// - Network connectivity
    /// - API endpoint is responding
    /// - Authentication is working
    ///
    /// This is useful for:
    /// - Startup validation
    /// - Provider selection (choose first healthy provider)
    /// - Monitoring and diagnostics
    ///
    /// # Errors
    ///
    /// - [`ProviderError::InvalidApiKey`]: Authentication failed
    /// - [`ProviderError::Unavailable`]: Provider is down
    /// - [`ProviderError::RequestFailed`]: Network error
    fn health_check(&self) -> impl Future<Output = Result<(), ProviderError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test that types are properly bounded
    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn completion_stream_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<CompletionStream>();
    }

    // A mock provider for testing trait bounds
    struct MockProvider;

    impl AiProvider for MockProvider {
        async fn complete(
            &self,
            _request: CompletionRequest,
        ) -> Result<CompletionStream, ProviderError> {
            let stream = futures::stream::empty();
            Ok(Box::pin(stream) as CompletionStream)
        }

        fn provider_id(&self) -> &'static str {
            "mock"
        }

        fn validate_config(&self) -> Result<(), ConfigError> {
            Ok(())
        }

        async fn health_check(&self) -> Result<(), ProviderError> {
            Ok(())
        }
    }

    #[test]
    fn mock_provider_is_send_sync() {
        assert_send_sync::<MockProvider>();
    }

    #[test]
    fn mock_provider_id() {
        let provider = MockProvider;
        assert_eq!(provider.provider_id(), "mock");
    }

    #[test]
    fn mock_validate_config() {
        let provider = MockProvider;
        assert!(provider.validate_config().is_ok());
    }

    #[tokio::test]
    async fn mock_health_check() {
        let provider = MockProvider;
        assert!(provider.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn mock_complete_returns_empty_stream() {
        use futures::StreamExt;

        let provider = MockProvider;
        let request = CompletionRequest::default();
        let mut stream = provider.complete(request).await.unwrap();

        // Empty stream should return None immediately
        assert!(stream.next().await.is_none());
    }
}
