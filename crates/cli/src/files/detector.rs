//! File reference detection from user messages

use std::path::{Path, PathBuf};

/// Detect file references in a user message
pub fn detect_file_references(_message: &str, _cwd: &Path) -> Vec<PathBuf> {
    vec![]
}

/// Check if a token looks like a file reference
pub fn is_file_reference(_token: &str) -> bool {
    false
}
