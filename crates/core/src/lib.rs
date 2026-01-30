//! Cherry2K Core Library
//!
//! This crate provides the core domain logic for Cherry2K, including:
//! - Error types for consistent error handling across the application
//! - Configuration loading with TOML file and environment variable support
//! - Provider abstractions for AI backends
//!
//! # Provider Architecture
//!
//! The [`provider`] module provides a unified interface for AI backends:
//!
//! ```ignore
//! use cherry2k_core::provider::{AiProvider, CompletionRequest, Message};
//!
//! async fn chat(provider: &impl AiProvider) {
//!     let request = CompletionRequest::new()
//!         .with_message(Message::user("Hello!"));
//!
//!     let stream = provider.complete(request).await.unwrap();
//!     // Process streaming response...
//! }
//! ```

pub mod config;
pub mod error;
pub mod provider;

pub use config::{
    AnthropicConfig, Config, GeneralConfig, OllamaConfig, OpenAiConfig, SafetyConfig, load_config,
};
pub use error::{CommandError, ConfigError, ProviderError, StorageError};
pub use provider::{AiProvider, CompletionRequest, CompletionStream, Message, Role};
