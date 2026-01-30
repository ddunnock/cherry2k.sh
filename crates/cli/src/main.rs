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
    /// Test Sentry integration (sends a test event)
    SentryTest {
        /// Trigger a panic to test panic handling
        #[arg(long)]
        panic: bool,
    },
}

/// Initialize Sentry error tracking.
///
/// Returns a guard that must be kept alive for the duration of the program.
/// Sentry is only active if SENTRY_DSN environment variable is set.
fn init_sentry() -> sentry::ClientInitGuard {
    sentry::init((
        std::env::var("SENTRY_DSN").ok(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            environment: std::env::var("SENTRY_ENVIRONMENT")
                .ok()
                .map(std::borrow::Cow::Owned),
            // Capture 100% of transactions for tracing (adjust in production)
            traces_sample_rate: 1.0,
            ..Default::default()
        },
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize Sentry first (before anything that might panic)
    let _sentry_guard = init_sentry();

    let cli = Cli::parse();

    // Initialize logging with Sentry integration
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cli.log_level));
    tracing_subscriber::fmt().with_env_filter(filter).init();

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
        Commands::SentryTest { panic } => {
            if std::env::var("SENTRY_DSN").is_err() {
                println!("SENTRY_DSN not set - Sentry is inactive");
                println!("Set SENTRY_DSN environment variable to enable");
                return Ok(());
            }

            println!("Sending test event to Sentry...");
            sentry::capture_message("Cherry2K test event", sentry::Level::Info);

            if panic {
                println!("Triggering test panic...");
                panic!("Cherry2K test panic - this is intentional!");
            }

            // Give Sentry time to send the event
            std::thread::sleep(std::time::Duration::from_secs(2));
            println!("Test event sent! Check your Sentry dashboard.");
        }
    }

    Ok(())
}
