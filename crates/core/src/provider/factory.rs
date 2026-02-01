//! Provider factory for dynamic provider registration and lookup.
//!
//! The [`ProviderFactory`] is responsible for:
//! - Creating provider instances from configuration
//! - Registering providers that pass validation
//! - Providing lookup by name and listing all available providers
//!
//! # Design
//!
//! The factory takes a [`Config`] and attempts to register each configured provider.
//! Invalid configurations are logged as warnings but don't block other providers.
//! At least one provider must be successfully registered for the factory to be usable.
//!
//! # Example
//!
//! ```ignore
//! use cherry2k_core::{Config, ProviderFactory};
//!
//! let config = Config::default();
//! let factory = ProviderFactory::from_config(&config)?;
//!
//! // Get default provider
//! let provider = factory.get_default();
//!
//! // Get specific provider
//! if let Some(anthropic) = factory.get("anthropic") {
//!     // Use Anthropic provider
//! }
//!
//! // List all available providers
//! for name in factory.list() {
//!     println!("Available: {}", name);
//! }
//! ```

use std::collections::HashMap;

use super::{AiProvider, AnthropicProvider, OllamaProvider, OpenAiProvider};
use crate::config::Config;
use crate::error::ConfigError;

/// Factory for creating and managing AI providers.
///
/// Registers providers based on configuration and provides lookup by name.
/// The factory ensures at least one provider is available after construction.
pub struct ProviderFactory {
    providers: HashMap<String, Box<dyn AiProvider>>,
    default_provider: String,
}

impl ProviderFactory {
    /// Create a new provider factory from configuration.
    ///
    /// This method:
    /// 1. Iterates through all provider configurations
    /// 2. Creates and validates each configured provider
    /// 3. Registers providers that pass validation (invalid ones are skipped with a warning)
    /// 4. Validates the default_provider setting
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::MissingField`] if no providers could be registered.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = Config::default();
    /// let factory = ProviderFactory::from_config(&config)?;
    /// ```
    pub fn from_config(config: &Config) -> Result<Self, ConfigError> {
        let mut providers: HashMap<String, Box<dyn AiProvider>> = HashMap::new();

        // Register OpenAI if configured
        if let Some(ref cfg) = config.openai {
            let provider = OpenAiProvider::new(cfg.clone());
            if let Err(e) = provider.validate_config() {
                tracing::warn!("OpenAI config invalid, skipping: {e}");
            } else {
                providers.insert("openai".to_string(), Box::new(provider));
            }
        }

        // Register Anthropic if configured
        if let Some(ref cfg) = config.anthropic {
            let provider = AnthropicProvider::new(cfg.clone());
            if let Err(e) = provider.validate_config() {
                tracing::warn!("Anthropic config invalid, skipping: {e}");
            } else {
                providers.insert("anthropic".to_string(), Box::new(provider));
            }
        }

        // Register Ollama if configured
        if let Some(ref cfg) = config.ollama {
            let provider = OllamaProvider::new(cfg.clone());
            if let Err(e) = provider.validate_config() {
                tracing::warn!("Ollama config invalid, skipping: {e}");
            } else {
                providers.insert("ollama".to_string(), Box::new(provider));
            }
        }

        // Validate we have at least one provider
        if providers.is_empty() {
            return Err(ConfigError::NoProviderAvailable {
                message: "Set OPENAI_API_KEY, ANTHROPIC_API_KEY, or configure Ollama.".to_string(),
            });
        }

        // Validate default_provider exists
        let default_provider = config.general.default_provider.clone();
        if !providers.contains_key(&default_provider) {
            // Pick first available provider as fallback (sorted for determinism)
            // SAFETY: We just verified providers is not empty above
            let mut available: Vec<_> = providers.keys().cloned().collect();
            available.sort();
            let fallback = available
                .into_iter()
                .next()
                .unwrap_or_else(|| unreachable!("providers verified non-empty above"));

            tracing::warn!(
                "Default provider '{}' not available, using '{}'",
                default_provider,
                fallback
            );

            return Ok(Self {
                providers,
                default_provider: fallback,
            });
        }

        Ok(Self {
            providers,
            default_provider,
        })
    }

    /// Get a provider by name.
    ///
    /// Returns `None` if the provider is not registered.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(provider) = factory.get("anthropic") {
    ///     let stream = provider.complete(request).await?;
    /// }
    /// ```
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&dyn AiProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    /// Get the default provider.
    ///
    /// This is guaranteed to return a valid provider after successful factory construction.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let provider = factory.get_default();
    /// let stream = provider.complete(request).await?;
    /// ```
    #[must_use]
    pub fn get_default(&self) -> &dyn AiProvider {
        // SAFETY: default_provider is guaranteed to exist after from_config succeeds.
        // The invariant is maintained by from_config which either:
        // - Sets default_provider to config value (after validating it exists), or
        // - Falls back to first available provider (after validating providers is non-empty)
        self.providers
            .get(&self.default_provider)
            .unwrap_or_else(|| unreachable!("default_provider invariant violated"))
            .as_ref()
    }

    /// Get the name of the default provider.
    ///
    /// # Example
    ///
    /// ```ignore
    /// println!("Using provider: {}", factory.default_provider_name());
    /// ```
    #[must_use]
    pub fn default_provider_name(&self) -> &str {
        &self.default_provider
    }

    /// List all registered provider names.
    ///
    /// Returns a sorted list for consistent ordering.
    ///
    /// # Example
    ///
    /// ```ignore
    /// for name in factory.list() {
    ///     println!("Available: {}", name);
    /// }
    /// ```
    #[must_use]
    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<_> = self.providers.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Check if a provider is registered.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if factory.contains("ollama") {
    ///     println!("Ollama is available");
    /// }
    /// ```
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AnthropicConfig, GeneralConfig, OllamaConfig, OpenAiConfig};

    mod fixtures {
        use super::*;

        pub fn config_openai_only() -> Config {
            Config {
                general: GeneralConfig {
                    default_provider: "openai".to_string(),
                    ..Default::default()
                },
                openai: Some(OpenAiConfig {
                    api_key: Some("sk-test123".to_string()),
                    ..Default::default()
                }),
                anthropic: None,
                ollama: None,
                ..Default::default()
            }
        }

        pub fn config_multiple_providers() -> Config {
            Config {
                general: GeneralConfig {
                    default_provider: "anthropic".to_string(),
                    ..Default::default()
                },
                openai: Some(OpenAiConfig {
                    api_key: Some("sk-test123".to_string()),
                    ..Default::default()
                }),
                anthropic: Some(AnthropicConfig {
                    api_key: Some("sk-ant-test123".to_string()),
                    ..Default::default()
                }),
                ollama: Some(OllamaConfig::default()),
                ..Default::default()
            }
        }

        pub fn config_no_providers() -> Config {
            Config {
                general: GeneralConfig::default(),
                openai: None,
                anthropic: None,
                ollama: None,
                ..Default::default()
            }
        }

        pub fn config_with_invalid_default() -> Config {
            Config {
                general: GeneralConfig {
                    default_provider: "nonexistent".to_string(),
                    ..Default::default()
                },
                openai: Some(OpenAiConfig {
                    api_key: Some("sk-test123".to_string()),
                    ..Default::default()
                }),
                anthropic: None,
                ollama: None,
                ..Default::default()
            }
        }

        pub fn config_with_invalid_provider() -> Config {
            Config {
                general: GeneralConfig {
                    default_provider: "openai".to_string(),
                    ..Default::default()
                },
                // OpenAI with empty key (invalid)
                openai: Some(OpenAiConfig {
                    api_key: Some("".to_string()),
                    ..Default::default()
                }),
                // Anthropic with valid key
                anthropic: Some(AnthropicConfig {
                    api_key: Some("sk-ant-test123".to_string()),
                    ..Default::default()
                }),
                ollama: None,
                ..Default::default()
            }
        }
    }

    mod from_config {
        use super::*;

        #[test]
        fn with_openai_only() {
            let config = fixtures::config_openai_only();
            let factory = ProviderFactory::from_config(&config).unwrap();

            assert!(factory.contains("openai"));
            assert!(!factory.contains("anthropic"));
            assert!(!factory.contains("ollama"));
            assert_eq!(factory.default_provider_name(), "openai");
        }

        #[test]
        fn with_multiple_providers() {
            let config = fixtures::config_multiple_providers();
            let factory = ProviderFactory::from_config(&config).unwrap();

            assert!(factory.contains("openai"));
            assert!(factory.contains("anthropic"));
            assert!(factory.contains("ollama"));
            assert_eq!(factory.default_provider_name(), "anthropic");
        }

        #[test]
        fn no_providers_fails() {
            let config = fixtures::config_no_providers();
            let result = ProviderFactory::from_config(&config);

            assert!(matches!(
                result,
                Err(ConfigError::NoProviderAvailable { .. })
            ));
        }

        #[test]
        fn invalid_default_falls_back() {
            let config = fixtures::config_with_invalid_default();
            let factory = ProviderFactory::from_config(&config).unwrap();

            // Should fall back to first available (openai)
            assert_eq!(factory.default_provider_name(), "openai");
        }

        #[test]
        fn invalid_provider_skipped() {
            let config = fixtures::config_with_invalid_provider();
            let factory = ProviderFactory::from_config(&config).unwrap();

            // OpenAI invalid (empty key), should not be registered
            assert!(!factory.contains("openai"));
            // Anthropic valid, should be registered
            assert!(factory.contains("anthropic"));
            // Default was openai, but it's invalid, so falls back to anthropic
            assert_eq!(factory.default_provider_name(), "anthropic");
        }
    }

    mod get {
        use super::*;

        #[test]
        fn returns_provider() {
            let config = fixtures::config_openai_only();
            let factory = ProviderFactory::from_config(&config).unwrap();

            let provider = factory.get("openai");
            assert!(provider.is_some());
            assert_eq!(provider.unwrap().provider_id(), "openai");
        }

        #[test]
        fn unknown_returns_none() {
            let config = fixtures::config_openai_only();
            let factory = ProviderFactory::from_config(&config).unwrap();

            assert!(factory.get("nonexistent").is_none());
        }
    }

    mod get_default {
        use super::*;

        #[test]
        fn returns_default_provider() {
            let config = fixtures::config_multiple_providers();
            let factory = ProviderFactory::from_config(&config).unwrap();

            let provider = factory.get_default();
            assert_eq!(provider.provider_id(), "anthropic");
        }
    }

    mod list {
        use super::*;

        #[test]
        fn returns_all_providers_sorted() {
            let config = fixtures::config_multiple_providers();
            let factory = ProviderFactory::from_config(&config).unwrap();

            let providers = factory.list();
            assert_eq!(providers, vec!["anthropic", "ollama", "openai"]);
        }

        #[test]
        fn single_provider() {
            let config = fixtures::config_openai_only();
            let factory = ProviderFactory::from_config(&config).unwrap();

            let providers = factory.list();
            assert_eq!(providers, vec!["openai"]);
        }
    }

    mod contains {
        use super::*;

        #[test]
        fn returns_true_for_registered() {
            let config = fixtures::config_openai_only();
            let factory = ProviderFactory::from_config(&config).unwrap();

            assert!(factory.contains("openai"));
        }

        #[test]
        fn returns_false_for_unregistered() {
            let config = fixtures::config_openai_only();
            let factory = ProviderFactory::from_config(&config).unwrap();

            assert!(!factory.contains("anthropic"));
        }
    }
}
