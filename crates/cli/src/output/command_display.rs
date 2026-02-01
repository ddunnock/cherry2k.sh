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
    if let Some(ctx) = context {
        if !ctx.is_empty() {
            println!();
            skin.print_text(ctx);
        }
    }

    // Format as markdown code block
    let markdown = format!("\n```bash\n{}\n```\n", command);
    skin.print_text(&markdown);
}
