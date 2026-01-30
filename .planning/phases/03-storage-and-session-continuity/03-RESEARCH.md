# Phase 03: Storage and Session Continuity - Research

**Researched:** 2026-01-30
**Domain:** SQLite persistence, async operations, session management, context window management
**Confidence:** HIGH

## Summary

Phase 03 implements conversation persistence using SQLite with an async-friendly wrapper, session management with per-directory scoping, and context window management through LLM-powered summarization. The research identifies proven patterns for each component:

**SQLite + Async:** Use `tokio-rusqlite` for async operations with rusqlite's mature API. This provides a lightweight thread-per-connection model that avoids async-await starvation issues common with blocking database calls.

**Token Counting:** Use provider-specific APIs for accurate token counting (Anthropic's free token counting endpoint, tiktoken-rs for OpenAI). Approximate counts acceptable for initial implementation, with provider APIs for precision.

**Context Management:** Implement hierarchical summarization where older messages are progressively compressed via LLM summarization, preserving recent exchanges verbatim. This is the current best practice for CLI tools with multi-turn conversations.

**Session Management:** Use XDG data directories (via `directories` crate, already in workspace), timestamp-based session IDs, and per-directory session scoping through working directory tracking.

**Primary recommendation:** Use tokio-rusqlite (0.7.x) with rusqlite bundled feature, implement LLM-based summarization for context management, leverage existing `directories` crate for XDG-compliant storage, and use simple embedded migrations pattern (SQL files in source code).

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rusqlite | 0.38.0+ | SQLite bindings | De facto Rust SQLite library, mature (10+ years), excellent API, zero-copy reads |
| tokio-rusqlite | 0.7.x | Async wrapper | Lightweight async wrapper, uses thread-per-connection to avoid blocking, 100% safe Rust |
| directories | 6.0.0 | XDG directories | Already in workspace, cross-platform, XDG-compliant, actively maintained |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tiktoken-rs | 0.9.1 | Token counting (OpenAI) | For OpenAI provider token estimation |
| refinery | 0.8.14 | Schema migrations | If complex migration needs emerge (not needed for Phase 03) |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tokio-rusqlite | sqlx | sqlx provides connection pooling and compile-time query checking, but adds significant complexity. For single-user CLI tool, thread-per-connection is simpler and sufficient. |
| tokio-rusqlite | spawn_blocking | More control but requires manual channel management. tokio-rusqlite provides cleaner API with same underlying pattern. |
| Embedded migrations | refinery | refinery adds migration versioning and rollback. Overkill for Phase 03 - simple SQL files sufficient for initial schema. |
| Provider APIs | tiktoken-rs only | tiktoken-rs is OpenAI-specific and doesn't support Claude accurately. Provider APIs are free and accurate but require network calls. Hybrid approach recommended. |

**Installation:**

```toml
[dependencies]
rusqlite = { version = "0.38", features = ["bundled"] }
tokio-rusqlite = "0.7"
directories.workspace = true  # Already in workspace
tiktoken-rs = "0.9"  # Optional for OpenAI token estimation
```

## Architecture Patterns

### Recommended Project Structure

```
crates/storage/
├── src/
│   ├── lib.rs              # Public API
│   ├── connection.rs       # tokio-rusqlite wrapper
│   ├── schema.rs           # Table definitions and migrations
│   ├── session.rs          # Session CRUD operations
│   ├── message.rs          # Message storage and retrieval
│   └── context_manager.rs  # Token counting and summarization
└── Cargo.toml
```

### Pattern 1: Async SQLite Operations

**What:** Use tokio-rusqlite's `Connection::call()` pattern for database operations
**When to use:** All database operations in async context

**Example:**

```rust
// Source: https://docs.rs/tokio-rusqlite/0.7.0/
use tokio_rusqlite::Connection;

// Open connection (async)
let conn = Connection::open(&db_path).await?;

// Execute operations via .call() with closure
let messages = conn.call(|conn| {
    let mut stmt = conn.prepare(
        "SELECT id, content, role FROM messages WHERE session_id = ?1"
    )?;

    let rows = stmt.query_map([session_id], |row| {
        Ok(Message {
            id: row.get(0)?,
            content: row.get(1)?,
            role: row.get(2)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>()
}).await?;
```

**Key insight:** The closure runs in a dedicated worker thread, returning results via oneshot channel. Connections are cheap to clone for sharing across tasks.

### Pattern 2: Per-Directory Session Management

**What:** Track session IDs by working directory to provide context isolation
**When to use:** Session creation, resume, and list operations

**Example:**

```rust
// Source: Research synthesis - common CLI pattern
use std::path::PathBuf;
use std::env;

struct SessionManager {
    db: Connection,
}

impl SessionManager {
    async fn get_or_create_session(&self) -> Result<String, StorageError> {
        let cwd = env::current_dir()?;
        let cwd_str = cwd.to_string_lossy();

        // Check for recent session in this directory
        let session_id = self.db.call(move |conn| {
            conn.query_row(
                "SELECT id FROM sessions
                 WHERE working_dir = ?1
                 AND last_message_at > datetime('now', '-4 hours')
                 ORDER BY last_message_at DESC LIMIT 1",
                [&cwd_str],
                |row| row.get(0)
            ).optional()
        }).await?;

        match session_id {
            Some(id) => Ok(id),
            None => self.create_session(&cwd).await,
        }
    }
}
```

### Pattern 3: Hierarchical Context Summarization

**What:** Compress older messages using LLM summarization while keeping recent messages verbatim
**When to use:** Before sending conversation history to provider API

**Example:**

```rust
// Source: Synthesis from https://www.getmaxim.ai/articles/context-window-management-strategies-for-long-context-ai-agents-and-chatbots
async fn manage_context(
    messages: Vec<Message>,
    token_budget: usize,
) -> Result<Vec<Message>, StorageError> {
    let token_count = estimate_tokens(&messages);

    if token_count <= token_budget {
        return Ok(messages);
    }

    // Split into recent (keep verbatim) and old (summarize)
    let split_point = messages.len() / 2;
    let (old_messages, recent_messages) = messages.split_at(split_point);

    // Use LLM to summarize old messages
    let summary = create_summary(old_messages).await?;

    // Combine summary + recent messages
    let mut result = vec![Message::summary(summary)];
    result.extend_from_slice(recent_messages);

    Ok(result)
}
```

**Key insight:** Summarization boundaries should align with conversation turns or topic shifts. Start conservative (50% split) and refine based on testing.

### Pattern 4: Embedded Migrations

**What:** Store schema SQL in source code, apply on first connection
**When to use:** Initial schema setup and simple version upgrades

**Example:**

```rust
// Source: Common Rust pattern, inspired by https://github.com/rust-db/refinery
const SCHEMA_VERSION: i32 = 1;

const INIT_SCHEMA: &str = r#"
    CREATE TABLE IF NOT EXISTS schema_version (
        version INTEGER PRIMARY KEY
    );

    CREATE TABLE IF NOT EXISTS sessions (
        id TEXT PRIMARY KEY,
        working_dir TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        last_message_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE INDEX idx_sessions_dir_time
        ON sessions(working_dir, last_message_at DESC);

    CREATE TABLE IF NOT EXISTS messages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        session_id TEXT NOT NULL,
        role TEXT NOT NULL,
        content TEXT NOT NULL,
        token_count INTEGER,
        is_summary BOOLEAN NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
    );

    CREATE INDEX idx_messages_session
        ON messages(session_id, created_at);
"#;

async fn ensure_schema(conn: &Connection) -> Result<(), StorageError> {
    conn.call(|conn| {
        conn.execute_batch(INIT_SCHEMA)?;

        let current: Option<i32> = conn.query_row(
            "SELECT version FROM schema_version LIMIT 1",
            [],
            |row| row.get(0)
        ).optional()?;

        if current.is_none() {
            conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                [SCHEMA_VERSION]
            )?;
        }

        Ok(())
    }).await
}
```

### Anti-Patterns to Avoid

- **Using rusqlite directly in async code:** Will block the executor. Always use tokio-rusqlite or spawn_blocking.
- **Creating new connections per query:** Connections are relatively expensive. Reuse via Clone (cheap) or connection pool.
- **Storing full conversation history in memory:** Load on-demand from database to avoid memory bloat for long sessions.
- **Over-aggressive summarization:** Don't summarize until actually needed (approaching token limit). Premature optimization loses context.
- **Using `.unwrap()` on database operations:** Database errors are recoverable. Use `?` and proper error handling.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Token counting | Character/word estimation | Provider APIs + tiktoken-rs | Different models tokenize differently. Estimation can be 20-50% off. Provider APIs are free and accurate. |
| XDG directory paths | Manual `~/.local/share` | `directories` crate | Already in workspace. Handles platform differences (macOS uses `~/Library/Application Support`, Windows uses `%APPDATA%`). |
| SQLite async wrapper | Manual spawn_blocking + channels | tokio-rusqlite | Already handles threading model, cleanup, and error propagation correctly. |
| Text summarization | Regex/truncation | LLM-based summarization | Simple truncation loses context coherence. LLM summarization preserves semantic meaning. |
| Transaction management | Manual BEGIN/COMMIT | rusqlite transaction API | Handles rollback on panic, prevents nested transaction bugs. |

**Key insight:** SQLite and async Rust have subtle edge cases. Use battle-tested libraries to avoid bugs around SQLITE_BUSY, connection lifecycle, and thread safety.

## Common Pitfalls

### Pitfall 1: SQLITE_BUSY Errors in Multi-Process Scenarios

**What goes wrong:** SQLite databases lock during writes. If another process holds the lock, operations fail with SQLITE_BUSY.

**Why it happens:** Default busy timeout is 0ms. Database doesn't wait for lock release.

**How to avoid:**

```rust
conn.call(|conn| {
    // Set 5 second busy timeout
    conn.busy_timeout(Duration::from_secs(5))?;
    Ok(())
}).await?;
```

**Warning signs:** Intermittent "database is locked" errors, especially when running multiple CLI instances simultaneously.

### Pitfall 2: Token Count Inaccuracy Across Providers

**What goes wrong:** Using tiktoken for Claude gives 20-30% inaccurate counts, causing premature or late summarization.

**Why it happens:** Each provider uses different tokenization (OpenAI uses tiktoken, Anthropic uses custom tokenizer).

**How to avoid:**

```rust
async fn count_tokens(
    provider: &str,
    messages: &[Message],
) -> Result<usize, Error> {
    match provider {
        "anthropic" => {
            // Use free token counting API
            let count = anthropic_client
                .count_tokens(messages)
                .await?;
            Ok(count.input_tokens)
        }
        "openai" => {
            // Use tiktoken-rs
            let encoding = tiktoken_rs::o200k_base()?;
            let text = messages_to_string(messages);
            Ok(encoding.encode_with_special_tokens(&text).len())
        }
        "ollama" => {
            // Local model, estimate conservatively
            // ~4 chars per token is safe approximation
            let chars: usize = messages.iter()
                .map(|m| m.content.len())
                .sum();
            Ok(chars / 4)
        }
        _ => Err(Error::UnsupportedProvider(provider.to_string())),
    }
}
```

**Warning signs:** Context summaries triggered too early or too late, unexpected token limit errors from provider APIs.

### Pitfall 3: Database File Permissions

**What goes wrong:** Database created with wrong permissions, readable by other users, potential security issue.

**Why it happens:** Default umask may create world-readable files.

**How to avoid:**

```rust
use std::fs;
use std::os::unix::fs::PermissionsExt;

async fn create_database(path: &Path) -> Result<Connection, StorageError> {
    let conn = Connection::open(path).await?;

    // Set 0600 (user read/write only)
    #[cfg(unix)]
    {
        let metadata = fs::metadata(path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(path, permissions)?;
    }

    Ok(conn)
}
```

**Warning signs:** Security audit tools flagging database file, conversations visible to other system users.

### Pitfall 4: Session ID Collisions with Timestamp-Based IDs

**What goes wrong:** Two sessions created in same second on same machine have identical IDs, causing data corruption.

**Why it happens:** Timestamp precision insufficient for collision resistance in rapid succession scenarios.

**How to avoid:**

```rust
use std::time::{SystemTime, UNIX_EPOCH};

fn generate_session_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();

    // Format: YYYY-MM-DD-HHMM-NANOS
    let dt = chrono::DateTime::from_timestamp(
        now.as_secs() as i64,
        now.subsec_nanos()
    ).unwrap();

    format!(
        "{}-{}",
        dt.format("%Y-%m-%d-%H%M"),
        now.subsec_nanos() / 1_000_000  // milliseconds for uniqueness
    )
}
```

**Warning signs:** Duplicate key errors on session creation, sessions mysteriously sharing history.

### Pitfall 5: Unbounded Context History Growth

**What goes wrong:** Long-running sessions accumulate thousands of messages, slowing down queries and API calls.

**Why it happens:** No automatic cleanup or archival strategy.

**How to avoid:**

```rust
// Implement 30-day expiration per requirements
async fn cleanup_old_sessions(conn: &Connection) -> Result<(), StorageError> {
    conn.call(|conn| {
        conn.execute(
            "DELETE FROM sessions
             WHERE last_message_at < datetime('now', '-30 days')",
            []
        )?;
        Ok(())
    }).await?;

    Ok(())
}

// Run on startup or periodically
```

**Warning signs:** Slow database queries over time, large database file size (>100MB per requirements), memory usage growing with session length.

## Code Examples

Verified patterns from official sources:

### Opening Database with Bundled SQLite

```rust
// Source: https://docs.rs/rusqlite/0.38.0/ + https://docs.rs/tokio-rusqlite/0.7.0/
use tokio_rusqlite::Connection;
use directories::ProjectDirs;

async fn open_database() -> Result<Connection, StorageError> {
    let proj_dirs = ProjectDirs::from("", "", "cherry2k")
        .ok_or(StorageError::NoHomeDir)?;

    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir)?;

    let db_path = data_dir.join("sessions.db");
    let conn = Connection::open(&db_path).await?;

    // Configure for robustness
    conn.call(|conn| {
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(())
    }).await?;

    Ok(conn)
}
```

### Saving Messages with Transaction

```rust
// Source: https://docs.rs/rusqlite/0.38.0/rusqlite/struct.Transaction.html
async fn save_message(
    conn: &Connection,
    session_id: &str,
    role: &str,
    content: &str,
    token_count: Option<usize>,
) -> Result<i64, StorageError> {
    let session_id = session_id.to_string();
    let role = role.to_string();
    let content = content.to_string();

    conn.call(move |conn| {
        let tx = conn.transaction()?;

        // Insert message
        tx.execute(
            "INSERT INTO messages (session_id, role, content, token_count)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![session_id, role, content, token_count],
        )?;

        let message_id = tx.last_insert_rowid();

        // Update session timestamp
        tx.execute(
            "UPDATE sessions
             SET last_message_at = datetime('now')
             WHERE id = ?1",
            [&session_id],
        )?;

        tx.commit()?;
        Ok(message_id)
    }).await
}
```

### Token Counting with Anthropic API

```rust
// Source: https://platform.claude.com/docs/en/build-with-claude/token-counting
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct TokenCountRequest {
    model: String,
    messages: Vec<ApiMessage>,
}

#[derive(Deserialize)]
struct TokenCountResponse {
    input_tokens: usize,
}

async fn count_anthropic_tokens(
    client: &reqwest::Client,
    api_key: &str,
    model: &str,
    messages: &[Message],
) -> Result<usize, ProviderError> {
    let api_messages: Vec<_> = messages.iter()
        .map(|m| ApiMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect();

    let response = client
        .post("https://api.anthropic.com/v1/messages/count_tokens")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&TokenCountRequest {
            model: model.to_string(),
            messages: api_messages,
        })
        .send()
        .await?;

    let count: TokenCountResponse = response.json().await?;
    Ok(count.input_tokens)
}
```

### Listing Sessions for Current Directory

```rust
// Source: Research synthesis
async fn list_sessions(
    conn: &Connection,
    cwd: &Path,
) -> Result<Vec<SessionInfo>, StorageError> {
    let cwd_str = cwd.to_string_lossy().to_string();

    conn.call(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT s.id, s.created_at, s.last_message_at,
                    (SELECT content FROM messages
                     WHERE session_id = s.id
                     ORDER BY created_at ASC LIMIT 1) as first_message
             FROM sessions s
             WHERE s.working_dir = ?1
             ORDER BY s.last_message_at DESC
             LIMIT 20"
        )?;

        let rows = stmt.query_map([&cwd_str], |row| {
            Ok(SessionInfo {
                id: row.get(0)?,
                created_at: row.get(1)?,
                last_message_at: row.get(2)?,
                first_message: row.get(3)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
    }).await
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Local tokenizers for all providers | Provider-specific token counting APIs | 2024-2025 | Anthropic removed local tokenizer for Claude 3+. Must use API for accuracy. OpenAI still supports tiktoken. |
| Simple message truncation | LLM-based summarization | 2024-2025 | Context windows expanded to 200K+ tokens, making intelligent summarization more valuable than truncation. |
| diesel/sqlx for simple apps | rusqlite + tokio-rusqlite | Ongoing | Diesel/sqlx are excellent for complex apps but overkill for single-user CLI tools. Community trend toward simpler stacks for small apps. |
| Manual spawn_blocking | Purpose-built async wrappers | 2023-2024 | tokio-rusqlite, async-sqlite emerged as battle-tested abstractions. Reduces boilerplate and bugs. |
| Character-based estimation | Hybrid: local + API | 2025 | Best practice: use local estimation for UX (instant feedback), verify with API for accuracy. |

**Deprecated/outdated:**

- **anthropic-tokenizer (pre-Claude 3):** Only works for legacy models. Claude 3+ requires API-based token counting or rough approximation.
- **sqlite with async-std:** Most Rust async ecosystem standardized on tokio. async-std-compatible wrappers exist but have smaller community.
- **VACUUM on every cleanup:** Modern SQLite with auto_vacuum=INCREMENTAL handles this automatically. Manual VACUUM rarely needed.

## Open Questions

Things that couldn't be fully resolved:

1. **Optimal summarization trigger point**
   - What we know: Should happen before hitting token limit (75% threshold common)
   - What's unclear: Cherry2K sets 16K token budget, but different providers have different limits (Claude 200K, GPT-4 128K). Should trigger be provider-specific or universal?
   - Recommendation: Start with universal 16K limit, make provider-specific in Phase 4 if needed. Simpler implementation for Phase 03.

2. **Summarization prompt design**
   - What we know: Prompt should emphasize preserving key context, decisions, and unresolved questions
   - What's unclear: Exact prompt engineering for best results varies by use case
   - Recommendation: Start with simple prompt (see example below), iterate based on user feedback. Document prompt in code for easy refinement.

```rust
// Recommended starting prompt for summarization
const SUMMARIZATION_PROMPT: &str = r#"
Summarize the following conversation history, preserving:
- Key facts and decisions made
- User's goals and preferences
- Unresolved questions or issues
- Technical context (file paths, commands, errors)

Be concise but preserve critical context. The summary will replace these messages in future conversations.

Conversation to summarize:
{conversation}

Provide only the summary, no preamble.
"#;
```

3. **Session cleanup timing**
   - What we know: 30-day expiration required, 100MB soft limit
   - What's unclear: Should cleanup run on every startup (simple but slower) or on-demand (complex but faster)?
   - Recommendation: Run on startup with quick check (SELECT COUNT, SELECT SUM). Only cleanup if needed. Startup impact minimal for typical database sizes.

4. **Cross-directory session discovery**
   - What we know: Requirements specify per-directory scope
   - What's unclear: Should `cherry2k resume --list` show all sessions (all directories) or only current directory?
   - Recommendation: Current directory only (matches requirements). Global list could be added later if users request it.

## Sources

### Primary (HIGH confidence)

- [tokio-rusqlite 0.7.0 documentation](https://docs.rs/tokio-rusqlite/0.7.0/) - API patterns and usage
- [rusqlite documentation](https://docs.rs/rusqlite/0.38.0/) - SQLite integration, transactions, features
- [Anthropic Token Counting API](https://platform.claude.com/docs/en/build-with-claude/token-counting) - Official API specification
- [tiktoken-rs repository](https://github.com/zurawiki/tiktoken-rs) - OpenAI token counting, version 0.9.1
- [SQLx repository](https://github.com/launchbadge/sqlx) - Alternative async SQLite approach
- [refinery repository](https://github.com/rust-db/refinery) - Migration toolkit, version 0.8.14
- [directories crate documentation](https://lib.rs/crates/directories) - XDG directory standards

### Secondary (MEDIUM confidence)

- [Context Window Management Strategies (getmaxim.ai)](https://www.getmaxim.ai/articles/context-window-management-strategies-for-long-context-ai-agents-and-chatbots) - Industry best practices, 2025
- [LLM Datasette Logging Schema](https://llm.datasette.io/en/stable/logging.html) - Production schema example
- [Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/) - CLI architecture patterns
- [Naurt Async SQLite article](https://www.naurt.com/blog-posts/naurt-async-sqlite-in-rust) - async patterns and tradeoffs
- [Token Counting Guide (Propel)](https://www.propelcode.ai/blog/token-counting-tiktoken-anthropic-gemini-guide-2025) - Multi-provider token counting, 2025
- [SQLite Best Practices (ProjectRules.AI)](https://www.projectrules.ai/rules/sqlite) - Performance and schema design
- [JetBrains Research: Context Management](https://blog.jetbrains.com/research/2025/12/efficient-context-management/) - Recent (Dec 2025) research on LLM context optimization

### Tertiary (LOW confidence)

- [Rust Users Forum: Using SQLite Asynchronously](https://users.rust-lang.org/t/using-sqlite-asynchronously/39658) - Community discussion, implementation patterns (dates vary, some old)
- [Medium: Unified Chat History System](https://medium.com/@mbonsign/unified-chat-history-and-logging-system-a-comprehensive-approach-to-ai-conversation-management-dc3b5d75499f) - Schema design concepts (no date, use with caution)
- [GeeksforGeeks: Database for Messaging Systems](https://www.geeksforgeeks.org/dbms/how-to-design-a-database-for-messaging-systems/) - General patterns (not Rust-specific)

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - All libraries have stable APIs, active maintenance, and clear use cases for this domain
- Architecture: HIGH - Patterns verified from official documentation and production examples (LLM Datasette)
- Pitfalls: MEDIUM-HIGH - Most based on documented issues (SQLITE_BUSY, token counting) and general SQLite knowledge. Specific edge cases may emerge during implementation.
- Token counting: MEDIUM - Provider APIs are authoritative (HIGH for Anthropic/OpenAI), but offline estimation strategies less standardized (MEDIUM)
- Context management: MEDIUM - Best practices established but exact implementation details (summarization prompts, trigger points) require experimentation

**Research date:** 2026-01-30
**Valid until:** 2026-03-30 (60 days - stable domain, but LLM APIs evolve rapidly)

**Notes:**

- tokio-rusqlite last release April 2024, but stable and sufficient for needs
- Anthropic tokenization policy changed significantly with Claude 3 (mid-2024), now settled
- Context window management best practices still evolving as models expand to 200K+ tokens
- SQLite and rusqlite are mature and stable (change risk: low)
- Most uncertainty is in LLM-specific aspects (token counting, summarization), not database layer
