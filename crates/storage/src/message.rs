//! Message repository for storing conversation messages.
//!
//! This module provides CRUD operations for messages within sessions.
//! Messages are stored with their role (user/assistant/system), content,
//! and optional token count for context window management.

use chrono::{DateTime, Utc};
use rusqlite::params;

use cherry2k_core::provider::Role;

use crate::StorageError;
use crate::connection::Database;
use crate::util::parse_datetime;

/// A stored message from the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredMessage {
    /// Unique message identifier (auto-incremented)
    pub id: i64,
    /// The session this message belongs to
    pub session_id: String,
    /// The role of the message sender (user, assistant, system)
    pub role: Role,
    /// The message content
    pub content: String,
    /// Optional token count for context window tracking
    pub token_count: Option<i64>,
    /// Whether this message is a summary of previous messages
    pub is_summary: bool,
    /// When the message was created
    pub created_at: DateTime<Utc>,
}

/// Saves a message to the database and updates the session timestamp.
///
/// This operation is atomic - both the message insert and session timestamp
/// update happen in a single transaction.
///
/// # Arguments
///
/// * `db` - The database connection
/// * `session_id` - The session to add the message to
/// * `role` - The message role (user, assistant, system)
/// * `content` - The message content
/// * `token_count` - Optional token count for context tracking
///
/// # Returns
///
/// The newly created message ID.
///
/// # Errors
///
/// Returns `StorageError::Database` if the insert fails (e.g., invalid session_id).
pub async fn save_message(
    db: &Database,
    session_id: &str,
    role: Role,
    content: &str,
    token_count: Option<i64>,
) -> Result<i64, StorageError> {
    let session_id = session_id.to_string();
    let role_str = role.to_string();
    let content = content.to_string();

    db.call(move |conn| {
        let tx = conn.transaction()?;

        // Insert the message
        tx.execute(
            "INSERT INTO messages (session_id, role, content, token_count, is_summary)
             VALUES (?1, ?2, ?3, ?4, 0)",
            params![session_id, role_str, content, token_count],
        )?;

        let message_id = tx.last_insert_rowid();

        // Update the session timestamp
        tx.execute(
            "UPDATE sessions SET last_message_at = datetime('now') WHERE id = ?1",
            params![session_id],
        )?;

        tx.commit()?;

        Ok(message_id)
    })
    .await
    .map_err(|e| StorageError::Database(e.to_string()))
}

/// Saves a summary message to the database.
///
/// Summary messages are marked with `is_summary=true` and have the System role.
/// They represent condensed versions of previous conversation history.
///
/// # Arguments
///
/// * `db` - The database connection
/// * `session_id` - The session to add the summary to
/// * `summary_content` - The summary text
///
/// # Returns
///
/// The newly created message ID.
///
/// # Errors
///
/// Returns `StorageError::Database` if the insert fails.
pub async fn save_summary(
    db: &Database,
    session_id: &str,
    summary_content: &str,
) -> Result<i64, StorageError> {
    let session_id = session_id.to_string();
    let content = summary_content.to_string();

    db.call(move |conn| {
        conn.execute(
            "INSERT INTO messages (session_id, role, content, is_summary)
             VALUES (?1, 'system', ?2, 1)",
            params![session_id, content],
        )?;

        Ok(conn.last_insert_rowid())
    })
    .await
    .map_err(|e| StorageError::Database(e.to_string()))
}

/// Retrieves all messages for a session.
///
/// Messages are ordered by creation time (oldest first).
///
/// # Arguments
///
/// * `db` - The database connection
/// * `session_id` - The session to retrieve messages for
///
/// # Returns
///
/// A vector of stored messages, ordered by created_at ASC.
///
/// # Errors
///
/// Returns `StorageError::Database` if the query fails.
pub async fn get_messages(
    db: &Database,
    session_id: &str,
) -> Result<Vec<StoredMessage>, StorageError> {
    let session_id = session_id.to_string();

    db.call(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, token_count, is_summary, created_at
             FROM messages
             WHERE session_id = ?1
             ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            let role_str: String = row.get(2)?;
            let is_summary_int: i64 = row.get(5)?;
            let created_at_str: String = row.get(6)?;

            Ok(StoredMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: parse_role(&role_str),
                content: row.get(3)?,
                token_count: row.get(4)?,
                is_summary: is_summary_int != 0,
                created_at: parse_datetime(&created_at_str),
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
    })
    .await
    .map_err(|e| StorageError::Database(e.to_string()))
}

/// Retrieves messages created after a given timestamp.
///
/// Useful for fetching only new messages since the last sync.
///
/// # Arguments
///
/// * `db` - The database connection
/// * `session_id` - The session to retrieve messages for
/// * `since` - Only return messages created after this time
///
/// # Returns
///
/// A vector of stored messages, ordered by created_at ASC.
///
/// # Errors
///
/// Returns `StorageError::Database` if the query fails.
pub async fn get_messages_since(
    db: &Database,
    session_id: &str,
    since: DateTime<Utc>,
) -> Result<Vec<StoredMessage>, StorageError> {
    let session_id = session_id.to_string();
    let since_str = since.format("%Y-%m-%d %H:%M:%S").to_string();

    db.call(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, token_count, is_summary, created_at
             FROM messages
             WHERE session_id = ?1 AND created_at > ?2
             ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map(params![session_id, since_str], |row| {
            let role_str: String = row.get(2)?;
            let is_summary_int: i64 = row.get(5)?;
            let created_at_str: String = row.get(6)?;

            Ok(StoredMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: parse_role(&role_str),
                content: row.get(3)?,
                token_count: row.get(4)?,
                is_summary: is_summary_int != 0,
                created_at: parse_datetime(&created_at_str),
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
    })
    .await
    .map_err(|e| StorageError::Database(e.to_string()))
}

/// Counts the number of messages in a session.
///
/// # Arguments
///
/// * `db` - The database connection
/// * `session_id` - The session to count messages for
///
/// # Returns
///
/// The number of messages in the session.
///
/// # Errors
///
/// Returns `StorageError::Database` if the query fails.
pub async fn count_messages(db: &Database, session_id: &str) -> Result<i64, StorageError> {
    let session_id = session_id.to_string();

    db.call(move |conn| {
        conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )
    })
    .await
    .map_err(|e| StorageError::Database(e.to_string()))
}

/// Deletes messages with ID less than the given ID.
///
/// This is used after summarization to remove old messages that have been
/// condensed into a summary.
///
/// # Arguments
///
/// * `db` - The database connection
/// * `session_id` - The session to delete messages from
/// * `before_id` - Delete messages with id < this value
///
/// # Returns
///
/// The number of messages deleted.
///
/// # Errors
///
/// Returns `StorageError::Database` if the delete fails.
pub async fn delete_messages_before(
    db: &Database,
    session_id: &str,
    before_id: i64,
) -> Result<usize, StorageError> {
    let session_id = session_id.to_string();

    db.call(move |conn| {
        conn.execute(
            "DELETE FROM messages WHERE session_id = ?1 AND id < ?2",
            params![session_id, before_id],
        )
    })
    .await
    .map_err(|e| StorageError::Database(e.to_string()))
}

/// Parses a role string into a Role enum.
///
/// Falls back to `Role::User` for unknown role strings.
fn parse_role(s: &str) -> Role {
    match s {
        "user" => Role::User,
        "assistant" => Role::Assistant,
        "system" => Role::System,
        other => {
            tracing::debug!("Unknown role '{}', defaulting to User", other);
            Role::User
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::create_session;
    use std::path::Path;
    use tempfile::TempDir;

    async fn setup_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open_at(db_path).await.unwrap();
        (db, temp_dir)
    }

    async fn setup_with_session() -> (Database, TempDir, String) {
        let (db, temp_dir) = setup_db().await;
        let working_dir = Path::new("/test/messages");
        let session_id = create_session(&db, working_dir).await.unwrap();
        (db, temp_dir, session_id)
    }

    mod save_message {
        use super::*;

        #[tokio::test]
        async fn saves_user_message() {
            let (db, _temp, session_id) = setup_with_session().await;

            let msg_id = save_message(&db, &session_id, Role::User, "Hello", None)
                .await
                .unwrap();

            assert!(msg_id > 0);
        }

        #[tokio::test]
        async fn saves_assistant_message() {
            let (db, _temp, session_id) = setup_with_session().await;

            let msg_id = save_message(&db, &session_id, Role::Assistant, "Hi there!", None)
                .await
                .unwrap();

            assert!(msg_id > 0);
        }

        #[tokio::test]
        async fn saves_with_token_count() {
            let (db, _temp, session_id) = setup_with_session().await;

            let msg_id = save_message(&db, &session_id, Role::User, "Test", Some(42))
                .await
                .unwrap();

            let messages = get_messages(&db, &session_id).await.unwrap();
            assert_eq!(messages[0].token_count, Some(42));
            assert_eq!(messages[0].id, msg_id);
        }

        #[tokio::test]
        async fn updates_session_timestamp() {
            let (db, _temp, session_id) = setup_with_session().await;

            // Get initial timestamp
            let session_before = crate::session::get_session(&db, &session_id)
                .await
                .unwrap()
                .unwrap();

            // Small delay
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Save a message
            save_message(&db, &session_id, Role::User, "Test", None)
                .await
                .unwrap();

            // Get updated timestamp
            let session_after = crate::session::get_session(&db, &session_id)
                .await
                .unwrap()
                .unwrap();

            assert!(session_after.last_message_at >= session_before.last_message_at);
        }

        #[tokio::test]
        async fn errors_for_invalid_session() {
            let (db, _temp) = setup_db().await;

            let result = save_message(&db, "nonexistent", Role::User, "Test", None).await;

            assert!(result.is_err());
        }
    }

    mod save_summary {
        use super::*;

        #[tokio::test]
        async fn saves_summary_message() {
            let (db, _temp, session_id) = setup_with_session().await;

            let msg_id = save_summary(&db, &session_id, "Summary of conversation")
                .await
                .unwrap();

            assert!(msg_id > 0);

            let messages = get_messages(&db, &session_id).await.unwrap();
            assert_eq!(messages.len(), 1);
            assert!(messages[0].is_summary);
            assert_eq!(messages[0].role, Role::System);
            assert_eq!(messages[0].content, "Summary of conversation");
        }
    }

    mod get_messages {
        use super::*;

        #[tokio::test]
        async fn returns_empty_for_no_messages() {
            let (db, _temp, session_id) = setup_with_session().await;

            let messages = get_messages(&db, &session_id).await.unwrap();

            assert!(messages.is_empty());
        }

        #[tokio::test]
        async fn returns_messages_in_order() {
            let (db, _temp, session_id) = setup_with_session().await;

            save_message(&db, &session_id, Role::User, "First", None)
                .await
                .unwrap();
            save_message(&db, &session_id, Role::Assistant, "Second", None)
                .await
                .unwrap();
            save_message(&db, &session_id, Role::User, "Third", None)
                .await
                .unwrap();

            let messages = get_messages(&db, &session_id).await.unwrap();

            assert_eq!(messages.len(), 3);
            assert_eq!(messages[0].content, "First");
            assert_eq!(messages[1].content, "Second");
            assert_eq!(messages[2].content, "Third");
        }

        #[tokio::test]
        async fn parses_roles_correctly() {
            let (db, _temp, session_id) = setup_with_session().await;

            save_message(&db, &session_id, Role::User, "User msg", None)
                .await
                .unwrap();
            save_message(&db, &session_id, Role::Assistant, "Assistant msg", None)
                .await
                .unwrap();
            save_message(&db, &session_id, Role::System, "System msg", None)
                .await
                .unwrap();

            let messages = get_messages(&db, &session_id).await.unwrap();

            assert_eq!(messages[0].role, Role::User);
            assert_eq!(messages[1].role, Role::Assistant);
            assert_eq!(messages[2].role, Role::System);
        }
    }

    mod get_messages_since {
        use super::*;
        use chrono::Duration;

        #[tokio::test]
        async fn returns_only_newer_messages() {
            let (db, _temp, session_id) = setup_with_session().await;

            // Save a message
            save_message(&db, &session_id, Role::User, "Old message", None)
                .await
                .unwrap();

            // Get timestamp after first message
            let cutoff = Utc::now();

            // Wait over 1 second since SQLite datetime('now') has second precision
            tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

            // Save another message
            save_message(&db, &session_id, Role::User, "New message", None)
                .await
                .unwrap();

            let messages = get_messages_since(&db, &session_id, cutoff).await.unwrap();

            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0].content, "New message");
        }

        #[tokio::test]
        async fn returns_empty_if_none_newer() {
            let (db, _temp, session_id) = setup_with_session().await;

            save_message(&db, &session_id, Role::User, "Old message", None)
                .await
                .unwrap();

            // Cutoff in the future
            let cutoff = Utc::now() + Duration::hours(1);

            let messages = get_messages_since(&db, &session_id, cutoff).await.unwrap();

            assert!(messages.is_empty());
        }
    }

    mod count_messages {
        use super::*;

        #[tokio::test]
        async fn returns_zero_for_empty_session() {
            let (db, _temp, session_id) = setup_with_session().await;

            let count = count_messages(&db, &session_id).await.unwrap();

            assert_eq!(count, 0);
        }

        #[tokio::test]
        async fn returns_correct_count() {
            let (db, _temp, session_id) = setup_with_session().await;

            save_message(&db, &session_id, Role::User, "One", None)
                .await
                .unwrap();
            save_message(&db, &session_id, Role::Assistant, "Two", None)
                .await
                .unwrap();
            save_message(&db, &session_id, Role::User, "Three", None)
                .await
                .unwrap();

            let count = count_messages(&db, &session_id).await.unwrap();

            assert_eq!(count, 3);
        }
    }

    mod delete_messages_before {
        use super::*;

        #[tokio::test]
        async fn deletes_older_messages() {
            let (db, _temp, session_id) = setup_with_session().await;

            let _id1 = save_message(&db, &session_id, Role::User, "First", None)
                .await
                .unwrap();
            let id2 = save_message(&db, &session_id, Role::Assistant, "Second", None)
                .await
                .unwrap();
            let _id3 = save_message(&db, &session_id, Role::User, "Third", None)
                .await
                .unwrap();

            // Delete messages before id2
            let deleted = delete_messages_before(&db, &session_id, id2).await.unwrap();

            assert_eq!(deleted, 1); // Only id1 deleted

            let messages = get_messages(&db, &session_id).await.unwrap();
            assert_eq!(messages.len(), 2);
            assert_eq!(messages[0].content, "Second");
            assert_eq!(messages[1].content, "Third");
        }

        #[tokio::test]
        async fn returns_zero_if_none_deleted() {
            let (db, _temp, session_id) = setup_with_session().await;

            save_message(&db, &session_id, Role::User, "Only message", None)
                .await
                .unwrap();

            // Try to delete with before_id = 1 (nothing has id < 1)
            let deleted = delete_messages_before(&db, &session_id, 1).await.unwrap();

            assert_eq!(deleted, 0);
        }
    }

    mod parse_role {
        use super::*;

        #[test]
        fn parses_known_roles() {
            assert_eq!(parse_role("user"), Role::User);
            assert_eq!(parse_role("assistant"), Role::Assistant);
            assert_eq!(parse_role("system"), Role::System);
        }

        #[test]
        fn defaults_to_user_for_unknown() {
            assert_eq!(parse_role("unknown"), Role::User);
            assert_eq!(parse_role(""), Role::User);
        }
    }
}
