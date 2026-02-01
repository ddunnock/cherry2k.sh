//! Unified diff generation and preview formatting
//!
//! Provides git-style unified diffs with colored output for file change previews.

use colored::Colorize;
use similar::TextDiff;

/// Generate a colored unified diff between old and new content.
///
/// Creates a git-style unified diff with:
/// - File headers (--- a/path, +++ b/path)
/// - Hunk headers (@@ -X,Y +X,Y @@)
/// - Color-coded changes (red for deletions, green for additions)
/// - 3 lines of context around changes
///
/// # Arguments
/// * `old` - Original file content (empty string for new files)
/// * `new` - New file content
/// * `filename` - File path to display in headers
///
/// # Returns
/// Formatted colored diff string ready for terminal display
///
/// # Example
/// ```
/// use cherry2k::files::generate_diff;
///
/// let old = "line 1\nline 2\nline 3";
/// let new = "line 1\nmodified line 2\nline 3";
/// let diff = generate_diff(old, new, "test.txt");
/// println!("{}", diff);
/// ```
pub fn generate_diff(old: &str, new: &str, filename: &str) -> String {
    let diff = TextDiff::from_lines(old, new);
    let mut output = String::new();

    // File headers
    output.push_str(&format!("--- a/{}\n", filename));
    output.push_str(&format!("+++ b/{}\n", filename));

    // Generate unified diff with 3 lines of context
    for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
        // Hunk header
        output.push_str(&format!("{}\n", hunk.header()));

        // Lines in this hunk
        for change in hunk.iter_changes() {
            let line = change.value();
            let colored_line = match change.tag() {
                similar::ChangeTag::Delete => format!("-{}", line).red().to_string(),
                similar::ChangeTag::Insert => format!("+{}", line).green().to_string(),
                similar::ChangeTag::Equal => format!(" {}", line),
            };
            output.push_str(&colored_line);
            // Only add newline if the line doesn't already end with one
            if !line.ends_with('\n') {
                output.push('\n');
            }
        }
    }

    output
}

/// Display a preview of new file content with formatting.
///
/// Shows the full content with:
/// - Green "New file:" header
/// - Line numbers for easy reference
/// - Separator lines for visual clarity
///
/// # Arguments
/// * `content` - The content of the new file
/// * `filename` - File path to display in header
///
/// # Example
/// ```
/// use cherry2k::files::display_new_file_preview;
///
/// let content = "fn main() {\n    println!(\"Hello\");\n}";
/// display_new_file_preview(content, "src/main.rs");
/// ```
pub fn display_new_file_preview(content: &str, filename: &str) {
    println!("{}", format!("New file: {}", filename).green().bold());
    println!("{}", "─".repeat(60));

    for (idx, line) in content.lines().enumerate() {
        println!("{:4} │ {}", (idx + 1).to_string().bright_black(), line);
    }

    println!("{}", "─".repeat(60));
}

/// Check if there are any changes between old and new content.
///
/// Quick check to determine if a diff would be empty.
///
/// # Arguments
/// * `old` - Original content
/// * `new` - New content
///
/// # Returns
/// `true` if content differs, `false` if identical
///
/// # Example
/// ```
/// use cherry2k::files::has_changes;
///
/// assert!(has_changes("old", "new"));
/// assert!(!has_changes("same", "same"));
/// ```
pub fn has_changes(old: &str, new: &str) -> bool {
    old != new
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_diff_unified_format() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nmodified line 2\nline 3\n";
        let diff = generate_diff(old, new, "test.txt");

        // Check for unified diff headers
        assert!(diff.contains("--- a/test.txt"));
        assert!(diff.contains("+++ b/test.txt"));

        // Check for hunk header format
        assert!(diff.contains("@@"));
    }

    #[test]
    fn test_generate_diff_colors_additions() {
        let old = "line 1\n";
        let new = "line 1\nline 2\n";
        let diff = generate_diff(old, new, "test.txt");

        // The added line should contain a + prefix (colored green in actual output)
        // Note: We can't easily test ANSI color codes in unit tests,
        // but we can verify the structure
        assert!(diff.contains("+line 2"));
    }

    #[test]
    fn test_generate_diff_colors_deletions() {
        let old = "line 1\nline 2\n";
        let new = "line 1\n";
        let diff = generate_diff(old, new, "test.txt");

        // The deleted line should contain a - prefix (colored red in actual output)
        assert!(diff.contains("-line 2"));
    }

    #[test]
    fn test_generate_diff_context_lines() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nmodified line 2\nline 3\n";
        let diff = generate_diff(old, new, "test.txt");

        // Context lines should have space prefix
        assert!(diff.contains(" line 1"));
        assert!(diff.contains(" line 3"));
    }

    #[test]
    fn test_generate_diff_empty_diff() {
        let content = "line 1\nline 2\n";
        let diff = generate_diff(content, content, "test.txt");

        // Should only have headers for identical content
        assert!(diff.contains("--- a/test.txt"));
        assert!(diff.contains("+++ b/test.txt"));
        // No hunk headers for empty diff
        assert!(!diff.contains("@@"));
    }

    #[test]
    fn test_has_changes_detects_differences() {
        assert!(has_changes("old content", "new content"));
        assert!(has_changes("line 1\nline 2", "line 1\nline 3"));
        assert!(has_changes("", "some content"));
        assert!(has_changes("some content", ""));
    }

    #[test]
    fn test_has_changes_detects_no_differences() {
        assert!(!has_changes("same", "same"));
        assert!(!has_changes("line 1\nline 2", "line 1\nline 2"));
        assert!(!has_changes("", ""));
    }

    #[test]
    fn test_new_file_preview_format() {
        // This test just ensures the function doesn't panic
        // Visual output is tested manually
        let content = "line 1\nline 2\nline 3";
        display_new_file_preview(content, "test.txt");
        // If we get here without panic, the test passes
    }
}
