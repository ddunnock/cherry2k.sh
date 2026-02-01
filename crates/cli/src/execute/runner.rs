//! Async command execution with real-time streaming output.
//!
//! Executes shell commands via `sh -c` with proper handling of:
//! - Piped stdout/stderr streams
//! - Cancellation via [`CancellationToken`]
//! - SIGINT forwarding to child process group
//! - Cleanup on drop (kill_on_drop)

use std::io;
use std::process::{ExitStatus, Stdio};

use colored::Colorize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

/// Result of command execution.
#[derive(Debug)]
pub struct CommandResult {
    /// Exit status of the command.
    pub status: ExitStatus,
    /// Whether execution was cancelled via Ctrl+C.
    pub was_cancelled: bool,
}

/// Execute a shell command with real-time streaming output.
///
/// - Runs command via `sh -c` for shell interpretation
/// - Streams stdout line-by-line to terminal
/// - Streams stderr line-by-line in red
/// - Forwards Ctrl+C to child process via SIGINT
/// - Uses `kill_on_drop(true)` for cleanup safety
///
/// # Arguments
///
/// * `cmd` - The command string to execute (passed to `sh -c`)
/// * `cancel_token` - Optional cancellation token for Ctrl+C handling
///
/// # Returns
///
/// [`CommandResult`] with exit status and cancellation flag.
///
/// # Errors
///
/// Returns an error if the command fails to spawn or wait completes abnormally.
///
/// # Example
///
/// ```no_run
/// use cherry2k::execute::execute_command;
///
/// async fn example() {
///     let result = execute_command("echo hello", None).await.unwrap();
///     assert!(result.status.success());
/// }
/// ```
pub async fn execute_command(
    cmd: &str,
    cancel_token: Option<CancellationToken>,
) -> io::Result<CommandResult> {
    let mut child = Command::new("sh")
        .args(["-c", cmd])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let child_id = child.id();
    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");

    // Spawn task to read stderr (in red)
    let stderr_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            eprintln!("{}", line.red());
        }
    });

    // Read stdout, handling cancellation
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut was_cancelled = false;

    loop {
        tokio::select! {
            biased; // Check cancellation first

            _ = async {
                if let Some(ref token) = cancel_token {
                    token.cancelled().await
                } else {
                    // Never completes if no token
                    std::future::pending::<()>().await
                }
            } => {
                // Ctrl+C received - send SIGINT to child process
                if let Some(id) = child_id {
                    #[cfg(unix)]
                    {
                        use nix::sys::signal::{kill, Signal};
                        use nix::unistd::Pid;
                        // Send SIGINT to child process (positive pid)
                        // This is more reliable than process group signaling
                        let pid = Pid::from_raw(id as i32);
                        let _ = kill(pid, Signal::SIGINT);
                    }
                }
                was_cancelled = true;
                break;
            }

            line = stdout_reader.next_line() => {
                match line {
                    Ok(Some(line)) => println!("{line}"),
                    Ok(None) => break, // EOF
                    Err(e) => {
                        eprintln!("{}", format!("Error reading output: {e}").red());
                        break;
                    }
                }
            }
        }
    }

    // Wait for stderr task
    let _ = stderr_handle.await;

    // Wait for child to exit
    let status = child.wait().await?;

    Ok(CommandResult {
        status,
        was_cancelled,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn execute_command_runs_echo() {
        let result = execute_command("echo hello", None).await.unwrap();
        assert!(result.status.success());
        assert!(!result.was_cancelled);
    }

    #[tokio::test]
    async fn execute_command_captures_exit_code() {
        let result = execute_command("exit 42", None).await.unwrap();
        assert!(!result.status.success());
        assert_eq!(result.status.code(), Some(42));
    }

    #[tokio::test]
    async fn execute_command_handles_stderr() {
        // This command writes to stderr
        let result = execute_command("echo error >&2", None).await.unwrap();
        assert!(result.status.success());
    }

    #[tokio::test]
    async fn execute_command_handles_both_streams() {
        let result = execute_command("echo stdout && echo stderr >&2", None)
            .await
            .unwrap();
        assert!(result.status.success());
    }

    #[tokio::test]
    async fn execute_command_handles_multiline_output() {
        let result = execute_command("echo line1; echo line2; echo line3", None)
            .await
            .unwrap();
        assert!(result.status.success());
    }

    #[tokio::test]
    async fn execute_command_respects_cancellation() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        // Start a long-running command
        let handle = tokio::spawn(async move {
            execute_command("sleep 60", Some(token_clone)).await
        });

        // Give it a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Cancel it
        token.cancel();

        // Should complete quickly
        let result = tokio::time::timeout(
            tokio::time::Duration::from_secs(2),
            handle,
        )
        .await
        .expect("command should complete after cancellation")
        .expect("join should succeed");

        let result = result.unwrap();
        assert!(result.was_cancelled);
    }

    #[tokio::test]
    async fn execute_command_handles_invalid_command() {
        // The shell returns exit code 127 for command not found
        let result = execute_command("nonexistent_command_xyz", None)
            .await
            .unwrap();
        assert!(!result.status.success());
        assert_eq!(result.status.code(), Some(127));
    }
}
