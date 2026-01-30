# Codebase Concerns

**Analysis Date:** 2026-01-29

## Overview

Cherry2K.sh is a blueprint/skeleton project with comprehensive standards in place but no implementation code yet. This document identifies concerns and risks that should be monitored during development, based on the architectural design and standards established.

---

## Tech Debt & Implementation Gaps

### 1. No Actual Implementation Exists

**Issue:** The project structure defines directories (`crates/core/src/provider/`, `crates/cli/src/commands/`, etc.) and imports them in documentation, but no actual `.rs` files are present.

**Files:**
- `crates/core/src/` (empty directories)
- `crates/storage/src/` (empty directories)
- `crates/cli/src/` (empty directories)

**Impact:**
- Cannot build the project: `cargo build` will fail with missing modules
- Cannot run tests: No test files exist
- Cannot verify standards compliance (linting, coverage, security)

**Fix Approach:**
1. Generate basic skeleton files for each module referenced in standards (lib.rs, mod.rs files)
2. Create minimal implementations of the AiProvider trait with stub methods
3. Implement error types and basic configuration structures
4. Build incrementally, verifying each module compiles before adding functionality

---

## Architecture Risks

### 2. Zsh Integration Path Not Implemented

**Issue:** Standards define ZLE widgets and completions in `zsh/widgets/` and `zsh/completions/` directories, but they are empty. The `cherry2k.plugin.zsh` file referenced in README doesn't exist.

**Files:**
- `zsh/cherry2k.plugin.zsh` (missing)
- `zsh/widgets/` (empty)
- `zsh/completions/` (empty)

**Impact:**
- Cannot install or test zsh integration
- Incomplete feature coverage compared to documented functionality
- Users following README steps will fail at "zsh integration" section

**Fix Approach:**
1. Create `cherry2k.plugin.zsh` with basic shell functions
2. Implement ZLE widgets for Ctrl+G and Ctrl+X Ctrl+A keybindings
3. Generate zsh completion function for commands and options
4. Test integration by sourcing plugin and verifying widgets register

---

### 3. Provider Implementations Missing

**Issue:** Three provider implementations are specified (OpenAI, Anthropic, Ollama) in standards and CLAUDE.md, but `crates/core/src/provider/` contains no implementation files.

**Files:**
- `crates/core/src/provider/openai.rs` (missing)
- `crates/core/src/provider/anthropic.rs` (missing)
- `crates/core/src/provider/ollama.rs` (missing)
- `crates/core/src/provider/trait.rs` (missing)

**Impact:**
- Core feature (multiple AI backends) cannot be realized
- No way to validate API integration design
- Cannot test provider selection logic or fallback behavior

**Fix Approach:**
1. Define `AiProvider` trait in `provider/trait.rs` with methods: `complete()`, `provider_id()`, `validate_config()`
2. Create provider modules that implement the trait with reqwest HTTP client
3. Add provider factory pattern for selecting provider from env vars
4. Write integration tests mocking each provider's API responses

---

## Security Concerns

### 4. Environment Variable Validation Not Implemented

**Issue:** Security standards (`security.md`) define strict API key loading patterns with validation, but no actual validation code exists.

**Standard Requirement:**
```rust
pub fn load_from_env() -> Result<ProviderConfig, ConfigError> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| ConfigError::MissingApiKey("OPENAI_API_KEY"))?;
    Ok(ProviderConfig { api_key, ..Default::default() })
}
```

**Files:**
- `crates/core/src/config/` (empty)

**Risk:**
- Could accidentally log API keys if error messages aren't carefully designed
- Config loading might fail silently instead of providing user guidance
- Potential for secrets in error messages or logs

**Recommendations:**
1. Implement structured error types that never contain secret values
2. Add validation that API key format matches provider requirements (e.g., "sk-" prefix for OpenAI)
3. Add safe error messages that guide users to fix problems without exposing secrets
4. Test that logging never includes secret values with automated grep checks

---

### 5. Database File Permissions Not Enforced

**Issue:** Security standards specify SQLite database must have 0600 permissions, but implementation is missing.

**Standard Requirement:**
```rust
pub fn open_secure_database(path: &Path) -> Result<Connection, StorageError> {
    let conn = Connection::open(path)?;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)?;
    Ok(conn)
}
```

**Files:**
- `crates/storage/src/repository.rs` (missing implementation)

**Risk:**
- Database containing conversation history could be world-readable
- Privacy violation if database is world-accessible

**Fix Approach:**
1. When opening or creating database file, immediately set permissions to 0o600
2. On startup, verify existing database has correct permissions
3. Document in error messages how users should fix incorrect permissions manually
4. Add integration test that verifies permissions are set correctly

---

### 6. Input Validation Framework Missing

**Issue:** Security standards require prompt length validation (max 100,000 chars) and null byte checking, but no validation module exists.

**Standard Requirement:**
```rust
const MAX_PROMPT_LENGTH: usize = 100_000;
pub fn validate_prompt(prompt: &str) -> Result<(), ValidationError> {
    if prompt.is_empty() {
        return Err(ValidationError::EmptyPrompt);
    }
    if prompt.len() > MAX_PROMPT_LENGTH {
        return Err(ValidationError::PromptTooLong { ... });
    }
    if prompt.contains('\0') {
        return Err(ValidationError::InvalidCharacters("null bytes"));
    }
    Ok(())
}
```

**Files:**
- `crates/core/src/` (no validation module)

**Impact:**
- Could send malformed or excessively large prompts to API providers
- Potential for API abuse or DoS against own service
- No protection against null byte injection

**Fix Approach:**
1. Create `crates/core/src/validation.rs` module
2. Implement prompt, path, and config validation functions
3. Call validation in all public API entry points before provider requests
4. Add comprehensive tests covering edge cases (empty prompts, max length, control characters)

---

## Performance & Scalability Concerns

### 7. SQLite Concurrency Not Addressed

**Issue:** README advertises "Conversation history" with SQLite backend, but no concurrency handling or busy timeout is implemented.

**Documentation Risk:**
- CLAUDE.md mentions "Check for concurrent processes; increase busy timeout" in troubleshooting
- Suggests the issue is known but not built-in to the solution

**Files:**
- `crates/storage/src/schema.rs` (missing)
- `crates/storage/src/repository.rs` (missing)

**Impact:**
- Multiple zsh instances accessing database could cause SQLITE_BUSY errors
- User experience degrades if one terminal blocks another
- No retry mechanism or automatic recovery

**Fix Approach:**
1. Set `PRAGMA busy_timeout = 5000` on all database connections to give 5 second retry window
2. Implement exponential backoff for connection failures
3. Log concurrency conflicts for debugging
4. Add integration test with concurrent database access

---

### 8. Streaming Response Buffering Strategy Undefined

**Issue:** Architecture specifies "streaming responses" and README claims "Real-time output as the AI generates responses", but no streaming implementation exists.

**Files:**
- `crates/core/src/provider/` (missing streaming implementation)

**Risk:**
- Could buffer entire response in memory before displaying (defeating streaming benefit)
- Large responses (>1MB) could cause memory pressure
- User experiences long delay before first token appears

**Design Questions to Resolve:**
1. How are SSE (Server-Sent Events) streams parsed and buffered?
2. What's the maximum buffered chunk size?
3. How is output flushed to terminal in real-time?
4. How are stream errors (connection drops mid-response) handled?

**Fix Approach:**
1. Define `CompletionStream` type that yields chunks asynchronously
2. Implement tokio StreamExt usage for handling server-sent events
3. Flush output immediately on each chunk (don't buffer)
4. Add tests with simulated network delays and stream disconnections

---

## Fragile Areas

### 9. Provider Fallback Logic Not Designed

**Issue:** Standards don't specify what happens if primary provider is unavailable. README shows switching providers, but no automatic fallback or retry strategy is defined.

**Risk:**
- User configures OpenAI as default, API is down, no fallback to Ollama
- No graceful degradation or user guidance

**Questions:**
1. Should there be a provider priority list with automatic fallback?
2. How should users be informed when provider is unavailable?
3. Should failed requests retry with exponential backoff?

**Fix Approach:**
1. Define fallback strategy in provider configuration
2. Implement retry logic with configurable max attempts
3. Provide clear error messages indicating which provider failed and why
4. Add tests for provider unavailability scenarios

---

### 10. REPL Command Parsing Not Implemented

**Issue:** README documents REPL commands (`/help`, `/quit`, `/clear`, `/provider`, etc.) but no interactive REPL implementation exists.

**Files:**
- `crates/cli/src/repl/` (empty)
- `crates/cli/src/repl/readline.rs` (missing)

**Risk:**
- Command parsing could be vulnerable to injection if not carefully validated
- REPL could hang on certain inputs
- Signal handling (Ctrl+C, Ctrl+D) not defined

**Fragile Implementation Points:**
1. Line reading from terminal (must handle EOF, signals, terminal resize)
2. Command parsing (must handle unknown commands gracefully)
3. Context switching between providers (state must be preserved)
4. History saving (must handle database failures without hanging)

**Fix Approach:**
1. Use a proven line-reading library (rustyline recommended)
2. Implement command parser that's resilient to typos
3. Store REPL state explicitly (current provider, model, conversation ID)
4. Handle all signals gracefully (Ctrl+C, Ctrl+Z, terminal resize)

---

## Testing Coverage Gaps

### 11. No Test Files Exist

**Issue:** Constitution mandates 80% line coverage minimum, but no test files are present anywhere in the codebase.

**Files:**
- No `#[cfg(test)] mod tests` blocks in any source files
- No integration tests in `tests/` directory
- No test fixtures or mock data

**Coverage Risk:**
- Cannot merge any code without 80% coverage
- Testing strategy must be built before code is written
- Risk of writing untestable code that needs refactoring

**Priority Areas to Test:**
1. Provider trait implementations (mock HTTP responses)
2. Configuration loading and validation
3. Error handling for missing API keys
4. SQLite migrations and schema
5. REPL command parsing
6. Streaming response handling
7. Signal handling in CLI

**Fix Approach:**
1. Create test modules alongside each implementation
2. Use `mockito` crate for mocking HTTP requests
3. Create `tests/common/mod.rs` for shared fixtures
4. Write tests before implementation (TDD approach)
5. Run `cargo llvm-cov --fail-under-lines 80` in CI

---

### 12. Integration Test Strategy Undefined

**Issue:** Constitution references integration tests but none are defined. How do we test the end-to-end flow (user input → provider request → response → storage)?

**Files:**
- `tests/` directory structure not defined

**Questions:**
1. Do integration tests hit real APIs (risky) or use mocks?
2. How do we test provider switching mid-conversation?
3. How do we test database migrations don't corrupt existing data?
4. How do we test zsh integration without a full terminal?

**Fix Approach:**
1. Create integration tests in `tests/integration_test.rs`
2. Use mock HTTP server (mockito or wiremock-rs)
3. Create throwaway test databases for each test
4. Document which tests require external dependencies

---

## Missing Critical Features

### 13. No Homebrew Formula or Installation

**Issue:** README advertises Homebrew installation, but no formula exists.

```bash
brew tap dunnock/tap
brew install cherry2k
```

**Risk:**
- Users cannot install via documented method
- Pre-built binaries don't exist
- Installation from source is the only option

**Fix Approach:**
1. Create Homebrew formula in `dunnock/homebrew-tap` repository
2. Add CI/CD step to build release binaries
3. Add GitHub Actions workflow for macOS/Linux binary builds
4. Test installation via homebrew locally before release

---

### 14. No .gitignore File

**Issue:** Security standards specify config files and API keys must not be committed, but no `.gitignore` exists.

**Risk:**
- Could accidentally commit `.env`, `config.toml`, database files
- Could commit IDE files (`.idea/`) that shouldn't be in repo
- Could expose secrets if user isn't careful

**Fix Approach:**
1. Create `.gitignore` with entries from security.md section 1.4
2. Add entries for Rust build artifacts (already standard)
3. Document in README what files should be added to .gitignore

---

## Dependency Risks

### 15. Bundled SQLite Feature

**Issue:** Cargo.toml specifies `rusqlite = { version = "0.32", features = ["bundled"] }`.

**Risk:**
- Bundled SQLite adds build time and binary size
- Distributing SQLite with binary could have licensing implications
- System SQLite might be more optimized/maintained

**Alternative:** The "bundled" feature is convenient for development but less optimal for distribution. Consider:
1. Using system SQLite in production builds
2. Only bundling for development/testing
3. Document SQLite version requirement (3.x+)

**Fix Approach:**
1. Keep "bundled" for dev builds (easier CI/local development)
2. Remove "bundled" from release builds
3. Add CI step to verify system SQLite is available

---

### 16. Async Runtime Not Pinned

**Issue:** Standards show `tokio = { version = "1.35", features = ["full"] }` but this is flexible versioning.

**Risk:**
- Tokio 1.36+ might have breaking changes to streaming APIs
- Different developers could end up with different Tokio versions
- No lock file exists yet

**Fix Approach:**
1. Use `Cargo.lock` to lock exact versions during development
2. When releasing stable, consider pinning to specific versions
3. Regularly audit dependencies with `cargo outdated` and `cargo audit`

---

## Documentation Issues

### 17. No CHANGELOG

**Issue:** Constitution references CHANGELOG.md in project structure, but file doesn't exist.

**Files:**
- `CHANGELOG.md` (missing)

**Impact:**
- Users can't see what changed between versions
- No release notes

**Fix Approach:**
1. Create `CHANGELOG.md` following Keep a Changelog format
2. Add entry for v0.1.0 (skeleton version) with "Initial project setup"
3. Document all features, fixes, and changes in future releases

---

### 18. Contributing Guidelines Missing

**Issue:** README mentions "Please read the contributing guidelines first" and references `CONTRIBUTING.md`, but file doesn't exist.

**Files:**
- `CONTRIBUTING.md` (missing)

**Fix Approach:**
1. Create `CONTRIBUTING.md` with:
   - How to set up development environment
   - Required tools and versions
   - How to run tests and coverage checks
   - Branch naming conventions
   - PR review process
   - Link to CLAUDE.md for standards
2. Reference this file from README

---

## Platform-Specific Concerns

### 19. Linux Support Untested

**Issue:** README says "macOS or Linux" but Zsh integration and SQLite paths might be macOS-specific.

**Specific Risks:**
- Database path defaults to `~/.local/share/cherry2k/` which is XDG-compliant Linux but non-standard on macOS
- Zsh plugin installation via `brew --prefix` works on macOS but may fail on Linux
- Xcode CLI tools requirement mentioned for macOS only

**Fix Approach:**
1. Use `dirs` crate for platform-aware config/data directories
2. Test on both macOS (GitHub Actions macos-latest) and Linux (ubuntu-latest)
3. Document platform-specific installation steps
4. CI should build and test on both platforms

---

### 20. No Windows Support

**Issue:** Zsh is Unix-only, but README doesn't explicitly exclude Windows.

**Risk:**
- Windows users might try to install and fail
- No clear indication that this tool is Unix-only

**Fix Approach:**
1. Add platform check in `build.rs` that errors on Windows with helpful message
2. Update README with explicit "macOS and Linux only" statement
3. Document why (Zsh integration requires Unix shell)

---

## Monitoring & Observability Gaps

### 21. No Logging Implementation

**Issue:** Standards mention `tracing` crate for structured logging, but no logging setup exists.

**Risk:**
- Cannot debug provider failures without logging
- Users cannot enable verbose logging to troubleshoot issues
- No record of API requests/responses for auditing

**Fix Approach:**
1. Add `tracing` and `tracing-subscriber` to workspace dependencies
2. Initialize tracing in CLI main.rs with subscriber
3. Add `CHERRY2K_LOG_LEVEL` env var to control verbosity
4. Use `#[instrument]` macro on provider methods
5. Never log API keys or prompts in logs

---

### 22. No Error Context Preservation

**Issue:** Error types are defined in standards, but no way to preserve error context or stack traces.

**Risk:**
- When a provider request fails, users see generic error messages
- Difficult to debug "why did my request fail?"

**Fix Approach:**
1. Use `anyhow` for error context in CLI
2. Use `thiserror` for library error types
3. Implement error chains that show what operation failed
4. Example: "Failed to load config at /path/to/config.toml: file not found"

---

## Summary of Priority Fixes

**CRITICAL (Block compilation):**
1. Create actual `.rs` files for modules referenced in architecture
2. Implement `AiProvider` trait and basic stub implementations
3. Create error types and config structures

**HIGH (Required for MVP):**
4. Implement SQLite schema and migrations
5. Implement zsh integration plugin file
6. Implement REPL with command parsing
7. Add provider implementations for OpenAI, Anthropic, Ollama

**MEDIUM (Before release):**
8. Security: Config file permission validation
9. Security: Input validation for prompts
10. Testing: Complete 80% line coverage
11. Logging: Structured logging setup
12. Documentation: Create CHANGELOG and CONTRIBUTING.md

**LOW (Nice to have):**
13. Performance: Implement streaming response buffering
14. Reliability: Provider fallback/retry logic
15. Platform: Test and document Linux support
16. Distribution: Create Homebrew formula

---

*Concerns audit: 2026-01-29*
