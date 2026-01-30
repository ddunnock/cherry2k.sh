//! Cherry2K CLI Application
//!
//! Zsh terminal AI assistant with provider-agnostic architecture.

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod commands;
mod confirm;

/// Cherry2K - Zsh Terminal AI Assistant
#[derive(Parser)]
#[command(name = "cherry2k")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Set log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Chat with AI (one-shot query)
    Chat {
        /// The message to send to the AI
        message: String,
    },
    /// Show current configuration
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cli.log_level));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    // Load configuration
    let config = cherry2k_core::config::load_config()?;
    tracing::debug!("Configuration loaded: {:?}", config.general);

    // Dispatch to command handlers
    match cli.command {
        Commands::Chat { message } => {
            commands::chat::run(&config, &message).await?;
        }
        Commands::Config => {
            commands::config::run(&config)?;
        }
    }

    Ok(())
}
