//! AI response parsing to extract file write proposals
//!
//! Parses AI responses for file write proposals using multiple patterns:
//! - Fenced code blocks with filename comments
//! - Inline filename after language tag
//! - FILE markers

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

/// A file write proposal extracted from AI response
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileProposal {
    /// Target file path (relative or absolute)
    pub path: PathBuf,
    /// Proposed file content
    pub content: String,
    /// Whether this is a new file (vs edit of existing)
    pub is_new: bool,
}

// Regex patterns for extracting file proposals
static FENCED_BLOCK: LazyLock<Regex> = LazyLock::new(|| {
    // Match: ```lang?[ \t]+(inline_path)?\n(content)```
    // Use [ \t] instead of \s to avoid capturing newline
    Regex::new(r"(?s)```(?:\w+)?(?:[ \t]+([^\n]+))?\n(.*?)```").unwrap()
});

static FILENAME_COMMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^//\s*filename:\s*(.+?)\s*$").unwrap()
});

static FILE_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)---\s*FILE:\s*([^\n]+?)\s*---\n(.*?)(?:---\s*END FILE\s*---|```)").unwrap()
});

/// Extract file write proposals from an AI response.
///
/// Searches for file proposals using multiple patterns:
/// 1. Fenced code blocks with `// filename: path` in first two lines
/// 2. Fenced code blocks with inline filename: ```rust path/to/file.rs
/// 3. FILE markers: `--- FILE: path ---` ... `--- END FILE ---`
///
/// # Arguments
/// * `response` - The AI response text to parse
/// * `cwd` - Current working directory for resolving relative paths
///
/// # Returns
/// Vector of file proposals found in the response
///
/// # Example
/// ```
/// use std::path::Path;
/// use cherry2k::files::extract_file_proposals;
///
/// let response = "--- FILE: src/main.rs ---\nfn main() {}\n--- END FILE ---";
///
/// let cwd = Path::new("/project");
/// let proposals = extract_file_proposals(response, cwd);
/// assert_eq!(proposals.len(), 1);
/// assert_eq!(proposals[0].path, Path::new("/project/src/main.rs"));
/// ```
pub fn extract_file_proposals(response: &str, cwd: &Path) -> Vec<FileProposal> {
    let mut proposals = Vec::new();

    // Pattern 1: FILE markers (highest priority)
    for cap in FILE_MARKER.captures_iter(response) {
        if let (Some(path_match), Some(content_match)) = (cap.get(1), cap.get(2)) {
            let path_str = path_match.as_str().trim();
            let content = content_match.as_str();

            if let Some(proposal) = create_proposal(path_str, content, cwd) {
                proposals.push(proposal);
            }
        }
    }

    // Pattern 2 & 3: Fenced code blocks
    for cap in FENCED_BLOCK.captures_iter(response) {
        let inline_path = cap.get(1).map(|m| m.as_str().trim());
        let content = cap.get(2).map(|m| m.as_str()).unwrap_or("");

        // Try inline filename first (```rust path/to/file.rs)
        if let Some(path_str) = inline_path
            && (path_str.contains('/') || path_str.contains('\\')
                || path_str.ends_with(".rs") || path_str.ends_with(".toml")
                || path_str.ends_with(".md") || path_str.ends_with(".txt"))
            && let Some(proposal) = create_proposal(path_str, content, cwd)
        {
            proposals.push(proposal);
            continue;
        }

        // Try filename comment in first two lines
        let first_two_lines: String = content
            .lines()
            .take(2)
            .collect::<Vec<_>>()
            .join("\n");

        if let Some(cap) = FILENAME_COMMENT.captures(&first_two_lines)
            && let Some(path_match) = cap.get(1)
        {
            let path_str = path_match.as_str().trim();
            // Remove the filename comment line from content
            let clean_content = content
                .lines()
                .filter(|line| !FILENAME_COMMENT.is_match(line))
                .collect::<Vec<_>>()
                .join("\n");

            if let Some(proposal) = create_proposal(path_str, &clean_content, cwd) {
                proposals.push(proposal);
            }
        }
    }

    proposals
}

/// Create a FileProposal from path and content
fn create_proposal(path_str: &str, content: &str, cwd: &Path) -> Option<FileProposal> {
    if path_str.is_empty() || content.is_empty() {
        return None;
    }

    let path = if path_str.starts_with('/') {
        PathBuf::from(path_str)
    } else {
        cwd.join(path_str)
    };

    let is_new = !path.exists();

    Some(FileProposal {
        path,
        content: content.to_string(),
        is_new,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_from_filename_comment() {
        let response = r#"
Here's the file:

```rust
// filename: src/main.rs
fn main() {
    println!("Hello!");
}
```
"#;
        let cwd = Path::new("/project");
        let proposals = extract_file_proposals(response, cwd);

        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].path, Path::new("/project/src/main.rs"));
        assert!(proposals[0].content.contains("fn main()"));
        assert!(!proposals[0].content.contains("// filename:"));
    }

    #[test]
    fn test_extract_from_inline_filename() {
        let response = r#"
```rust src/lib.rs
pub fn hello() {
    println!("Hi!");
}
```
"#;
        let cwd = Path::new("/project");
        let proposals = extract_file_proposals(response, cwd);

        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].path, Path::new("/project/src/lib.rs"));
        assert!(proposals[0].content.contains("pub fn hello()"));
    }

    #[test]
    fn test_extract_from_file_markers() {
        let response = r#"
Create this file:

--- FILE: config.toml ---
[settings]
enabled = true
--- END FILE ---
"#;
        let cwd = Path::new("/project");
        let proposals = extract_file_proposals(response, cwd);

        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].path, Path::new("/project/config.toml"));
        assert!(proposals[0].content.contains("[settings]"));
    }

    #[test]
    fn test_multiple_proposals() {
        let response = r#"
```rust
// filename: src/main.rs
fn main() {}
```

```rust src/lib.rs
pub fn lib() {}
```

--- FILE: Cargo.toml ---
[package]
name = "test"
--- END FILE ---
"#;
        let cwd = Path::new("/project");
        let proposals = extract_file_proposals(response, cwd);

        assert_eq!(proposals.len(), 3);
    }

    #[test]
    fn test_non_file_code_blocks_ignored() {
        let response = r#"
Here's how to use it:

```rust
let x = 42;
println!("{}", x);
```

This is just an example, not a file.
"#;
        let cwd = Path::new("/project");
        let proposals = extract_file_proposals(response, cwd);

        assert_eq!(proposals.len(), 0);
    }

    #[test]
    fn test_relative_paths_resolved() {
        let cwd = Path::new("/absolute/path");
        let response = r#"
```rust relative/file.rs
fn test() {}
```
"#;
        let proposals = extract_file_proposals(response, cwd);

        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].path, Path::new("/absolute/path/relative/file.rs"));
    }

    #[test]
    fn test_is_new_correctly_set() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = temp_dir.path().join("existing.txt");
        fs::write(&existing_file, "old content").unwrap();

        let response = format!(
            r#"
```rust {}
new content
```

```rust {}
new file content
```
"#,
            existing_file.display(),
            temp_dir.path().join("new.txt").display()
        );

        let proposals = extract_file_proposals(&response, temp_dir.path());

        assert_eq!(proposals.len(), 2);
        assert_eq!(proposals[0].is_new, false); // existing file
        assert_eq!(proposals[1].is_new, true);  // new file
    }

    #[test]
    fn test_absolute_paths_preserved() {
        let response = r#"
```rust /tmp/absolute.rs
fn test() {}
```
"#;
        let cwd = Path::new("/project");
        let proposals = extract_file_proposals(response, cwd);

        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].path, Path::new("/tmp/absolute.rs"));
    }

    #[test]
    fn test_empty_content_ignored() {
        let response = r#"
```rust src/empty.rs
```
"#;
        let cwd = Path::new("/project");
        let proposals = extract_file_proposals(response, cwd);

        assert_eq!(proposals.len(), 0);
    }

    #[test]
    fn test_filename_comment_in_second_line() {
        let response = r#"
```rust
// Some comment
// filename: src/test.rs
fn test() {}
```
"#;
        let cwd = Path::new("/project");
        let proposals = extract_file_proposals(response, cwd);

        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].path, Path::new("/project/src/test.rs"));
    }
}
