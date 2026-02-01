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
use cherry2k_core::{CompletionRequest, Message, ProviderFactory, command_mode_system_prompt};
use cherry2k_storage::message::save_message;
use cherry2k_storage::session::{cleanup_old_sessions, get_or_create_session};
use cherry2k_storage::{Database, prepare_context};
use serde::Deserialize;
use tokio_stream::StreamExt;

use cherry2k::confirm::{ConfirmResult, check_blocked_patterns, confirm_command, edit_command};
use cherry2k::execute::{display_exit_status, execute_command};
use cherry2k::files;
use cherry2k::intent::{Intent, detect_intent};
use cherry2k::output::{
    ResponseSpinner, StreamWriter, display_provider_error, display_suggested_command,
};
use cherry2k::signal::setup_cancellation;
use colored::Colorize;
use tokio_util::sync::CancellationToken;

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

    // Parse message for command mode markers
    let user_message = message.trim();
    let (actual_message, force_command_mode) =
        if let Some(stripped) = user_message.strip_prefix('!') {
            (stripped.trim(), true)
        } else if let Some(stripped) = user_message.strip_prefix("/run ") {
            (stripped.trim(), true)
        } else {
            (user_message, false)
        };

    // Check for question mode marker (? suffix)
    let force_question_mode = actual_message.ends_with('?') && !force_command_mode;

    // Detect and inject file references before sending to AI
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let file_refs = files::detect_file_references(actual_message, &cwd);

    let mut file_context = String::new();
    for path in &file_refs {
        match files::FileReader::read_file(path) {
            Ok(files::ReadResult::Content(content)) => {
                file_context.push_str(&format!(
                    "\n--- File: {} ---\n{}\n",
                    path.display(),
                    content
                ));
            }
            Ok(files::ReadResult::TooLarge { size, .. }) => {
                eprintln!("Skipping {} (too large: {} bytes)", path.display(), size);
            }
            Ok(files::ReadResult::Binary { .. }) => {
                eprintln!("Skipping {} (binary file)", path.display());
            }
            Ok(files::ReadResult::Error { error, .. }) => {
                eprintln!("Warning: Could not read {}: {}", path.display(), error);
            }
            Err(e) => {
                eprintln!("Warning: Could not read {}: {}", path.display(), e);
            }
        }
    }

    // Build augmented message with file context if any
    let augmented_message = if file_context.is_empty() {
        actual_message.to_string()
    } else {
        tracing::debug!("Injected {} file(s) into context", file_refs.len());
        format!(
            "The user referenced these files:\n{}\n\nUser message: {}",
            file_context, actual_message
        )
    };

    // Save user message before sending request (use actual_message for cleaner history)
    save_message(&db, &session_id, Role::User, actual_message, None)
        .await
        .context("Failed to save message")?;

    // Build request with history + new message (using augmented version)
    // Always include command mode system prompt - AI decides based on context
    let request = CompletionRequest::new()
        .with_message(Message::system(command_mode_system_prompt()))
        .with_messages(context.messages)
        .with_message(Message::user(&augmented_message));

    tracing::debug!(
        "Request mode: force_command={}, force_question={}",
        force_command_mode,
        force_question_mode
    );

    // Setup cancellation handler (before streaming, can be reused for command execution)
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

    // Detect if response contains a command suggestion (skip if force_question_mode)
    // Intent::Question means response was just an explanation, already displayed
    if !force_question_mode && let Intent::Command(detected) = detect_intent(&collected_response) {
        // Check for blocked dangerous patterns first
        if let Some(pattern) =
            check_blocked_patterns(&detected.command, &config.safety.blocked_patterns)
        {
            println!();
            println!(
                "{} Command matches dangerous pattern: {}",
                "BLOCKED:".red(),
                pattern
            );
            println!("This command has been blocked for safety reasons.");
            return Ok(());
        }

        // Display the command with syntax highlighting
        display_suggested_command(&detected.command, detected.context.as_deref());

        // Check if confirmation is required (respect config)
        let mut command_to_run = detected.command.clone();

        if config.safety.confirm_commands {
            // Ask for confirmation
            loop {
                match confirm_command(&command_to_run)? {
                    ConfirmResult::Yes => {
                        // Re-check blocked patterns after edit
                        if let Some(pattern) =
                            check_blocked_patterns(&command_to_run, &config.safety.blocked_patterns)
                        {
                            println!();
                            println!(
                                "{} Command matches dangerous pattern: {}",
                                "BLOCKED:".red(),
                                pattern
                            );
                            println!("This command has been blocked for safety reasons.");
                            return Ok(());
                        }

                        run_command(&command_to_run, &cancel_token).await?;
                        break;
                    }
                    ConfirmResult::No => {
                        println!("Command cancelled.");
                        break;
                    }
                    ConfirmResult::Edit => {
                        command_to_run = edit_command(&command_to_run)?;
                        // Re-display the edited command
                        display_suggested_command(&command_to_run, None);
                        // Loop continues to re-confirm
                    }
                }
            }
        } else {
            // Auto-execute without confirmation (confirm_commands = false)
            run_command(&command_to_run, &cancel_token).await?;
        }
    }

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

/// Execute a command with signal handling and display results.
///
/// Extracted helper to reduce duplication in the confirmation and auto-execute paths.
async fn run_command(command: &str, cancel_token: &CancellationToken) -> Result<()> {
    println!(); // Blank line before execution

    // Execute with signal handling
    let result = execute_command(command, Some(cancel_token.clone())).await?;

    // Display exit status
    display_exit_status(result.status);

    if result.was_cancelled {
        println!("Command interrupted.");
    }

    Ok(())
}
