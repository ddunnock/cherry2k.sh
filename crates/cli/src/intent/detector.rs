//! Intent detection from AI responses
//!
//! Parses AI responses to detect command suggestions in bash/sh/shell code blocks.

use regex::Regex;
use std::sync::LazyLock;

use super::types::{DetectedCommand, Intent};

/// Regex pattern for bash/sh/shell code blocks.
/// Captures the content between ```bash/sh/shell and ```.
static CODE_BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"```(?:bash|sh|shell)\n([\s\S]*?)\n```").expect("valid regex")
});

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
pub fn parse_command_from_response(response: &str) -> Option<DetectedCommand> {
    let captures = CODE_BLOCK_RE.captures(response)?;
    let command = captures.get(1)?.as_str().trim();

    // Empty code blocks don't count as commands
    if command.is_empty() {
        return None;
    }

    // Find text before the code block (context)
    let match_start = captures.get(0)?.start();
    let context = if match_start > 0 {
        let before = response[..match_start].trim();
        if before.is_empty() {
            None
        } else {
            Some(before.to_string())
        }
    } else {
        None
    };

    Some(DetectedCommand {
        command: command.to_string(),
        context,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_code_block_returns_command() {
        let response = "Here's how to list files:\n```bash\nls -la\n```";
        let intent = detect_intent(response);
        match intent {
            Intent::Command(cmd) => {
                assert_eq!(cmd.command, "ls -la");
            }
            Intent::Question => panic!("Expected Command intent"),
        }
    }

    #[test]
    fn sh_code_block_returns_command() {
        let response = "```sh\necho hello\n```";
        let intent = detect_intent(response);
        match intent {
            Intent::Command(cmd) => {
                assert_eq!(cmd.command, "echo hello");
            }
            Intent::Question => panic!("Expected Command intent"),
        }
    }

    #[test]
    fn shell_code_block_returns_command() {
        let response = "```shell\npwd\n```";
        let intent = detect_intent(response);
        match intent {
            Intent::Command(cmd) => {
                assert_eq!(cmd.command, "pwd");
            }
            Intent::Question => panic!("Expected Command intent"),
        }
    }

    #[test]
    fn no_code_block_returns_question() {
        let response = "To list files, you can use the ls command with the -la flags.";
        let intent = detect_intent(response);
        assert!(matches!(intent, Intent::Question));
    }

    #[test]
    fn python_code_block_returns_question() {
        let response = "```python\nprint('hello')\n```";
        let intent = detect_intent(response);
        assert!(matches!(intent, Intent::Question));
    }

    #[test]
    fn javascript_code_block_returns_question() {
        let response = "```js\nconsole.log('hello');\n```";
        let intent = detect_intent(response);
        assert!(matches!(intent, Intent::Question));
    }

    #[test]
    fn multiline_command_captured_correctly() {
        let response = "```bash\nfor f in *.txt; do\n  echo $f\ndone\n```";
        let intent = detect_intent(response);
        match intent {
            Intent::Command(cmd) => {
                assert!(cmd.command.contains("for f in"));
                assert!(cmd.command.contains("echo $f"));
                assert!(cmd.command.contains("done"));
            }
            Intent::Question => panic!("Expected Command intent"),
        }
    }

    #[test]
    fn context_before_code_block_captured() {
        let response = "Here's the command to list files:\n```bash\nls -la\n```";
        let intent = detect_intent(response);
        match intent {
            Intent::Command(cmd) => {
                assert_eq!(cmd.command, "ls -la");
                assert!(cmd.context.is_some());
                assert!(cmd.context.as_ref().unwrap().contains("list files"));
            }
            Intent::Question => panic!("Expected Command intent"),
        }
    }

    #[test]
    fn empty_code_block_returns_question() {
        let response = "```bash\n\n```";
        let intent = detect_intent(response);
        assert!(matches!(intent, Intent::Question));
    }

    #[test]
    fn whitespace_only_code_block_returns_question() {
        let response = "```bash\n   \n```";
        let intent = detect_intent(response);
        assert!(matches!(intent, Intent::Question));
    }

    #[test]
    fn first_bash_block_is_used() {
        let response = "First:\n```bash\necho first\n```\nSecond:\n```bash\necho second\n```";
        let intent = detect_intent(response);
        match intent {
            Intent::Command(cmd) => {
                assert_eq!(cmd.command, "echo first");
            }
            Intent::Question => panic!("Expected Command intent"),
        }
    }

    #[test]
    fn no_context_when_code_block_at_start() {
        let response = "```bash\nls\n```";
        let intent = detect_intent(response);
        match intent {
            Intent::Command(cmd) => {
                assert_eq!(cmd.command, "ls");
                assert!(cmd.context.is_none());
            }
            Intent::Question => panic!("Expected Command intent"),
        }
    }
}
