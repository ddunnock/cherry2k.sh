//! File reference detection from user messages
//!
//! Detects when users mention file paths in chat messages, enabling automatic
//! file content inclusion for AI context.

use std::path::{Path, PathBuf};

/// Detect file references in a user message.
///
/// Scans the message for tokens that look like file paths, validates they exist,
/// and returns canonicalized paths.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use cherry2k::files::detect_file_references;
///
/// let cwd = Path::new("/project");
/// let files = detect_file_references("fix main.rs", cwd);
/// // Returns vec![PathBuf::from("/project/main.rs")] if file exists
/// ```
pub fn detect_file_references(message: &str, cwd: &Path) -> Vec<PathBuf> {
    let mut found_files = Vec::new();

    // Extract potential file path tokens from the message
    let tokens = extract_tokens(message);

    for token in tokens {
        if let Some(path) = validate_file_path(&token, cwd) {
            // Avoid duplicates
            if !found_files.contains(&path) {
                found_files.push(path);
            }
        }
    }

    found_files
}

/// Check if a token looks like a file reference (heuristic).
///
/// Returns true if the token contains path separators or common file extensions.
pub fn is_file_reference(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }

    // Too generic to be a file path
    if token.len() <= 2 {
        return false;
    }

    // Contains path separator
    if token.contains('/') || token.contains('\\') {
        return true;
    }

    // Has common file extension
    let extensions = [
        ".rs", ".py", ".js", ".ts", ".jsx", ".tsx", ".go", ".java", ".c", ".cpp",
        ".h", ".hpp", ".cs", ".rb", ".php", ".swift", ".kt", ".sh", ".bash", ".zsh",
        ".toml", ".yaml", ".yml", ".json", ".xml", ".md", ".txt", ".csv", ".sql",
    ];

    for ext in &extensions {
        if token.ends_with(ext) {
            return true;
        }
    }

    false
}

/// Extract potential file path tokens from message.
fn extract_tokens(message: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';
    let mut in_backticks = false;

    for ch in message.chars() {
        match ch {
            '"' | '\'' if !in_backticks => {
                if in_quotes && ch == quote_char {
                    // End of quoted string
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                    in_quotes = false;
                } else if !in_quotes {
                    // Start of quoted string
                    in_quotes = true;
                    quote_char = ch;
                }
            }
            '`' if !in_quotes => {
                if in_backticks {
                    // End of backticked string
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                    in_backticks = false;
                } else {
                    // Start of backticked string
                    in_backticks = true;
                }
            }
            ' ' | '\t' | '\n' | ',' | ';' | ':' if !in_quotes && !in_backticks => {
                // Token separator
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Don't forget the last token
    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Validate that a token represents an existing file path.
fn validate_file_path(token: &str, cwd: &Path) -> Option<PathBuf> {
    // Try as absolute path
    let path = Path::new(token);
    if path.is_absolute() && path.is_file() {
        return path.canonicalize().ok();
    }

    // Try relative to cwd
    let relative_path = cwd.join(token);
    if relative_path.is_file() {
        return relative_path.canonicalize().ok();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Create test files
        let main_rs = temp_path.join("main.rs");
        fs::write(&main_rs, "fn main() {}").unwrap();

        let src_dir = temp_path.join("src");
        fs::create_dir(&src_dir).unwrap();
        let lib_rs = src_dir.join("lib.rs");
        fs::write(&lib_rs, "// lib").unwrap();

        (temp_dir, temp_path)
    }

    #[test]
    fn detects_simple_filename_in_cwd() {
        let (_temp_dir, temp_path) = setup_test_env();

        let message = "fix main.rs";
        let files = detect_file_references(message, &temp_path);

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("main.rs"));
    }

    #[test]
    fn detects_relative_path() {
        let (_temp_dir, temp_path) = setup_test_env();

        let message = "look at src/lib.rs";
        let files = detect_file_references(message, &temp_path);

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("lib.rs"));
    }

    #[test]
    fn detects_quoted_paths() {
        let (_temp_dir, temp_path) = setup_test_env();

        let message = r#"check "main.rs" and 'src/lib.rs'"#;
        let files = detect_file_references(message, &temp_path);

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn detects_backticked_paths() {
        let (_temp_dir, temp_path) = setup_test_env();

        let message = "check `main.rs` please";
        let files = detect_file_references(message, &temp_path);

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("main.rs"));
    }

    #[test]
    fn ignores_non_file_words() {
        let (_temp_dir, temp_path) = setup_test_env();

        let message = "what is Rust?";
        let files = detect_file_references(message, &temp_path);

        assert_eq!(files.len(), 0);
    }

    #[test]
    fn filters_out_nonexistent_files() {
        let (_temp_dir, temp_path) = setup_test_env();

        let message = "fix nonexistent.rs";
        let files = detect_file_references(message, &temp_path);

        assert_eq!(files.len(), 0);
    }

    #[test]
    fn avoids_duplicates() {
        let (_temp_dir, temp_path) = setup_test_env();

        let message = "main.rs and main.rs again";
        let files = detect_file_references(message, &temp_path);

        assert_eq!(files.len(), 1);
    }

    #[test]
    fn is_file_reference_detects_extensions() {
        assert!(is_file_reference("main.rs"));
        assert!(is_file_reference("test.py"));
        assert!(is_file_reference("config.toml"));
    }

    #[test]
    fn is_file_reference_detects_paths() {
        assert!(is_file_reference("src/main.rs"));
        assert!(is_file_reference("./test"));
        assert!(is_file_reference("../parent"));
    }

    #[test]
    fn is_file_reference_rejects_generic_words() {
        assert!(!is_file_reference("file"));
        assert!(!is_file_reference("main"));
        assert!(!is_file_reference("Rust"));
        assert!(!is_file_reference(""));
        assert!(!is_file_reference("a"));
    }

    #[test]
    fn handles_multiple_files_in_complex_message() {
        let (_temp_dir, temp_path) = setup_test_env();

        let message = "Compare main.rs with src/lib.rs, they should match";
        let files = detect_file_references(message, &temp_path);

        assert_eq!(files.len(), 2);
    }
}
