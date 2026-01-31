//! Async database connection wrapper
//!
//! This module provides an async interface to SQLite using tokio-rusqlite,
//! handling database initialization, XDG paths, and proper file permissions.

use std::path::PathBuf;
use std::time::Duration;

use directories::ProjectDirs;
use tokio_rusqlite::Connection;

use crate::StorageError;
use crate::schema::ensure_schema;

/// Async SQLite database wrapper
///
/// Provides async access to the SQLite database for session and message storage.
/// Handles automatic schema migration, XDG-compliant paths, and secure file permissions.
///
/// # Example
///
/// ```no_run
/// use cherry2k_storage::Database;
///
/// # async fn example() -> Result<(), cherry2k_core::StorageError> {
/// let db = Database::open().await?;
/// // Use db.call() for database operations
/// # Ok(())
/// # }
/// ```
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Opens the database, creating it if necessary
    ///
    /// This function:
    /// 1. Determines the XDG data directory for cherry2k
    /// 2. Creates the directory if it doesn't exist
    /// 3. Opens/creates the SQLite database file
    /// 4. Sets file permissions to 0600 (owner read/write only)
    /// 5. Configures SQLite for robustness (busy timeout, foreign keys)
    /// 6. Runs schema migrations if needed
    ///
    /// # Database Location
    ///
    /// - Linux: `~/.local/share/cherry2k/sessions.db`
    /// - macOS: `~/Library/Application Support/cherry2k/sessions.db`
    /// - Windows: `C:\Users\<User>\AppData\Roaming\cherry2k\sessions.db`
    ///
    /// # Errors
    ///
    /// Returns `StorageError` if:
    /// - XDG directories cannot be determined (no home directory)
    /// - Directory creation fails
    /// - Database file cannot be opened/created
    /// - Schema migration fails
    pub async fn open() -> Result<Self, StorageError> {
        let db_path = Self::database_path()?;
        Self::open_at(db_path).await
    }

    /// Opens the database at a specific path
    ///
    /// This is useful for testing or custom database locations.
    /// The parent directory must exist.
    ///
    /// # Errors
    ///
    /// Returns `StorageError` if the database cannot be opened or initialized.
    pub async fn open_at(path: PathBuf) -> Result<Self, StorageError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                StorageError::IoError(format!("Failed to create database directory: {e}"))
            })?;
        }

        tracing::debug!("Opening database at {:?}", path);

        // Open the connection
        let conn = Connection::open(&path)
            .await
            .map_err(|e| StorageError::Database(format!("Failed to open database: {e}")))?;

        // Set file permissions on Unix (0600 = owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if path.exists() {
                let permissions = std::fs::Permissions::from_mode(0o600);
                std::fs::set_permissions(&path, permissions).map_err(|e| {
                    StorageError::IoError(format!("Failed to set database permissions: {e}"))
                })?;
            }
        }

        // Configure SQLite and run migrations
        conn.call(|conn| {
            // Set busy timeout to 5 seconds for concurrent access
            conn.busy_timeout(Duration::from_secs(5))?;

            // Enable foreign key constraints
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;

            // Run schema migrations
            ensure_schema(conn)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            Ok::<(), rusqlite::Error>(())
        })
        .await
        .map_err(|e| StorageError::Database(format!("Failed to initialize database: {e}")))?;

        Ok(Self { conn })
    }

    /// Returns the default database path based on XDG directories
    ///
    /// # Errors
    ///
    /// Returns `StorageError::NoHomeDir` if the home directory cannot be determined.
    pub fn database_path() -> Result<PathBuf, StorageError> {
        let proj_dirs = ProjectDirs::from("", "", "cherry2k").ok_or(StorageError::NoHomeDir)?;

        Ok(proj_dirs.data_dir().join("sessions.db"))
    }

    /// Executes a closure with the underlying rusqlite connection
    ///
    /// This is the primary way to interact with the database. The closure
    /// runs on a dedicated thread pool, allowing async code to wait for
    /// database operations without blocking the async runtime.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cherry2k_storage::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::open().await?;
    /// let count: i64 = db.call(|conn| {
    ///     conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the error from the closure if it fails.
    pub async fn call<F, R>(&self, f: F) -> Result<R, rusqlite::Error>
    where
        F: FnOnce(&mut rusqlite::Connection) -> Result<R, rusqlite::Error> + Send + 'static,
        R: Send + 'static,
    {
        self.conn.call(f).await.map_err(|e| match e {
            tokio_rusqlite::Error::Error(e) | tokio_rusqlite::Error::Close((_, e)) => e,
            _ => rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ABORT),
                Some("Connection closed or unavailable".to_string()),
            ),
        })
    }

    /// Executes a closure that may return a custom error type
    ///
    /// Similar to `call`, but allows returning `StorageError` instead of
    /// `rusqlite::Error`. Useful for higher-level operations that need
    /// to return domain-specific errors.
    pub async fn call_storage<F, R>(&self, f: F) -> Result<R, StorageError>
    where
        F: FnOnce(&mut rusqlite::Connection) -> Result<R, StorageError> + Send + 'static,
        R: Send + 'static,
    {
        self.conn
            .call(move |conn| {
                f(conn).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })
            .await
            .map_err(|e| StorageError::Database(e.to_string()))
    }

    /// Returns a reference to the underlying tokio-rusqlite connection
    ///
    /// This is primarily for advanced use cases where direct access
    /// to the connection is needed.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn database_opens_and_initializes() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::open_at(db_path.clone()).await.unwrap();

        // Verify schema was created
        let table_count: i64 = db
            .call(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                    [],
                    |row| row.get(0),
                )
            })
            .await
            .unwrap();

        // Should have schema_version, sessions, and messages tables
        assert!(
            table_count >= 3,
            "Expected at least 3 tables, got {table_count}"
        );
    }

    #[tokio::test]
    async fn database_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("deep").join("test.db");

        // Parent directories don't exist yet
        assert!(!nested_path.parent().unwrap().exists());

        let _db = Database::open_at(nested_path.clone()).await.unwrap();

        // Now they should exist
        assert!(nested_path.exists());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn database_has_secure_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("secure.db");

        let _db = Database::open_at(db_path.clone()).await.unwrap();

        let metadata = std::fs::metadata(&db_path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "Database should have 0600 permissions");
    }

    #[tokio::test]
    async fn database_enables_foreign_keys() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("fk.db");

        let db = Database::open_at(db_path).await.unwrap();

        // Foreign keys should be enabled
        let fk_enabled: i64 = db
            .call(|conn| conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0)))
            .await
            .unwrap();

        assert_eq!(fk_enabled, 1, "Foreign keys should be enabled");
    }

    #[tokio::test]
    async fn database_path_is_correct() {
        let path = Database::database_path().unwrap();

        // Should end with sessions.db
        assert!(path.ends_with("sessions.db"));

        // Should contain cherry2k in the path
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains("cherry2k"),
            "Path should contain 'cherry2k': {path_str}"
        );
    }

    #[tokio::test]
    async fn call_storage_converts_errors() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("error.db");

        let db = Database::open_at(db_path).await.unwrap();

        let result: Result<(), StorageError> = db
            .call_storage(|_conn| {
                Err(StorageError::SessionNotFound {
                    id: "test".to_string(),
                })
            })
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, StorageError::Database(_)));
    }
}
