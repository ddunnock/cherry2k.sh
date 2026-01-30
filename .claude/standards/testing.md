# Testing Standards

> **Applies to**: All automated tests in Cherry2K.sh
> **Parent**: `constitution.md`

---

## 1. Coverage Requirements

All code **MUST** meet or exceed these thresholds:

| Metric | Minimum | Target | Blocking |
|--------|---------|--------|----------|
| Line Coverage | 80% | 90% | Yes |
| Branch Coverage | 75% | 85% | Yes |
| Function Coverage | 85% | 95% | Yes |

### 1.1 Exclusions

These **MAY** be excluded (with documented rationale):

- Generated code (build scripts, macros)
- Debug-only code paths (`#[cfg(debug_assertions)]`)
- Integration test fixtures
- Unreachable error handlers (e.g., `unreachable!()` with invariant comments)

---

## 2. Enforcement Commands

### 2.1 Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in specific crate
cargo test -p cherry2k-core
```

### 2.2 Coverage with cargo-llvm-cov

```bash
# Install
cargo install cargo-llvm-cov

# Run with coverage threshold
cargo llvm-cov --fail-under-lines 80

# Generate HTML report
cargo llvm-cov --html
open target/llvm-cov/html/index.html

# Generate lcov for CI
cargo llvm-cov --lcov --output-path lcov.info
```

---

## 3. Test Organization

### 3.1 Directory Structure

```
crates/
├── core/
│   └── src/
│       ├── provider/
│       │   ├── mod.rs
│       │   ├── openai.rs         # Contains #[cfg(test)] mod tests
│       │   └── anthropic.rs      # Contains #[cfg(test)] mod tests
│       └── lib.rs
├── storage/
│   └── src/
│       ├── repository.rs         # Contains #[cfg(test)] mod tests
│       └── lib.rs
└── cli/
    └── src/
        └── main.rs
tests/                            # Workspace-level integration tests
├── integration_test.rs
├── provider_switching.rs
└── fixtures/
    ├── mod.rs
    └── mock_responses.rs
```

### 3.2 Unit Tests (In-Module)

Unit tests live alongside the code they test:

```rust
// src/provider/openai.rs

pub struct OpenAiProvider { /* ... */ }

impl OpenAiProvider {
    pub fn new(config: ProviderConfig) -> Result<Self, ConfigError> {
        // Implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fixtures {
        use super::*;

        pub fn mock_config() -> ProviderConfig {
            ProviderConfig {
                api_key: "test-key".into(),
                base_url: "http://localhost:8080".into(),
                model: "gpt-4".into(),
            }
        }
    }

    mod new_tests {
        use super::*;

        #[test]
        fn creates_provider_with_valid_config() {
            let config = fixtures::mock_config();
            let result = OpenAiProvider::new(config);
            assert!(result.is_ok());
        }

        #[test]
        fn rejects_empty_api_key() {
            let config = ProviderConfig {
                api_key: "".into(),
                ..fixtures::mock_config()
            };
            let result = OpenAiProvider::new(config);
            assert!(matches!(result, Err(ConfigError::MissingApiKey)));
        }
    }
}
```

### 3.3 Integration Tests (Workspace-Level)

```rust
// tests/provider_switching.rs

use cherry2k_core::provider::{AiProvider, ProviderFactory};
use cherry2k_storage::ConversationRepository;

#[tokio::test]
async fn switches_provider_mid_conversation() {
    // Arrange
    let repo = ConversationRepository::in_memory().unwrap();
    let openai = ProviderFactory::create("openai", test_config()).unwrap();
    let anthropic = ProviderFactory::create("anthropic", test_config()).unwrap();

    // Act - Start with OpenAI
    let response1 = openai.complete(request("Hello")).await.unwrap();
    repo.save_message(&response1).unwrap();

    // Switch to Anthropic
    let response2 = anthropic.complete(request("Continue")).await.unwrap();
    repo.save_message(&response2).unwrap();

    // Assert - Both messages saved
    let history = repo.list_messages().unwrap();
    assert_eq!(history.len(), 2);
}
```

---

## 4. Test Naming Conventions

### 4.1 Test Functions

Use descriptive snake_case names that describe the scenario:

```rust
// Pattern: <action>_<condition>_<expected_result>

#[test]
fn creates_provider_with_valid_config() { }

#[test]
fn rejects_empty_api_key() { }

#[test]
fn returns_none_for_missing_conversation() { }

#[test]
fn streams_response_chunks_in_order() { }

#[test]
fn handles_rate_limit_with_retry() { }
```

### 4.2 Test Modules

Group tests by the function/method being tested:

```rust
#[cfg(test)]
mod tests {
    mod new_tests { /* tests for ::new() */ }
    mod complete_tests { /* tests for .complete() */ }
    mod validate_config_tests { /* tests for .validate_config() */ }
}
```

---

## 5. Test Patterns

### 5.1 Arrange-Act-Assert (AAA)

All tests **MUST** follow the AAA pattern:

```rust
#[test]
fn saves_conversation_to_database() {
    // Arrange
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    let repo = ConversationRepository::new(conn);
    let conversation = Conversation::new("test-id");

    // Act
    let result = repo.save(&conversation);

    // Assert
    assert!(result.is_ok());
    let loaded = repo.find("test-id").unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().id, "test-id");
}
```

### 5.2 Test Fixtures

Create reusable fixtures in a `fixtures` submodule:

```rust
#[cfg(test)]
mod tests {
    mod fixtures {
        use super::*;

        pub fn mock_config() -> ProviderConfig {
            ProviderConfig {
                api_key: "test-key".into(),
                base_url: "http://localhost:8080".into(),
                model: "test-model".into(),
            }
        }

        pub fn sample_conversation() -> Conversation {
            Conversation {
                id: "test-conv-1".into(),
                messages: vec![
                    Message::user("Hello"),
                    Message::assistant("Hi there!"),
                ],
                created_at: Utc::now(),
            }
        }

        pub fn sample_request() -> CompletionRequest {
            CompletionRequest {
                prompt: "Test prompt".into(),
                max_tokens: 100,
                temperature: 0.7,
            }
        }
    }
}
```

### 5.3 Testing Async Code

Use `#[tokio::test]` for async tests:

```rust
#[tokio::test]
async fn fetches_completion_from_api() {
    // Arrange
    let mock_server = MockServer::start().await;
    setup_mock_response(&mock_server).await;

    let provider = OpenAiProvider::new_with_base_url(
        fixtures::mock_config(),
        mock_server.uri(),
    );

    // Act
    let result = provider.complete(fixtures::sample_request()).await;

    // Assert
    assert!(result.is_ok());
}
```

---

## 6. Mocking

### 6.1 HTTP Mocking with wiremock

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};

#[tokio::test]
async fn handles_successful_api_response() {
    // Arrange
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "id": "chatcmpl-123",
                    "choices": [{
                        "message": {
                            "role": "assistant",
                            "content": "Hello!"
                        }
                    }]
                }))
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = ProviderConfig {
        base_url: mock_server.uri(),
        ..fixtures::mock_config()
    };
    let provider = OpenAiProvider::new(config).unwrap();

    // Act
    let result = provider.complete(request).await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn handles_rate_limit_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "30")
                .set_body_json(serde_json::json!({
                    "error": {
                        "message": "Rate limit exceeded",
                        "type": "rate_limit_error"
                    }
                }))
        )
        .mount(&mock_server)
        .await;

    let provider = create_provider_with_mock(&mock_server);

    // Act
    let result = provider.complete(request).await;

    // Assert
    assert!(matches!(result, Err(ProviderError::RateLimited { retry_after: 30 })));
}
```

### 6.2 Trait-Based Mocking

Use traits to enable mocking without external crates:

```rust
// In production code
pub trait Repository {
    fn save(&self, item: &Item) -> Result<(), StorageError>;
    fn find(&self, id: &str) -> Result<Option<Item>, StorageError>;
}

// In test code
#[cfg(test)]
mod tests {
    struct MockRepository {
        items: std::cell::RefCell<HashMap<String, Item>>,
    }

    impl Repository for MockRepository {
        fn save(&self, item: &Item) -> Result<(), StorageError> {
            self.items.borrow_mut().insert(item.id.clone(), item.clone());
            Ok(())
        }

        fn find(&self, id: &str) -> Result<Option<Item>, StorageError> {
            Ok(self.items.borrow().get(id).cloned())
        }
    }
}
```

---

## 7. SQLite Testing

### 7.1 In-Memory Database

Use in-memory SQLite for fast, isolated tests:

```rust
#[test]
fn saves_and_retrieves_conversation() {
    // Arrange - in-memory database
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    let repo = ConversationRepository::new(conn);

    let conversation = fixtures::sample_conversation();

    // Act
    repo.save(&conversation).unwrap();
    let loaded = repo.find(&conversation.id).unwrap();

    // Assert
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().messages.len(), 2);
}
```

### 7.2 Migration Testing

Test that migrations apply cleanly:

```rust
#[test]
fn migrations_apply_cleanly() {
    let conn = Connection::open_in_memory().unwrap();

    // Should not panic
    let result = run_migrations(&conn);

    assert!(result.is_ok());

    // Verify tables exist
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    assert!(tables.contains(&"conversations".to_string()));
    assert!(tables.contains(&"migrations".to_string()));
}

#[test]
fn migrations_are_idempotent() {
    let conn = Connection::open_in_memory().unwrap();

    // Run twice
    run_migrations(&conn).unwrap();
    let result = run_migrations(&conn);

    // Should succeed without error
    assert!(result.is_ok());
}
```

---

## 8. CLI Testing

### 8.1 Command Parsing Tests

```rust
use clap::Parser;

#[test]
fn parses_chat_command() {
    let cli = Cli::try_parse_from(["cherry2k", "chat", "Hello world"]).unwrap();

    match cli.command {
        Commands::Chat { message, provider } => {
            assert_eq!(message, "Hello world");
            assert!(provider.is_none());
        }
        _ => panic!("Expected Chat command"),
    }
}

#[test]
fn parses_chat_with_provider() {
    let cli = Cli::try_parse_from([
        "cherry2k", "chat", "-p", "anthropic", "Hello"
    ]).unwrap();

    match cli.command {
        Commands::Chat { message, provider } => {
            assert_eq!(message, "Hello");
            assert_eq!(provider, Some("anthropic".into()));
        }
        _ => panic!("Expected Chat command"),
    }
}
```

### 8.2 Output Format Tests

```rust
#[test]
fn formats_response_as_json() {
    let response = CompletionResponse {
        content: "Hello!".into(),
        model: "gpt-4".into(),
    };

    let output = format_output(&response, OutputFormat::Json);

    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["content"], "Hello!");
}
```

---

## 9. CI Integration

### 9.1 GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test

on:
  push:
    branches: [main, develop]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Run tests with coverage
        run: cargo llvm-cov --lcov --output-path lcov.info --fail-under-lines 80

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: true
```

### 9.2 Quality Gates

**Blocking** (MUST pass):
- All tests pass
- Coverage ≥80% lines
- No test warnings

**Non-Blocking** (warnings):
- Coverage <90% (target)
- Slow tests (>1s)

---

## 10. Test Quality Checklist

Before submitting PR:

- [ ] All new code has tests
- [ ] Tests follow AAA pattern
- [ ] Descriptive test names
- [ ] Fixtures used for common setup
- [ ] Async code uses `#[tokio::test]`
- [ ] HTTP calls mocked with wiremock
- [ ] SQLite tests use in-memory database
- [ ] Coverage threshold met (80%)
- [ ] No `.unwrap()` in test assertions (use `assert!` instead)