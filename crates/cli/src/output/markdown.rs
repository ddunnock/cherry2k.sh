//! Terminal markdown rendering
//!
//! Renders markdown text with terminal formatting using termimad.
//! Supports a plain mode for environments without color support or
//! when piping output.

use termimad::{MadSkin, StyledChar};

/// Render markdown text for terminal display.
///
/// Converts markdown syntax to terminal-formatted text with colors
/// and styling. When `plain` is true, returns the text unmodified
/// for use in pipes or terminals without color support.
///
/// # Arguments
///
/// * `text` - The markdown text to render
/// * `plain` - If true, return text as-is without formatting
///
/// # Returns
///
/// Formatted text suitable for terminal output.
///
/// # Example
///
/// ```
/// use cherry2k::output::render_markdown;
///
/// let formatted = render_markdown("**bold** and *italic*", false);
/// let plain = render_markdown("**bold** and *italic*", true);
/// assert_eq!(plain, "**bold** and *italic*");
/// ```
#[must_use]
pub fn render_markdown(text: &str, plain: bool) -> String {
    if plain {
        return text.to_string();
    }

    let skin = create_skin();
    skin.term_text(text).to_string()
}

/// Create a customized MadSkin for terminal rendering.
fn create_skin() -> MadSkin {
    let mut skin = MadSkin::default();

    // Customize colors for better terminal visibility
    // Bold: Yellow for emphasis
    skin.bold.set_fg(termimad::crossterm::style::Color::Yellow);

    // Italic: Cyan for subtle emphasis
    skin.italic.set_fg(termimad::crossterm::style::Color::Cyan);

    // Inline code: Green on default background
    skin.inline_code
        .set_fg(termimad::crossterm::style::Color::Green);

    // Code blocks: Green text
    skin.code_block
        .set_fg(termimad::crossterm::style::Color::Green);

    // Headers: Bold yellow
    skin.headers[0].set_fg(termimad::crossterm::style::Color::Yellow);
    skin.headers[1].set_fg(termimad::crossterm::style::Color::Yellow);

    // Bullet points: Use a nice character
    skin.bullet = StyledChar::from_fg_char(termimad::crossterm::style::Color::Cyan, '*');

    skin
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_mode_returns_unchanged() {
        let text = "**bold** and *italic*";
        let result = render_markdown(text, true);
        assert_eq!(result, text);
    }

    #[test]
    fn formatted_mode_produces_output() {
        let text = "Hello **world**";
        let result = render_markdown(text, false);
        // Result should have some content (may include ANSI codes)
        assert!(!result.is_empty());
    }

    #[test]
    fn handles_empty_string() {
        assert_eq!(render_markdown("", true), "");
        // Formatted empty string might have some whitespace
        let formatted = render_markdown("", false);
        assert!(formatted.len() <= 2); // May have newline
    }

    #[test]
    fn handles_code_blocks() {
        let text = "```rust\nlet x = 1;\n```";
        let result = render_markdown(text, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn handles_lists() {
        let text = "- item 1\n- item 2";
        let result = render_markdown(text, false);
        assert!(!result.is_empty());
    }
}
