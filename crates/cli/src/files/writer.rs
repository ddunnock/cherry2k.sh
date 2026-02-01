//! File writing with user approval flow
//!
//! Provides safe file writing with diff preview and [y/n/e] confirmation.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::confirm::{confirm, ConfirmResult};
use crate::files::{display_new_file_preview, generate_diff, has_changes};

/// Result of a file write operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteResult {
    /// File was written successfully
    Written { path: PathBuf },
    /// User cancelled the write operation
    Cancelled,
    /// No changes detected, write skipped
    Skipped,
}

/// Write a file with user approval after showing diff preview.
///
/// Shows a colored diff preview (or new file preview) and prompts the user
/// to approve, reject, or edit the changes before writing to disk.
///
/// # Arguments
/// * `path` - Target file path
/// * `new_content` - Content to write
/// * `auto_write` - If true, bypass confirmation and write immediately
///
/// # Returns
/// * `WriteResult::Written` - File was written
/// * `WriteResult::Cancelled` - User rejected the write
/// * `WriteResult::Skipped` - No changes detected
///
/// # Errors
/// Returns error if file I/O fails
///
/// # Example
/// ```no_run
/// use std::path::Path;
/// use cherry2k::files::write_file_with_approval;
///
/// let result = write_file_with_approval(
///     Path::new("config.toml"),
///     "new content",
///     false
/// ).unwrap();
/// ```
pub fn write_file_with_approval(
    path: &Path,
    new_content: &str,
    auto_write: bool,
) -> Result<WriteResult> {
    // Read existing content (empty string if new file)
    let old_content = fs::read_to_string(path).unwrap_or_default();

    // Check if there are any changes
    if !has_changes(&old_content, new_content) {
        eprintln!("No changes detected in {}", path.display());
        return Ok(WriteResult::Skipped);
    }

    // Display diff or new file preview
    if old_content.is_empty() {
        // New file
        display_new_file_preview(new_content, &path.display().to_string());
    } else {
        // Existing file - show diff
        let diff = generate_diff(&old_content, new_content, &path.display().to_string());
        println!("{}", diff);
    }

    // Auto-write mode bypasses confirmation
    if auto_write {
        write_file(path, new_content)?;
        eprintln!("Wrote {}", path.display());
        return Ok(WriteResult::Written {
            path: path.to_path_buf(),
        });
    }

    // Approval loop with edit support
    let mut content = new_content.to_string();
    loop {
        match confirm("Write this file?", true)? {
            ConfirmResult::Yes => {
                write_file(path, &content)?;
                eprintln!("Wrote {}", path.display());
                return Ok(WriteResult::Written {
                    path: path.to_path_buf(),
                });
            }
            ConfirmResult::No => {
                eprintln!("Cancelled write to {}", path.display());
                return Ok(WriteResult::Cancelled);
            }
            ConfirmResult::Edit => {
                // Open in $EDITOR
                content = edit::edit(&content)
                    .context("Failed to open editor")?;

                // Re-display diff with edited content
                println!();
                eprintln!("Updated diff after editing:");
                if old_content.is_empty() {
                    display_new_file_preview(&content, &path.display().to_string());
                } else {
                    let diff = generate_diff(&old_content, &content, &path.display().to_string());
                    println!("{}", diff);
                }
                // Loop continues to ask for confirmation again
            }
        }
    }
}

/// Write multiple files with batch or step-by-step approval.
///
/// Shows all diffs first, then offers to write all at once, cancel all,
/// or process files one at a time with individual approval.
///
/// # Arguments
/// * `files` - Vector of (path, content) tuples
/// * `auto_write` - If true, bypass all confirmations
///
/// # Returns
/// Vector of WriteResult for each file, in the same order as input
///
/// # Errors
/// Returns error if file I/O fails
///
/// # Example
/// ```no_run
/// use std::path::PathBuf;
/// use cherry2k::files::write_multiple_files;
///
/// let files = vec![
///     (PathBuf::from("file1.txt"), "content 1".to_string()),
///     (PathBuf::from("file2.txt"), "content 2".to_string()),
/// ];
/// let results = write_multiple_files(&files, false).unwrap();
/// ```
pub fn write_multiple_files(
    files: &[(PathBuf, String)],
    auto_write: bool,
) -> Result<Vec<WriteResult>> {
    if files.is_empty() {
        return Ok(vec![]);
    }

    // Show all diffs first
    eprintln!("\n{} file(s) to write:\n", files.len());
    for (path, new_content) in files {
        let old_content = fs::read_to_string(path).unwrap_or_default();

        if !has_changes(&old_content, new_content) {
            eprintln!("Skipping {} (no changes)", path.display());
            continue;
        }

        eprintln!("─────────────────────────────────────");
        if old_content.is_empty() {
            display_new_file_preview(new_content, &path.display().to_string());
        } else {
            let diff = generate_diff(&old_content, new_content, &path.display().to_string());
            println!("{}", diff);
        }
    }
    eprintln!("─────────────────────────────────────\n");

    // Auto-write mode
    if auto_write {
        let mut results = Vec::new();
        for (path, content) in files {
            let result = write_file_with_approval(path, content, true)?;
            results.push(result);
        }
        return Ok(results);
    }

    // Prompt for batch or step-by-step processing
    print!("Write all files? [y/n/step] ");
    io::Write::flush(&mut io::stdout())?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim().to_lowercase();

    match choice.as_str() {
        "y" | "yes" => {
            // Write all files
            let mut results = Vec::new();
            for (path, content) in files {
                write_file(path, content)?;
                eprintln!("Wrote {}", path.display());
                results.push(WriteResult::Written {
                    path: path.clone(),
                });
            }
            Ok(results)
        }
        "step" => {
            // Process each file individually
            let mut results = Vec::new();
            for (path, content) in files {
                let result = write_file_with_approval(path, content, false)?;
                results.push(result);
            }
            Ok(results)
        }
        _ => {
            // Default to cancel
            eprintln!("Cancelled all writes");
            Ok(vec![WriteResult::Cancelled; files.len()])
        }
    }
}

/// Internal helper to write file with parent directory creation.
fn write_file(path: &Path, content: &str) -> Result<()> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    fs::write(path, content)
        .with_context(|| format!("Failed to write file {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_result_equality() {
        let path = PathBuf::from("/tmp/test.txt");
        assert_eq!(
            WriteResult::Written { path: path.clone() },
            WriteResult::Written { path: path.clone() }
        );
        assert_eq!(WriteResult::Cancelled, WriteResult::Cancelled);
        assert_eq!(WriteResult::Skipped, WriteResult::Skipped);
        assert_ne!(
            WriteResult::Written { path: path.clone() },
            WriteResult::Cancelled
        );
    }

    #[test]
    fn test_auto_write_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new_file.txt");
        let content = "test content";

        let result = write_file_with_approval(&file_path, content, true).unwrap();

        assert_eq!(
            result,
            WriteResult::Written {
                path: file_path.clone()
            }
        );
        assert_eq!(fs::read_to_string(&file_path).unwrap(), content);
    }

    #[test]
    fn test_auto_write_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.txt");

        // Create initial file
        fs::write(&file_path, "old content").unwrap();

        // Update with new content
        let new_content = "new content";
        let result = write_file_with_approval(&file_path, new_content, true).unwrap();

        assert_eq!(
            result,
            WriteResult::Written {
                path: file_path.clone()
            }
        );
        assert_eq!(fs::read_to_string(&file_path).unwrap(), new_content);
    }

    #[test]
    fn test_no_changes_returns_skipped() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("unchanged.txt");
        let content = "same content";

        // Create file with content
        fs::write(&file_path, content).unwrap();

        // Try to write same content
        let result = write_file_with_approval(&file_path, content, false).unwrap();

        assert_eq!(result, WriteResult::Skipped);
    }

    #[test]
    fn test_write_file_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("deeply/nested/file.txt");
        let content = "test";

        write_file(&file_path, content).unwrap();

        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), content);
    }

    #[test]
    fn test_write_multiple_files_auto_write() {
        let temp_dir = TempDir::new().unwrap();

        let files = vec![
            (temp_dir.path().join("file1.txt"), "content 1".to_string()),
            (temp_dir.path().join("file2.txt"), "content 2".to_string()),
        ];

        let results = write_multiple_files(&files, true).unwrap();

        assert_eq!(results.len(), 2);
        assert!(matches!(results[0], WriteResult::Written { .. }));
        assert!(matches!(results[1], WriteResult::Written { .. }));

        assert_eq!(fs::read_to_string(&files[0].0).unwrap(), "content 1");
        assert_eq!(fs::read_to_string(&files[1].0).unwrap(), "content 2");
    }

    #[test]
    fn test_write_multiple_files_empty_list() {
        let files: Vec<(PathBuf, String)> = vec![];
        let results = write_multiple_files(&files, false).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_write_multiple_files_with_no_changes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("unchanged.txt");
        let content = "same content";

        // Create file with content
        fs::write(&file_path, content).unwrap();

        let files = vec![
            (file_path, content.to_string()),
        ];

        // This should handle the "no changes" case gracefully
        let results = write_multiple_files(&files, true).unwrap();

        // Should return Skipped for unchanged file
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], WriteResult::Skipped);
    }
}
