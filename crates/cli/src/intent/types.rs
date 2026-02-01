//! Intent detection types
//!
//! Types for distinguishing AI responses that suggest commands from explanatory answers.

use crate::files::FileProposal;

/// Detected intent from AI response
#[derive(Debug, Clone)]
pub enum Intent {
    /// AI provided an explanatory answer (no command to run)
    Question,
    /// AI suggested a command to execute
    Command(DetectedCommand),
    /// AI proposed file write operations
    FileOperation(Vec<FileProposal>),
}

/// A command detected in AI response
#[derive(Debug, Clone)]
pub struct DetectedCommand {
    /// The command string extracted from code block
    pub command: String,
    /// Any explanation text before the code block
    pub context: Option<String>,
}

impl DetectedCommand {
    /// Create a new detected command with optional context.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            context: None,
        }
    }

    /// Create a new detected command with context.
    pub fn with_context(command: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            context: Some(context.into()),
        }
    }
}
