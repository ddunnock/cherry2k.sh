//! Configuration type definitions for Cherry2K
//!
//! All configuration types use serde for deserialization and provide sensible defaults.

use serde::Deserialize;

/// Root configuration structure
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,
    /// OpenAI provider settings
    pub openai: Option<OpenAiConfig>,
    /// Anthropic provider settings
    pub anthropic: Option<AnthropicConfig>,
    /// Ollama provider settings
    pub ollama: Option<OllamaConfig>,
    /// Safety settings
    pub safety: SafetyConfig,
}

/// General application settings
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Default provider to use (openai, anthropic, ollama)
    pub default_provider: String,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            log_level: "info".to_string(),
        }
    }
}

/// OpenAI provider configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OpenAiConfig {
    /// API key (prefer env var OPENAI_API_KEY)
    pub api_key: Option<String>,
    /// Base URL for API (default: <https://api.openai.com/v1>)
    /// Allows using OpenAI-compatible APIs
    pub base_url: String,
    /// Model to use (default: gpt-4o)
    pub model: String,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o".to_string(),
        }
    }
}

/// Anthropic provider configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AnthropicConfig {
    /// API key (prefer env var ANTHROPIC_API_KEY)
    pub api_key: Option<String>,
    /// Model to use (default: claude-sonnet-4-20250514)
    pub model: String,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: "claude-sonnet-4-20250514".to_string(),
        }
    }
}

/// Ollama provider configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OllamaConfig {
    /// Ollama host URL (default: <http://localhost:11434>)
    pub host: String,
    /// Model to use (default: llama3.2)
    pub model: String,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: "http://localhost:11434".to_string(),
            model: "llama3.2".to_string(),
        }
    }
}

/// Safety configuration for command execution
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SafetyConfig {
    /// Require confirmation before executing commands (default: true)
    pub confirm_commands: bool,
    /// Require confirmation before file writes (default: true)
    pub confirm_file_writes: bool,
    /// List of blocked command patterns
    pub blocked_patterns: Vec<String>,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            confirm_commands: true,
            confirm_file_writes: true,
            blocked_patterns: vec![
                "rm -rf /".to_string(),
                "rm -rf ~".to_string(),
                "> /dev/sda".to_string(),
                "mkfs".to_string(),
                ":(){:|:&};:".to_string(), // fork bomb
            ],
        }
    }
}
