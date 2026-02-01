//! Chat command handler
//!
//! Sends a one-shot query to the configured AI provider and streams the response.
//! Supports spinner animation while waiting, line-buffered streaming output,
//! and Ctrl+C cancellation with confirmation.
//!
//! Sessions are automatically managed per-directory. Conversation history is
//! loaded and sent to the provider for context. Messages are saved after each
//! exchange.

/// Probability threshold for session cleanup (26/256 ≈ 10.2%).
///
/// On each chat completion, we roll a random u8. If it's below this threshold,
/// we trigger cleanup of old sessions. This spreads the cleanup work across
/// many requests rather than doing it all at once.
const CLEANUP_PROBABILITY_THRESHOLD: u8 = 26;

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};
use cherry2k_core::config::Config;
use cherry2k_core::provider::Role;
use cherry2k_core::{CompletionRequest, Message, ProviderFactory};
use cherry2k_storage::message::save_message;
use cherry2k_storage::session::{cleanup_old_sessions, get_or_create_session};
use cherry2k_storage::{Database, prepare_context};
use serde::Deserialize;
use tokio_stream::StreamExt;

use cherry2k::output::{ResponseSpinner, StreamWriter, display_provider_error};
use cherry2k::signal::setup_cancellation;

/// Shell context passed from zsh integration.
#[derive(Debug, Deserialize)]
struct ShellContext {
    /// Current working directory
    pwd: String,
    /// Shell executable path
    shell: String,
    /// Zsh version (if zsh) - intentionally unused, reserved for Phase 6 intent detection
    #[serde(default)]
    #[allow(dead_code)]
    zsh_version: Option<String>,
    /// Recent command history
    #[serde(default)]
    history: Vec<HistoryEntry>,
    /// Filtered environment variables
    #[serde(default)]
    env: HashMap<String, String>,
}

/// A single history entry from shell context.
#[derive(Debug, Deserialize)]
struct HistoryEntry {
    /// Timestamp of the command (ISO format, optional)
    #[serde(default)]
    timestamp: Option<String>,
    /// The command that was executed
    command: String,
}

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
/// * `context_file` - Optional path to JSON file with shell context (from zsh integration)
///
/// # Errors
///
/// Returns an error if:
/// - OpenAI is not configured (missing API key)
/// - The API request fails
/// - Network errors occur during streaming
/// - Database operations fail
/// - Context file cannot be read or parsed (if provided)
pub async fn run(
    config: &Config,
    message: &str,
    _plain: bool,
    context_file: Option<&Path>,
) -> Result<()> {
    // TODO(Phase 5): Use _plain flag to disable markdown rendering

    // Parse shell context if provided
    if let Some(path) = context_file {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read context file: {}", path.display()))?;

        let shell_context: ShellContext = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse context file: {}", path.display()))?;

        tracing::debug!(
            "Shell context: pwd={}, shell={}, history_len={}, env_keys={:?}",
            shell_context.pwd,
            shell_context.shell,
            shell_context.history.len(),
            shell_context.env.keys().collect::<Vec<_>>()
        );

        // Log history entries at trace level
        for entry in &shell_context.history {
            tracing::trace!(
                "History: {} - {}",
                entry.timestamp.as_deref().unwrap_or("no timestamp"),
                entry.command
            );
        }
    }

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

    // Create provider factory from config
    let factory = ProviderFactory::from_config(config)
        .map_err(|e| anyhow::anyhow!("{}", e))
        .context("Failed to initialize providers")?;

    // Check for in-session provider override
    let active_provider_name = super::provider::get_active_provider()
        .filter(|name| factory.contains(name))
        .unwrap_or_else(|| factory.default_provider_name().to_string());

    let provider = factory
        .get(&active_provider_name)
        .ok_or_else(|| anyhow::anyhow!("Provider '{}' not available", active_provider_name))?;

    tracing::debug!("Using provider: {}", provider.provider_id());

    // Load conversation history
    let context = prepare_context(&db, &session_id, provider)
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
    // Using random to avoid timing-based patterns
    if rand::random::<u8>() < CLEANUP_PROBABILITY_THRESHOLD
        && let Ok(count) = cleanup_old_sessions(&db).await
        && count > 0
    {
        tracing::debug!("Cleaned up {} old sessions", count);
    }

    Ok(())
}
