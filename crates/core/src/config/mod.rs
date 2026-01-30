//! Configuration module for Cherry2K
//!
//! This module provides configuration loading with support for:
//! - TOML configuration files (~/.config/cherry2k/config.toml)
//! - Environment variable overrides (OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.)
//! - Sensible defaults when no configuration is provided
//!
//! # Priority (highest to lowest)
//! 1. Environment variables
//! 2. Config file
//! 3. Compiled defaults
//!
//! # Example
//! ```no_run
//! use cherry2k_core::config::load_config;
//!
//! let config = load_config().expect("Failed to load config");
//! println!("Default provider: {}", config.general.default_provider);
//! ```

mod loader;
mod types;

pub use loader::{get_config_path, load_config};
pub use types::{AnthropicConfig, Config, GeneralConfig, OllamaConfig, OpenAiConfig, SafetyConfig};
