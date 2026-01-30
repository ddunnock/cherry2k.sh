//! Session management commands.
//!
//! Provides commands for managing conversation sessions:
//! - `resume`: List or resume sessions
//! - `new`: Force create a new session
//! - `clear`: Delete all sessions with confirmation

use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};
use cherry2k_storage::session::{create_session, get_session, list_sessions};
use cherry2k_storage::Database;

/// Resume a session or list available sessions.
///
/// # Arguments
///
/// * `db` - The database connection
/// * `session_id` - Optional specific session ID to resume
/// * `list` - If true, list all sessions instead of resuming
/// * `working_dir` - The current working directory
///
/// # Returns
///
/// Ok(Some(session_id)) if a session was resumed, Ok(None) if listing or no session found.
pub async fn resume(
    db: &Database,
    session_id: Option<&str>,
    list: bool,
    working_dir: &Path,
) -> Result<Option<String>> {
    if list {
        // List all sessions for this directory
        let sessions = list_sessions(db, working_dir, 20)
            .await
            .context("Failed to list sessions")?;

        if sessions.is_empty() {
            println!("No sessions found in this directory.");
            return Ok(None);
        }

        println!("Sessions in {}:", working_dir.display());
        println!();
        println!("{:<22} {:<22} Preview", "ID", "Last Active");
        println!("{}", "-".repeat(70));

        for session in sessions {
            let preview = session
                .first_message_preview
                .as_deref()
                .unwrap_or("(no messages)");
            // Truncate preview to 50 chars
            let preview_truncated = if preview.chars().count() > 50 {
                format!("{}...", preview.chars().take(47).collect::<String>())
            } else {
                preview.to_string()
            };

            println!(
                "{:<22} {:<22} {}",
                session.id,
                session.last_message_at.format("%Y-%m-%d %H:%M"),
                preview_truncated
            );
        }

        return Ok(None);
    }

    if let Some(id) = session_id {
        // Resume specific session
        let session = get_session(db, id)
            .await
            .context("Failed to get session")?;

        match session {
            Some(s) => {
                println!("Resumed session {}", s.id);
                Ok(Some(s.id))
            }
            None => {
                anyhow::bail!("Session not found: {}", id);
            }
        }
    } else {
        // Get most recent session
        let sessions = list_sessions(db, working_dir, 1)
            .await
            .context("Failed to list sessions")?;

        match sessions.first() {
            Some(session) => {
                println!("Resumed session {}", session.id);
                Ok(Some(session.id.clone()))
            }
            None => {
                println!("No sessions found in this directory.");
                Ok(None)
            }
        }
    }
}

/// Create a new session in the current directory.
///
/// # Arguments
///
/// * `db` - The database connection
/// * `working_dir` - The current working directory
///
/// # Returns
///
/// The newly created session ID.
pub async fn new_session(db: &Database, working_dir: &Path) -> Result<String> {
    let session_id = create_session(db, working_dir)
        .await
        .context("Failed to create session")?;

    println!("Created new session {}", session_id);
    Ok(session_id)
}

/// Delete all sessions with user confirmation.
///
/// # Arguments
///
/// * `db` - The database connection
///
/// # Returns
///
/// Ok(()) on success or cancellation.
pub async fn clear(db: &Database) -> Result<()> {
    // Prompt for confirmation
    print!("Delete all sessions? [y/n]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    if input != "y" && input != "yes" {
        println!("Cancelled");
        return Ok(());
    }

    // Get all sessions and delete them
    // We need to iterate through directories, but for simplicity we'll just
    // clean up old sessions and delete the database file
    // Actually, let's use a more targeted approach - delete sessions by listing them

    // For now, use cleanup with a future timestamp to delete everything
    // This is a bit of a hack - we should add a delete_all_sessions function
    // But we can use the cleanup function with a very recent threshold

    // Actually, let's just count sessions in common directories and delete them
    // For a simpler approach, we'll query all session IDs directly

    let count = db
        .call(|conn| {
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;
            if count > 0 {
                conn.execute("DELETE FROM messages", [])?;
                conn.execute("DELETE FROM sessions", [])?;
            }
            Ok(count)
        })
        .await
        .context("Failed to delete sessions")?;

    if count > 0 {
        println!("Deleted {} session(s)", count);
    } else {
        println!("No sessions to delete");
    }

    Ok(())
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

    mod new_session {
        use super::*;

        #[tokio::test]
        async fn creates_session() {
            let (db, temp_dir) = setup_db().await;
            let working_dir = temp_dir.path();

            let session_id = new_session(&db, working_dir).await.unwrap();

            assert!(!session_id.is_empty());
        }
    }

    mod resume {
        use super::*;

        #[tokio::test]
        async fn returns_none_for_no_sessions() {
            let (db, temp_dir) = setup_db().await;
            let working_dir = temp_dir.path();

            let result = resume(&db, None, false, working_dir).await.unwrap();

            assert!(result.is_none());
        }

        #[tokio::test]
        async fn returns_session_id_when_exists() {
            let (db, temp_dir) = setup_db().await;
            let working_dir = temp_dir.path();

            // Create a session first
            let created_id = new_session(&db, working_dir).await.unwrap();

            // Resume should return that session
            let result = resume(&db, None, false, working_dir).await.unwrap();

            assert_eq!(result, Some(created_id));
        }

        #[tokio::test]
        async fn resumes_specific_session() {
            let (db, temp_dir) = setup_db().await;
            let working_dir = temp_dir.path();

            let created_id = new_session(&db, working_dir).await.unwrap();

            let result = resume(&db, Some(&created_id), false, working_dir)
                .await
                .unwrap();

            assert_eq!(result, Some(created_id));
        }

        #[tokio::test]
        async fn errors_for_nonexistent_session() {
            let (db, temp_dir) = setup_db().await;
            let working_dir = temp_dir.path();

            let result = resume(&db, Some("nonexistent"), false, working_dir).await;

            assert!(result.is_err());
        }
    }
}
