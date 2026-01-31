//! Database schema definitions and migrations
//!
//! This module contains the SQL schema for Cherry2K's SQLite database,
//! including tables for sessions and messages.

use rusqlite::Connection;

use crate::StorageError;

/// Current schema version for migration tracking
pub const SCHEMA_VERSION: i32 = 1;

/// Initial database schema SQL
///
/// Creates:
/// - `schema_version` table for tracking migrations
/// - `sessions` table for conversation sessions
/// - `messages` table for individual messages within sessions
/// - Indexes for efficient queries
const INIT_SCHEMA: &str = r#"
-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Sessions table: groups messages by working directory and time
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    working_dir TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_message_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Messages table: individual messages within a session
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    token_count INTEGER,
    is_summary INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Index for finding sessions by directory (most recent first)
CREATE INDEX IF NOT EXISTS idx_sessions_dir_time
    ON sessions(working_dir, last_message_at DESC);

-- Index for finding messages by session
CREATE INDEX IF NOT EXISTS idx_messages_session
    ON messages(session_id, created_at ASC);

-- Partial index for summary messages (used in context management)
CREATE INDEX IF NOT EXISTS idx_messages_summary
    ON messages(session_id, id DESC) WHERE is_summary = 1;

-- Record schema version
INSERT OR IGNORE INTO schema_version (version) VALUES (1);
"#;

/// Ensures the database schema is up to date
///
/// This function:
/// 1. Checks if the schema_version table exists
/// 2. If not, runs the initial schema migration
/// 3. If yes, verifies the version matches expected
///
/// # Errors
///
/// Returns `StorageError::Migration` if schema creation fails
/// or if the database has an incompatible schema version.
pub fn ensure_schema(conn: &Connection) -> Result<(), StorageError> {
    // Check if schema_version table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| StorageError::Database(e.to_string()))?;

    if !table_exists {
        // Fresh database - run initial schema
        tracing::info!("Initializing database schema (version {})", SCHEMA_VERSION);
        conn.execute_batch(INIT_SCHEMA)
            .map_err(|e| StorageError::Migration(format!("Failed to create schema: {e}")))?;
        return Ok(());
    }

    // Check current version
    let current_version: i32 = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
            row.get(0)
        })
        .map_err(|e| StorageError::Database(e.to_string()))?;

    if current_version > SCHEMA_VERSION {
        return Err(StorageError::Migration(format!(
            "Database schema version {} is newer than supported version {}. \
             Please upgrade cherry2k.",
            current_version, SCHEMA_VERSION
        )));
    }

    if current_version < SCHEMA_VERSION {
        // Future: run incremental migrations here
        // For now, we only have version 1
        tracing::warn!(
            "Database schema version {} is older than expected {}",
            current_version,
            SCHEMA_VERSION
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn schema_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();

        // Verify sessions table exists
        let sessions_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='sessions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(sessions_exists, "sessions table should exist");

        // Verify messages table exists
        let messages_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='messages'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(messages_exists, "messages table should exist");

        // Verify schema version was recorded
        let version: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn schema_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Run schema twice
        ensure_schema(&conn).unwrap();
        ensure_schema(&conn).unwrap();

        // Should still work
        let version: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn indexes_are_created() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();

        // Check indexes exist
        let idx_sessions: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name='idx_sessions_dir_time'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(idx_sessions, "idx_sessions_dir_time index should exist");

        let idx_messages: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name='idx_messages_session'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(idx_messages, "idx_messages_session index should exist");
    }

    #[test]
    fn foreign_key_constraint_works() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        ensure_schema(&conn).unwrap();

        // Inserting a message without a valid session should fail
        let result = conn.execute(
            "INSERT INTO messages (session_id, role, content) VALUES ('nonexistent', 'user', 'test')",
            [],
        );
        assert!(
            result.is_err(),
            "Foreign key constraint should prevent orphan messages"
        );
    }
}
