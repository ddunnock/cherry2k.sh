//! Chat command handler
//!
//! Sends a one-shot query to the configured AI provider.
//! Currently a placeholder - Phase 2 will add actual provider integration.

use anyhow::Result;
use cherry2k_core::config::Config;

use crate::confirm::{self, ConfirmResult};

/// Run the chat command.
///
/// Currently a placeholder that demonstrates the confirmation flow.
/// Phase 2 will add actual provider integration.
pub async fn run(config: &Config, message: &str) -> Result<()> {
    println!("Cherry2K Chat");
    println!("=============");
    println!();
    println!("Provider: {}", config.general.default_provider);
    println!("Message: \"{}\"", message);
    println!();
    println!("[Phase 2 will add AI provider integration]");
    println!();

    // Demonstrate the confirmation flow (scaffolded for Phase 6)
    // This simulates what happens when AI suggests a command
    demonstrate_confirmation_flow(config)?;

    Ok(())
}

/// Demonstrate the confirmation flow that will be used for command execution.
fn demonstrate_confirmation_flow(config: &Config) -> Result<()> {
    println!("--- Confirmation Flow Demo ---");
    println!();

    // Example: AI would suggest this command
    let suggested_command = "echo 'Hello from Cherry2K!'";

    // Check against blocked patterns first
    if let Some(pattern) = confirm::check_blocked_patterns(suggested_command, &config.safety.blocked_patterns) {
        println!("BLOCKED: Command matches dangerous pattern: {}", pattern);
        return Ok(());
    }

    // If confirmation is enabled, ask user
    if config.safety.confirm_commands {
        match confirm::confirm_command(suggested_command)? {
            ConfirmResult::Yes => {
                println!();
                println!("User confirmed. [Would execute in Phase 6]");
            }
            ConfirmResult::No => {
                println!();
                println!("User cancelled.");
            }
            ConfirmResult::Edit => {
                println!();
                println!("User wants to edit. [Would open editor in Phase 6]");
            }
        }
    } else {
        println!("(Confirmation disabled - would auto-execute)");
        println!("Command: {}", suggested_command);
    }

    println!();
    println!("--- End Demo ---");

    Ok(())
}
