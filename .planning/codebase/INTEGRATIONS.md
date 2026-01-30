# External Integrations

**Analysis Date:** 2026-01-29

## APIs & External Services

**AI Providers (Cloud):**
- OpenAI - Send completion requests, receive streaming responses
  - SDK/Client: `reqwest` HTTP client
  - Auth: `OPENAI_API_KEY` environment variable (format: `sk-*`)
  - Endpoint: `https://api.openai.com/v1/chat/completions`
  - Models supported: GPT-4, GPT-4 Turbo, GPT-3.5 Turbo
  - Features: Streaming responses, configurable temperature/max_tokens

- Anthropic - Send completion requests, receive streaming responses
  - SDK/Client: `reqwest` HTTP client
  - Auth: `ANTHROPIC_API_KEY` environment variable (format: `sk-ant-*`)
  - Endpoint: `https://api.anthropic.com/v1/messages`
  - Models supported: Claude 3 Opus, Claude 3 Sonnet, Claude 3 Haiku
  - Features: Streaming responses, configurable temperature/max_tokens

- Ollama (Local) - No external API; runs locally on user's machine
  - SDK/Client: `reqwest` HTTP client
  - Auth: None (no authentication required)
  - Endpoint: `http://localhost:11434` (configurable via `OLLAMA_HOST`)
  - Models supported: Any Ollama-compatible model (Llama 2, Mistral, CodeLlama, etc.)
  - Features: Offline operation, no external network required

## Data Storage

**Databases:**
- SQLite 3 (Bundled via rusqlite)
  - Connection: Configured via config file path (`CHERRY2K_CONFIG_PATH` or `~/.config/cherry2k/config.toml`)
  - Client: `rusqlite` Rust crate with bundled SQLite
  - Storage location: `~/.local/share/cherry2k/conversations.db` (configurable)
  - Features: Conversation history, user preferences, provider configurations, session metadata
  - Concurrency: WAL (Write-Ahead Logging) mode enabled for better concurrent access
  - Security: Database file requires 0600 permissions; parameterized queries used exclusively

**File Storage:**
- Local filesystem only - No external file storage service
  - Config file: `~/.config/cherry2k/config.toml` (TOML format)
  - Database file: `~/.local/share/cherry2k/conversations.db` (SQLite)
  - Both stored locally with restricted file permissions (0600)

**Caching:**
- None - No caching layer configured; responses fetched fresh from providers or SQLite

## Authentication & Identity

**Auth Provider:**
- Custom implementation via environment variables
  - OpenAI API key validation and header injection
  - Anthropic API key validation and header injection
  - Ollama: No authentication required for localhost
  - Implementation approach: Keys loaded from environment or config file; never logged or stored in code

## Monitoring & Observability

**Error Tracking:**
- None - No external error tracking service (Sentry, Bugsnag, etc.)
- Error propagation via `thiserror` and `anyhow` crates
- Errors surfaced to user via CLI output

**Logs:**
- Structured logging via Tracing framework
  - Output: stderr by default
  - Format: Controlled by `CHERRY2K_LOG_LEVEL` environment variable (info, debug, warn, error)
  - Tracing-subscriber with env-filter for dynamic filtering
  - Security: No API keys, tokens, or user prompts logged (instrumentation skips sensitive fields)

## CI/CD & Deployment

**Hosting:**
- User's local machine (macOS or Linux)
- Homebrew distribution via `dunnock/tap` (optional)
- Manual installation: `cargo build --release` and binary placement

**CI Pipeline:**
- None specified in codebase (to be configured by team)
- Recommended: GitHub Actions workflow
  - Cargo fmt, clippy, test, coverage, audit before merge

## Environment Configuration

**Required env vars:**
- One of: `OPENAI_API_KEY` (for OpenAI) OR `ANTHROPIC_API_KEY` (for Anthropic) OR `OLLAMA_HOST` (for Ollama)
- Everything else is optional with sensible defaults

**Optional env vars:**
- `CHERRY2K_CONFIG_PATH` - Path to config TOML file (default: `~/.config/cherry2k/config.toml`)
- `CHERRY2K_LOG_LEVEL` - Log verbosity (default: `info`)
- `OLLAMA_HOST` - Ollama server URL (default: `http://localhost:11434`)

**Secrets location:**
- Environment variables: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`
- Config file: `~/.config/cherry2k/config.toml` (treated as sensitive; must have 0600 permissions)
- Never committed to git (listed in `.gitignore`)

## Webhooks & Callbacks

**Incoming:**
- None - Cherry2K does not expose any HTTP endpoints or webhooks

**Outgoing:**
- None - Cherry2K only initiates requests to AI provider APIs; no outbound webhooks

## API Response Handling

**OpenAI:**
- Request: POST to `/v1/chat/completions` with JSON body
- Response: SSE (Server-Sent Events) for streaming
- Error handling: 401 (invalid key), 429 (rate limit), generic 4xx/5xx errors

**Anthropic:**
- Request: POST to `/v1/messages` with JSON body
- Response: Streaming JSON format
- Error handling: 401 (invalid key), 429 (rate limit), generic 4xx/5xx errors

**Ollama:**
- Request: POST to `/api/generate` with JSON body
- Response: Streaming JSON format
- Error handling: Local connection errors, model not found

## Network Configuration

**HTTPS Requirements:**
- OpenAI: HTTPS only (enforced via `reqwest` client config)
- Anthropic: HTTPS only (enforced via `reqwest` client config)
- Ollama: HTTP allowed (localhost exception for development/local use)

**Timeouts:**
- Cloud providers: 10s connection timeout, 120s request timeout
- Ollama: 5s connection timeout, 300s request timeout (longer for local inference)

**User Agent:**
- `cherry2k/{VERSION}` - Added to all HTTP requests for identification

## Request/Response Flow

### Chat Command
1. User: `cherry2k chat "prompt"`
2. CLI parses input, validates prompt length (<100KB)
3. Loads provider config (from env or config file)
4. Makes async request to selected provider (OpenAI, Anthropic, or Ollama)
5. Streams response chunks as they arrive
6. Saves conversation to SQLite
7. Outputs formatted response to terminal

### Interactive REPL
1. User: `cherry2k repl`
2. REPL loop reads user input line-by-line
3. Commands starting with `/` processed locally (e.g., `/provider anthropic`, `/history`)
4. Regular prompts sent to provider with accumulated conversation context
5. Responses streamed in real-time
6. Conversation persisted to SQLite after each exchange

---

*Integration audit: 2026-01-29*
