//! Signal handling for graceful cancellation.
//!
//! Provides Ctrl+C handling with user confirmation before cancelling operations.
//! This is important for streaming responses where the user might accidentally
//! press Ctrl+C and want to continue.
//!
//! # Behavior
//!
//! When Ctrl+C is pressed during a streaming response:
//! 1. The stream pauses (but doesn't cancel yet)
//! 2. User sees prompt: "Cancel response? [y/n]: "
//! 3. If user types 'y' or 'Y', the operation is cancelled
//! 4. If user types anything else, streaming continues
//!
//! # Example
//!
//! ```no_run
//! use cherry2k::signal::setup_cancellation;
//!
//! async fn streaming_operation() {
//!     let cancel_token = setup_cancellation();
//!
//!     loop {
//!         tokio::select! {
//!             // ... do streaming work ...
//!             _ = cancel_token.cancelled() => {
//!                 println!("Cancelled!");
//!                 break;
//!             }
//!         }
//!     }
//! }
//! ```

use std::io::{self, BufRead, Write};

use tokio_util::sync::CancellationToken;

/// Set up Ctrl+C handling with confirmation prompt.
///
/// Returns a [`CancellationToken`] that will be cancelled when the user confirms
/// they want to stop the operation. The token can be used with `tokio::select!`
/// to race against async operations.
///
/// # Confirmation Flow
///
/// When Ctrl+C is pressed:
/// 1. Prints "\n\nCancel response? [y/n]: " to stderr
/// 2. Waits for user input (blocking on stdin)
/// 3. If input starts with 'y' or 'Y', cancels the token
/// 4. Otherwise prints "Continuing..." and waits for next Ctrl+C
///
/// # Returns
///
/// A `CancellationToken` that can be awaited with `.cancelled()`.
#[must_use]
pub fn setup_cancellation() -> CancellationToken {
    let token = CancellationToken::new();
    let token_clone = token.clone();

    tokio::spawn(async move {
        loop {
            // Wait for Ctrl+C signal
            if tokio::signal::ctrl_c().await.is_err() {
                // Signal handling failed, just return
                break;
            }

            // Ask for confirmation using spawn_blocking to avoid blocking the async runtime
            let confirmed = tokio::task::spawn_blocking(|| {
                // Print prompt to stderr so it doesn't mix with streaming output
                eprint!("\n\nCancel response? [y/n]: ");
                io::stderr().flush().ok();

                // Read user input
                let stdin = io::stdin();
                let mut line = String::new();
                if stdin.lock().read_line(&mut line).is_ok() {
                    let input = line.trim();
                    input.starts_with('y') || input.starts_with('Y')
                } else {
                    // On read error, default to not cancelling (safer)
                    false
                }
            })
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Confirmation task panicked: {:?}", e);
                false
            });

            if confirmed {
                token_clone.cancel();
                break;
            } else {
                eprintln!("Continuing...");
            }
        }
    });

    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancellation_token_is_not_cancelled_initially() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
    }

    #[tokio::test]
    async fn cancellation_token_can_be_cancelled() {
        let token = CancellationToken::new();
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn cloned_tokens_share_state() {
        let token = CancellationToken::new();
        let clone = token.clone();

        token.cancel();

        assert!(clone.is_cancelled());
    }
}
