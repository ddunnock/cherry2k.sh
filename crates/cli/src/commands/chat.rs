//! Chat command handler
//!
//! Sends a one-shot query to the configured AI provider and streams the response.
//! Supports spinner animation while waiting, line-buffered streaming output,
//! and Ctrl+C cancellation with confirmation.

use std::io::{self, Write};

use anyhow::{Context, Result};
use cherry2k_core::config::Config;
use cherry2k_core::{AiProvider, CompletionRequest, Message, OpenAiProvider};
use tokio_stream::StreamExt;

use crate::output::{ResponseSpinner, StreamWriter, display_provider_error};
use crate::signal::setup_cancellation;

/// Run the chat command.
///
/// Sends the message to the configured AI provider and streams the response
/// to the terminal with line buffering for smooth output.
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
pub async fn run(config: &Config, message: &str, _plain: bool) -> Result<()> {
    // Get OpenAI config or error
    let openai_config = config.openai.clone().ok_or_else(|| {
        anyhow::anyhow!("OpenAI not configured. Set OPENAI_API_KEY environment variable.")
    })?;

    // Create and validate provider
    let provider = OpenAiProvider::new(openai_config);
    provider
        .validate_config()
        .context("Invalid OpenAI configuration")?;

    // Build request
    let request = CompletionRequest::new().with_message(Message::user(message));

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

    // Stream response with cancellation support
    let mut writer = StreamWriter::new();
    tokio::pin!(stream);

    loop {
        tokio::select! {
            chunk = stream.next() => {
                match chunk {
                    Some(Ok(text)) => {
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
                return Ok(());
            }
        }
    }

    // Flush any remaining buffered content
    writer.flush()?;
    println!(); // Blank line after response

    Ok(())
}
