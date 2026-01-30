//! Config command handler
//!
//! Displays the current configuration.

use anyhow::Result;
use cherry2k_core::config::Config;

/// Format API key status for display (never shows actual key).
fn api_key_status(key: &Option<String>) -> &'static str {
    if key.is_some() {
        "configured"
    } else {
        "not set"
    }
}

/// Show current configuration.
pub fn run(config: &Config) -> Result<()> {
    println!("Cherry2K Configuration");
    println!("======================");
    println!();

    println!("[General]");
    println!("  Default provider: {}", config.general.default_provider);
    println!("  Log level: {}", config.general.log_level);
    println!();

    println!("[Safety]");
    println!("  Confirm commands: {}", config.safety.confirm_commands);
    println!(
        "  Confirm file writes: {}",
        config.safety.confirm_file_writes
    );
    println!(
        "  Blocked patterns: {}",
        config.safety.blocked_patterns.len()
    );
    println!();

    if let Some(ref openai) = config.openai {
        println!("[OpenAI]");
        println!("  Base URL: {}", openai.base_url);
        println!("  Model: {}", openai.model);
        println!("  API key: {}", api_key_status(&openai.api_key));
        println!();
    }

    if let Some(ref anthropic) = config.anthropic {
        println!("[Anthropic]");
        println!("  Model: {}", anthropic.model);
        println!("  API key: {}", api_key_status(&anthropic.api_key));
        println!();
    }

    if let Some(ref ollama) = config.ollama {
        println!("[Ollama]");
        println!("  Host: {}", ollama.host);
        println!("  Model: {}", ollama.model);
        println!();
    }

    Ok(())
}
