//! Provider management commands.
//!
//! Commands for listing, showing, and switching AI providers.
//! Provider selection persists in a state file for in-session switching.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use cherry2k_core::config::Config;
use cherry2k_core::ProviderFactory;
use directories::ProjectDirs;

// ============================================================================
// State File Management
// ============================================================================

/// Get the state directory path.
///
/// Uses XDG conventions via the directories crate.
fn get_state_dir() -> Option<PathBuf> {
    ProjectDirs::from("", "", "cherry2k")
        .map(|dirs| dirs.state_dir().unwrap_or(dirs.data_dir()).to_path_buf())
}

/// Get the currently active provider from state file.
///
/// Returns `None` if:
/// - State directory cannot be determined
/// - State file doesn't exist
/// - State file cannot be read
pub fn get_active_provider() -> Option<String> {
    let state_dir = get_state_dir()?;
    let path = state_dir.join("active_provider");
    match fs::read_to_string(&path) {
        Ok(s) => Some(s.trim().to_string()).filter(|s| !s.is_empty()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            tracing::debug!("Failed to read active_provider state: {e}");
            None
        }
    }
}

/// Set the active provider in state file.
///
/// Creates the state directory if it doesn't exist.
fn set_active_provider(name: &str) -> Result<()> {
    let state_dir =
        get_state_dir().ok_or_else(|| anyhow::anyhow!("Could not determine state directory"))?;
    fs::create_dir_all(&state_dir).context("Failed to create state directory")?;
    fs::write(state_dir.join("active_provider"), name).context("Failed to write state file")?;
    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Initialize provider factory and determine active provider.
///
/// Returns the factory and the name of the currently active provider
/// (either from state file or config default).
fn get_factory_and_active(config: &Config) -> Result<(ProviderFactory, String)> {
    let factory = ProviderFactory::from_config(config)
        .map_err(|e| anyhow::anyhow!("{}", e))
        .context("Failed to initialize providers")?;

    let active_name = get_active_provider()
        .filter(|name| factory.contains(name))
        .unwrap_or_else(|| factory.default_provider_name().to_string());

    Ok((factory, active_name))
}

/// Get the model name for a provider from config.
fn get_model_for_provider(config: &Config, provider: &str) -> String {
    match provider {
        "openai" => config.openai.as_ref().map(|c| c.model.clone()),
        "anthropic" => config.anthropic.as_ref().map(|c| c.model.clone()),
        "ollama" => config.ollama.as_ref().map(|c| c.model.clone()),
        _ => None,
    }
    .unwrap_or_else(|| "unknown".to_string())
}

// ============================================================================
// Command Handlers
// ============================================================================

/// List all configured providers.
///
/// Shows all providers registered in the factory, marking the active one.
///
/// Format:
/// ```text
/// Available providers:
///   * anthropic (claude-sonnet-4-20250514) [active]
///     ollama (llama3.2)
///     openai (gpt-4o)
/// ```
pub fn run_list(config: &Config) -> Result<()> {
    let (factory, active_name) = get_factory_and_active(config)?;

    println!("Available providers:");
    for name in factory.list() {
        let model = get_model_for_provider(config, name);
        let marker = if name == active_name { "*" } else { " " };
        let active_label = if name == active_name { " [active]" } else { "" };
        println!("  {} {} ({}){}", marker, name, model, active_label);
    }

    Ok(())
}

/// Show the current provider and model.
///
/// Format: `Currently using: anthropic (claude-sonnet-4-20250514)`
pub fn run_current(config: &Config) -> Result<()> {
    let (_factory, active_name) = get_factory_and_active(config)?;

    let model = get_model_for_provider(config, &active_name);
    println!("Currently using: {} ({})", active_name, model);

    Ok(())
}

/// Switch to a different provider.
///
/// Validates that the provider exists in the factory before switching.
/// If the provider doesn't exist, shows an error and lists available providers.
pub fn run_switch(config: &Config, provider_name: &str) -> Result<()> {
    let (factory, _) = get_factory_and_active(config)?;

    // Validate provider exists
    if !factory.contains(provider_name) {
        let available = factory.list().join(", ");
        anyhow::bail!(
            "Provider '{}' not configured. Available: {}",
            provider_name,
            available
        );
    }

    // Save to state file
    set_active_provider(provider_name)?;

    let model = get_model_for_provider(config, provider_name);
    println!("Switched to: {} ({})", provider_name, model);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get_model_for_provider {
        use super::*;
        use cherry2k_core::config::{AnthropicConfig, OllamaConfig, OpenAiConfig};

        #[test]
        fn returns_configured_model() {
            let config = Config {
                openai: Some(OpenAiConfig {
                    model: "gpt-4o".to_string(),
                    ..Default::default()
                }),
                anthropic: Some(AnthropicConfig {
                    model: "claude-sonnet-4-20250514".to_string(),
                    ..Default::default()
                }),
                ollama: Some(OllamaConfig {
                    model: "llama3.2".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            };

            assert_eq!(get_model_for_provider(&config, "openai"), "gpt-4o");
            assert_eq!(
                get_model_for_provider(&config, "anthropic"),
                "claude-sonnet-4-20250514"
            );
            assert_eq!(get_model_for_provider(&config, "ollama"), "llama3.2");
        }

        #[test]
        fn returns_unknown_for_missing_config() {
            let config = Config::default();
            assert_eq!(get_model_for_provider(&config, "openai"), "unknown");
        }

        #[test]
        fn returns_unknown_for_unknown_provider() {
            let config = Config::default();
            assert_eq!(get_model_for_provider(&config, "nonexistent"), "unknown");
        }
    }
}
