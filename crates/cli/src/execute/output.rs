//! Exit status display formatting for command execution.
//!
//! Provides user-friendly display of command exit status with appropriate
//! colors:
//! - Success: Green "OK"
//! - Failure: Red "FAILED (exit N)"
//! - Signal termination: Yellow "Terminated by signal N"

use std::process::ExitStatus;

use colored::Colorize;

/// Display the exit status of a completed command.
///
/// Shows:
/// - Green "OK" on success
/// - Red "FAILED (exit N)" with exit code on failure
/// - Yellow "Terminated by signal N" for signal termination
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
    // Status messages go to stderr (data to stdout, metadata to stderr)
    if status.success() {
        eprintln!("{}", "OK".green());
    } else {
        match status.code() {
            Some(code) => {
                eprintln!("{}", format!("FAILED (exit {})", code).red());
            }
            None => {
                // Process terminated by signal
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    if let Some(sig) = status.signal() {
                        eprintln!("{}", format!("Terminated by signal {}", sig).yellow());
                    } else {
                        eprintln!("{}", "Terminated abnormally".yellow());
                    }
                }
                #[cfg(not(unix))]
                {
                    eprintln!("{}", "Terminated abnormally".yellow());
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
        let status = Command::new("sh").args(["-c", "exit 42"]).status().unwrap();
        display_exit_status(status);
        // If we get here without panic, the test passes
    }
}
