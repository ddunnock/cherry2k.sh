//! Cherry2K CLI Library
//!
//! Provides command execution, output formatting, and signal handling
//! for the Cherry2K terminal AI assistant.
//!
//! # Modules
//!
//! - [`execute`] - Async command execution with streaming output
//! - [`output`] - Terminal output formatting (markdown, spinner, streaming)
//! - [`signal`] - Ctrl+C signal handling with confirmation
//! - [`intent`] - AI response intent detection

pub mod execute;
pub mod intent;
pub mod output;
pub mod signal;
