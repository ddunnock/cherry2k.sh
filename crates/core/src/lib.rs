//! Cherry2K Core Library
//!
//! This crate provides the core domain logic for Cherry2K, including:
//! - Error types for consistent error handling across the application
//! - Configuration loading with TOML file and environment variable support
//! - Provider abstractions for AI backends (coming in Phase 2)

pub mod config;
pub mod error;

pub use config::{
    AnthropicConfig, Config, GeneralConfig, OllamaConfig, OpenAiConfig, SafetyConfig, load_config,
};
pub use error::{CommandError, ConfigError, ProviderError, StorageError};
