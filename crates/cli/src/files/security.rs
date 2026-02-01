//! Security validations for file operations
//!
//! Provides secrets detection to prevent accidental exposure of sensitive files,
//! and path validation to enforce project scope boundaries.

use std::path::Path;

use super::scope::ProjectScope;

/// Filenames and patterns that should never be written by the AI.
///
/// These files typically contain secrets, credentials, or private keys.
const BLOCKED_FILENAMES: &[&str] = &[
    ".env",
    ".env.local",
    ".env.production",
    ".env.development",
    ".env.staging",
    ".env.test",
    "credentials.json",
    "secrets.json",
    "secrets.yaml",
    "secrets.yml",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
    "id_dsa",
    ".npmrc",
    ".pypirc",
    ".netrc",
    ".dockercfg",
    "docker-config.json",
];

/// Check if a path refers to a secrets file.
///
/// Returns `true` if the filename matches any blocked pattern or contains
/// sensitive path segments like `.aws/credentials`.
///
/// # Arguments
///
/// * `path` - Path to check
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use cherry2k::files::is_secrets_file;
///
/// assert!(is_secrets_file(Path::new(".env")));
/// assert!(is_secrets_file(Path::new("/path/to/.env.local")));
/// assert!(is_secrets_file(Path::new(".aws/credentials")));
/// assert!(!is_secrets_file(Path::new("config.toml")));
/// ```
pub fn is_secrets_file(path: &Path) -> bool {
    // Check filename against blocked list
    if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
        if BLOCKED_FILENAMES.contains(&filename) {
            return true;
        }
    }

    // Check for .aws/credentials pattern in path
    let path_str = path.to_string_lossy();
    if path_str.contains(".aws/credentials") || path_str.contains(".aws\\credentials") {
        return true;
    }

    false
}

/// Result of path validation for write operations.
#[derive(Debug, PartialEq, Eq)]
pub enum ValidationResult {
    /// Path is safe to write
    Ok,
    /// Path is outside project scope - needs extra confirmation
    OutOfScope {
        /// The path that's out of scope
        path: String,
        /// The project root
        root: String,
    },
    /// Path is a secrets file - writing is blocked
    BlockedSecrets {
        /// The blocked path
        path: String,
    },
}

/// Validate a write path against security constraints.
///
/// Checks both secrets file patterns and project scope boundaries.
///
/// # Arguments
///
/// * `path` - Path to validate for writing
/// * `scope` - Project scope to check against
///
/// # Returns
///
/// - `ValidationResult::Ok` - Path is safe to write
/// - `ValidationResult::OutOfScope` - Path is outside project, needs extra confirmation
/// - `ValidationResult::BlockedSecrets` - Path is a secrets file, cannot write
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use cherry2k::files::{ProjectScope, validate_write_path, ValidationResult};
///
/// let scope = ProjectScope::detect().unwrap();
///
/// match validate_write_path(Path::new("src/main.rs"), &scope) {
///     ValidationResult::Ok => println!("Safe to write"),
///     ValidationResult::OutOfScope { .. } => println!("Outside project"),
///     ValidationResult::BlockedSecrets { .. } => println!("Blocked secrets file"),
/// }
/// ```
pub fn validate_write_path(path: &Path, scope: &ProjectScope) -> ValidationResult {
    // First check for secrets - these are blocked regardless of scope
    if is_secrets_file(path) {
        return ValidationResult::BlockedSecrets {
            path: path.display().to_string(),
        };
    }

    // Check scope
    if !scope.is_within_scope(path) {
        return ValidationResult::OutOfScope {
            path: path.display().to_string(),
            root: scope.root().display().to_string(),
        };
    }

    ValidationResult::Ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn detects_env_file() {
        assert!(is_secrets_file(Path::new(".env")));
    }

    #[test]
    fn detects_env_local() {
        assert!(is_secrets_file(Path::new(".env.local")));
    }

    #[test]
    fn detects_env_production() {
        assert!(is_secrets_file(Path::new(".env.production")));
    }

    #[test]
    fn detects_credentials_json() {
        assert!(is_secrets_file(Path::new("credentials.json")));
        assert!(is_secrets_file(Path::new("/path/to/credentials.json")));
    }

    #[test]
    fn detects_secrets_yaml() {
        assert!(is_secrets_file(Path::new("secrets.yaml")));
        assert!(is_secrets_file(Path::new("secrets.yml")));
    }

    #[test]
    fn detects_ssh_keys() {
        assert!(is_secrets_file(Path::new("id_rsa")));
        assert!(is_secrets_file(Path::new("id_ed25519")));
        assert!(is_secrets_file(Path::new("id_ecdsa")));
    }

    #[test]
    fn detects_aws_credentials() {
        assert!(is_secrets_file(Path::new(".aws/credentials")));
        assert!(is_secrets_file(Path::new("/home/user/.aws/credentials")));
        // Windows path
        assert!(is_secrets_file(Path::new(".aws\\credentials")));
    }

    #[test]
    fn allows_regular_files() {
        assert!(!is_secrets_file(Path::new("config.toml")));
        assert!(!is_secrets_file(Path::new("src/main.rs")));
        assert!(!is_secrets_file(Path::new("README.md")));
        assert!(!is_secrets_file(Path::new(".gitignore")));
    }

    #[test]
    fn validate_blocks_secrets_file() {
        let temp = TempDir::new().unwrap();
        let scope = ProjectScope::new_for_test(temp.path().to_path_buf(), false);

        let env_path = temp.path().join(".env");
        let result = validate_write_path(&env_path, &scope);

        assert!(matches!(result, ValidationResult::BlockedSecrets { .. }));
    }

    #[test]
    fn validate_allows_in_scope_file() {
        let temp = TempDir::new().unwrap();
        let scope = ProjectScope::new_for_test(temp.path().to_path_buf(), false);

        let file_path = temp.path().join("src").join("main.rs");
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();

        let result = validate_write_path(&file_path, &scope);

        assert_eq!(result, ValidationResult::Ok);
    }

    #[test]
    fn validate_flags_out_of_scope() {
        let temp = TempDir::new().unwrap();
        let scope = ProjectScope::new_for_test(temp.path().to_path_buf(), false);

        // Path outside scope (parent directory)
        let outside_path = temp.path().parent().unwrap().join("outside.txt");
        let result = validate_write_path(&outside_path, &scope);

        assert!(matches!(result, ValidationResult::OutOfScope { .. }));
    }

    #[test]
    fn validate_blocks_secrets_even_outside_scope() {
        let temp = TempDir::new().unwrap();
        let scope = ProjectScope::new_for_test(temp.path().to_path_buf(), false);

        // Secrets file outside scope should still be blocked
        let env_path = temp.path().parent().unwrap().join(".env");
        let result = validate_write_path(&env_path, &scope);

        // Secrets check happens first, so this should be BlockedSecrets not OutOfScope
        assert!(matches!(result, ValidationResult::BlockedSecrets { .. }));
    }
}
