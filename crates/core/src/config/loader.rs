//! Configuration loading logic for Cherry2K

use crate::config::types::*;
use crate::error::ConfigError;
use directories::ProjectDirs;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Load configuration from file and environment variables.
///
/// Priority (highest to lowest):
/// 1. Environment variables (OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.)
/// 2. Config file (~/.config/cherry2k/config.toml or CHERRY2K_CONFIG_PATH)
/// 3. Compiled defaults
///
/// # Errors
/// Returns ConfigError if config file exists but is malformed.
/// Missing config file is NOT an error - defaults are used.
pub fn load_config() -> Result<Config, ConfigError> {
    // Find config file path
    let config_path = get_config_path();

    // Load from file if exists
    let mut config = if config_path.exists() {
        let content = fs::read_to_string(&config_path).map_err(ConfigError::ReadError)?;
        toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?
    } else {
        Config::default()
    };

    // Apply environment variable overrides
    apply_env_overrides(&mut config);

    Ok(config)
}

/// Get the config file path.
/// Uses CHERRY2K_CONFIG_PATH if set, otherwise ~/.config/cherry2k/config.toml
pub fn get_config_path() -> PathBuf {
    if let Ok(path) = env::var("CHERRY2K_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    if let Some(proj_dirs) = ProjectDirs::from("com", "cherry2k", "cherry2k") {
        proj_dirs.config_dir().join("config.toml")
    } else {
        // Fallback if home directory detection fails - use current directory
        PathBuf::from(".cherry2k/config.toml")
    }
}

/// Apply environment variable overrides to config.
fn apply_env_overrides(config: &mut Config) {
    // Log level override
    if let Ok(level) = env::var("CHERRY2K_LOG_LEVEL") {
        config.general.log_level = level;
    }

    // Default provider override
    if let Ok(provider) = env::var("CHERRY2K_PROVIDER") {
        config.general.default_provider = provider;
    }

    // OpenAI overrides
    if let Ok(key) = env::var("OPENAI_API_KEY") {
        config
            .openai
            .get_or_insert_with(OpenAiConfig::default)
            .api_key = Some(key);
    }
    if let Ok(base_url) = env::var("OPENAI_BASE_URL") {
        config
            .openai
            .get_or_insert_with(OpenAiConfig::default)
            .base_url = base_url;
    }
    if let Ok(model) = env::var("OPENAI_MODEL") {
        config
            .openai
            .get_or_insert_with(OpenAiConfig::default)
            .model = model;
    }

    // Anthropic overrides
    if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
        config
            .anthropic
            .get_or_insert_with(AnthropicConfig::default)
            .api_key = Some(key);
    }
    if let Ok(model) = env::var("ANTHROPIC_MODEL") {
        config
            .anthropic
            .get_or_insert_with(AnthropicConfig::default)
            .model = model;
    }

    // Ollama overrides
    if let Ok(host) = env::var("OLLAMA_HOST") {
        config.ollama.get_or_insert_with(OllamaConfig::default).host = host;
    }
    if let Ok(model) = env::var("OLLAMA_MODEL") {
        config
            .ollama
            .get_or_insert_with(OllamaConfig::default)
            .model = model;
    }

    // Safety overrides (for testing/power users)
    if let Ok(val) = env::var("CHERRY2K_CONFIRM_COMMANDS") {
        config.safety.confirm_commands = val.parse().unwrap_or(true);
    }
    if let Ok(val) = env::var("CHERRY2K_CONFIRM_FILE_WRITES") {
        config.safety.confirm_file_writes = val.parse().unwrap_or(true);
    }
}

#[cfg(test)]
#[allow(unsafe_code)] // Required for env::set_var/remove_var in Rust 2024
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // SAFETY: These tests run sequentially (cargo test runs each test in its own thread
    // by default, but we're modifying process-global env vars). The unsafe blocks are
    // required in Rust 2024 edition because env::set_var/remove_var can cause data races.

    #[test]
    fn test_default_config_when_no_file() {
        // Ensure no config file exists at test path
        // SAFETY: Test environment, single-threaded test execution
        unsafe {
            env::set_var("CHERRY2K_CONFIG_PATH", "/nonexistent/path/config.toml");
        }
        let config = load_config().unwrap();
        assert_eq!(config.general.default_provider, "openai");
        assert!(config.safety.confirm_commands);
        // SAFETY: Cleanup after test
        unsafe {
            env::remove_var("CHERRY2K_CONFIG_PATH");
        }
    }

    #[test]
    fn test_env_override() {
        // SAFETY: Test environment, single-threaded test execution
        unsafe {
            env::set_var("CHERRY2K_CONFIG_PATH", "/nonexistent/path/config.toml");
            env::set_var("OPENAI_API_KEY", "test-key-123");
        }
        let config = load_config().unwrap();
        assert_eq!(
            config.openai.as_ref().unwrap().api_key,
            Some("test-key-123".to_string())
        );
        // SAFETY: Cleanup after test
        unsafe {
            env::remove_var("CHERRY2K_CONFIG_PATH");
            env::remove_var("OPENAI_API_KEY");
        }
    }

    #[test]
    fn test_config_file_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[general]
default_provider = "anthropic"
log_level = "debug"

[safety]
confirm_commands = false
"#
        )
        .unwrap();

        // SAFETY: Test environment, single-threaded test execution
        unsafe {
            env::set_var("CHERRY2K_CONFIG_PATH", file.path().to_str().unwrap());
        }
        let config = load_config().unwrap();
        assert_eq!(config.general.default_provider, "anthropic");
        assert_eq!(config.general.log_level, "debug");
        assert!(!config.safety.confirm_commands);
        // SAFETY: Cleanup after test
        unsafe {
            env::remove_var("CHERRY2K_CONFIG_PATH");
        }
    }

    #[test]
    fn test_invalid_toml_returns_error() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "this is not valid toml {{{{").unwrap();

        // SAFETY: Test environment, single-threaded test execution
        unsafe {
            env::set_var("CHERRY2K_CONFIG_PATH", file.path().to_str().unwrap());
        }
        let result = load_config();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::ParseError(_)));
        // SAFETY: Cleanup after test
        unsafe {
            env::remove_var("CHERRY2K_CONFIG_PATH");
        }
    }

    #[test]
    fn test_env_model_override_without_api_key() {
        // SAFETY: Cleanup any leftover env vars from other tests
        unsafe {
            env::remove_var("OPENAI_API_KEY");
            env::remove_var("OPENAI_MODEL");
            env::remove_var("OPENAI_BASE_URL");
        }

        // Test that OPENAI_MODEL creates config even without OPENAI_API_KEY
        // SAFETY: Test environment, single-threaded test execution
        unsafe {
            env::set_var("CHERRY2K_CONFIG_PATH", "/nonexistent/path/config.toml");
            env::set_var("OPENAI_MODEL", "gpt-4-turbo");
        }
        let config = load_config().unwrap();
        assert!(config.openai.is_some(), "openai config should exist");
        assert_eq!(config.openai.as_ref().unwrap().model, "gpt-4-turbo");
        // api_key should be None since we only set OPENAI_MODEL
        assert!(
            config.openai.as_ref().unwrap().api_key.is_none(),
            "api_key should be None when only OPENAI_MODEL is set"
        );
        // SAFETY: Cleanup after test
        unsafe {
            env::remove_var("CHERRY2K_CONFIG_PATH");
            env::remove_var("OPENAI_MODEL");
        }
    }
}
