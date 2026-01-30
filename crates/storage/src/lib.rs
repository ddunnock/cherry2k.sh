//! Cherry2K Storage Layer
//!
//! This crate provides SQLite-based persistence for Cherry2K, including:
//! - Conversation history storage
//! - Session management
//! - User preferences
//!
//! # Usage
//!
//! ```no_run
//! use cherry2k_storage::Database;
//!
//! # async fn example() -> Result<(), cherry2k_core::StorageError> {
//! // Open the database (creates it if needed)
//! let db = Database::open().await?;
//!
//! // Use db.call() for database operations
//! let count: i64 = db.call(|conn| {
//!     conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
//! }).await.map_err(|e| cherry2k_core::StorageError::Database(e.to_string()))?;
//! # Ok(())
//! # }
//! ```
//!
//! # Database Location
//!
//! The database is stored in the XDG data directory:
//! - Linux: `~/.local/share/cherry2k/sessions.db`
//! - macOS: `~/Library/Application Support/cherry2k/sessions.db`
//!
//! # Security
//!
//! The database file is created with 0600 permissions (owner read/write only)
//! to protect conversation history.

mod connection;
mod schema;

// Re-export the main types
pub use connection::Database;

// Re-export core error types for convenience
pub use cherry2k_core::StorageError;
