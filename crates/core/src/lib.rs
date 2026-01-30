//! Cherry2K Core Library
//!
//! This crate provides the core domain logic for Cherry2K, including:
//! - Error types for consistent error handling across the application
//! - Provider abstractions for AI backends (coming in Phase 2)
//! - Configuration types (coming in Phase 1, Plan 02)

pub mod error;

pub use error::{CommandError, ConfigError, ProviderError, StorageError};
