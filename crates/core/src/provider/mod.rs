//! AI Provider abstraction layer.
//!
//! This module provides a unified interface for interacting with various AI backends.
//! The design is provider-agnostic, allowing Cherry2K to support multiple AI services
//! (OpenAI, Anthropic, Ollama) through a common trait.
//!
//! # Architecture
//!
//! ```text
//! +-----------------+
//! |   AiProvider    |  <-- Core trait (trait.rs)
//! +-----------------+
//!         |
//!    implements
//!         |
//! +-------+-------+-------+
//! |       |       |       |
//! v       v       v       v
//! OpenAI  Anthropic  Ollama  ...
//! ```
//!
//! # Key Types
//!
//! - [`AiProvider`]: The core trait all providers implement
//! - [`CompletionStream`]: Streaming response type
//! - [`CompletionRequest`]: Request configuration
//! - [`Message`]: A single conversation message
//! - [`Role`]: Message sender role (System, User, Assistant)
//!
//! # Example
//!
//! ```ignore
//! use cherry2k_core::provider::{AiProvider, CompletionRequest, Message};
//!
//! async fn chat(provider: &impl AiProvider) -> Result<String, ProviderError> {
//!     let request = CompletionRequest::new()
//!         .with_message(Message::system("You are a helpful assistant."))
//!         .with_message(Message::user("Hello!"));
//!
//!     let mut stream = provider.complete(request).await?;
//!     let mut response = String::new();
//!
//!     while let Some(chunk) = stream.next().await {
//!         response.push_str(&chunk?);
//!     }
//!
//!     Ok(response)
//! }
//! ```

mod r#trait;
mod types;

pub use r#trait::{AiProvider, CompletionStream};
pub use types::{CompletionRequest, Message, Role};
