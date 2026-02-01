//! Exit status display formatting for command execution.
//!
//! Provides user-friendly display of command exit status with appropriate
//! colors and symbols:
//! - Success: Green checkmark with "exit 0"
//! - Failure: Red X with exit code
//! - Signal termination: Yellow warning with signal number

use std::process::ExitStatus;

use colored::Colorize;

/// Display the exit status of a completed command.
///
/// Shows:
/// - Green checkmark with "exit 0" on success
/// - Red X with exit code on failure
/// - Yellow warning for signal termination
///
/// # Example
///
/// ```no_run
/// use std::process::Command;
/// use cherry2k::execute::display_exit_status;
///
/// let status = Command::new("ls").status().unwrap();
/// display_exit_status(status);
/// ```
pub fn display_exit_status(status: ExitStatus) {
    if status.success() {
        println!("{}", "OK".green());
    } else {
        match status.code() {
            Some(code) => {
                println!("{}", format!("FAILED (exit {})", code).red());
            }
            None => {
                // Process terminated by signal
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    if let Some(sig) = status.signal() {
                        println!(
                            "{}",
                            format!("Terminated by signal {}", sig).yellow()
                        );
                    } else {
                        println!("{}", "Terminated abnormally".yellow());
                    }
                }
                #[cfg(not(unix))]
                {
                    println!("{}", "Terminated abnormally".yellow());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn display_exit_status_runs_without_panic_on_success() {
        // Use a command that should always succeed on Unix
        let status = Command::new("true").status().unwrap();
        display_exit_status(status);
        // If we get here without panic, the test passes
    }

    #[test]
    fn display_exit_status_runs_without_panic_on_failure() {
        // Use a command that should always fail on Unix
        let status = Command::new("false").status().unwrap();
        display_exit_status(status);
        // If we get here without panic, the test passes
    }

    #[test]
    fn display_exit_status_runs_without_panic_on_exit_code() {
        // Use sh -c to get a specific exit code
        let status = Command::new("sh")
            .args(["-c", "exit 42"])
            .status()
            .unwrap();
        display_exit_status(status);
        // If we get here without panic, the test passes
    }
}
