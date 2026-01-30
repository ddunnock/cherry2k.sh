//! Boxed error display with actionable guidance
//!
//! Displays errors in a visually distinct boxed frame using Unicode
//! box-drawing characters. Provides custom messages for known error
//! types (rate limiting, invalid API key) with actionable guidance.

use cherry2k_core::ProviderError;
use colored::Colorize;

// Unicode box-drawing characters (double-line style)
const BOX_TOP_LEFT: char = '\u{2554}'; // ╔
const BOX_TOP_RIGHT: char = '\u{2557}'; // ╗
const BOX_BOTTOM_LEFT: char = '\u{255A}'; // ╚
const BOX_BOTTOM_RIGHT: char = '\u{255D}'; // ╝
const BOX_HORIZONTAL: char = '\u{2550}'; // ═
const BOX_VERTICAL: char = '\u{2551}'; // ║

/// Display an error in a boxed frame with actionable guidance.
///
/// The error is displayed with:
/// - A red boxed border for visibility
/// - Actionable guidance for common issues
///
/// # Example
///
/// ```no_run
/// use cherry2k::output::display_error;
/// use std::io;
///
/// let error = io::Error::new(io::ErrorKind::NotFound, "file not found");
/// display_error(&error);
/// ```
pub fn display_error(error: &dyn std::error::Error) {
    display_error_box(&format!("Error: {error}"));
}

/// Display a ProviderError with custom formatting and actionable guidance.
///
/// Provides specific guidance for:
/// - Rate limiting: shows retry time and quota suggestions
/// - Invalid API key: shows environment variable and config file locations
/// - Unavailable providers: suggests retry or alternative providers
///
/// # Example
///
/// ```no_run
/// use cherry2k::output::display_provider_error;
/// use cherry2k_core::ProviderError;
///
/// let error = ProviderError::InvalidApiKey { provider: "OpenAI".to_string() };
/// display_provider_error(&error);
/// ```
pub fn display_provider_error(error: &ProviderError) {
    let message = format_provider_error(error);
    display_error_box(&message);
}

/// Display a message in a boxed frame.
fn display_error_box(message: &str) {
    let lines: Vec<&str> = message.lines().collect();

    // Get terminal width or default
    let width = terminal_width().unwrap_or(60);
    let inner_width = width.saturating_sub(4); // Account for borders and padding

    // Print top border
    print_top_border(width);

    // Print each line with side borders
    for line in &lines {
        print_content_line(line, inner_width);
    }

    // Print bottom border
    print_bottom_border(width);
}

/// Format ProviderError variants with actionable guidance.
fn format_provider_error(error: &ProviderError) -> String {
    match error {
        ProviderError::RateLimited {
            provider,
            retry_after_secs,
        } => {
            format!(
                "Rate Limited by {provider}\n\n\
                 You've exceeded the API rate limit.\n\
                 Retry after: {retry_after_secs} seconds\n\n\
                 Suggestion: Wait and try again, or check your\n\
                 usage quota at the provider's dashboard."
            )
        }
        ProviderError::InvalidApiKey { provider } => {
            let env_var = provider_env_var(provider);
            format!(
                "Invalid API Key for {provider}\n\n\
                 The API key was rejected by the provider.\n\n\
                 To fix:\n\
                 1. Set {env_var} environment variable, or\n\
                 2. Update ~/.config/cherry2k/config.toml\n\n\
                 Get your API key from the {provider} dashboard."
            )
        }
        ProviderError::Unavailable { provider, reason } => {
            format!(
                "Provider Unavailable: {provider}\n\n\
                 Reason: {reason}\n\n\
                 The provider may be experiencing issues.\n\
                 Try again later or switch to a different provider."
            )
        }
        ProviderError::RequestFailed(msg) => {
            format!(
                "Request Failed\n\n\
                 {msg}\n\n\
                 Check your network connection and try again."
            )
        }
        ProviderError::ParseError(msg) => {
            format!(
                "Response Parse Error\n\n\
                 {msg}\n\n\
                 This may indicate an API change or network issue."
            )
        }
        ProviderError::StreamInterrupted(msg) => {
            format!(
                "Stream Interrupted\n\n\
                 {msg}\n\n\
                 The response was cut off. Try again."
            )
        }
    }
}

/// Get the environment variable name for a provider's API key.
fn provider_env_var(provider: &str) -> String {
    match provider.to_lowercase().as_str() {
        "openai" => "OPENAI_API_KEY".into(),
        "anthropic" => "ANTHROPIC_API_KEY".into(),
        "ollama" => "OLLAMA_HOST".into(),
        _ => format!("{}_API_KEY", provider.to_uppercase()),
    }
}

/// Get terminal width, if available.
fn terminal_width() -> Option<usize> {
    // Try to get terminal size from environment or default
    // In the future, could use terminal_size crate
    std::env::var("COLUMNS").ok().and_then(|s| s.parse().ok())
}

/// Generate a horizontal line of the given width.
fn horizontal_line(width: usize) -> String {
    std::iter::repeat_n(BOX_HORIZONTAL, width.saturating_sub(2)).collect()
}

/// Print the top border of the error box.
fn print_top_border(width: usize) {
    let line = horizontal_line(width);
    eprintln!("{}", format!("{BOX_TOP_LEFT}{line}{BOX_TOP_RIGHT}").red());
}

/// Print the bottom border of the error box.
fn print_bottom_border(width: usize) {
    let line = horizontal_line(width);
    eprintln!(
        "{}",
        format!("{BOX_BOTTOM_LEFT}{line}{BOX_BOTTOM_RIGHT}").red()
    );
}

/// Print a content line with side borders.
///
/// Uses character-aware truncation to avoid panics on multi-byte UTF-8 characters.
fn print_content_line(content: &str, inner_width: usize) {
    // Truncate or pad content to fit (character-aware to handle UTF-8 safely)
    let display_content: String = if content.chars().count() > inner_width {
        content.chars().take(inner_width).collect()
    } else {
        content.to_string()
    };
    let padding = inner_width.saturating_sub(display_content.chars().count());
    let padding_str = " ".repeat(padding);

    eprintln!(
        "{} {}{} {}",
        format!("{BOX_VERTICAL}").red(),
        display_content,
        padding_str,
        format!("{BOX_VERTICAL}").red()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_rate_limited_error() {
        let error = ProviderError::RateLimited {
            provider: "OpenAI".to_string(),
            retry_after_secs: 60,
        };
        let message = format_provider_error(&error);
        assert!(message.contains("Rate Limited"));
        assert!(message.contains("OpenAI"));
        assert!(message.contains("60 seconds"));
    }

    #[test]
    fn format_invalid_api_key_error() {
        let error = ProviderError::InvalidApiKey {
            provider: "Anthropic".to_string(),
        };
        let message = format_provider_error(&error);
        assert!(message.contains("Invalid API Key"));
        assert!(message.contains("ANTHROPIC_API_KEY"));
    }

    #[test]
    fn provider_env_var_mapping() {
        assert_eq!(provider_env_var("openai"), "OPENAI_API_KEY");
        assert_eq!(provider_env_var("anthropic"), "ANTHROPIC_API_KEY");
        assert_eq!(provider_env_var("ollama"), "OLLAMA_HOST");
        assert_eq!(provider_env_var("Custom"), "CUSTOM_API_KEY");
    }

    #[test]
    fn format_request_failed_error() {
        let error = ProviderError::RequestFailed("connection timeout".to_string());
        let message = format_provider_error(&error);
        assert!(message.contains("Request Failed"));
        assert!(message.contains("connection timeout"));
    }

    #[test]
    fn format_unavailable_error() {
        let error = ProviderError::Unavailable {
            provider: "OpenAI".to_string(),
            reason: "service maintenance".to_string(),
        };
        let message = format_provider_error(&error);
        assert!(message.contains("Provider Unavailable"));
        assert!(message.contains("service maintenance"));
    }

    #[test]
    fn print_content_line_handles_utf8() {
        // Test that multi-byte UTF-8 characters don't cause panics
        // The function prints to stderr, so we just verify it doesn't panic
        print_content_line("Hello 世界! 🎉 émojis", 10);
        print_content_line("日本語テスト", 5);
        print_content_line("Ñoño señor", 20);
    }

    #[test]
    fn print_content_line_truncates_correctly() {
        // Verify character-based truncation works
        let content = "Hello 世界";
        // "Hello 世界" is 8 characters, should not panic when truncating to 5
        print_content_line(content, 5);
    }
}
