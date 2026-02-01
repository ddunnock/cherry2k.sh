//! Cherry2K CLI Library
//!
//! Provides command execution, output formatting, and signal handling
//! for the Cherry2K terminal AI assistant.
//!
//! # Modules
//!
//! - [`confirm`] - User confirmation prompts for safety-critical operations
//! - [`execute`] - Async command execution with streaming output
//! - [`files`] - File detection and safe reading
//! - [`intent`] - AI response intent detection
//! - [`output`] - Terminal output formatting (markdown, spinner, streaming)
//! - [`signal`] - Ctrl+C signal handling with confirmation

pub mod confirm;
pub mod execute;
pub mod files;
pub mod intent;
pub mod output;
pub mod signal;
