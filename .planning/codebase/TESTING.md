# Testing Patterns

**Analysis Date:** 2026-01-29

## Test Framework

**Runner:**
- `cargo test` - Built-in test runner
- Config: No separate config file needed (uses Cargo.toml)
- Edition: Rust 2021

**Assertion Library:**
- Built-in `assert!()`, `assert_eq!()`, `assert_ne!()`
- No external assertion library required

**Run Commands:**
```bash
cargo test                                 # Run all tests
cargo test --doc                          # Run documentation examples as tests
cargo test -- --nocapture                 # Run with println! output visible
cargo test test_name                      # Run specific test by name
cargo test -p cherry2k-core               # Run tests in specific crate
cargo test -- --test-threads=1            # Run serially (for debugging)
```

**Coverage:**
```bash
cargo install cargo-llvm-cov              # Install coverage tool
cargo llvm-cov --fail-under-lines 80      # Generate coverage, fail if <80%
cargo llvm-cov --html                     # Generate HTML report
open target/llvm-cov/html/index.html      # View report
```

## Test File Organization

**Location:**
- **Unit tests**: Inline in source files under `#[cfg(test)] mod tests`
- **Integration tests**: In `tests/` directory at workspace root
- **Fixtures**: In `tests/fixtures/` directory

**Naming:**
- Test files: `*.rs` in `tests/` directory (e.g., `integration_test.rs`, `provider_switching.rs`)
- Test functions: `snake_case` descriptive names (e.g., `test_creates_provider_with_valid_config()`)
- Test modules: `test_name` following pattern `<function>_tests` (e.g., `mod new_tests`)

**Structure:**
```
crates/
├── core/
│   └── src/
│       ├── provider/
│       │   ├── openai.rs         # Contains #[cfg(test)] mod tests
│       │   └── trait.rs          # Contains #[cfg(test)] mod tests
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

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod fixtures {
        // Reusable test data
    }

    mod new_tests {
        // Tests for ::new() method
    }

    mod complete_tests {
        // Tests for .complete() method
    }

    mod validate_config_tests {
        // Tests for .validate_config() method
    }
}
```

**Patterns:**
- **Arrange-Act-Assert (AAA)**: Every test follows this structure
- **Fixtures submodule**: Common setup lives in `mod fixtures { }` block
- **Grouped by method**: Tests grouped in submodules by the function being tested
- **No setup/teardown macros**: Use explicit fixtures instead

## Arrange-Act-Assert Pattern

All tests **MUST** follow AAA structure:

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

**Key Points:**
- Clear separation with comments
- Arrange: Set up preconditions
- Act: Execute the code being tested
- Assert: Verify results

## Test Naming Conventions

**Function Names:**
Pattern: `<action>_<condition>_<expected_result>` or `<test_behavior>`

```rust
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

#[test]
fn logs_authentication_failure() { }
```

**Module Names:**
Group by function being tested:

```rust
#[cfg(test)]
mod tests {
    mod new_tests { /* tests for ::new() */ }
    mod complete_tests { /* tests for .complete() */ }
    mod validate_config_tests { /* tests for .validate_config() */ }
}
```

## Fixtures

**Test Data Fixtures:**
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

**Location:**
- All fixtures live in `mod fixtures` block within `#[cfg(test)] mod tests`
- Reusable across multiple test functions in the same module
- Build upon each other for complex scenarios

## Mocking

**HTTP Mocking with wiremock:**
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
```

**Mocking Error Responses:**
```rust
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

**Trait-Based Mocking (for Storage):**
```rust
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

    #[test]
    fn uses_mock_repository() {
        let repo = MockRepository {
            items: RefCell::new(HashMap::new()),
        };
        // Use repo in test
    }
}
```

**What to Mock:**
- External HTTP APIs (use wiremock)
- Database access (use in-memory SQLite or trait mocks)
- Time-sensitive operations (use tokio-test)
- File system operations (use temporary directories)

**What NOT to Mock:**
- Core business logic being tested
- Error types and error paths
- Serialization/deserialization logic
- Small utility functions

## Async Testing

**Using `#[tokio::test]`:**
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

**Key Points:**
- Always use `#[tokio::test]`, never `#[test]` for async code
- Tests run on tokio runtime automatically
- Supports `.await` syntax

## SQLite Testing

**In-Memory Database:**
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

**Migration Testing:**
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

## CLI Testing

**Command Parsing Tests:**
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

**Output Format Tests:**
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

## Test Coverage

**Requirements:**
- **Minimum**: 80% line coverage (blocking)
- **Target**: 90% line coverage
- **Branch coverage**: 75% minimum, 85% target
- **Function coverage**: 85% minimum, 95% target

**Exclusions** (with documented rationale):
- Generated code (build scripts, macros)
- Debug-only code paths (`#[cfg(debug_assertions)]`)
- Integration test fixtures
- Unreachable error handlers (with `unreachable!()` and comment explaining invariant)

**View Coverage:**
```bash
cargo llvm-cov --fail-under-lines 80     # Check and fail if below 80%
cargo llvm-cov --html                    # Generate HTML report
open target/llvm-cov/html/index.html     # View in browser
```

## Integration Tests

**Location:** `tests/` directory at workspace root

**Example Structure:**
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

## Documentation Tests

**In Rustdoc Comments:**
Tests embedded in doc comments run automatically with `cargo test --doc`:

```rust
/// Send a completion request to the AI provider.
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
pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionStream, ProviderError> {
    // ...
}
```

**Key Directives:**
- `#[test]`: Example compiles and runs (default)
- `,no_run`: Example compiles but doesn't run (for examples that need manual setup)
- `,ignore`: Example is skipped
- `#[...]`: Lines starting with `#` are hidden from documentation (setup/cleanup)

## CI/CD Integration

**GitHub Actions Test Workflow:**
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

**Quality Gates:**

**Blocking** (MUST pass):
- All tests pass: `cargo test`
- Coverage ≥80% lines: `cargo llvm-cov --fail-under-lines 80`
- No test warnings

**Non-Blocking** (warnings):
- Coverage <90% (target, not required)
- Slow tests (>1s execution)

## Test Quality Checklist

Before submitting PR:

- [ ] All new code has tests
- [ ] Tests follow AAA pattern (Arrange-Act-Assert)
- [ ] Descriptive test names following naming conventions
- [ ] Fixtures used for common setup, no duplication
- [ ] Async code uses `#[tokio::test]`
- [ ] HTTP calls mocked with wiremock
- [ ] SQLite tests use in-memory database
- [ ] Coverage threshold met (80% minimum)
- [ ] No `.unwrap()` in test assertions (use `assert!()` instead)
- [ ] Documentation tests compile and run (`cargo test --doc`)
- [ ] Integration tests in workspace `tests/` directory
- [ ] Error paths tested, not just happy paths

## Common Test Patterns

**Testing Error Cases:**
```rust
#[test]
fn rejects_empty_api_key() {
    let config = ProviderConfig {
        api_key: "".into(),
        ..fixtures::mock_config()
    };
    let result = OpenAiProvider::new(config);
    assert!(matches!(result, Err(ConfigError::MissingApiKey)));
}
```

**Testing Streaming:**
```rust
#[tokio::test]
async fn streams_response_chunks_in_order() {
    // Arrange
    let mock_server = MockServer::start().await;
    setup_streaming_response(&mock_server).await;
    let provider = create_provider_with_mock(&mock_server);

    // Act
    let mut stream = provider.complete(request).await.unwrap();
    let mut chunks = Vec::new();
    while let Some(chunk) = stream.next().await {
        chunks.push(chunk.unwrap());
    }

    // Assert
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], "Hello");
    assert_eq!(chunks[1], " ");
    assert_eq!(chunks[2], "world");
}
```

**Testing Database Operations:**
```rust
#[test]
fn handles_concurrent_writes() {
    let conn = Arc::new(Connection::open_in_memory().unwrap());
    run_migrations(&conn).unwrap();

    let mut handles = vec![];
    for i in 0..10 {
        let conn = Arc::clone(&conn);
        let handle = std::thread::spawn(move || {
            let repo = ConversationRepository::new(conn);
            let conversation = Conversation::new(&format!("test-{}", i));
            repo.save(&conversation)
        });
        handles.push(handle);
    }

    for handle in handles {
        assert!(handle.join().unwrap().is_ok());
    }
}
```

---

*Testing analysis: 2026-01-29*
