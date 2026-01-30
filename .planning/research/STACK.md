# Technology Stack Research

**Project:** Cherry2K - Zsh Terminal AI Assistant
**Researched:** 2026-01-29
**Research Mode:** Ecosystem (Stack Dimension)

---

## Executive Summary

The Rust ecosystem for building terminal AI assistants has matured significantly. The stack is well-established with clear winners in each category. Tokio dominates async runtime, reqwest owns HTTP, rusqlite handles SQLite, and ratatui has become the standard TUI library after forking from tui-rs. The key architectural decision is whether to use existing multi-provider AI libraries (like rust-genai) or build a provider abstraction from scratch.

**Recommendation:** Build your own lightweight provider abstraction using reqwest directly. This gives you full control over streaming behavior, error handling, and provider-specific quirks without adding dependency on rapidly-evolving AI client libraries.

---

## Recommended Stack

### Core Runtime

| Technology | Version | Purpose | Confidence |
|------------|---------|---------|------------|
| **Rust** | 1.75+ (1.77 recommended) | Native async traits, improved diagnostics | HIGH |
| **Tokio** | 1.49.x | Async runtime, I/O, networking | HIGH |
| **Futures** | 0.3.x | Stream trait, combinators | HIGH |

**Why Tokio:**
- LTS releases (1.47.x supported until Sept 2026)
- De facto standard - 99% of async Rust libraries assume Tokio
- async-std was discontinued March 2025; smol is niche
- Required by reqwest, ratatui (via crossterm), and most ecosystem crates

**Version Rationale:**
- Use `tokio = { version = "1.49", features = ["full"] }` for development convenience
- In production, trim to `features = ["rt-multi-thread", "net", "io-util", "time", "sync", "macros"]`
- MSRV is Rust 1.71 for tokio 1.48+

### HTTP Client

| Technology | Version | Purpose | Confidence |
|------------|---------|---------|------------|
| **reqwest** | 0.13.x | HTTP client for AI provider APIs | HIGH |

**Why reqwest:**
- 306M+ downloads, de facto standard for HTTP in Rust
- Native async/await with Tokio
- Built-in SSE streaming support via `response.bytes_stream()`
- TLS via rustls (default) or native-tls
- JSON support via serde integration

**Critical Version Notes:**
- v0.13 changed default TLS provider to rustls with aws-lc crypto
- `query` and `form` are now opt-in features (add them explicitly)
- `trust-dns` renamed to `hickory-dns`

**Recommended Configuration:**
```toml
reqwest = { version = "0.13", features = ["json", "stream", "rustls-tls"] }
```

**Why NOT hyper directly:** reqwest is built on hyper but adds connection pooling, redirects, cookies, and a sane API. Building on raw hyper is unnecessary complexity.

### Database

| Technology | Version | Purpose | Confidence |
|------------|---------|---------|------------|
| **rusqlite** | 0.38.x | SQLite persistence | HIGH |

**Why rusqlite:**
- Bundles SQLite 3.51.1 (with `bundled` feature)
- No runtime dependency on system SQLite
- Excellent migration support
- 40M+ downloads, actively maintained

**Recommended Configuration:**
```toml
rusqlite = { version = "0.38", features = ["bundled", "chrono", "serde_json"] }
```

**Why NOT sqlx:** sqlx is excellent for PostgreSQL/MySQL but adds compile-time query checking complexity. For embedded SQLite with simple schemas, rusqlite is simpler and faster to iterate.

**Why NOT diesel:** ORM overhead is unnecessary for a CLI tool with ~5 tables. rusqlite's raw SQL is more transparent and debuggable.

### CLI Framework

| Technology | Version | Purpose | Confidence |
|------------|---------|---------|------------|
| **clap** | 4.5.x | CLI argument parsing | HIGH |

**Why clap:**
- De facto standard (no real competitor in 2026)
- Derive macros for type-safe args
- Excellent shell completion generation
- Subcommand support for `cherry2k chat`, `cherry2k config`, etc.

**Recommended Configuration:**
```toml
clap = { version = "4.5", features = ["derive", "env"] }
```

**Why `env` feature:** Allows `#[arg(env = "OPENAI_API_KEY")]` for API key configuration.

### TUI Framework (Optional Mode)

| Technology | Version | Purpose | Confidence |
|------------|---------|---------|------------|
| **ratatui** | 0.30.x | Terminal UI for optional TUI mode | HIGH |
| **crossterm** | 0.29.x | Cross-platform terminal control | HIGH |

**Why ratatui:**
- Fork of abandoned tui-rs, actively maintained
- Sub-millisecond rendering, zero-cost abstractions
- Constraint-based layouts
- 17k+ GitHub stars, used in 1900+ crates

**Why crossterm (not termion):**
- Windows support (termion is Unix-only)
- Better maintained
- Default backend for ratatui

**Recommended Configuration:**
```toml
ratatui = { version = "0.30", features = ["crossterm"] }
crossterm = { version = "0.29", features = ["event-stream"] }
```

**Note on v0.30:** Major reorganization into modular workspace. Use `ratatui::init()` and `ratatui::run()` convenience functions added in 0.28.1.

### Serialization

| Technology | Version | Purpose | Confidence |
|------------|---------|---------|------------|
| **serde** | 1.0.x | Serialization framework | HIGH |
| **serde_json** | 1.0.x | JSON for API payloads | HIGH |
| **toml** | 0.9.x | Configuration files | HIGH |

**Why these versions:**
- serde is stable at 1.0, no 2.0 planned
- serde_json 1.0.145+ has latest features
- toml 0.9 is current stable with serde support

**Recommended Configuration:**
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.9"
```

### Error Handling

| Technology | Version | Purpose | Confidence |
|------------|---------|---------|------------|
| **thiserror** | 2.0.x | Library error types | HIGH |
| **anyhow** | 2.0.x | Application error handling | HIGH |

**Pattern:**
- `thiserror` in library crates (`core`, `storage`) for typed errors
- `anyhow` in binary crate (`cli`) for error context and display

**Why 2.0:** Both crates hit 2.0 in late 2025 with improved ergonomics. Use 2.0 for new projects.

```toml
thiserror = "2.0"
anyhow = "2.0"
```

### Logging/Tracing

| Technology | Version | Purpose | Confidence |
|------------|---------|---------|------------|
| **tracing** | 0.1.x | Structured logging | HIGH |
| **tracing-subscriber** | 0.3.x | Log output formatting | HIGH |

**Why tracing (not log):**
- Structured spans with begin/end times
- Better async support (tracks across await points)
- Maintained by Tokio team
- Industry standard for new Rust projects in 2025+

**Recommended Configuration:**
```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### Configuration Paths

| Technology | Version | Purpose | Confidence |
|------------|---------|---------|------------|
| **directories** | 5.x | XDG-compliant config paths | HIGH |

**Why directories (not dirs or xdg):**
- Cross-platform (macOS, Linux, Windows)
- Handles app-specific paths (not just base dirs)
- 1.3M+ monthly downloads

```toml
directories = "5"
```

**Usage:**
```rust
use directories::ProjectDirs;
if let Some(proj_dirs) = ProjectDirs::from("", "", "cherry2k") {
    let config_path = proj_dirs.config_dir(); // ~/.config/cherry2k on Linux
}
```

---

## AI Provider Integration

### Decision: Build vs Buy

**Recommendation:** Build lightweight provider abstraction, do NOT use rust-genai or async-openai.

**Why:**

| Factor | Build Own | Use rust-genai/async-openai |
|--------|-----------|----------------------------|
| Control | Full control over streaming, errors | Limited by library design |
| Dependencies | Only reqwest + serde | Additional crate + transitive deps |
| Stability | You control the code | Library churn (0.5.x, breaking changes) |
| Learning | Understand the APIs | Black box |
| Maintenance | You maintain | Dependent on maintainer |

**The APIs are simple.** OpenAI/Anthropic/Ollama are just REST endpoints with SSE streaming. The complexity is in handling the responses, not making the requests.

### SSE Streaming Pattern

```rust
use futures::StreamExt;
use reqwest::Client;

pub async fn stream_completion(
    client: &Client,
    url: &str,
    body: serde_json::Value,
    api_key: &str,
) -> impl Stream<Item = Result<String, Error>> {
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    // Parse SSE events from byte stream
    response
        .bytes_stream()
        .map(|chunk| parse_sse_event(chunk?))
}
```

**This is ~100 lines of code per provider.** Not worth adding a dependency for.

### Provider-Specific Notes

| Provider | Endpoint | Auth | Streaming Format |
|----------|----------|------|------------------|
| OpenAI | `api.openai.com/v1/chat/completions` | Bearer token | SSE with `data: {json}` |
| Anthropic | `api.anthropic.com/v1/messages` | `x-api-key` header | SSE with `event:` + `data:` |
| Ollama | `localhost:11434/api/chat` | None | NDJSON (newline-delimited JSON) |

**Key difference:** Anthropic uses `event:` lines to indicate event type, OpenAI does not. Ollama uses NDJSON, not SSE. Your abstraction must handle all three.

---

## Testing Stack

| Technology | Version | Purpose | Confidence |
|------------|---------|---------|------------|
| **wiremock** | 0.6.x | HTTP mocking for API tests | HIGH |
| **tokio-test** | 0.4.x | Async test utilities | HIGH |
| **cargo-llvm-cov** | 0.6.23 | Code coverage | HIGH |

**Why wiremock:**
- Designed for async (works with tokio)
- Isolated MockServer per test (no test pollution)
- Connection pooling for performance
- Request matching and response templating

**Coverage Setup:**
```bash
cargo install cargo-llvm-cov
cargo llvm-cov --fail-under-lines 80
```

---

## What NOT to Use

### Avoid These Libraries

| Library | Why Avoid |
|---------|-----------|
| **async-trait** | Rust 1.75+ has native async traits. No longer needed. |
| **async-std** | Discontinued March 2025. Tokio is the standard. |
| **tui** | Abandoned. Use ratatui (active fork). |
| **diesel** | ORM overhead unnecessary for CLI tool with SQLite. |
| **sqlx** | Compile-time query checking adds complexity for simple schemas. |
| **rust-genai** | Rapidly evolving (0.5.x), adds unnecessary abstraction. |
| **async-openai** | Specific to OpenAI, doesn't help with Anthropic/Ollama. |
| **log** | Older logging crate. tracing is the modern choice. |

### Avoid These Patterns

| Pattern | Why Avoid | Do Instead |
|---------|-----------|------------|
| `.unwrap()` in library code | Panics are infectious | Return `Result<T, E>` |
| Global `static` for config | Hard to test | Pass config as parameter |
| Blocking calls in async | Deadlocks the runtime | Use `tokio::fs`, `spawn_blocking` |
| String errors | No context, hard to match | Use `thiserror` enums |

---

## Complete Cargo.toml

```toml
[workspace]
resolver = "2"
members = ["crates/core", "crates/storage", "crates/cli"]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "MIT"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.49", features = ["full"] }
futures = "0.3"

# HTTP
reqwest = { version = "0.13", features = ["json", "stream", "rustls-tls"] }

# Database
rusqlite = { version = "0.38", features = ["bundled", "chrono", "serde_json"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.9"

# CLI
clap = { version = "4.5", features = ["derive", "env"] }

# TUI (optional)
ratatui = { version = "0.30", features = ["crossterm"] }
crossterm = { version = "0.29", features = ["event-stream"] }

# Error handling
thiserror = "2.0"
anyhow = "2.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities
directories = "5"

# Testing
wiremock = "0.6"
tokio-test = "0.4"
```

---

## Version Verification Sources

| Library | Source | Confidence |
|---------|--------|------------|
| tokio 1.49 | [crates.io](https://crates.io/crates/tokio), [docs.rs](https://docs.rs/crate/tokio/latest) | HIGH |
| reqwest 0.13 | [GitHub](https://github.com/seanmonstar/reqwest) | HIGH |
| rusqlite 0.38 | [docs.rs](https://docs.rs/crate/rusqlite/latest), [crates.io](https://crates.io/crates/rusqlite) | HIGH |
| clap 4.5.54 | [crates.io](https://crates.io/crates/clap) | HIGH |
| ratatui 0.30 | [ratatui.rs](https://ratatui.rs/highlights/v030/) | HIGH |
| crossterm 0.29 | [crates.io](https://crates.io/crates/crossterm/0.29.0) | HIGH |
| thiserror 2.0 | [Multiple 2025 guides](https://dev.to/leapcell/rust-error-handling-compared-anyhow-vs-thiserror-vs-snafu-2003) | HIGH |
| anyhow 2.0 | [Multiple 2025 guides](https://dev.to/leapcell/rust-error-handling-compared-anyhow-vs-thiserror-vs-snafu-2003) | HIGH |
| wiremock 0.6 | [lib.rs](https://lib.rs/crates/wiremock), [crates.io](https://crates.io/crates/wiremock) | MEDIUM |
| cargo-llvm-cov 0.6.23 | [docs.rs](https://docs.rs/crate/cargo-llvm-cov/latest), [crates.io](https://crates.io/crates/cargo-llvm-cov) | HIGH |

---

## Open Questions

1. **TUI as separate binary?** Consider `cherry2k-tui` as optional binary vs feature flag in main binary. Reduces compile time if TUI not needed.

2. **Ollama streaming format:** Need to verify NDJSON vs SSE - some sources conflict. Test against actual Ollama 0.5.x.

3. **reqwest 0.13 breaking changes:** Verify `query` and `form` features are explicitly added if used.

---

*Research conducted 2026-01-29. Verify versions against crates.io before implementation.*