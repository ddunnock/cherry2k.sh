//! Command execution module for running shell commands.
//!
//! Provides async command execution with real-time streaming output,
//! signal handling (Ctrl+C forwarding), and exit status display.
//!
//! # Features
//!
//! - Runs commands via `sh -c` for shell interpretation
//! - Streams stdout line-by-line to terminal
//! - Streams stderr line-by-line in red
//! - Forwards Ctrl+C to child process via SIGINT
//! - Uses `kill_on_drop(true)` for cleanup safety
//!
//! # Example
//!
//! ```no_run
//! use cherry2k::execute::{execute_command, display_exit_status};
//!
//! async fn run() {
//!     let result = execute_command("ls -la", None).await.unwrap();
//!     display_exit_status(result.status);
//! }
//! ```

mod output;
mod runner;

pub use output::display_exit_status;
pub use runner::{execute_command, CommandResult};
