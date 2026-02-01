//! Confirmation prompt utilities
//!
//! This module provides user confirmation prompts for safety-critical operations.
//! It is used to ensure the user explicitly approves AI-suggested commands before
//! they are executed.
//!
//! Functions are used by main.rs in Phase 06-04 CLI integration.

// Public API for command execution flow - used in 06-04
#![allow(dead_code)]

use std::io::{self, BufRead, Write};

/// Result of a confirmation prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmResult {
    /// User confirmed (yes)
    Yes,
    /// User denied (no)
    No,
    /// User wants to edit before confirming
    Edit,
}

/// Prompt the user for confirmation.
///
/// Displays the prompt and waits for y/n/e input.
/// - y/Y/yes -> ConfirmResult::Yes
/// - n/N/no -> ConfirmResult::No
/// - e/E/edit -> ConfirmResult::Edit
///
/// # Arguments
/// * `prompt` - The question to ask
/// * `allow_edit` - Whether to show the (e)dit option
///
/// # Example
/// ```no_run
/// use cherry2k::confirm::{confirm, ConfirmResult};
///
/// let result = confirm("Run this command?", false).unwrap();
/// match result {
///     ConfirmResult::Yes => println!("Proceeding..."),
///     ConfirmResult::No => println!("Cancelled."),
///     ConfirmResult::Edit => println!("Editing..."),
/// }
/// ```
pub fn confirm(prompt: &str, allow_edit: bool) -> io::Result<ConfirmResult> {
    let options = if allow_edit { "[y/n/e]" } else { "[y/n]" };

    loop {
        print!("{} {} ", prompt, options);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "y" | "yes" => return Ok(ConfirmResult::Yes),
            "n" | "no" => return Ok(ConfirmResult::No),
            "e" | "edit" if allow_edit => return Ok(ConfirmResult::Edit),
            "" => {
                // Empty input defaults to No for safety
                return Ok(ConfirmResult::No);
            }
            _ => {
                if allow_edit {
                    println!("Please enter 'y' for yes, 'n' for no, or 'e' to edit.");
                } else {
                    println!("Please enter 'y' for yes or 'n' for no.");
                }
            }
        }
    }
}

/// Confirm a potentially dangerous command before execution.
///
/// Displays the command and asks for confirmation.
/// Returns the user's choice (Yes, No, or Edit).
pub fn confirm_command(command: &str) -> io::Result<ConfirmResult> {
    println!();
    println!("Suggested command:");
    println!("  {}", command);
    println!();
    confirm("Run this?", true)
}

/// Prompt user to edit a command.
///
/// Displays the current command and asks for a new version.
/// If user enters empty line, returns the original command.
///
/// # Arguments
/// * `original` - The original command to edit
///
/// # Returns
/// The edited command (or original if unchanged)
///
/// # Example
/// ```no_run
/// use cherry2k::confirm::edit_command;
///
/// let edited = edit_command("ls -la").unwrap();
/// // User can type new command or press Enter to keep original
/// ```
pub fn edit_command(original: &str) -> io::Result<String> {
    println!();
    println!("Current command: {}", original);
    print!("Enter new command (or press Enter to keep): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let edited = input.trim();

    if edited.is_empty() {
        Ok(original.to_string())
    } else {
        Ok(edited.to_string())
    }
}

/// Confirm a file operation before execution.
///
/// Displays the operation details and asks for confirmation.
/// Note: Used in Phase 7 (File Operations).
pub fn confirm_file_operation(operation: &str, path: &str) -> io::Result<ConfirmResult> {
    println!();
    println!("{}: {}", operation, path);
    println!();
    confirm("Proceed?", false)
}

/// Check if a command matches any blocked patterns.
///
/// Returns Some(pattern) if blocked, None if allowed.
pub fn check_blocked_patterns<'a>(command: &str, patterns: &'a [String]) -> Option<&'a str> {
    patterns
        .iter()
        .find(|p| command.contains(p.as_str()))
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_patterns_match() {
        let patterns = vec!["rm -rf /".to_string(), "rm -rf ~".to_string()];
        assert!(check_blocked_patterns("rm -rf /", &patterns).is_some());
        assert!(check_blocked_patterns("sudo rm -rf /", &patterns).is_some());
        assert!(check_blocked_patterns("rm file.txt", &patterns).is_none());
    }

    #[test]
    fn test_blocked_patterns_empty() {
        let patterns: Vec<String> = vec![];
        assert!(check_blocked_patterns("rm -rf /", &patterns).is_none());
    }

    #[test]
    fn test_blocked_patterns_returns_matching_pattern() {
        let patterns = vec!["rm -rf /".to_string(), "mkfs".to_string()];
        assert_eq!(
            check_blocked_patterns("rm -rf /home", &patterns),
            Some("rm -rf /")
        );
        assert_eq!(
            check_blocked_patterns("sudo mkfs.ext4 /dev/sda", &patterns),
            Some("mkfs")
        );
    }

    #[test]
    fn test_confirm_result_equality() {
        assert_eq!(ConfirmResult::Yes, ConfirmResult::Yes);
        assert_eq!(ConfirmResult::No, ConfirmResult::No);
        assert_eq!(ConfirmResult::Edit, ConfirmResult::Edit);
        assert_ne!(ConfirmResult::Yes, ConfirmResult::No);
    }
}
