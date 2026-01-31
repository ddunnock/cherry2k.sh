//! Context window management with LLM-based summarization.
//!
//! This module handles conversation context preparation for AI providers,
//! including token estimation and automatic summarization when approaching
//! context limits.
//!
//! # Token Budget
//!
//! The context window is managed with a 16,000 token budget. When the
//! conversation exceeds 75% of this budget, older messages are summarized
//! using the AI provider to preserve context while staying within limits.

use futures::StreamExt;

use cherry2k_core::provider::{AiProvider, CompletionRequest, Message, Role};

use crate::Database;
use crate::StorageError;
use crate::message::{StoredMessage, get_messages};

/// Maximum tokens for conversation history.
const TOKEN_BUDGET: usize = 16_000;

/// Trigger summarization at 75% of token budget.
const SUMMARIZE_THRESHOLD: f32 = 0.75;

/// Conservative estimate: 4 characters per token.
const CHARS_PER_TOKEN: usize = 4;

/// Prompt template for summarizing conversation history.
const SUMMARIZATION_PROMPT: &str = r#"Summarize the following conversation history, preserving:
- Key facts and decisions made
- User's goals and preferences
- Unresolved questions or issues
- Technical context (file paths, commands, errors)

Be concise but preserve critical context. The summary will replace these messages.

Conversation:
{conversation}

Summary:"#;

/// Result of context preparation.
///
/// Contains the messages ready to send to the provider and indicates
/// whether summarization occurred.
#[derive(Debug, Clone)]
#[must_use = "ContextResult contains was_summarized flag that should be checked"]
pub struct ContextResult {
    /// Messages to send to provider (converted from StoredMessage).
    pub messages: Vec<Message>,
    /// True if summarization occurred during preparation.
    pub was_summarized: bool,
}

/// Estimates token count for a list of messages.
///
/// Uses a conservative heuristic of 4 characters per token.
/// This is a simple but sufficient approximation for Phase 03.
///
/// # Arguments
///
/// * `messages` - The messages to estimate tokens for
///
/// # Returns
///
/// Estimated token count.
#[must_use]
pub fn estimate_tokens(messages: &[StoredMessage]) -> usize {
    let total_chars: usize = messages.iter().map(|m| m.content.len()).sum();
    total_chars / CHARS_PER_TOKEN
}

/// Formats messages for summarization.
///
/// Creates a readable format: "Role: content\n\n" for each message.
fn format_for_summary(messages: &[StoredMessage]) -> String {
    messages
        .iter()
        .map(|m| format!("{}: {}", role_to_string(m.role), m.content))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Converts Role enum to title case string for summarization.
fn role_to_string(role: Role) -> &'static str {
    match role {
        Role::User => "User",
        Role::Assistant => "Assistant",
        Role::System => "System",
    }
}

/// Converts a StoredMessage to a provider Message.
fn stored_to_message(stored: &StoredMessage) -> Message {
    Message::new(stored.role, &stored.content)
}

/// Prepares conversation context for the AI provider.
///
/// Loads messages for the session and checks if summarization is needed.
/// If the estimated token count exceeds 75% of the budget, older messages
/// are summarized using the provider.
///
/// # Arguments
///
/// * `db` - The database connection
/// * `session_id` - The session to load context for
/// * `provider` - The AI provider to use for summarization
///
/// # Returns
///
/// A `ContextResult` containing messages ready for the provider and
/// a flag indicating if summarization occurred.
///
/// # Errors
///
/// Returns `StorageError` if database operations fail or summarization fails.
pub async fn prepare_context(
    db: &Database,
    session_id: &str,
    provider: &dyn AiProvider,
) -> Result<ContextResult, StorageError> {
    // Load all messages for the session
    let messages = get_messages(db, session_id).await?;

    // Check if we're under the threshold
    let estimated_tokens = estimate_tokens(&messages);
    let threshold_tokens = ((TOKEN_BUDGET as f32) * SUMMARIZE_THRESHOLD) as usize;

    if estimated_tokens < threshold_tokens {
        // Under threshold - convert and return without summarization
        let provider_messages: Vec<Message> = messages.iter().map(stored_to_message).collect();
        return Ok(ContextResult {
            messages: provider_messages,
            was_summarized: false,
        });
    }

    // Over threshold - need to summarize
    tracing::info!(
        "Context exceeds threshold ({} tokens > {}), summarizing...",
        estimated_tokens,
        threshold_tokens
    );

    // Split messages at 50% point
    let split_point = messages.len() / 2;
    let (old_messages, recent_messages) = messages.split_at(split_point);

    // Get the ID of the first message to keep (for deletion)
    let first_kept_id = if recent_messages.is_empty() {
        i64::MAX
    } else {
        recent_messages[0].id
    };

    // Format old messages for summarization
    let conversation_text = format_for_summary(old_messages);
    let prompt = SUMMARIZATION_PROMPT.replace("{conversation}", &conversation_text);

    // Call provider to get summary
    let request = CompletionRequest::new()
        .with_message(Message::user(&prompt))
        .with_max_tokens(1000);

    let stream = provider
        .complete(request)
        .await
        .map_err(|e| StorageError::Database(format!("Summarization failed: {e}")))?;

    // Collect the summary from the stream
    tokio::pin!(stream);
    let mut summary = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(text) => summary.push_str(&text),
            Err(e) => {
                return Err(StorageError::Database(format!(
                    "Summarization stream error: {e}"
                )));
            }
        }
    }

    // Atomically delete old messages and save summary in a single transaction
    // This prevents data loss if save_summary fails after deletion
    let session_id_owned = session_id.to_string();
    let summary_clone = summary.clone();
    db.call(move |conn| {
        let tx = conn.transaction()?;

        // Delete old messages
        let deleted = tx.execute(
            "DELETE FROM messages WHERE session_id = ?1 AND id < ?2",
            rusqlite::params![session_id_owned, first_kept_id],
        )?;
        tracing::debug!("Deleted {} old messages during summarization", deleted);

        // Save summary as system message
        tx.execute(
            "INSERT INTO messages (session_id, role, content, is_summary) VALUES (?1, 'system', ?2, 1)",
            rusqlite::params![session_id_owned, summary_clone],
        )?;

        tx.commit()?;
        Ok(())
    })
    .await
    .map_err(|e| StorageError::Database(format!("Failed to save summary: {e}")))?;

    // Build final message list: summary + recent messages
    let mut result_messages = Vec::with_capacity(recent_messages.len() + 1);
    result_messages.push(Message::system(&summary));
    result_messages.extend(recent_messages.iter().map(stored_to_message));

    Ok(ContextResult {
        messages: result_messages,
        was_summarized: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::save_message;
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
        let working_dir = Path::new("/test/context");
        let session_id = create_session(&db, working_dir).await.unwrap();
        (db, temp_dir, session_id)
    }

    mod estimate_tokens {
        use super::*;

        #[test]
        fn empty_messages_returns_zero() {
            let messages: Vec<StoredMessage> = vec![];
            assert_eq!(estimate_tokens(&messages), 0);
        }

        #[test]
        fn estimates_with_4_chars_per_token() {
            // 100 chars / 4 = 25 tokens
            let messages = vec![StoredMessage {
                id: 1,
                session_id: "test".to_string(),
                role: Role::User,
                content: "a".repeat(100),
                token_count: None,
                is_summary: false,
                created_at: chrono::Utc::now(),
            }];
            assert_eq!(estimate_tokens(&messages), 25);
        }

        #[test]
        fn sums_across_messages() {
            let messages = vec![
                StoredMessage {
                    id: 1,
                    session_id: "test".to_string(),
                    role: Role::User,
                    content: "a".repeat(40), // 10 tokens
                    token_count: None,
                    is_summary: false,
                    created_at: chrono::Utc::now(),
                },
                StoredMessage {
                    id: 2,
                    session_id: "test".to_string(),
                    role: Role::Assistant,
                    content: "b".repeat(80), // 20 tokens
                    token_count: None,
                    is_summary: false,
                    created_at: chrono::Utc::now(),
                },
            ];
            // Total: 120 chars / 4 = 30 tokens
            assert_eq!(estimate_tokens(&messages), 30);
        }
    }

    mod format_for_summary {
        use super::*;

        #[test]
        fn formats_single_message() {
            let messages = vec![StoredMessage {
                id: 1,
                session_id: "test".to_string(),
                role: Role::User,
                content: "Hello".to_string(),
                token_count: None,
                is_summary: false,
                created_at: chrono::Utc::now(),
            }];
            let formatted = format_for_summary(&messages);
            assert_eq!(formatted, "User: Hello");
        }

        #[test]
        fn formats_multiple_messages() {
            let messages = vec![
                StoredMessage {
                    id: 1,
                    session_id: "test".to_string(),
                    role: Role::User,
                    content: "Hi".to_string(),
                    token_count: None,
                    is_summary: false,
                    created_at: chrono::Utc::now(),
                },
                StoredMessage {
                    id: 2,
                    session_id: "test".to_string(),
                    role: Role::Assistant,
                    content: "Hello!".to_string(),
                    token_count: None,
                    is_summary: false,
                    created_at: chrono::Utc::now(),
                },
            ];
            let formatted = format_for_summary(&messages);
            assert_eq!(formatted, "User: Hi\n\nAssistant: Hello!");
        }

        #[test]
        fn handles_all_roles() {
            let messages = vec![
                StoredMessage {
                    id: 1,
                    session_id: "test".to_string(),
                    role: Role::System,
                    content: "System".to_string(),
                    token_count: None,
                    is_summary: false,
                    created_at: chrono::Utc::now(),
                },
                StoredMessage {
                    id: 2,
                    session_id: "test".to_string(),
                    role: Role::User,
                    content: "User".to_string(),
                    token_count: None,
                    is_summary: false,
                    created_at: chrono::Utc::now(),
                },
                StoredMessage {
                    id: 3,
                    session_id: "test".to_string(),
                    role: Role::Assistant,
                    content: "Assistant".to_string(),
                    token_count: None,
                    is_summary: false,
                    created_at: chrono::Utc::now(),
                },
            ];
            let formatted = format_for_summary(&messages);
            assert!(formatted.contains("System: System"));
            assert!(formatted.contains("User: User"));
            assert!(formatted.contains("Assistant: Assistant"));
        }
    }

    mod stored_to_message {
        use super::*;

        #[test]
        fn converts_correctly() {
            let stored = StoredMessage {
                id: 1,
                session_id: "test".to_string(),
                role: Role::User,
                content: "Hello".to_string(),
                token_count: Some(10),
                is_summary: false,
                created_at: chrono::Utc::now(),
            };
            let message = stored_to_message(&stored);
            assert_eq!(message.role, Role::User);
            assert_eq!(message.content, "Hello");
        }
    }

    mod prepare_context {
        use super::*;

        // Use a shared dummy provider definition for tests
        struct DummyProvider;

        impl AiProvider for DummyProvider {
            fn complete(
                &self,
                _request: CompletionRequest,
            ) -> impl std::future::Future<
                Output = Result<
                    cherry2k_core::provider::CompletionStream,
                    cherry2k_core::ProviderError,
                >,
            > + Send {
                async {
                    Err(cherry2k_core::ProviderError::InvalidApiKey {
                        provider: "dummy".to_string(),
                    })
                }
            }

            fn provider_id(&self) -> &'static str {
                "dummy"
            }

            fn validate_config(&self) -> Result<(), cherry2k_core::ConfigError> {
                Ok(())
            }

            fn health_check(
                &self,
            ) -> impl std::future::Future<Output = Result<(), cherry2k_core::ProviderError>> + Send
            {
                async { Ok(()) }
            }
        }

        #[tokio::test]
        async fn returns_empty_for_no_messages() {
            let (db, _temp, session_id) = setup_with_session().await;

            let result = prepare_context(&db, &session_id, &DummyProvider)
                .await
                .unwrap();

            assert!(result.messages.is_empty());
            assert!(!result.was_summarized);
        }

        #[tokio::test]
        async fn returns_messages_without_summarization_when_under_threshold() {
            let (db, _temp, session_id) = setup_with_session().await;

            // Add a few short messages (well under 12K tokens)
            save_message(&db, &session_id, Role::User, "Hello", None)
                .await
                .unwrap();
            save_message(&db, &session_id, Role::Assistant, "Hi there!", None)
                .await
                .unwrap();

            let result = prepare_context(&db, &session_id, &DummyProvider)
                .await
                .unwrap();

            assert_eq!(result.messages.len(), 2);
            assert_eq!(result.messages[0].role, Role::User);
            assert_eq!(result.messages[0].content, "Hello");
            assert_eq!(result.messages[1].role, Role::Assistant);
            assert_eq!(result.messages[1].content, "Hi there!");
            assert!(!result.was_summarized);
        }

        #[tokio::test]
        async fn converts_stored_messages_to_provider_messages() {
            let (db, _temp, session_id) = setup_with_session().await;

            save_message(&db, &session_id, Role::System, "Be helpful", None)
                .await
                .unwrap();
            save_message(&db, &session_id, Role::User, "Question", None)
                .await
                .unwrap();
            save_message(&db, &session_id, Role::Assistant, "Answer", None)
                .await
                .unwrap();

            let result = prepare_context(&db, &session_id, &DummyProvider)
                .await
                .unwrap();

            assert_eq!(result.messages.len(), 3);
            assert_eq!(result.messages[0].role, Role::System);
            assert_eq!(result.messages[1].role, Role::User);
            assert_eq!(result.messages[2].role, Role::Assistant);
        }
    }

    mod threshold_calculation {
        use super::*;

        #[test]
        fn threshold_is_75_percent_of_budget() {
            let threshold = ((TOKEN_BUDGET as f32) * SUMMARIZE_THRESHOLD) as usize;
            assert_eq!(threshold, 12_000); // 16000 * 0.75 = 12000
        }

        #[test]
        fn chars_needed_for_threshold() {
            // To hit 12K tokens at 4 chars/token, need 48K chars
            let chars_for_threshold = 12_000 * CHARS_PER_TOKEN;
            assert_eq!(chars_for_threshold, 48_000);
        }
    }
}
