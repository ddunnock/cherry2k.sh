# Coding Conventions

**Analysis Date:** 2026-01-29

## Naming Patterns

**Files:**
- Rust module files: `snake_case.rs` (e.g., `openai.rs`, `provider_config.rs`)
- Zsh functions: `kebab-case` (e.g., `cherry2k-complete`, `cherry2k-assist-widget`)
- Config files: `snake_case` (e.g., `cherry2k_config.toml`)
- SQL migrations: `NNNN_description.sql` (e.g., `0001_initial_schema.sql`, `0002_conversations.sql`)

**Functions:**
- Rust functions: `snake_case` (e.g., `complete_request()`, `validate_prompt()`)
- Test functions: `snake_case` descriptive names following pattern `<action>_<condition>_<expected_result>` (e.g., `creates_provider_with_valid_config()`, `handles_rate_limit_with_retry()`)
- Async functions: Prefix with `async fn` and maintain `snake_case` (e.g., `async fn fetch_completion()`)

**Variables:**
- Local variables: `snake_case` (e.g., `request`, `api_key`, `max_tokens`)
- Constants: `UPPER_SNAKE_CASE` (e.g., `MAX_PROMPT_LENGTH`, `DEFAULT_TIMEOUT`)
- Module-level constants: `UPPER_SNAKE_CASE` for all caps (e.g., `MIGRATIONS: &[(&str, &str)]`)

**Types:**
- Structs: `PascalCase` (e.g., `OpenAiProvider`, `CompletionRequest`, `StorageError`)
- Enums: `PascalCase` (e.g., `ProviderError`, `OutputFormat`, `ConfigError`)
- Traits: `PascalCase` (e.g., `AiProvider`, `Repository`)
- Type aliases: `PascalCase` (e.g., `CompletionStream`)

## Code Style

**Formatting:**
- Use `cargo fmt` with default Rust settings
- Run before every commit: `cargo fmt --check`
- Max line width: 100 characters (configured in `rustfmt.toml`)

**Linting:**
- Use `cargo clippy -- -D warnings` to enforce all checks
- Primary rule sets: `clippy::all`, `clippy::pedantic`, `clippy::nursery`
- Deny: `unwrap_used`, `panic` (in library code), `unsafe_code`
- Warn: `expect_used`, `todo`, `dbg_macro`, `print_stdout`, `print_stderr`

**Clippy Settings** (in `clippy.toml`):
```toml
cognitive-complexity-threshold = 25
too-many-arguments-threshold = 7
type-complexity-threshold = 250
```

## Import Organization

**Order:**
1. Standard library imports (e.g., `use std::...`)
2. External crate imports (e.g., `use tokio::...`, `use serde::...`)
3. Internal crate imports (e.g., `use cherry2k_core::...`)
4. Private module imports (e.g., `use super::...`)

**Path Aliases:**
- Workspace-level aliases configured in each `Cargo.toml`
- Use fully qualified paths from crate root for clarity
- No `use crate::*` wildcards in library code

**Example:**
```rust
use std::path::Path;
use tokio::fs;
use serde::{Serialize, Deserialize};
use cherry2k_core::provider::AiProvider;
use super::fixtures;
```

## Error Handling

**Patterns:**
- Use `thiserror` for all library error types
- All error types **MUST** derive `Debug` and `Error`
- Use `Result<T, ErrorType>` type alias pattern for consistency
- Propagate errors with `?` operator, never use `.unwrap()` or `.expect()` in library code
- Add context with `anyhow::Context` trait when propagating errors

**Error Type Pattern:**
```rust
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("database connection failed: {0}")]
    Connection(#[from] rusqlite::Error),

    #[error("migration {version} failed: {message}")]
    Migration { version: u32, message: String },

    #[error("record not found: {0}")]
    NotFound(String),
}
```

**Acceptable `.expect()` usage (main.rs only):**
```rust
let config = Config::from_env()
    .expect("CONFIG: Missing required environment variables");
```

## Logging

**Framework:** `tracing` crate with structured logging

**Patterns:**
- Use `#[instrument]` macro on functions to add context
- Log operation outcomes, not intermediate steps
- Use `info!()` for normal operations, `warn!()` for concerning situations, `error!()` for failures
- Log metadata, not secrets: Never log API keys, tokens, or user prompts
- Use structured fields: `info!(tokens = 100, model = "gpt-4", "request sent")`

**Example:**
```rust
use tracing::{info, instrument};

#[instrument(skip(request), fields(provider = %self.provider_id()))]
async fn complete(&self, request: CompletionRequest) -> Result<Response, ProviderError> {
    info!(
        model = %request.model,
        max_tokens = request.max_tokens,
        "Sending completion request"
    );
    // ...
}
```

**What NOT to Log:**
- API keys or tokens (even partial)
- User prompts in production
- Full error bodies from API responses
- Stack traces to user-facing output

## Comments

**When to Comment:**
- Non-obvious algorithms or complex logic
- Performance-critical sections with reasoning
- Workarounds and their reasons with issue references
- Safety invariants for unsafe code blocks
- TODO items with issue numbers (e.g., `// TODO(#42): implement retry logic`)

**When NOT to Comment:**
- Self-explanatory code
- What the code does (doc comments handle this)
- Obvious implementations or variable assignments

**Comment Style:**
- Single-line comments: `// Brief explanation`
- Multi-line comments: Break at natural points, keep lines under 80 chars
- Prefix special comments: `TODO(#123):`, `HACK:`, `SAFETY:`, `FIXME:`

**Example:**
```rust
// TODO(#123): Implement exponential backoff for rate limiting
// Currently we just fail on first rate limit error

// HACK: Work around issue in reqwest where streaming responses
// hang without final newline. Remove when upstream fixed.

// SAFETY: We verified input is valid UTF-8 above via is_ascii() check
let text = unsafe { std::str::from_utf8_unchecked(bytes) };
```

## Function Design

**Size:** Target <50 lines for most functions
- Break complex logic into smaller helper functions
- If >100 lines: likely needs refactoring

**Parameters:**
- Limit to 7 parameters (Clippy enforces)
- Use structs for related parameters
- Accept references to avoid unnecessary clones
- Use trait objects for polymorphism (e.g., `dyn AiProvider`)

**Return Values:**
- Return `Result<T, ErrorType>` for fallible operations
- Return `Option<T>` for nullable values (prefer Result with descriptive error)
- Use type aliases for complex return types: `type CompletionStream = Pin<Box<dyn Stream<...>>>`

## Module Design

**Exports:**
- Use `pub mod` for submodules, `pub use` for re-exports
- Keep public API minimal and clear
- Document public items at module level

**Barrel Files (mod.rs):**
- `crates/core/src/provider/mod.rs` exports `AiProvider`, provider implementations
- `crates/storage/src/lib.rs` exports main storage types
- Avoid re-exporting implementation details

**Example Structure:**
```rust
// src/provider/mod.rs
pub mod trait;
pub mod openai;
pub mod anthropic;

pub use self::trait::AiProvider;
pub use self::openai::OpenAiProvider;
pub use self::anthropic::AnthropicProvider;
```

## Documentation

**Module Documentation:**
Every module **MUST** have module-level documentation (starts with `//!`):
```rust
//! AI Provider abstraction for Cherry2K.sh.
//!
//! This module defines the [`AiProvider`] trait that all AI backends
//! must implement, enabling seamless switching between OpenAI, Anthropic,
//! and Ollama.
//!
//! # Examples
//!
//! ```rust,no_run
//! use cherry2k_core::provider::{AiProvider, OpenAiProvider};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let provider = OpenAiProvider::new(config)?;
//! let response = provider.complete(request).await?;
//! # Ok(())
//! # }
//! ```
```

**Public Item Documentation:**
All public types, functions, and traits **MUST** have doc comments:
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
/// # Errors
///
/// This function returns an error if:
/// - Network connection fails
/// - API returns an error response
/// - Rate limit is exceeded
/// - Authentication fails
///
/// # Examples
///
/// ```rust,no_run
/// use cherry2k_core::provider::{AiProvider, CompletionRequest};
/// # async fn example(provider: impl AiProvider) -> anyhow::Result<()> {
/// let response = provider.complete(request).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Panics
///
/// This function does not panic.
pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionStream, ProviderError>
```

**Error Type Documentation:**
```rust
/// Errors that can occur during provider operations.
///
/// # Error Handling
///
/// Most errors are recoverable:
/// - [`ProviderError::RateLimited`] - Wait and retry
/// - [`ProviderError::Network`] - May be transient, retry with backoff
#[derive(Debug, Error)]
pub enum ProviderError {
    /// Network request failed. May be transient—consider retrying.
    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),

    /// Rate limit exceeded. Wait before retrying.
    #[error("rate limited, retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },
}
```

## Type Safety and Patterns

**Generic Constraints:**
- Use trait bounds for clear intent: `T: Send + Sync`
- Prefer specific trait bounds over generic where possible
- Document lifetime parameters in public APIs

**Async Traits:**
Use native async/await on traits (Rust 1.75+), no `async-trait` crate:
```rust
pub trait AiProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionStream, ProviderError>;
}
```

**Streaming Patterns:**
Use `futures::Stream` with `Pin<Box<...>>` for return types:
```rust
pub type CompletionStream = Pin<Box<dyn Stream<Item = Result<String, ProviderError>> + Send>>;
```

## SQLite Patterns

**Connection Management:**
```rust
pub fn open_database(path: &Path) -> Result<Connection, StorageError> {
    let conn = Connection::open_with_flags(path, flags)?;
    // Enable WAL mode for better concurrency
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
    Ok(conn)
}
```

**Parameterized Queries (MANDATORY):**
```rust
// ✅ Good
conn.query_row(
    "SELECT * FROM conversations WHERE id = ?",
    [id],
    |row| row.get::<_, String>(0),
)?;

// ❌ Bad - SQL injection risk
let query = format!("SELECT * FROM conversations WHERE id = '{}'", id);
```

**Repository Pattern:**
Use repository trait for data access abstraction:
```rust
pub trait Repository {
    fn save(&self, item: &Item) -> Result<(), StorageError>;
    fn find(&self, id: &str) -> Result<Option<Item>, StorageError>;
}
```

## CLI Patterns

**Command Structure (using clap):**
```rust
#[derive(Parser)]
#[command(name = "cherry2k")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(long, default_value = "text", global = true)]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}
```

**Output Formatting:**
Provide both human-readable and machine-readable (JSON) output:
```rust
#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

pub fn format_output<T: Serialize + std::fmt::Display>(
    value: &T,
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Text => value.to_string(),
        OutputFormat::Json => serde_json::to_string_pretty(value)
            .unwrap_or_else(|_| r#"{"error": "serialization failed"}"#.to_string())
    }
}
```

## Workspace Dependencies

**Workspace-level Cargo.toml configuration:**
- Pin all workspace dependencies at workspace level
- Crates inherit via `workspace = true`
- Keep consistent versions across crates

**Key Dependencies:**
- `tokio` (1.35+): Async runtime with full features
- `reqwest` (0.12+): HTTP client with JSON and streaming
- `rusqlite` (0.32+): SQLite with bundled library
- `serde` (1.0+): Serialization with derive
- `thiserror` (2.0+): Error handling
- `clap` (4.5+): CLI argument parsing
- `tracing` (0.1+): Structured logging

---

*Convention analysis: 2026-01-29*
