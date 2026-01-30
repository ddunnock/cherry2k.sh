# Security Standards

> **Applies to**: All code, configuration, and processes in Cherry2K.sh
> **Parent**: `constitution.md`

---

## 1. API Key Management

### 1.1 Requirements

| Rule | Enforcement |
|------|-------------|
| No secrets in code | API keys **MUST NOT** be committed to version control |
| Environment variables | Keys **MUST** be loaded from environment or config file |
| File permissions | Config files with secrets **MUST** have 0600 permissions |
| No logging | API keys **MUST NOT** be logged, even partially |

### 1.2 Loading API Keys

```rust
use std::env;

/// Load provider configuration from environment.
///
/// # Environment Variables
///
/// - `OPENAI_API_KEY` - OpenAI API key
/// - `ANTHROPIC_API_KEY` - Anthropic API key
/// - `OLLAMA_HOST` - Ollama server URL (default: http://localhost:11434)
pub fn load_from_env() -> Result<ProviderConfig, ConfigError> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| ConfigError::MissingApiKey("OPENAI_API_KEY"))?;

    Ok(ProviderConfig {
        api_key,
        ..Default::default()
    })
}
```

### 1.3 Config File Security

```rust
use std::fs;
use std::os::unix::fs::PermissionsExt;

/// Load configuration from file, verifying permissions.
pub fn load_from_file(path: &Path) -> Result<Config, ConfigError> {
    // Check file permissions first
    let metadata = fs::metadata(path)?;
    let permissions = metadata.permissions();

    // Reject if group/other readable
    if permissions.mode() & 0o077 != 0 {
        return Err(ConfigError::InsecurePermissions(
            format!("Config file {} has insecure permissions. Run: chmod 600 {}",
                    path.display(), path.display())
        ));
    }

    let content = fs::read_to_string(path)?;
    toml::from_str(&content).map_err(ConfigError::Parse)
}
```

### 1.4 Git Ignore

```gitignore
# .gitignore

# Config files with secrets
config.toml
.env
.env.local
*.key
*.pem

# Local database
*.db
*.sqlite

# IDE
.idea/
.vscode/
```

---

## 2. Input Validation

### 2.1 Requirements

| Rule | Enforcement |
|------|-------------|
| All external input | **MUST** be validated before use |
| User prompts | **MUST** be length-limited |
| File paths | **MUST** be canonicalized and checked |
| Provider responses | **MUST** be validated before processing |

### 2.2 User Input Validation

```rust
/// Maximum prompt length in characters.
const MAX_PROMPT_LENGTH: usize = 100_000;

/// Maximum conversation history to send.
const MAX_HISTORY_MESSAGES: usize = 100;

/// Validate user prompt before sending to provider.
pub fn validate_prompt(prompt: &str) -> Result<(), ValidationError> {
    if prompt.is_empty() {
        return Err(ValidationError::EmptyPrompt);
    }

    if prompt.len() > MAX_PROMPT_LENGTH {
        return Err(ValidationError::PromptTooLong {
            length: prompt.len(),
            max: MAX_PROMPT_LENGTH,
        });
    }

    // Check for null bytes (could cause issues in C FFI)
    if prompt.contains('\0') {
        return Err(ValidationError::InvalidCharacters("null bytes"));
    }

    Ok(())
}
```

### 2.3 Path Validation

```rust
use std::path::{Path, PathBuf};

/// Validate and canonicalize a file path.
///
/// # Security
///
/// - Resolves symlinks to prevent directory traversal
/// - Rejects paths outside allowed directories
pub fn validate_path(path: &Path, allowed_base: &Path) -> Result<PathBuf, ValidationError> {
    // Canonicalize to resolve .. and symlinks
    let canonical = path.canonicalize()
        .map_err(|_| ValidationError::InvalidPath("path does not exist"))?;

    let allowed_canonical = allowed_base.canonicalize()
        .map_err(|_| ValidationError::InvalidPath("base path does not exist"))?;

    // Verify path is under allowed base
    if !canonical.starts_with(&allowed_canonical) {
        return Err(ValidationError::PathTraversal);
    }

    Ok(canonical)
}
```

---

## 3. SQLite Security

### 3.1 Requirements

| Rule | Enforcement |
|------|-------------|
| No secrets in DB | API keys **MUST NOT** be stored in SQLite |
| Parameterized queries | **MUST** use prepared statements |
| File permissions | Database file **MUST** have 0600 permissions |

### 3.2 Parameterized Queries

```rust
// ❌ Bad - SQL injection vulnerability
let query = format!("SELECT * FROM conversations WHERE id = '{}'", id);
conn.execute(&query, [])?;

// ✅ Good - parameterized query
conn.query_row(
    "SELECT * FROM conversations WHERE id = ?",
    [id],
    |row| /* ... */
)?;
```

### 3.3 Database File Permissions

```rust
use std::os::unix::fs::PermissionsExt;

/// Create or open database with secure permissions.
pub fn open_secure_database(path: &Path) -> Result<Connection, StorageError> {
    let conn = Connection::open(path)?;

    // Set file permissions to owner-only
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)?;

    Ok(conn)
}
```

---

## 4. Network Security

### 4.1 Requirements

| Rule | Enforcement |
|------|-------------|
| HTTPS only | Cloud providers **MUST** use HTTPS |
| Certificate validation | TLS certificates **MUST** be validated |
| Timeouts | All requests **MUST** have timeouts |
| Local exception | Ollama localhost **MAY** use HTTP |

### 4.2 HTTP Client Configuration

```rust
use reqwest::Client;
use std::time::Duration;

/// Create secure HTTP client for API requests.
pub fn create_http_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        // Enforce HTTPS for non-localhost
        .https_only(true)
        // Validate certificates (default, but be explicit)
        .danger_accept_invalid_certs(false)
        // Timeouts
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(120))
        // User agent
        .user_agent(format!("cherry2k/{}", env!("CARGO_PKG_VERSION")))
        .build()
}

/// Create client for local Ollama (HTTP allowed).
pub fn create_ollama_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(300))  // Longer for local inference
        .build()
}
```

### 4.3 Request Security

```rust
impl OpenAiProvider {
    async fn make_request(&self, request: &CompletionRequest) -> Result<Response, ProviderError> {
        let response = self.client
            .post(&format!("{}/v1/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;

        // Check for error responses
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();

            return Err(match status {
                401 => ProviderError::Auth("Invalid API key".into()),
                429 => ProviderError::RateLimited { retry_after: parse_retry_after(&body) },
                _ => ProviderError::Api { status, message: body },
            });
        }

        Ok(response)
    }
}
```

---

## 5. Logging Security

### 5.1 Requirements

| Rule | Enforcement |
|------|-------------|
| No secrets | **MUST NOT** log API keys, tokens, or passwords |
| No PII | **SHOULD NOT** log user prompts in production |
| Structured logging | **SHOULD** use tracing with structured fields |

### 5.2 What to Log

```rust
use tracing::{info, warn, error, instrument};

// ✅ Good - log metadata, not content
#[instrument(skip(request), fields(provider = %self.provider_id()))]
async fn complete(&self, request: CompletionRequest) -> Result<Response, ProviderError> {
    info!(
        model = %request.model,
        max_tokens = request.max_tokens,
        "Sending completion request"
    );

    // ...

    info!(
        response_tokens = response.usage.completion_tokens,
        "Received completion response"
    );
}

// ❌ Bad - logging sensitive data
info!("API key: {}", api_key);
info!("User prompt: {}", prompt);
```

### 5.3 Error Logging

```rust
// ✅ Good - log error type, not full details that might contain secrets
error!(
    error_type = "api_error",
    status = %status,
    "Provider request failed"
);

// ❌ Bad - might log API key in URL or body
error!("Request failed: {:?}", response);
```

---

## 6. Error Messages

### 6.1 Requirements

| Rule | Enforcement |
|------|-------------|
| No secrets in errors | Error messages **MUST NOT** contain API keys |
| User-friendly | Errors **SHOULD** suggest remediation |
| No stack traces to users | Debug info **SHOULD** go to logs, not output |

### 6.2 Safe Error Messages

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Missing API key - suggest how to set it.
    #[error("Missing API key. Set {0} environment variable or add to config file.")]
    MissingApiKey(&'static str),

    /// Invalid API key - don't include the actual key.
    #[error("Invalid API key format. Keys should start with 'sk-'.")]
    InvalidApiKeyFormat,

    /// Config file not found - include path for debugging.
    #[error("Config file not found: {0}")]
    NotFound(PathBuf),

    /// Config file has insecure permissions.
    #[error("Config file has insecure permissions. Run: chmod 600 {0}")]
    InsecurePermissions(String),
}
```

---

## 7. Dependency Security

### 7.1 Requirements

| Rule | Enforcement |
|------|-------------|
| Audit dependencies | Run `cargo audit` before releases |
| Pin versions | Use exact versions in `Cargo.lock` |
| Minimal features | Only enable necessary crate features |
| Update regularly | Check for updates monthly |

### 7.2 Audit Commands

```bash
# Install cargo-audit
cargo install cargo-audit

# Run security audit
cargo audit

# Fix vulnerabilities (updates Cargo.lock)
cargo audit fix

# Deny all advisories in CI
cargo audit --deny warnings
```

### 7.3 Dependabot Configuration

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
    groups:
      rust-dependencies:
        patterns:
          - "*"
```

---

## 8. CI Security Scanning

```yaml
# .github/workflows/security.yml
name: Security

on:
  push:
    branches: [main]
  pull_request:
  schedule:
    - cron: "0 0 * * 0"  # Weekly

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Run security audit
        run: cargo audit --deny warnings

  clippy-security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Run clippy with security lints
        run: |
          cargo clippy -- \
            -D clippy::unwrap_used \
            -D clippy::expect_used \
            -D clippy::panic
```

---

## 9. Security Checklist

### Before Release

- [ ] No API keys or secrets in code
- [ ] Config files require 0600 permissions
- [ ] All user input is validated
- [ ] SQLite queries use parameters
- [ ] HTTPS enforced for cloud providers
- [ ] No secrets in logs or error messages
- [ ] `cargo audit` passes with no warnings
- [ ] Dependencies are up to date

### Code Review

- [ ] New inputs are validated
- [ ] No hardcoded secrets
- [ ] Error messages don't leak sensitive info
- [ ] Logging doesn't include secrets or PII
- [ ] File operations validate paths