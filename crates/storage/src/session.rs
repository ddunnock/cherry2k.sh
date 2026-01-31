//! Session repository for managing conversation sessions.
//!
//! This module provides CRUD operations for sessions, which group messages
//! by working directory and time. Sessions auto-continue if the last message
//! was within 4 hours.

use std::path::Path;

use chrono::{DateTime, Duration, Utc};
use rusqlite::OptionalExtension;
use rusqlite::params;

use crate::StorageError;
use crate::connection::Database;
use crate::util::parse_datetime;

/// A full session record from the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Unique session identifier (timestamp-based: "YYYY-MM-DD-HHMM-SSS")
    pub id: String,
    /// The working directory where this session was created
    pub working_dir: String,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the last message was added
    pub last_message_at: DateTime<Utc>,
}

/// A lightweight session info for list views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    /// Unique session identifier
    pub id: String,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the last message was added
    pub last_message_at: DateTime<Utc>,
    /// First 100 characters of the first user message (if any)
    pub first_message_preview: Option<String>,
}

/// Generates a timestamp-based session ID with random suffix.
///
/// Format: "YYYY-MM-DD-HHMM-SSS-XXXX" where SSS is milliseconds and XXXX is
/// a random hex suffix for collision avoidance.
///
/// # Example
///
/// ```
/// use cherry2k_storage::session::generate_session_id;
///
/// let id = generate_session_id();
/// // e.g., "2026-01-30-1423-456-a3f2"
/// ```
#[must_use]
pub fn generate_session_id() -> String {
    let now = Utc::now();
    let random_suffix: u16 = rand::random();
    format!(
        "{}-{:03}-{:04x}",
        now.format("%Y-%m-%d-%H%M"),
        now.timestamp_subsec_millis(),
        random_suffix
    )
}

/// Validates a session ID format.
///
/// Session IDs should match the format "YYYY-MM-DD-HHMM-SSS-XXXX" (24 chars)
/// where SSS is milliseconds and XXXX is a hex random suffix.
///
/// Also accepts the legacy format "YYYY-MM-DD-HHMM-SSS" (19 chars) for
/// backwards compatibility.
#[must_use]
pub fn is_valid_session_id(id: &str) -> bool {
    // New format: 24 chars (YYYY-MM-DD-HHMM-SSS-XXXX)
    // Legacy format: 19 chars (YYYY-MM-DD-HHMM-SSS)
    let len = id.len();
    if len != 24 && len != 19 {
        return false;
    }

    // All chars must be digits or hyphens (and hex for new format)
    id.chars()
        .all(|c| c.is_ascii_digit() || c == '-' || c.is_ascii_hexdigit())
}

/// Creates a new session for the given working directory.
///
/// # Arguments
///
/// * `db` - The database connection
/// * `working_dir` - The directory path for this session
///
/// # Returns
///
/// The newly created session ID.
///
/// # Errors
///
/// Returns `StorageError::Database` if the insert fails.
pub async fn create_session(db: &Database, working_dir: &Path) -> Result<String, StorageError> {
    let session_id = generate_session_id();
    let working_dir_str = working_dir.to_string_lossy().to_string();

    let id = session_id.clone();
    db.call(move |conn| {
        conn.execute(
            "INSERT INTO sessions (id, working_dir) VALUES (?1, ?2)",
            params![id, working_dir_str],
        )
    })
    .await
    .map_err(|e| StorageError::Database(e.to_string()))?;

    tracing::debug!(
        "Created session {} for {}",
        session_id,
        working_dir.display()
    );
    Ok(session_id)
}

/// Gets an existing session or creates a new one.
///
/// A session is reused if:
/// 1. It's for the same working directory
/// 2. The last message was within the last 4 hours
///
/// Otherwise, a new session is created.
///
/// # Arguments
///
/// * `db` - The database connection
/// * `working_dir` - The directory path for this session
///
/// # Returns
///
/// The session ID (existing or newly created).
///
/// # Errors
///
/// Returns `StorageError::Database` if database operations fail.
pub async fn get_or_create_session(
    db: &Database,
    working_dir: &Path,
) -> Result<String, StorageError> {
    let working_dir_str = working_dir.to_string_lossy().to_string();
    let idle_threshold = Utc::now() - Duration::hours(4);
    let threshold_str = idle_threshold.format("%Y-%m-%d %H:%M:%S").to_string();

    // Try to find an active session for this directory
    let existing_session: Option<String> = db
        .call(move |conn| {
            conn.query_row(
                "SELECT id FROM sessions
                 WHERE working_dir = ?1
                   AND last_message_at >= ?2
                 ORDER BY last_message_at DESC
                 LIMIT 1",
                params![working_dir_str, threshold_str],
                |row| row.get(0),
            )
            .optional()
        })
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

    match existing_session {
        Some(id) => {
            tracing::debug!(
                "Continuing existing session {} for {}",
                id,
                working_dir.display()
            );
            Ok(id)
        }
        None => create_session(db, working_dir).await,
    }
}

/// Retrieves a session by ID.
///
/// # Arguments
///
/// * `db` - The database connection
/// * `session_id` - The session ID to retrieve
///
/// # Returns
///
/// The session if found, or `None` if not found.
///
/// # Errors
///
/// Returns `StorageError::Database` if the query fails.
pub async fn get_session(db: &Database, session_id: &str) -> Result<Option<Session>, StorageError> {
    let id = session_id.to_string();

    db.call(move |conn| {
        conn.query_row(
            "SELECT id, working_dir, created_at, last_message_at
             FROM sessions WHERE id = ?1",
            params![id],
            |row| {
                let created_at_str: String = row.get(2)?;
                let last_message_at_str: String = row.get(3)?;

                Ok(Session {
                    id: row.get(0)?,
                    working_dir: row.get(1)?,
                    created_at: parse_datetime(&created_at_str),
                    last_message_at: parse_datetime(&last_message_at_str),
                })
            },
        )
        .optional()
    })
    .await
    .map_err(|e| StorageError::Database(e.to_string()))
}

/// Lists sessions for a directory with first message preview.
///
/// Sessions are ordered by `last_message_at` descending (most recent first).
///
/// # Arguments
///
/// * `db` - The database connection
/// * `working_dir` - The directory to filter by
/// * `limit` - Maximum number of sessions to return
///
/// # Returns
///
/// A vector of session info with optional first message preview.
///
/// # Errors
///
/// Returns `StorageError::Database` if the query fails.
pub async fn list_sessions(
    db: &Database,
    working_dir: &Path,
    limit: usize,
) -> Result<Vec<SessionInfo>, StorageError> {
    let working_dir_str = working_dir.to_string_lossy().to_string();

    db.call(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT s.id, s.created_at, s.last_message_at,
                    (SELECT SUBSTR(m.content, 1, 100)
                     FROM messages m
                     WHERE m.session_id = s.id AND m.role = 'user'
                     ORDER BY m.created_at ASC
                     LIMIT 1) as preview
             FROM sessions s
             WHERE s.working_dir = ?1
             ORDER BY s.last_message_at DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![working_dir_str, limit as i64], |row| {
            let created_at_str: String = row.get(1)?;
            let last_message_at_str: String = row.get(2)?;

            Ok(SessionInfo {
                id: row.get(0)?,
                created_at: parse_datetime(&created_at_str),
                last_message_at: parse_datetime(&last_message_at_str),
                first_message_preview: row.get(3)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
    })
    .await
    .map_err(|e| StorageError::Database(e.to_string()))
}

/// Updates the session's last_message_at timestamp to now.
///
/// # Arguments
///
/// * `db` - The database connection
/// * `session_id` - The session ID to update
///
/// # Errors
///
/// Returns `StorageError::SessionNotFound` if the session doesn't exist.
/// Returns `StorageError::Database` if the update fails.
pub async fn update_session_timestamp(db: &Database, session_id: &str) -> Result<(), StorageError> {
    let id = session_id.to_string();

    let rows_affected = db
        .call(move |conn| {
            conn.execute(
                "UPDATE sessions SET last_message_at = datetime('now') WHERE id = ?1",
                params![id],
            )
        })
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

    if rows_affected == 0 {
        return Err(StorageError::SessionNotFound {
            id: session_id.to_string(),
        });
    }

    Ok(())
}

/// Deletes a session and all its messages (cascade).
///
/// # Arguments
///
/// * `db` - The database connection
/// * `session_id` - The session ID to delete
///
/// # Errors
///
/// Returns `StorageError::SessionNotFound` if the session doesn't exist.
/// Returns `StorageError::Database` if the delete fails.
pub async fn delete_session(db: &Database, session_id: &str) -> Result<(), StorageError> {
    let id = session_id.to_string();

    let rows_affected = db
        .call(move |conn| conn.execute("DELETE FROM sessions WHERE id = ?1", params![id]))
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

    if rows_affected == 0 {
        return Err(StorageError::SessionNotFound {
            id: session_id.to_string(),
        });
    }

    tracing::debug!("Deleted session {}", session_id);
    Ok(())
}

/// Deletes sessions older than 30 days.
///
/// This should be called periodically to clean up old conversation history.
///
/// # Arguments
///
/// * `db` - The database connection
///
/// # Returns
///
/// The number of sessions deleted.
///
/// # Errors
///
/// Returns `StorageError::Database` if the delete fails.
pub async fn cleanup_old_sessions(db: &Database) -> Result<usize, StorageError> {
    let threshold = Utc::now() - Duration::days(30);
    let threshold_str = threshold.format("%Y-%m-%d %H:%M:%S").to_string();

    let rows_deleted = db
        .call(move |conn| {
            conn.execute(
                "DELETE FROM sessions WHERE last_message_at < ?1",
                params![threshold_str],
            )
        })
        .await
        .map_err(|e| StorageError::Database(e.to_string()))?;

    if rows_deleted > 0 {
        tracing::info!("Cleaned up {} old sessions", rows_deleted);
    }

    Ok(rows_deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open_at(db_path).await.unwrap();
        (db, temp_dir)
    }

    mod generate_session_id {
        use super::*;

        #[test]
        fn generates_valid_format() {
            let id = generate_session_id();
            // Format: YYYY-MM-DD-HHMM-SSS-XXXX (24 chars)
            assert_eq!(id.len(), 24, "ID should be 24 characters: {id}");
            assert!(id.contains('-'), "ID should contain dashes");

            // Parse parts
            let parts: Vec<&str> = id.split('-').collect();
            assert_eq!(parts.len(), 6, "Should have 6 parts separated by dashes");

            // Year
            let year: i32 = parts[0].parse().unwrap();
            assert!(year >= 2024, "Year should be reasonable");

            // Month (01-12)
            let month: u32 = parts[1].parse().unwrap();
            assert!((1..=12).contains(&month), "Month should be 1-12");

            // Day (01-31)
            let day: u32 = parts[2].parse().unwrap();
            assert!((1..=31).contains(&day), "Day should be 1-31");

            // Hour+Minute (0000-2359)
            let hhmm: u32 = parts[3].parse().unwrap();
            assert!(hhmm <= 2359, "HHMM should be <= 2359");

            // Milliseconds (000-999)
            let ms: u32 = parts[4].parse().unwrap();
            assert!(ms <= 999, "Milliseconds should be 0-999");

            // Random hex suffix (4 hex digits)
            assert_eq!(parts[5].len(), 4, "Hex suffix should be 4 characters");
            assert!(
                parts[5].chars().all(|c| c.is_ascii_hexdigit()),
                "Suffix should be hex digits"
            );
        }

        #[test]
        fn generates_unique_ids() {
            // IDs should always be unique due to random suffix
            let id1 = generate_session_id();
            let id2 = generate_session_id();

            // Random suffix makes collisions extremely unlikely
            assert_ne!(id1, id2, "IDs should differ due to random suffix");
        }
    }

    mod validate_session_id {
        use super::*;

        #[test]
        fn accepts_new_format() {
            assert!(is_valid_session_id("2026-01-30-1423-456-a3f2"));
        }

        #[test]
        fn accepts_legacy_format() {
            assert!(is_valid_session_id("2026-01-30-1423-456"));
        }

        #[test]
        fn rejects_too_short() {
            assert!(!is_valid_session_id("2026-01-30"));
        }

        #[test]
        fn rejects_too_long() {
            assert!(!is_valid_session_id("2026-01-30-1423-456-a3f2-extra"));
        }

        #[test]
        fn rejects_invalid_chars() {
            assert!(!is_valid_session_id("2026-01-30-1423-45x"));
            assert!(!is_valid_session_id("hello-world-test-1234"));
        }

        #[test]
        fn validates_generated_ids() {
            let id = generate_session_id();
            assert!(is_valid_session_id(&id), "Generated ID should be valid: {id}");
        }
    }

    mod create_session {
        use super::*;

        #[tokio::test]
        async fn creates_session_with_id() {
            let (db, _temp) = setup_db().await;
            let working_dir = Path::new("/test/path");

            let id = create_session(&db, working_dir).await.unwrap();

            assert!(!id.is_empty());
            assert_eq!(id.len(), 24, "Session ID should be 24 characters: {id}");
            assert!(is_valid_session_id(&id), "Session ID should be valid format");
        }

        #[tokio::test]
        async fn stores_working_directory() {
            let (db, _temp) = setup_db().await;
            let working_dir = Path::new("/test/path");

            let id = create_session(&db, working_dir).await.unwrap();
            let session = get_session(&db, &id).await.unwrap().unwrap();

            assert_eq!(session.working_dir, "/test/path");
        }
    }

    mod get_or_create_session {
        use super::*;
        use std::time::Duration as StdDuration;
        use tokio::time::sleep;

        #[tokio::test]
        async fn creates_new_session_when_none_exists() {
            let (db, _temp) = setup_db().await;
            let working_dir = Path::new("/test/new");

            let id = get_or_create_session(&db, working_dir).await.unwrap();

            assert!(!id.is_empty());
            let session = get_session(&db, &id).await.unwrap();
            assert!(session.is_some());
        }

        #[tokio::test]
        async fn reuses_recent_session() {
            let (db, _temp) = setup_db().await;
            let working_dir = Path::new("/test/reuse");

            // Create first session
            let id1 = create_session(&db, working_dir).await.unwrap();
            // Update timestamp to ensure it's recent
            update_session_timestamp(&db, &id1).await.unwrap();

            // Small delay to ensure different timestamp
            sleep(StdDuration::from_millis(10)).await;

            // Should reuse the existing session
            let id2 = get_or_create_session(&db, working_dir).await.unwrap();

            assert_eq!(id1, id2);
        }

        #[tokio::test]
        async fn creates_new_for_different_directory() {
            let (db, _temp) = setup_db().await;
            let dir1 = Path::new("/test/dir1");
            let dir2 = Path::new("/test/dir2");

            let id1 = get_or_create_session(&db, dir1).await.unwrap();
            let id2 = get_or_create_session(&db, dir2).await.unwrap();

            assert_ne!(id1, id2);
        }
    }

    mod get_session {
        use super::*;

        #[tokio::test]
        async fn returns_none_for_nonexistent() {
            let (db, _temp) = setup_db().await;

            let session = get_session(&db, "nonexistent").await.unwrap();

            assert!(session.is_none());
        }

        #[tokio::test]
        async fn returns_session_with_all_fields() {
            let (db, _temp) = setup_db().await;
            let working_dir = Path::new("/test/fields");

            let id = create_session(&db, working_dir).await.unwrap();
            let session = get_session(&db, &id).await.unwrap().unwrap();

            assert_eq!(session.id, id);
            assert_eq!(session.working_dir, "/test/fields");
            // created_at and last_message_at should be recent
            let now = Utc::now();
            let diff = now - session.created_at;
            assert!(
                diff.num_seconds() < 60,
                "Session should be created recently"
            );
        }
    }

    mod list_sessions {
        use super::*;

        #[tokio::test]
        async fn returns_empty_for_no_sessions() {
            let (db, _temp) = setup_db().await;
            let working_dir = Path::new("/test/empty");

            let sessions = list_sessions(&db, working_dir, 10).await.unwrap();

            assert!(sessions.is_empty());
        }

        #[tokio::test]
        async fn returns_sessions_for_directory() {
            let (db, _temp) = setup_db().await;
            let working_dir = Path::new("/test/list");

            // Insert sessions with known unique IDs to avoid collision
            db.call(|conn| {
                conn.execute(
                    "INSERT INTO sessions (id, working_dir) VALUES ('list-test-1', ?1)",
                    params!["/test/list"],
                )?;
                conn.execute(
                    "INSERT INTO sessions (id, working_dir) VALUES ('list-test-2', ?1)",
                    params!["/test/list"],
                )?;
                Ok(())
            })
            .await
            .unwrap();

            let sessions = list_sessions(&db, working_dir, 10).await.unwrap();

            assert_eq!(sessions.len(), 2);
        }

        #[tokio::test]
        async fn respects_limit() {
            let (db, _temp) = setup_db().await;
            let working_dir = Path::new("/test/limit");

            // Create multiple sessions by using different directories
            for i in 0..5 {
                // Force new session by using unique milliseconds
                let id = format!("test-session-{i}");
                db.call(move |conn| {
                    conn.execute(
                        "INSERT INTO sessions (id, working_dir) VALUES (?1, ?2)",
                        params![id, "/test/limit"],
                    )
                })
                .await
                .unwrap();
            }

            let sessions = list_sessions(&db, working_dir, 3).await.unwrap();

            assert_eq!(sessions.len(), 3);
        }

        #[tokio::test]
        async fn ordered_by_last_message_desc() {
            let (db, _temp) = setup_db().await;
            let working_dir = Path::new("/test/order");

            // Create sessions with known timestamps
            db.call(|conn| {
                conn.execute(
                    "INSERT INTO sessions (id, working_dir, last_message_at) VALUES ('old', ?1, '2020-01-01 00:00:00')",
                    params!["/test/order"],
                )?;
                conn.execute(
                    "INSERT INTO sessions (id, working_dir, last_message_at) VALUES ('new', ?1, '2025-01-01 00:00:00')",
                    params!["/test/order"],
                )?;
                Ok(())
            }).await.unwrap();

            let sessions = list_sessions(&db, working_dir, 10).await.unwrap();

            assert_eq!(sessions.len(), 2);
            assert_eq!(sessions[0].id, "new");
            assert_eq!(sessions[1].id, "old");
        }
    }

    mod update_session_timestamp {
        use super::*;

        #[tokio::test]
        async fn updates_timestamp() {
            let (db, _temp) = setup_db().await;
            let working_dir = Path::new("/test/update");

            let id = create_session(&db, working_dir).await.unwrap();
            let before = get_session(&db, &id).await.unwrap().unwrap();

            // Small delay
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            update_session_timestamp(&db, &id).await.unwrap();

            let after = get_session(&db, &id).await.unwrap().unwrap();

            // last_message_at should be updated (or at least not earlier)
            assert!(after.last_message_at >= before.last_message_at);
        }

        #[tokio::test]
        async fn errors_for_nonexistent() {
            let (db, _temp) = setup_db().await;

            let result = update_session_timestamp(&db, "nonexistent").await;

            assert!(matches!(result, Err(StorageError::SessionNotFound { .. })));
        }
    }

    mod delete_session {
        use super::*;

        #[tokio::test]
        async fn deletes_session() {
            let (db, _temp) = setup_db().await;
            let working_dir = Path::new("/test/delete");

            let id = create_session(&db, working_dir).await.unwrap();
            delete_session(&db, &id).await.unwrap();

            let session = get_session(&db, &id).await.unwrap();
            assert!(session.is_none());
        }

        #[tokio::test]
        async fn errors_for_nonexistent() {
            let (db, _temp) = setup_db().await;

            let result = delete_session(&db, "nonexistent").await;

            assert!(matches!(result, Err(StorageError::SessionNotFound { .. })));
        }
    }

    mod cleanup_old_sessions {
        use super::*;

        #[tokio::test]
        async fn deletes_old_sessions() {
            let (db, _temp) = setup_db().await;

            // Insert an old session
            db.call(|conn| {
                conn.execute(
                    "INSERT INTO sessions (id, working_dir, last_message_at) VALUES ('old', '/test', '2020-01-01 00:00:00')",
                    [],
                )
            })
            .await
            .unwrap();

            let count = cleanup_old_sessions(&db).await.unwrap();

            assert_eq!(count, 1);
        }

        #[tokio::test]
        async fn keeps_recent_sessions() {
            let (db, _temp) = setup_db().await;
            let working_dir = Path::new("/test/keep");

            let id = create_session(&db, working_dir).await.unwrap();

            let count = cleanup_old_sessions(&db).await.unwrap();

            assert_eq!(count, 0);
            let session = get_session(&db, &id).await.unwrap();
            assert!(session.is_some());
        }
    }
}
