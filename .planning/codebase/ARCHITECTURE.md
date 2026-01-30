# Architecture

**Analysis Date:** 2026-01-29

## Pattern Overview

**Overall:** Layered architecture with trait-based provider abstraction

**Key Characteristics:**
- Multi-crate workspace design (core, storage, cli) with clear separation of concerns
- Provider-agnostic abstraction via `AiProvider` trait enabling pluggable AI backends
- Async-first architecture using Tokio for non-blocking I/O
- SQLite-backed persistence layer for conversation history
- Zero-unsafe code policy (forbid unsafe in workspace lints)

## Layers

**Provider Layer:**
- Purpose: Abstract interface for AI service integrations
- Location: `crates/core/src/provider/`
- Contains: Trait definition (`trait.rs`), OpenAI, Anthropic, and Ollama implementations
- Depends on: HTTP client (reqwest), serialization (serde)
- Used by: CLI commands, REPL, storage layer for provider metadata

**Domain Layer:**
- Purpose: Core business logic and conversation management
- Location: `crates/core/src/`
- Contains: Provider trait, conversation models, configuration, error types
- Depends on: Provider abstractions, serialization
- Used by: CLI and storage layers

**Storage Layer:**
- Purpose: Persistent data access and SQLite integration
- Location: `crates/storage/src/`
- Contains: Schema definitions, migrations, repository pattern data access
- Depends on: rusqlite with bundled SQLite, domain types from core
- Used by: CLI commands for history, conversation persistence

**CLI/Terminal Layer:**
- Purpose: User interaction through terminal commands and REPL
- Location: `crates/cli/src/`
- Contains: Command handlers (chat, repl, config, history), output formatting
- Depends on: Core domain logic, storage repository, async runtime
- Used by: End users via binary executable

**Shell Integration Layer:**
- Purpose: Zsh-native integration without external dependencies
- Location: `zsh/`
- Contains: Pure zsh shell functions, ZLE widgets, tab completions
- Depends on: Binary executable only
- Used by: Zsh terminal environment

## Data Flow

**One-Shot Query Flow:**

1. User invokes `cherry2k chat "question"` via CLI
2. CLI command handler (`crates/cli/src/commands/chat.rs`) parses arguments
3. Configuration loader (`crates/core/src/config/`) reads env vars and config file
4. Provider factory selects appropriate implementation based on config
5. Provider sends request to external API or local Ollama service
6. Response stream received and passed to output formatter
7. Storage repository saves exchange to SQLite conversation database
8. Formatted output displayed to terminal

**Interactive REPL Flow:**

1. User runs `cherry2k repl`
2. REPL initialization (`crates/cli/src/repl/`) starts readline loop
3. User input captured and sent to current provider
4. Streaming response written to terminal line-by-line
5. After response completes, exchange persisted to storage
6. REPL prompt returned for next user input
7. `/provider` and `/model` commands switch active provider without recreating connection

**Persistence Flow:**

1. After completion request/response cycle, storage layer receives exchange
2. Repository (`crates/storage/src/repository.rs`) executes INSERT statements
3. SQLite applies schema validation
4. Conversation context retrieved for subsequent requests via JOIN queries
5. History commands query conversation table with optional filtering/limiting

**State Management:**
- No global mutable state; each operation is request-scoped
- Provider instances are immutable after construction
- Configuration loaded once at startup
- SQLite connection pooling handled internally by rusqlite
- Async runtime (Tokio) owns all background tasks

## Key Abstractions

**AiProvider Trait:**
- Purpose: Enables seamless provider switching without code changes
- Examples: `crates/core/src/provider/openai.rs`, `crates/core/src/provider/anthropic.rs`, `crates/core/src/provider/ollama.rs`
- Pattern: Async trait with streaming response support; each implementation handles provider-specific request/response mapping

**CompletionRequest:**
- Purpose: Provider-agnostic request format
- Pattern: Builder or struct with required fields (message content, optional context/history); serialized appropriately per provider

**CompletionStream:**
- Purpose: Async iterator for streaming responses
- Pattern: Futures stream that yields response chunks; allows incremental display without buffering entire response

**Configuration Schema:**
- Purpose: Unified config structure supporting multiple providers
- Pattern: TOML file at `~/.config/cherry2k/config.toml` with env var overrides; defaults applied if unspecified

**Repository Pattern:**
- Purpose: Abstract database operations for storage layer
- Pattern: Data access layer (`crates/storage/src/repository.rs`) with methods for CRUD operations; enables test mocking

## Entry Points

**Binary Executable:**
- Location: `crates/cli/src/main.rs`
- Triggers: User runs `cherry2k` command with subcommands (chat, repl, config, history)
- Responsibilities: Parse CLI args, initialize logger, load config, delegate to command handlers

**Library Exports (crates/core):**
- Location: `crates/core/src/lib.rs`
- Triggers: External code imports provider trait or configuration types
- Responsibilities: Export public API (AiProvider trait, ProviderConfig, error types) for library consumers

**Shell Functions:**
- Location: `zsh/cherry2k.plugin.zsh`
- Triggers: User sources plugin file in .zshrc
- Responsibilities: Define zsh functions and ZLE widgets that invoke cherry2k binary

## Error Handling

**Strategy:** Layered error propagation with context via `thiserror` enums

**Patterns:**
- Library code defines error enums with `#[derive(Error)]` and `#[from]` attributes for automatic conversion
- CLI converts errors to user-friendly messages with suggestions
- Network errors wrapped with exponential backoff retry logic
- API-specific errors (rate limits, invalid auth) caught early to avoid retry loops
- Never use `.unwrap()` or `.expect()` in library code (enforced by clippy lint)

**Error Types:**
- `ProviderError` - API request failures, auth issues, rate limiting
- `ConfigError` - Missing/invalid configuration
- Storage errors propagated with database context

## Cross-Cutting Concerns

**Logging:** Uses `tracing` crate for structured logging; `tracing-subscriber` filters by `CHERRY2K_LOG_LEVEL` env var; sensitive data (API keys) never logged via audit in all provider implementations

**Validation:** Input validation occurs at layer boundaries; CLI validates command args before passing to core; provider implementations validate API keys before requests; storage validates SQLite schema compatibility

**Authentication:** Provider config loads API keys from environment variables or secure config file (0600 permissions); no in-memory caching of credentials across requests; each provider validates key format before use

**Streaming:** All provider implementations support async streaming via futures Stream trait; output formatter consumes stream chunks and writes incrementally to terminal; large responses never fully buffered in memory

---

*Architecture analysis: 2026-01-29*
