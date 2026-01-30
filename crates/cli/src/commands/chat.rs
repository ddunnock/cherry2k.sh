//! Chat command handler
//!
//! Sends a one-shot query to the configured AI provider.
//! Currently a placeholder - Phase 2 will add actual provider integration.

use anyhow::Result;
use cherry2k_core::config::Config;

/// Run the chat command.
///
/// Currently a placeholder that shows the message would be sent.
/// Phase 2 will add actual provider integration.
pub async fn run(config: &Config, message: &str) -> Result<()> {
    println!("Chat command received.");
    println!("Provider: {}", config.general.default_provider);
    println!("Message: {}", message);
    println!();
    println!("[Phase 2 will add AI provider integration]");

    // Demonstrate the confirmation flow would happen here
    if config.safety.confirm_commands {
        println!();
        println!("(Safety: Command confirmation is ENABLED)");
    }

    Ok(())
}
