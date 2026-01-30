# Rust Standards

> **Applies to**: All Rust code in Cherry2K.sh
> **Version Constraint**: ≥1.75 (stable)
> **Parent**: `constitution.md`

---

## 1. Version and Toolchain

| Constraint      | Value   | Rationale                                     |
|-----------------|---------|-----------------------------------------------|
| Minimum Version | 1.75    | Async trait stability, RPITIT                 |
| Channel         | stable  | No nightly features without documented reason |
| Edition         | 2021    | Current stable edition                        |

---

## 2. Project Structure

### 2.1 Workspace Layout

```
crates/
├── core/                       # Domain logic + provider abstraction
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── provider/
│       │   ├── mod.rs
│       │   ├── trait.rs        # AiProvider trait definition
│       │   ├── openai.rs
│       │   ├── anthropic.rs
│       │   └── ollama.rs
│       ├── conversation/
│       │   ├── mod.rs
│       │   ├── message.rs
│       │   └── context.rs
│       ├── config/
│       │   ├── mod.rs
│       │   └── provider_config.rs
│       └── error.rs
├── storage/                    # SQLite persistence
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── schema.rs
│       ├── migrations/
│       │   ├── mod.rs
│       │   ├── 0001_initial.sql
│       │   └── 0002_conversations.sql
│       └── repository.rs
└── cli/                        # Terminal interface
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── commands/
        │   ├── mod.rs
        │   ├── chat.rs
        │   ├── config.rs
        │   └── history.rs
        ├── repl/
        │   ├── mod.rs
        │   └── readline.rs
        └── output/
            ├── mod.rs
            └── formatter.rs
```

### 2.2 Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["David Dunnock"]
license = "MIT"
repository = "https://github.com/dunnock/cherry2k"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# HTTP client
reqwest = { version = "0.12", features = ["json", "stream"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
rusqlite = { version = "0.32", features = ["bundled"] }

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# CLI
clap = { version = "4.5", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Streaming
futures = "0.3"
async-stream = "0.3"

# Testing
tokio-test = "0.4"
wiremock = "0.6"

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
unwrap_used = "deny"
expect_used = "warn"
panic = "deny"
```

### 2.3 Crate Cargo.toml Pattern

```toml
[package]
name = "cherry2k-core"
version.workspace = true
edition.workspace = true
authors.workspace = true

[dependencies]
tokio.workspace = true
reqwest.workspace = true
serde.workspace = true
thiserror.workspace = true

[dev-dependencies]
tokio-test.workspace = true
wiremock.workspace = true

[lints]
workspace = true
```

---

## 3. Documentation Standards

### 3.1 Module Documentation

Every module **MUST** have module-level documentation:

```rust
//! AI Provider abstraction for Cherry2K.sh.
//!
//! This module defines the [`AiProvider`] trait that all AI backends
//! must implement, enabling seamless switching between OpenAI, Anthropic,
//! and Ollama.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     ┌──────────────┐
//! │   CLI/REPL  │────▶│  AiProvider  │
//! └─────────────┘     └──────┬───────┘
//!                            │
//!        ┌───────────────────┼───────────────────┐
//!        ▼                   ▼                   ▼
//! ┌────────────┐     ┌─────────────┐     ┌────────────┐
//! │   OpenAI   │     │  Anthropic  │     │   Ollama   │
//! └────────────┘     └─────────────┘     └────────────┘
//! ```
//!
//! # Examples
//!
//! ```rust,no_run
//! use cherry2k_core::provider::{AiProvider, OpenAiProvider, ProviderConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = ProviderConfig::from_env()?;
//!     let provider = OpenAiProvider::new(config)?;
//!
//!     let response = provider.complete("Hello, world!").await?;
//!     println!("{}", response.content);
//!     Ok(())
//! }
//! ```
```

### 3.2 Trait Documentation

```rust
/// Core trait for AI provider implementations.
///
/// All AI backends (OpenAI, Anthropic, Ollama) **MUST** implement this trait
/// to be usable with Cherry2K.sh.
///
/// # Implementor Requirements
///
/// - Handle rate limiting with exponential backoff
/// - Validate configuration before first request
/// - Support streaming responses via [`CompletionStream`]
/// - Propagate errors without panicking
///
/// # Examples
///
/// ```rust
/// use cherry2k_core::provider::{AiProvider, CompletionRequest};
///
/// async fn use_provider(provider: &dyn AiProvider) -> anyhow::Result<()> {
///     let request = CompletionRequest::new("Explain Rust ownership");
///     let mut stream = provider.complete(request).await?;
///
///     while let Some(chunk) = stream.next().await {
///         print!("{}", chunk?);
///     }
///     Ok(())
/// }
/// ```
pub trait AiProvider: Send + Sync {
    /// Send a completion request and receive a streaming response.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] if:
    /// - Network request fails
    /// - API returns an error response
    /// - Rate limit is exceeded
    /// - Authentication fails
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionStream, ProviderError>;

    /// Returns the provider identifier for logging and configuration.
    fn provider_id(&self) -> &'static str;

    /// Validate provider configuration before use.
    fn validate_config(&self) -> Result<(), ConfigError>;
}
```

### 3.3 Error Type Documentation

```rust
/// Errors that can occur when interacting with AI providers.
///
/// # Error Handling Strategy
///
/// - **Retryable errors** (rate limits, timeouts): Implement exponential backoff
/// - **Configuration errors**: Fail fast, prompt user to fix
/// - **API errors**: Log and surface to user
#[derive(Debug, Error)]
pub enum ProviderError {
    /// Network request failed. May be transient—consider retrying.
    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error ({status}): {message}")]
    Api {
        status: u16,
        message: String,
    },

    /// Rate limit exceeded. Wait before retrying.
    #[error("rate limited, retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    /// Invalid API key or authentication failure.
    #[error("authentication failed: {0}")]
    Auth(String),

    /// Provider configuration is invalid.
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),
}
```

---

## 4. Type Safety and Linting

### 4.1 Clippy Configuration

Create `clippy.toml` in workspace root:

```toml
cognitive-complexity-threshold = 25
too-many-arguments-threshold = 7
type-complexity-threshold = 250
```

### 4.2 Required Lints

```toml
[workspace.lints.clippy]
# Error on dangerous patterns
unwrap_used = "deny"      # Use ? or proper error handling
panic = "deny"            # No panics in library code

# Warn on code quality issues
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"

# Additional warnings
expect_used = "warn"      # Prefer ? over .expect()
todo = "warn"             # No TODOs in production code
dbg_macro = "warn"        # No debug macros in production
print_stdout = "warn"     # Use tracing instead
print_stderr = "warn"     # Use tracing instead
```

### 4.3 rustfmt Configuration

Create `rustfmt.toml` in workspace root:

```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Max"
imports_granularity = "Module"
group_imports = "StdExternalCrate"
reorder_imports = true
reorder_modules = true
```

---

## 5. Error Handling

### 5.1 Error Type Pattern

Use `thiserror` for all error types:

```rust
use thiserror::Error;

/// Errors from the storage layer.
#[derive(Debug, Error)]
pub enum StorageError {
    /// Database connection failed.
    #[error("database connection failed: {0}")]
    Connection(#[from] rusqlite::Error),

    /// Migration failed.
    #[error("migration {version} failed: {message}")]
    Migration { version: u32, message: String },

    /// Record not found.
    #[error("record not found: {0}")]
    NotFound(String),

    /// Serialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),
}
```

### 5.2 Result Handling Rules

**MUST NOT** use `.unwrap()` or `.expect()` in library code:

```rust
// ❌ Bad - panics on error
let value = some_option.unwrap();
let result = fallible_operation().expect("should work");

// ✅ Good - proper error propagation
let value = some_option.ok_or(StorageError::NotFound("value".into()))?;
let result = fallible_operation()?;

// ✅ Acceptable in main.rs with clear context
let config = Config::from_env().expect("CONFIG: Missing required environment variables");
```

### 5.3 Error Context

Add context when propagating errors:

```rust
use anyhow::Context;

fn load_conversation(id: &str) -> anyhow::Result<Conversation> {
    let row = db.query_row(/* ... */)
        .context(format!("failed to load conversation {id}"))?;

    serde_json::from_str(&row)
        .context("failed to deserialize conversation")
}
```

---

## 6. Async Patterns

### 6.1 Tokio Runtime

**Library code** accepts an executor, doesn't create a runtime:

```rust
// ✅ Library function - async, no runtime
pub async fn fetch_completion(request: &CompletionRequest) -> Result<Response, ProviderError> {
    // Implementation
}
```

**Binary entry point** creates the runtime:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = create_provider()?;
    let response = provider.complete(request).await?;
    Ok(())
}
```

### 6.2 Async Traits (Rust 1.75+)

Use native async traits (no `async-trait` crate needed):

```rust
pub trait AiProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionStream, ProviderError>;
}

impl AiProvider for OpenAiProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionStream, ProviderError> {
        // Implementation
    }
}
```

### 6.3 Streaming Responses

Use `futures::Stream` for streaming:

```rust
use futures::Stream;
use std::pin::Pin;

/// A stream of completion response chunks.
pub type CompletionStream = Pin<Box<dyn Stream<Item = Result<String, ProviderError>> + Send>>;

impl OpenAiProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionStream, ProviderError> {
        let response = self.client.post(/* ... */).send().await?;

        let stream = response
            .bytes_stream()
            .map(|chunk| /* parse SSE */)
            .boxed();

        Ok(stream)
    }
}
```

---

## 7. SQLite Patterns

### 7.1 Connection Management

```rust
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

/// Opens database connection with appropriate flags.
pub fn open_database(path: &Path) -> Result<Connection, StorageError> {
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_CREATE
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;

    let conn = Connection::open_with_flags(path, flags)?;

    // Enable WAL mode for better concurrency
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;

    Ok(conn)
}
```

### 7.2 Migrations

```rust
/// Run all pending migrations.
pub fn run_migrations(conn: &Connection) -> Result<(), StorageError> {
    const MIGRATIONS: &[(&str, &str)] = &[
        ("0001", include_str!("migrations/0001_initial.sql")),
        ("0002", include_str!("migrations/0002_conversations.sql")),
    ];

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS migrations (
            version TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL
        );"
    )?;

    for (version, sql) in MIGRATIONS {
        let applied: bool = conn.query_row(
            "SELECT 1 FROM migrations WHERE version = ?",
            [version],
            |_| Ok(true),
        ).unwrap_or(false);

        if !applied {
            conn.execute_batch(sql)
                .map_err(|e| StorageError::Migration {
                    version: version.parse().unwrap_or(0),
                    message: e.to_string(),
                })?;

            conn.execute(
                "INSERT INTO migrations (version, applied_at) VALUES (?, datetime('now'))",
                [version],
            )?;
        }
    }

    Ok(())
}
```

### 7.3 Repository Pattern

```rust
/// Repository for conversation persistence.
pub struct ConversationRepository {
    conn: Connection,
}

impl ConversationRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    pub fn save(&self, conversation: &Conversation) -> Result<(), StorageError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO conversations (id, data, updated_at)
             VALUES (?1, ?2, datetime('now'))",
            (
                &conversation.id,
                serde_json::to_string(conversation)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?,
            ),
        )?;
        Ok(())
    }

    pub fn find(&self, id: &str) -> Result<Option<Conversation>, StorageError> {
        let result = self.conn.query_row(
            "SELECT data FROM conversations WHERE id = ?",
            [id],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(data) => {
                let conversation = serde_json::from_str(&data)
                    .map_err(|e| StorageError::Serialization(e.to_string()))?;
                Ok(Some(conversation))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(StorageError::Connection(e)),
        }
    }
}
```

---

## 8. CLI Patterns

### 8.1 Clap Command Structure

```rust
use clap::{Parser, Subcommand};

/// Cherry2K.sh - AI assistant for your terminal
#[derive(Parser)]
#[command(name = "cherry2k")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output format
    #[arg(long, default_value = "text", global = true)]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Send a one-shot message
    Chat {
        /// The message to send
        message: String,

        /// Provider to use (openai, anthropic, ollama)
        #[arg(short, long)]
        provider: Option<String>,
    },

    /// Start interactive REPL
    Repl {
        #[arg(short, long)]
        provider: Option<String>,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// View conversation history
    History {
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}
```

### 8.2 Output Formatting

```rust
use serde::Serialize;

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

---

## 9. Zsh Integration

### 9.1 Plugin Structure

```zsh
# zsh/cherry2k.plugin.zsh

# Ensure binary is in PATH
export PATH="${0:A:h}/../target/release:$PATH"

# Main function
cherry2k() {
    command cherry2k "$@"
}

# Widget for inline AI assist
cherry2k-assist-widget() {
    local result
    result=$(cherry2k chat --format=text "$BUFFER" 2>/dev/null)
    if [[ -n "$result" ]]; then
        BUFFER="$result"
        CURSOR=${#BUFFER}
    fi
    zle redisplay
}
zle -N cherry2k-assist-widget

# Default keybinding (Ctrl+G)
bindkey '^G' cherry2k-assist-widget
```

### 9.2 Completions

```zsh
# zsh/completions/_cherry2k

#compdef cherry2k

_cherry2k() {
    local -a commands
    commands=(
        'chat:Send a one-shot message'
        'repl:Start interactive REPL'
        'config:Manage configuration'
        'history:View conversation history'
    )

    _arguments -C \
        '-v[Enable verbose output]' \
        '--verbose[Enable verbose output]' \
        '--format=[Output format]:format:(text json)' \
        '1:command:->command' \
        '*::arg:->args'

    case "$state" in
        command)
            _describe 'command' commands
            ;;
        args)
            case "$words[1]" in
                chat)
                    _arguments \
                        '-p[Provider]:provider:(openai anthropic ollama)' \
                        '--provider=[Provider]:provider:(openai anthropic ollama)' \
                        '*:message:'
                    ;;
            esac
            ;;
    esac
}

_cherry2k "$@"
```

---

## 10. Testing

See `testing.md` for comprehensive standards. Key patterns:

### 10.1 Unit Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod fixtures {
        use super::*;

        pub fn mock_config() -> ProviderConfig {
            ProviderConfig {
                api_key: "test-key".into(),
                base_url: "http://localhost:8080".into(),
                model: "test-model".into(),
            }
        }
    }

    mod complete_tests {
        use super::*;
        use wiremock::{MockServer, Mock, ResponseTemplate};
        use wiremock::matchers::{method, path};

        #[tokio::test]
        async fn returns_response_for_valid_request() {
            // Arrange
            let mock_server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/v1/completions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(/* ... */))
                .mount(&mock_server)
                .await;

            let config = fixtures::mock_config();
            let provider = OpenAiProvider::new_with_base_url(config, mock_server.uri());

            // Act
            let result = provider.complete(request).await;

            // Assert
            assert!(result.is_ok());
        }
    }
}
```

### 10.2 Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --fail-under-lines 80
cargo llvm-cov --html  # Generate report
```