//! Error types for Cherry2K
//!
//! This module defines all error types used throughout the Cherry2K application.
//! Errors are designed to be:
//! - Descriptive: Clear messages for debugging and logging
//! - Actionable: Users know what went wrong and how to fix it
//! - Typed: Different error categories for programmatic handling

use std::path::PathBuf;
use thiserror::Error;

/// Errors from AI provider operations
#[derive(Debug, Error)]
pub enum ProviderError {
    // TODO: Uncomment when reqwest is added in Phase 2
    // #[error("API request failed: {0}")]
    // RequestFailed(#[from] reqwest::Error),
    /// API request failed with a message
    #[error("API request failed: {0}")]
    RequestFailed(String),

    /// Invalid API key for a provider
    #[error("Invalid API key for {provider}")]
    InvalidApiKey {
        /// The provider that rejected the key
        provider: String,
    },

    /// Rate limited by the provider
    #[error("Rate limited by {provider}, retry after {retry_after_secs} seconds")]
    RateLimited {
        /// The provider that rate limited us
        provider: String,
        /// Seconds to wait before retrying
        retry_after_secs: u64,
    },

    /// Provider is unavailable
    #[error("Provider {provider} is unavailable: {reason}")]
    Unavailable {
        /// The unavailable provider
        provider: String,
        /// Reason for unavailability
        reason: String,
    },

    /// Response parsing failed
    #[error("Response parsing failed: {0}")]
    ParseError(String),

    /// Streaming was interrupted
    #[error("Streaming interrupted: {0}")]
    StreamInterrupted(String),
}

/// Errors from configuration loading
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found at {path}")]
    NotFound {
        /// Path where config was expected
        path: PathBuf,
    },

    /// Failed to read configuration file
    #[error("Failed to read configuration: {0}")]
    ReadError(#[from] std::io::Error),

    /// Invalid configuration format
    #[error("Invalid configuration format: {0}")]
    ParseError(String),

    /// Missing required configuration field
    #[error("Missing required configuration: {field}")]
    MissingField {
        /// The missing field name
        field: String,
    },

    /// Invalid value for a configuration field
    #[error("Invalid value for {field}: {reason}")]
    InvalidValue {
        /// The field with invalid value
        field: String,
        /// Why the value is invalid
        reason: String,
    },
}

/// Errors from storage operations
#[derive(Debug, Error)]
pub enum StorageError {
    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Migration failed
    #[error("Migration failed: {0}")]
    Migration(String),

    /// Conversation not found
    #[error("Conversation not found: {id}")]
    ConversationNotFound {
        /// The conversation ID that was not found
        id: String,
    },

    /// Session not found
    #[error("Session not found: {id}")]
    SessionNotFound {
        /// The session ID that was not found
        id: String,
    },
}

/// Errors from command execution (Phase 6)
#[derive(Debug, Error)]
pub enum CommandError {
    /// User denied command execution
    #[error("Command execution denied by user")]
    UserDenied,

    /// Command blocked for safety reasons
    #[error("Command blocked for safety: {reason}")]
    Blocked {
        /// Why the command was blocked
        reason: String,
    },

    /// Command execution failed
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    /// Command timed out
    #[error("Command timed out after {timeout_secs} seconds")]
    Timeout {
        /// Timeout duration in seconds
        timeout_secs: u64,
    },
}
