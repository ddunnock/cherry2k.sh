//! Intent detection module
//!
//! Detects whether AI responses contain command suggestions (bash code blocks)
//! or explanatory answers.

mod detector;
mod types;

pub use detector::{detect_intent, parse_command_from_response};
pub use types::{DetectedCommand, Intent};
