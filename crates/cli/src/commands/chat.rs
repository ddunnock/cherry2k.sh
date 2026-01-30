//! Chat command handler
//!
//! Sends a one-shot query to the configured AI provider and streams the response.
//! Supports spinner animation while waiting, line-buffered streaming output,
//! and Ctrl+C cancellation with confirmation.
//!
//! Sessions are automatically managed per-directory. Conversation history is
//! loaded and sent to the provider for context. Messages are saved after each
//! exchange.

use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use cherry2k_core::config::Config;
use cherry2k_core::provider::Role;
use cherry2k_core::{AiProvider, CompletionRequest, Message, OpenAiProvider};
use cherry2k_storage::message::save_message;
use cherry2k_storage::session::{cleanup_old_sessions, get_or_create_session};
use cherry2k_storage::{prepare_context, Database};
use tokio_stream::StreamExt;

use crate::output::{display_provider_error, ResponseSpinner, StreamWriter};
use crate::signal::setup_cancellation;

/// Run the chat command.
///
/// Sends the message to the configured AI provider and streams the response
/// to the terminal with line buffering for smooth output. Automatically manages
/// conversation sessions per-directory, loading history and saving messages.
///
/// # Arguments
///
/// * `config` - Application configuration
/// * `message` - The user's message to send to the AI
/// * `_plain` - If true, skip markdown rendering (currently unused, for future enhancement)
///
/// # Errors
///
/// Returns an error if:
/// - OpenAI is not configured (missing API key)
/// - The API request fails
/// - Network errors occur during streaming
/// - Database operations fail
pub async fn run(config: &Config, message: &str, _plain: bool) -> Result<()> {
    // Open database for session management
    let db = Database::open()
        .await
        .context("Failed to open session database")?;

    // Get or create session for current directory
    let working_dir = std::env::current_dir().context("Failed to get current directory")?;
    let session_id = get_or_create_session(&db, &working_dir)
        .await
        .context("Failed to get session")?;

    tracing::debug!("Using session {} in {}", session_id, working_dir.display());

    // Get OpenAI config or error
    let openai_config = config.openai.clone().ok_or_else(|| {
        anyhow::anyhow!("OpenAI not configured. Set OPENAI_API_KEY environment variable.")
    })?;

    // Create and validate provider
    let provider = OpenAiProvider::new(openai_config);
    provider
        .validate_config()
        .context("Invalid OpenAI configuration")?;

    // Load conversation history
    let context = prepare_context(&db, &session_id, &provider)
        .await
        .context("Failed to load conversation history")?;

    // Show indicator if summarization occurred
    if context.was_summarized {
        println!("(context summarized)");
    }

    // Save user message before sending request
    save_message(&db, &session_id, Role::User, message, None)
        .await
        .context("Failed to save message")?;

    // Build request with history + new message
    let request = CompletionRequest::new()
        .with_messages(context.messages)
        .with_message(Message::user(message));

    // Setup cancellation handler
    let cancel_token = setup_cancellation();

    // Show spinner while waiting for initial response
    let spinner = ResponseSpinner::new();
    spinner.start();

    // Get stream from provider
    let stream = match provider.complete(request).await {
        Ok(s) => s,
        Err(e) => {
            spinner.stop();
            display_provider_error(&e);
            return Err(e.into());
        }
    };

    // Stop spinner and prepare for streaming output
    spinner.stop();
    println!(); // Blank line before response
    print!("\u{25B6} "); // Subtle icon prefix (black right-pointing triangle)
    io::stdout().flush()?;

    // Stream response with cancellation support, accumulating for save
    let mut writer = StreamWriter::new();
    let mut collected_response = String::new();
    tokio::pin!(stream);

    loop {
        tokio::select! {
            chunk = stream.next() => {
                match chunk {
                    Some(Ok(text)) => {
                        collected_response.push_str(&text);
                        writer.write_chunk(&text)?;
                    }
                    Some(Err(e)) => {
                        writer.flush()?;
                        println!();
                        display_provider_error(&e);
                        return Err(e.into());
                    }
                    None => break, // Stream ended
                }
            }
            _ = cancel_token.cancelled() => {
                writer.flush()?;
                println!("\n\nCancelled by user.");
                // Save partial response if we got any
                if !collected_response.is_empty() {
                    let _ = save_message(&db, &session_id, Role::Assistant, &collected_response, None).await;
                }
                return Ok(());
            }
        }
    }

    // Flush any remaining buffered content
    writer.flush()?;
    println!(); // Blank line after response

    // Save assistant response
    save_message(&db, &session_id, Role::Assistant, &collected_response, None)
        .await
        .context("Failed to save response")?;

    // Probabilistic cleanup (~10% of the time)
    if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH)
        && duration.as_millis() % 10 == 0
        && let Ok(count) = cleanup_old_sessions(&db).await
        && count > 0
    {
        tracing::debug!("Cleaned up {} old sessions", count);
    }

    Ok(())
}
