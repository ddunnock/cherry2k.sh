//! Cherry2K CLI Application
//!
//! Zsh terminal AI assistant with provider-agnostic architecture.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use cherry2k_storage::Database;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

mod commands;

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
        /// Output plain text without markdown rendering
        #[arg(short, long)]
        plain: bool,
        /// Path to JSON file with shell context (for zsh integration)
        #[arg(long)]
        context_file: Option<PathBuf>,
    },
    /// Show current configuration
    Config,
    /// Show or switch AI providers
    Provider {
        /// Provider to switch to (omit to show current)
        name: Option<String>,
        /// List all available providers
        #[arg(short, long)]
        list: bool,
    },
    /// Resume a previous session or list sessions
    Resume {
        /// List all sessions instead of resuming
        #[arg(short, long)]
        list: bool,
        /// Specific session ID to resume
        session_id: Option<String>,
    },
    /// Start a new session (ignoring any existing session)
    New,
    /// Delete all sessions
    Clear,
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
    // Use lower sample rate in production to control costs
    let sample_rate = std::env::var("SENTRY_ENVIRONMENT")
        .map(|env| if env == "production" { 0.1 } else { 1.0 })
        .unwrap_or(1.0);

    sentry::init((
        std::env::var("SENTRY_DSN").ok(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            environment: std::env::var("SENTRY_ENVIRONMENT")
                .ok()
                .map(std::borrow::Cow::Owned),
            traces_sample_rate: sample_rate,
            // Attach stacktraces to all messages for better debugging
            attach_stacktrace: true,
            ..Default::default()
        },
    ))
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // Capture error to Sentry before reporting to user
            sentry::capture_error(&*e);
            eprintln!("Error: {e:?}");
            ExitCode::FAILURE
        }
    }
}

/// Main application logic.
///
/// Separated from main() to enable proper exit code propagation.
/// CLI domain rule: non-zero exit on error for script integration.
async fn run() -> Result<()> {
    // Load .env file if present (ignore errors if not found)
    let _ = dotenvy::dotenv();

    // Initialize Sentry first (before anything that might panic)
    let _sentry_guard = init_sentry();

    let cli = Cli::parse();

    // Initialize logging with Sentry integration
    // Sentry layer captures warn/error logs as breadcrumbs
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cli.log_level));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(sentry::integrations::tracing::layer())
        .init();

    // Load configuration
    let config = cherry2k_core::config::load_config()?;
    tracing::debug!("Configuration loaded: {:?}", config.general);

    // Dispatch to command handlers
    match cli.command {
        Commands::Chat {
            message,
            plain,
            context_file,
        } => {
            commands::chat::run(&config, &message, plain, context_file.as_deref()).await?;
        }
        Commands::Config => {
            commands::config::run(&config)?;
        }
        Commands::Provider { name, list } => {
            if list {
                commands::provider::run_list(&config)?;
            } else if let Some(provider_name) = name {
                commands::provider::run_switch(&config, &provider_name)?;
            } else {
                commands::provider::run_current(&config)?;
            }
        }
        Commands::Resume { list, session_id } => {
            let db = Database::open()
                .await
                .context("Failed to open session database")?;
            let working_dir = std::env::current_dir().context("Failed to get current directory")?;
            commands::session::resume(&db, session_id.as_deref(), list, &working_dir).await?;
        }
        Commands::New => {
            let db = Database::open()
                .await
                .context("Failed to open session database")?;
            let working_dir = std::env::current_dir().context("Failed to get current directory")?;
            commands::session::new_session(&db, &working_dir).await?;
        }
        Commands::Clear => {
            let db = Database::open()
                .await
                .context("Failed to open session database")?;
            commands::session::clear(&db).await?;
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

            // Flush Sentry client to ensure events are sent
            sentry::Hub::current()
                .client()
                .map(|c| c.flush(Some(std::time::Duration::from_secs(5))));
            println!("Test event sent! Check your Sentry dashboard.");
        }
    }

    Ok(())
}
