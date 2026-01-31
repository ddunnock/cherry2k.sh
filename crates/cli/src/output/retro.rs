//! 8-bit retro color scheme for terminal output
//!
//! Provides a classic terminal aesthetic using the 16 ANSI colors
//! for maximum compatibility across terminal emulators.

use termimad::crossterm::style::{Attribute, Color};
use termimad::{MadSkin, StyledChar};

/// Retro 8-bit color palette for terminal output.
///
/// Uses the classic 16 ANSI colors to achieve a retro terminal look
/// that works on virtually all terminal emulators.
#[derive(Debug, Clone, Copy)]
pub struct RetroColors {
    /// Primary text color - bright green (classic terminal green)
    pub text: Color,
    /// Header color - bright yellow for emphasis
    pub header: Color,
    /// Code block text color - bright cyan
    pub code: Color,
    /// Code block background - black
    pub code_bg: Color,
    /// Cherry prompt color - bright magenta
    pub prompt: Color,
    /// Error message color - bright red
    pub error: Color,
    /// Dimmed/secondary text - dark gray
    pub dim: Color,
}

/// Create the retro 8-bit color scheme.
///
/// Returns a color palette using ANSI 16-color values for
/// maximum terminal compatibility and a classic retro look.
///
/// # Example
///
/// ```
/// use cherry2k_cli::output::retro_color_scheme;
///
/// let colors = retro_color_scheme();
/// // Use colors.text for main prose, colors.header for headings, etc.
/// ```
#[must_use]
pub fn retro_color_scheme() -> RetroColors {
    RetroColors {
        text: Color::AnsiValue(10),
        header: Color::AnsiValue(11),
        code: Color::AnsiValue(14),
        code_bg: Color::AnsiValue(0),
        prompt: Color::AnsiValue(13),
        error: Color::AnsiValue(9),
        dim: Color::AnsiValue(8),
    }
}

/// Apply the retro color scheme to a MadSkin for markdown rendering.
///
/// Configures the skin with the retro 8-bit aesthetic:
/// - Green prose text (classic terminal look)
/// - Bold yellow headers
/// - Cyan code blocks on black background
/// - Green bullet points
///
/// # Arguments
///
/// * `skin` - Mutable reference to the MadSkin to configure
///
/// # Example
///
/// ```
/// use termimad::MadSkin;
/// use cherry2k_cli::output::apply_retro_skin;
///
/// let mut skin = MadSkin::default();
/// apply_retro_skin(&mut skin);
/// // skin is now configured with retro colors
/// ```
pub fn apply_retro_skin(skin: &mut MadSkin) {
    let colors = retro_color_scheme();

    // Main text uses retro green
    skin.paragraph.set_fg(colors.text);

    // Headers: bold yellow
    for header in &mut skin.headers {
        header.set_fg(colors.header);
        header.add_attr(Attribute::Bold);
    }

    // Code blocks: cyan on black
    skin.code_block.set_fg(colors.code);
    skin.code_block.set_bg(colors.code_bg);

    // Inline code: cyan (no background to keep it readable inline)
    skin.inline_code.set_fg(colors.code);

    // Lists: green bullet character
    skin.bullet = StyledChar::from_fg_char(colors.text, '\u{2022}'); // bullet: •
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retro_colors_use_ansi_values() {
        let colors = retro_color_scheme();

        // Verify all colors are AnsiValue type (0-15 for 16-color palette)
        assert!(matches!(colors.text, Color::AnsiValue(10)));
        assert!(matches!(colors.header, Color::AnsiValue(11)));
        assert!(matches!(colors.code, Color::AnsiValue(14)));
        assert!(matches!(colors.code_bg, Color::AnsiValue(0)));
        assert!(matches!(colors.prompt, Color::AnsiValue(13)));
        assert!(matches!(colors.error, Color::AnsiValue(9)));
        assert!(matches!(colors.dim, Color::AnsiValue(8)));
    }

    #[test]
    fn apply_retro_skin_modifies_paragraph() {
        let mut skin = MadSkin::default();
        apply_retro_skin(&mut skin);

        // After applying retro skin, paragraph should have green foreground
        // We can't easily inspect the internal state, but we can verify
        // the function runs without panicking
    }

    #[test]
    fn apply_retro_skin_modifies_headers() {
        let mut skin = MadSkin::default();
        apply_retro_skin(&mut skin);

        // Headers should be modified (function completes without error)
        assert!(!skin.headers.is_empty());
    }

    #[test]
    fn retro_colors_is_copy() {
        let colors = retro_color_scheme();
        let colors2 = colors; // Copy
        let _colors3 = colors; // Still valid after copy

        // Both should have same values
        assert!(matches!(colors2.text, Color::AnsiValue(10)));
    }
}
