//! Command display with syntax highlighting
//!
//! Displays suggested commands in a formatted code block with bash
//! syntax highlighting for improved readability.

use termimad::MadSkin;

/// Display a suggested command with bash syntax highlighting.
///
/// Renders the command in a formatted code block using termimad's
/// markdown rendering, which provides syntax highlighting for bash.
///
/// If context text is provided (explanation before the command),
/// it is displayed first.
///
/// # Arguments
/// * `command` - The command string to display
/// * `context` - Optional context/explanation text
pub fn display_suggested_command(command: &str, context: Option<&str>) {
    let skin = MadSkin::default();

    // Display context if provided
    if let Some(ctx) = context
        && !ctx.is_empty()
    {
        println!();
        skin.print_text(ctx);
    }

    // Format as markdown code block
    let markdown = format_command_markdown(command);
    skin.print_text(&markdown);
}

/// Format a command as a markdown code block.
///
/// This is separated for testability.
fn format_command_markdown(command: &str) -> String {
    format!("\n```bash\n{}\n```\n", command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_command_markdown_wraps_in_code_block() {
        let result = format_command_markdown("ls -la");
        assert!(result.contains("```bash"));
        assert!(result.contains("ls -la"));
        assert!(result.contains("```"));
    }

    #[test]
    fn format_command_markdown_preserves_multiline() {
        let cmd = "for f in *.txt; do\n  echo $f\ndone";
        let result = format_command_markdown(cmd);
        assert!(result.contains("for f in"));
        assert!(result.contains("echo $f"));
        assert!(result.contains("done"));
    }

    #[test]
    fn format_command_markdown_handles_special_chars() {
        let cmd = "echo 'hello $USER' && ls | grep test";
        let result = format_command_markdown(cmd);
        assert!(result.contains("'hello $USER'"));
        assert!(result.contains("grep test"));
    }
}
