//! Intent detection from AI responses
//!
//! Parses AI responses to detect command suggestions in bash/sh/shell code blocks.

use super::types::{DetectedCommand, Intent};

/// Detect intent from an AI response.
///
/// Returns `Intent::Command` if the response contains a bash/sh/shell code block,
/// otherwise returns `Intent::Question`.
pub fn detect_intent(response: &str) -> Intent {
    match parse_command_from_response(response) {
        Some(cmd) => Intent::Command(cmd),
        None => Intent::Question,
    }
}

/// Parse a command from an AI response.
///
/// Looks for ```bash, ```sh, or ```shell code blocks and extracts
/// the command from the first matching block.
///
/// Returns `None` if no matching code block is found or if the code block is empty.
pub fn parse_command_from_response(_response: &str) -> Option<DetectedCommand> {
    // TODO: Implement in Task 2
    None
}
