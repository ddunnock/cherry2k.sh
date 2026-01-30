# Technology Stack

**Analysis Date:** 2026-01-29

## Languages

**Primary:**
- Rust 1.93+ (stable, Edition 2024) - Core application logic, provider implementations, CLI
- Zsh - Terminal integration, widgets, completions

**Secondary:**
- SQL - SQLite schema and migrations
- TOML - Configuration files

## Runtime

**Environment:**
- Rust stable toolchain (1.93 or newer, Edition 2024)
- Tokio async runtime (1.49+)

**Package Manager:**
- Cargo - Rust dependency management
- Lockfile: `Cargo.lock` (required in workspace)

## Frameworks

**Core:**
- Tokio 1.49 (full features) - Async runtime for concurrent operations
- Reqwest (json, stream features) - HTTP client for API calls to providers

**CLI:**
- Clap 4.5.56 (derive features) - Command-line argument parsing
- Tracing 0.1.44 + tracing-subscriber 0.3.22 (env-filter) - Structured logging framework

**Data:**
- Serde 1.0.228 (derive) - Serialization/deserialization
- Serde JSON 1.0.149 - JSON handling
- TOML 0.9.11 - Configuration files

**Async Utilities:**
- Futures 0.3 - Stream abstractions for streaming responses
- Async-stream 0.3 - Helper macros for async streams

## Key Dependencies

**Critical:**
- tokio - Async runtime; enables all async/await patterns
- reqwest - HTTP client; handles API communication with cloud providers
- rusqlite - SQLite driver; manages conversation persistence with bundled SQLite
- serde/serde_json - Serialization; handles config files, API payloads, conversation storage

**Infrastructure:**
- thiserror 2.0 - Error type derivation; enables propagation with `?` operator
- anyhow 1.0 - Flexible error handling; used for error context
- tracing - Instrumentation; structured logging without logging secrets
- clap - CLI parsing; command structure and help generation

**Testing:**
- tokio-test 0.4 - Test utilities for async code
- wiremock 0.6 - HTTP mocking; simulates API responses in tests

## Configuration

**Environment:**
- API keys via environment variables (OPENAI_API_KEY, ANTHROPIC_API_KEY)
- Optional CHERRY2K_CONFIG_PATH for config file location
- CHERRY2K_LOG_LEVEL for log verbosity
- OLLAMA_HOST for local Ollama endpoint

**Build:**
- `Cargo.toml` (workspace root) - Workspace configuration with shared dependencies
- `Cargo.toml` (per crate) - Individual crate metadata
- `Cargo.lock` - Pinned dependency versions
- `rustfmt.toml` - Code formatting (max_width: 100, Edition 2024)
- `clippy.toml` - Linter configuration (cognitive-complexity: 25, etc.)

## Platform Requirements

**Development:**
- macOS or Linux
- Rust 1.93+ (for building from source, Edition 2024)
- SQLite 3 (on macOS: installed via Homebrew for native performance)
- One API key: OpenAI (OPENAI_API_KEY=sk-*), Anthropic (ANTHROPIC_API_KEY=sk-ant-*), or Ollama running locally

**Production:**
- macOS or Linux binary
- No network access required if using Ollama locally
- HTTPS for OpenAI and Anthropic API calls

## Workspace Structure

```toml
[workspace]
resolver = "2"
members = ["crates/core", "crates/storage", "crates/cli"]

[workspace.package]
version = "0.1.0"
edition = "2024"
authors = ["Cherry2K Contributors"]
license = "AGPL-3.0"

[workspace.dependencies]
# Shared across all crates (see Cargo.toml for current versions)
```

**Three crates:**
- `crates/core` - Provider trait, implementations (OpenAI, Anthropic, Ollama), conversation logic, config parsing
- `crates/storage` - SQLite schema, migrations, data access repository
- `crates/cli` - Binary entry point, commands (chat, repl, config, history), REPL implementation, terminal output formatting

## Code Quality Gates

All code must pass before merge:

```bash
cargo fmt --check              # Formatting enforced
cargo clippy -- -D warnings    # All warnings become errors
cargo test                     # All tests pass
cargo llvm-cov --fail-under-lines 80  # Minimum 80% line coverage
cargo audit --deny warnings    # No vulnerable dependencies
```

## Error Handling

- `thiserror` for library crate error definitions
- `anyhow::Context` for error propagation with context
- NO `.unwrap()` or `.expect()` in library code; main.rs acceptable with justification
- All Result types propagated with `?` operator

## Async Patterns

- Native async traits (Rust 2024 edition, no async-trait crate needed)
- Streaming responses via `Pin<Box<dyn Stream<Item = Result<T, E>> + Send>>`
- Tokio runtime created only in binary main.rs
- Library functions are async-ready but don't create runtime

---

*Stack analysis: 2026-01-29, updated 2026-01-30*
