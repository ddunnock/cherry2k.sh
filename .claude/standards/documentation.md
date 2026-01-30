# Documentation Standards

> **Applies to**: All documentation in Cherry2K.sh
> **Parent**: `constitution.md`

---

## 1. Documentation Types

Cherry2K.sh uses Rust's native documentation system:

| Type               | Tool                  | Location        |
|--------------------|-----------------------|-----------------|
| API Documentation  | `cargo doc` (rustdoc) | `target/doc/`   |
| User Documentation | README.md, docs/      | Repository root |
| Code Comments      | Inline comments       | Source files    |

---

## 2. Rustdoc Standards

### 2.1 Module Documentation

Every module **MUST** have module-level documentation:

```rust
//! AI Provider abstraction for Cherry2K.sh.
//!
//! This module defines the core [`AiProvider`] trait and implementations
//! for OpenAI, Anthropic, and Ollama backends.
//!
//! # Overview
//!
//! The provider system enables seamless switching between AI backends
//! without changing application code.
//!
//! # Examples
//!
//! ```rust,no_run
//! use cherry2k_core::provider::{ProviderFactory, CompletionRequest};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let provider = ProviderFactory::from_env()?;
//! let request = CompletionRequest::new("Hello!");
//! let response = provider.complete(request).await?;
//! println!("{}", response.content);
//! # Ok(())
//! # }
//! ```
//!
//! # Feature Flags
//!
//! - `openai` - Enable OpenAI provider (default)
//! - `anthropic` - Enable Anthropic provider (default)
//! - `ollama` - Enable Ollama provider (default)
```

### 2.2 Public Item Documentation

All public items **MUST** have documentation:

```rust
/// A completion request to send to an AI provider.
///
/// This struct encapsulates all parameters needed for a completion request,
/// including the prompt, model settings, and optional conversation context.
///
/// # Examples
///
/// ```rust
/// use cherry2k_core::provider::CompletionRequest;
///
/// // Simple request
/// let request = CompletionRequest::new("Explain Rust ownership");
///
/// // Request with options
/// let request = CompletionRequest::builder()
///     .prompt("Explain Rust ownership")
///     .max_tokens(500)
///     .temperature(0.7)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// The prompt text to send to the model.
    pub prompt: String,

    /// Maximum tokens in the response.
    ///
    /// Defaults to 1000. Set to 0 for no limit (provider default).
    pub max_tokens: usize,

    /// Sampling temperature (0.0 - 2.0).
    ///
    /// Lower values are more deterministic, higher values more creative.
    /// Defaults to 0.7.
    pub temperature: f32,

    /// Previous messages for conversation context.
    ///
    /// Optional. If provided, the prompt is treated as a continuation.
    pub history: Option<Vec<Message>>,
}
```

### 2.3 Function Documentation

```rust
/// Send a completion request to the AI provider.
///
/// This method handles the full request lifecycle including rate limiting,
/// retries, and streaming response parsing.
///
/// # Arguments
///
/// * `request` - The completion request containing prompt and parameters.
///
/// # Returns
///
/// Returns a stream of response chunks on success.
///
/// # Errors
///
/// This function returns an error if:
///
/// * Network connection fails ([`ProviderError::Network`])
/// * API returns an error response ([`ProviderError::Api`])
/// * Rate limit is exceeded ([`ProviderError::RateLimited`])
/// * Authentication fails ([`ProviderError::Auth`])
///
/// # Examples
///
/// ```rust,no_run
/// use cherry2k_core::provider::{AiProvider, CompletionRequest};
/// use futures::StreamExt;
///
/// # async fn example(provider: impl AiProvider) -> anyhow::Result<()> {
/// let request = CompletionRequest::new("Hello!");
/// let mut stream = provider.complete(request).await?;
///
/// while let Some(chunk) = stream.next().await {
///     print!("{}", chunk?);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Panics
///
/// This function does not panic.
pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionStream, ProviderError> {
    // ...
}
```

### 2.4 Error Documentation

```rust
/// Errors that can occur during provider operations.
///
/// # Error Handling
///
/// Most errors are recoverable:
///
/// - [`ProviderError::RateLimited`] - Wait and retry
/// - [`ProviderError::Network`] - May be transient, retry with backoff
/// - [`ProviderError::Auth`] - Check API key configuration
/// - [`ProviderError::Api`] - Log and report to user
///
/// # Examples
///
/// ```rust
/// use cherry2k_core::provider::ProviderError;
///
/// fn handle_error(err: ProviderError) {
///     match err {
///         ProviderError::RateLimited { retry_after } => {
///             eprintln!("Rate limited. Retry in {} seconds.", retry_after);
///         }
///         ProviderError::Auth(msg) => {
///             eprintln!("Authentication failed: {}", msg);
///         }
///         _ => eprintln!("Error: {}", err),
///     }
/// }
/// ```
#[derive(Debug, Error)]
pub enum ProviderError {
    /// Network request failed.
    ///
    /// This may be transient. Consider retrying with exponential backoff.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Rate limit exceeded.
    ///
    /// The `retry_after` field indicates how long to wait before retrying.
    #[error("rate limited, retry after {retry_after} seconds")]
    RateLimited {
        /// Seconds to wait before retrying.
        retry_after: u64,
    },

    // ...
}
```

---

## 3. Documentation Commands

### 3.1 Building Documentation

```bash
# Build documentation
cargo doc

# Build and open in browser
cargo doc --open

# Build with private items (for development)
cargo doc --document-private-items

# Build specific crate
cargo doc -p cherry2k-core
```

### 3.2 Documentation Tests

```bash
# Run documentation examples as tests
cargo test --doc

# Run all tests including doc tests
cargo test
```

### 3.3 CI Documentation Check

```bash
# Check documentation builds without warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

---

## 4. README.md Structure

### 4.1 Required Sections

```markdown
# Cherry2K.sh

> AI assistant for your zsh terminal

Brief description (1-2 sentences).

## Features

- Feature 1
- Feature 2
- Feature 3

## Installation

### Homebrew (Recommended)

\`\`\`bash
brew tap dunnock/tap
brew install cherry2k
\`\`\`

### From Source

\`\`\`bash
cargo install --git https://github.com/dunnock/cherry2k
\`\`\`

## Quick Start

\`\`\`bash
# Set your API key
export OPENAI_API_KEY=sk-...

# One-shot query
cherry2k chat "Explain Rust ownership"

# Interactive mode
cherry2k repl
\`\`\`

## Configuration

Configuration details...

## Usage

Detailed usage examples...

## Providers

### OpenAI

...

### Anthropic

...

### Ollama

...

## Zsh Integration

...

## Contributing

...

## License

MIT
```

---

## 5. Code Comments

### 5.1 When to Comment

**DO** comment:

- Non-obvious algorithms
- Performance-critical code
- Workarounds and their reasons
- Safety invariants
- TODO items (with issue reference)

**DON'T** comment:

- Self-explanatory code
- What the code does (doc comments handle this)
- Obvious implementations

### 5.2 Comment Style

```rust
// Single-line comments for brief explanations
let timeout = Duration::from_secs(30);

// Multi-line comments for longer explanations.
// Break at natural points in the explanation.
// Keep lines under 80 characters.
let complex_value = calculate_something();

// TODO(#123): Implement retry logic with exponential backoff
// Currently we just fail on first error.

// HACK: Work around issue in reqwest where streaming responses
// can hang if the server doesn't send a final newline.
// Remove when https://github.com/seanmonstar/reqwest/issues/XXX is fixed.
response.bytes_stream().chain(stream::once(async { Ok(Bytes::new()) }))

// SAFETY: We've verified that the slice is valid UTF-8 in the line above.
// The check is: input.iter().all(|b| b.is_ascii())
let text = unsafe { std::str::from_utf8_unchecked(bytes) };
```

### 5.3 Inline Documentation

For complex functions, use inline comments to explain sections:

```rust
pub fn parse_sse_event(data: &str) -> Result<SseEvent, ParseError> {
    // SSE format: "data: {json}\n\n"
    // Multiple data lines are concatenated.

    let mut lines = data.lines();

    // Skip empty lines at the start
    let first_line = lines
        .find(|line| !line.is_empty())
        .ok_or(ParseError::EmptyEvent)?;

    // Extract the event type if present
    let event_type = if first_line.starts_with("event:") {
        Some(first_line.strip_prefix("event:").unwrap().trim())
    } else {
        None
    };

    // Collect all data lines
    let data: String = lines
        .filter(|line| line.starts_with("data:"))
        .map(|line| line.strip_prefix("data:").unwrap().trim())
        .collect::<Vec<_>>()
        .join("\n");

    // Parse the JSON payload
    let payload = serde_json::from_str(&data)?;

    Ok(SseEvent { event_type, payload })
}
```

---

## 6. CHANGELOG.md

### 6.1 Format

Follow [Keep a Changelog](https://keepachangelog.com/):

```markdown
# Changelog

All notable changes to Cherry2K.sh will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- New feature description (#PR)

### Changed

- Changed behavior description (#PR)

### Deprecated

- Deprecated feature description (#PR)

### Removed

- Removed feature description (#PR)

### Fixed

- Bug fix description (#PR)

### Security

- Security fix description (#PR)

## [0.2.0] - 2026-02-15

### Added

- Streaming response support for all providers (#45)
- Conversation history persistence (#48)

### Changed

- Improved error messages for configuration issues (#52)

### Fixed

- SQLite busy timeout handling (#42)

## [0.1.0] - 2026-01-29

### Added

- Initial release
- OpenAI provider support
- Anthropic provider support
- Ollama provider support
- SQLite conversation storage
- Zsh integration with widgets
```

---

## 7. Documentation Quality Checklist

### Before Merge

- [ ] All public items have doc comments
- [ ] Doc comments include examples
- [ ] Examples compile (tested with `cargo test --doc`)
- [ ] `# Errors` section lists all error conditions
- [ ] `# Panics` section present (even if "does not panic")
- [ ] README is up to date
- [ ] CHANGELOG updated for user-facing changes

### Periodic Review

- [ ] Documentation matches current behavior
- [ ] Examples still work with latest API
- [ ] Links are not broken
- [ ] No outdated information