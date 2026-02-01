//! System prompts for AI behavior configuration.
//!
//! Provides system prompt snippets that configure AI behavior for different
//! modes of operation.

/// System prompt snippet for command suggestion mode.
///
/// Instructs the AI to respond with bash code blocks when the user
/// wants a command executed. Includes explicit marker documentation.
///
/// This is appended to any existing system prompt.
pub const COMMAND_MODE_PROMPT: &str = r#"
You are a terminal assistant that helps with shell commands.

When the user wants to run a command or perform a shell action:
- Respond with the command in a bash code block like this:
```bash
command here
```
- Keep explanations brief, focus on the command
- If multiple steps needed, suggest one command at a time

When the user wants an explanation or information:
- Provide a clear, concise answer without code blocks
- Only include code blocks if demonstrating syntax

Explicit mode markers (user can force a mode):
- `!` at start or `/run` at start = always suggest a command
- `?` at end = always provide explanation, never suggest command
"#;

/// Get the command mode system prompt.
#[must_use]
pub fn command_mode_system_prompt() -> &'static str {
    COMMAND_MODE_PROMPT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_mode_prompt_not_empty() {
        assert!(!COMMAND_MODE_PROMPT.is_empty());
        assert!(!command_mode_system_prompt().is_empty());
    }

    #[test]
    fn command_mode_prompt_contains_bash_block() {
        assert!(COMMAND_MODE_PROMPT.contains("```bash"));
    }

    #[test]
    fn command_mode_prompt_documents_markers() {
        assert!(COMMAND_MODE_PROMPT.contains('!'));
        assert!(COMMAND_MODE_PROMPT.contains("/run"));
        assert!(COMMAND_MODE_PROMPT.contains('?'));
    }
}
