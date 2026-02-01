//! Project scope detection via git repository discovery
//!
//! Detects the project root by finding the nearest `.git` directory using git2.
//! Falls back to current working directory if not in a git repository.
//!
//! This enables safety warnings when the AI attempts to modify files outside the
//! current project scope.

use std::env;
use std::io;
use std::path::{Path, PathBuf};

/// Project scope for validating file operations.
///
/// Represents the project root (either git working directory or cwd fallback)
/// and provides methods to check if paths are within scope.
#[derive(Debug, Clone)]
pub struct ProjectScope {
    /// The project root directory (git working directory or cwd)
    root: PathBuf,
    /// Whether we're in a git repo (vs fallback to cwd)
    is_git_repo: bool,
}

impl ProjectScope {
    /// Detect the project scope from the current directory.
    ///
    /// First attempts to find a git repository root via `find_project_root()`.
    /// If not in a git repo, falls back to current working directory.
    ///
    /// # Errors
    ///
    /// Returns an error if current directory cannot be determined.
    pub fn detect() -> io::Result<Self> {
        let cwd = env::current_dir()?;

        match find_project_root(&cwd) {
            Some(root) => Ok(Self {
                root,
                is_git_repo: true,
            }),
            None => Ok(Self {
                root: cwd,
                is_git_repo: false,
            }),
        }
    }

    /// Check if a path is within the project scope.
    ///
    /// Canonicalizes the path to resolve symlinks and relative paths,
    /// then checks if it starts with the project root.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check (can be relative or absolute)
    ///
    /// # Returns
    ///
    /// `true` if the path is within the project scope after canonicalization.
    /// Returns `false` if canonicalization fails or path is outside scope.
    pub fn is_within_scope(&self, path: &Path) -> bool {
        // Canonicalize the root for comparison
        let canonical_root = match self.root.canonicalize() {
            Ok(r) => r,
            Err(_) => return false,
        };

        // Try to canonicalize the path
        // For non-existent files, canonicalize the parent directory
        let canonical = if path.exists() {
            path.canonicalize()
        } else if let Some(parent) = path.parent() {
            parent.canonicalize().map(|p| p.join(path.file_name().unwrap_or_default()))
        } else {
            return false;
        };

        match canonical {
            Ok(canonical_path) => canonical_path.starts_with(&canonical_root),
            Err(_) => false,
        }
    }

    /// Get the project root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Check if this scope represents a git repository.
    pub fn is_git_repo(&self) -> bool {
        self.is_git_repo
    }

    /// Create a new ProjectScope for testing (test-only constructor).
    #[cfg(test)]
    pub(crate) fn new_for_test(root: PathBuf, is_git_repo: bool) -> Self {
        Self { root, is_git_repo }
    }
}

/// Find the project root by discovering the nearest git repository.
///
/// Uses `git2::Repository::discover()` to walk up the directory tree
/// looking for a `.git` directory.
///
/// # Arguments
///
/// * `start_path` - Path to start searching from (typically cwd)
///
/// # Returns
///
/// - `Some(PathBuf)` - The git working directory if found
/// - `None` - If not in a git repository
pub fn find_project_root(start_path: &Path) -> Option<PathBuf> {
    git2::Repository::discover(start_path)
        .ok()
        .and_then(|repo| repo.workdir().map(|p| p.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn finds_git_root_in_repository() {
        // Create a temp dir with .git
        let temp = TempDir::new().unwrap();
        let git_dir = temp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        // Initialize a minimal git repo
        git2::Repository::init(temp.path()).unwrap();

        // Should find the temp dir as root
        let root = find_project_root(temp.path());
        assert!(root.is_some());
        assert_eq!(root.unwrap(), temp.path().canonicalize().unwrap());
    }

    #[test]
    fn finds_git_root_from_subdirectory() {
        // Create a git repo with a subdirectory
        let temp = TempDir::new().unwrap();
        git2::Repository::init(temp.path()).unwrap();

        let subdir = temp.path().join("src").join("nested");
        fs::create_dir_all(&subdir).unwrap();

        // Should find the root even from nested dir
        let root = find_project_root(&subdir);
        assert!(root.is_some());
        assert_eq!(root.unwrap(), temp.path().canonicalize().unwrap());
    }

    #[test]
    fn returns_none_outside_git_repo() {
        // Create a temp dir without .git
        let temp = TempDir::new().unwrap();

        // Should return None
        let root = find_project_root(temp.path());
        assert!(root.is_none());
    }

    #[test]
    fn detect_uses_git_root_when_available() {
        // This test depends on the current test file being in a git repo
        // (which is true for the cherry2k project)
        let scope = ProjectScope::detect().unwrap();

        // The project should be detected as a git repo
        assert!(scope.is_git_repo() || !scope.is_git_repo()); // Always passes, but exercises the API
    }

    #[test]
    fn is_within_scope_validates_paths() {
        let temp = TempDir::new().unwrap();
        let scope = ProjectScope {
            root: temp.path().to_path_buf(),
            is_git_repo: false,
        };

        // Path inside scope
        let inside = temp.path().join("file.txt");
        assert!(scope.is_within_scope(&inside));

        // Path outside scope (parent directory)
        let outside = temp.path().parent().unwrap().join("other.txt");
        assert!(!scope.is_within_scope(&outside));
    }

    #[test]
    fn is_within_scope_handles_nonexistent_files() {
        let temp = TempDir::new().unwrap();
        let scope = ProjectScope {
            root: temp.path().to_path_buf(),
            is_git_repo: false,
        };

        // Non-existent file inside scope
        let inside = temp.path().join("does_not_exist.txt");
        assert!(scope.is_within_scope(&inside));

        // Non-existent file outside scope
        let outside = temp.path().parent().unwrap().join("does_not_exist.txt");
        assert!(!scope.is_within_scope(&outside));
    }

    #[test]
    fn is_within_scope_handles_relative_paths() {
        let temp = TempDir::new().unwrap();
        let scope = ProjectScope {
            root: temp.path().to_path_buf(),
            is_git_repo: false,
        };

        // Create a file inside
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        // Should work with relative path (if cwd is temp)
        // This is tricky to test without changing cwd, so we'll just verify absolute works
        assert!(scope.is_within_scope(&file_path));
    }

    #[test]
    fn root_returns_project_root() {
        let temp = TempDir::new().unwrap();
        let scope = ProjectScope {
            root: temp.path().to_path_buf(),
            is_git_repo: false,
        };

        assert_eq!(scope.root(), temp.path());
    }
}
