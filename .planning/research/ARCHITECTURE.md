# Architecture Patterns

**Domain:** Terminal AI Assistant with Shell Integration
**Researched:** 2026-01-29
**Overall Confidence:** HIGH

## Executive Summary

Terminal AI assistants with shell integration follow a well-established architectural pattern: a compiled binary handles AI communication and heavy lifting, while shell-native code (ZLE widgets, functions) provides the user-facing integration. This separation is critical for both performance and maintainability.

Research of existing tools (aichat, Claude Code, gptme, zsh_codex) reveals consistent patterns:

1. **Binary + Shell Script separation** - Shell scripts invoke a compiled binary
2. **Provider abstraction via traits** - Unified interface across OpenAI, Anthropic, Ollama
3. **SQLite for persistence** - Local-first conversation storage
4. **Async streaming** - Tokio-based streaming for responsive output
5. **Subprocess communication** - stdio-based IPC between shell and binary

## Recommended Architecture

```
+------------------+     stdin/stdout     +-------------------+
|   Zsh Session    | <------------------> |  cherry2k binary  |
|                  |                      |                   |
|  ZLE Widget      |   invokes binary,    |  CLI Parser       |
|  (Ctrl+G or * )  |   captures output    |  (clap)           |
|                  |                      |                   |
|  Shell Functions |                      |  Command Router   |
|  (cherry2k-*)    |                      |                   |
+------------------+                      +-------------------+
                                                   |
                                    +------+-------+-------+
                                    |      |               |
                              +-----v---+  |         +-----v-----+
                              |  Core   |  |         |  Storage  |
                              | Library |  |         |  (SQLite) |
                              +---------+  |         +-----------+
                                    |      |
                    +---------------+------+---------------+
                    |               |                      |
              +-----v-----+  +------v-----+  +-------------v+
              |  OpenAI   |  | Anthropic  |  |    Ollama    |
              |  Provider |  |  Provider  |  |   Provider   |
              +-----------+  +------------+  +--------------+
                    |               |                |
                    v               v                v
              [External APIs / Local Server]
```

### Component Boundaries

| Component                    | Responsibility                                     | Communicates With                        | Technology          |
|------------------------------|----------------------------------------------------|------------------------------------------|---------------------|
| **ZLE Widget**               | Capture `* ` prefix, invoke binary, display output | Shell environment, binary via subprocess | Pure zsh            |
| **Shell Functions**          | Helper commands (`cherry2k-config`, etc.)          | Binary via subprocess                    | Pure zsh            |
| **CLI Layer**                | Parse args, route commands, format output          | Core library, Storage                    | Rust (clap)         |
| **Core Library**             | Provider abstraction, conversation logic, config   | Providers, Storage types                 | Rust (async traits) |
| **Provider Implementations** | API-specific request/response handling             | External APIs via HTTP                   | Rust (reqwest)      |
| **Storage Layer**            | Conversation persistence, session management       | SQLite database                          | Rust (rusqlite)     |

### Critical Boundary: Shell vs Binary

**Shell layer (zsh/) is THIN:**
- Capture user input
- Invoke binary with arguments
- Display streamed output
- Handle shell-specific keybindings

**Binary layer (crates/) is THICK:**
- All AI communication
- All persistence
- All configuration parsing
- All business logic

This separation ensures:
- Shell startup remains fast (no heavy initialization)
- Binary can be tested independently
- Shell scripts remain portable and simple

## Data Flow Patterns

### Pattern 1: Inline AI Query (Primary Flow)

```
User types: "* how do I list files by size?"
    |
    v
[ZLE Widget intercepts input]
    |
    v
[Shell spawns: cherry2k chat --inline "how do I list files by size?"]
    |
    v
[CLI parses, loads config, selects provider]
    |
    v
[Core: build CompletionRequest with context]
    |
    v
[Provider: POST to API, return Stream<Chunk>]
    |
    v
[CLI: write chunks to stdout as received]
    |
    v
[Shell: ZLE displays output inline]
    |
    v
[Storage: persist exchange to SQLite]
    |
    v
[User returned to prompt]
```

**Key characteristics:**
- Synchronous from user perspective (they wait for response)
- Asynchronous internally (streaming chunks)
- Output written incrementally (no buffering entire response)
- Persistence happens after response complete

### Pattern 2: Session Continuity

```
First query: "* what is rustc?"
    |
    v
[Binary creates/loads session from SQLite]
[Session ID stored in shell environment: $CHERRY2K_SESSION]
    |
    v
[Response + session ID returned]
    |
    v
Follow-up: "* give me an example"
    |
    v
[Binary loads session by $CHERRY2K_SESSION]
[Previous context included in request]
    |
    v
[Coherent follow-up response]
```

**Implementation notes:**
- Session ID as environment variable enables cross-invocation continuity
- Each binary invocation is stateless; state lives in SQLite
- Session expiry/cleanup handled by storage layer

### Pattern 3: Command Execution Flow

```
User: "* list all rust files"
    |
    v
[AI responds: "ls *.rs" with intent: COMMAND]
    |
    v
[CLI outputs command + prompt: "Run this? [y/n]"]
    |
    v
[User confirms: y]
    |
    v
[Shell executes: ls *.rs in user's context]
    |
    v
[Output displayed normally]
```

**Critical security consideration:**
- Binary NEVER executes commands directly
- Binary outputs suggested command + confirmation prompt
- Shell layer handles actual execution (maintains user context, env vars)
- This separation prevents privilege escalation

### Pattern 4: File Operations Flow

```
User: "* create a rust hello world"
    |
    v
[AI generates file content with intent: FILE_WRITE]
    |
    v
[CLI outputs diff preview to stderr]
[CLI prompts: "Write this file? [y/n]"]
    |
    v
[User confirms: y]
    |
    v
[Binary writes file directly (has filesystem access)]
    |
    v
[Success message to stdout]
```

**File operations stay in binary because:**
- Diff generation requires file system access
- Atomic write operations are easier in Rust
- Error handling is more robust

## Provider Abstraction Pattern

### Trait Design (HIGH confidence - verified pattern)

Based on research of rust-genai, aichat, and RLLM, the standard pattern:

```rust
/// Core provider trait - enables seamless provider switching
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Send completion request, receive streaming response
    async fn complete(
        &self,
        request: CompletionRequest
    ) -> Result<CompletionStream, ProviderError>;

    /// Provider identifier for logging and config
    fn provider_id(&self) -> &'static str;

    /// Validate configuration before use
    fn validate_config(&self) -> Result<(), ConfigError>;

    /// List available models (optional)
    fn available_models(&self) -> Vec<ModelInfo> {
        vec![] // Default empty, providers can override
    }
}
```

### Provider Selection Pattern

```rust
/// Factory pattern for provider instantiation
pub fn create_provider(config: &Config) -> Result<Box<dyn AiProvider>, Error> {
    match config.provider.as_str() {
        "openai" => Ok(Box::new(OpenAiProvider::new(config)?)),
        "anthropic" => Ok(Box::new(AnthropicProvider::new(config)?)),
        "ollama" => Ok(Box::new(OllamaProvider::new(config)?)),
        // Model-name-based inference (aichat pattern)
        _ if config.model.starts_with("gpt") => {
            Ok(Box::new(OpenAiProvider::new(config)?))
        }
        _ if config.model.starts_with("claude") => {
            Ok(Box::new(AnthropicProvider::new(config)?))
        }
        _ => Ok(Box::new(OllamaProvider::new(config)?)) // Default to Ollama
    }
}
```

### Request/Response Normalization

Each provider translates to/from a common format:

```rust
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

pub struct Message {
    pub role: Role,  // System, User, Assistant
    pub content: String,
}

// Providers implement internal translation
impl OpenAiProvider {
    fn to_openai_format(&self, req: CompletionRequest) -> OpenAiRequest { ... }
    fn from_openai_chunk(&self, chunk: OpenAiChunk) -> ResponseChunk { ... }
}
```

## Storage Architecture

### Schema Design (HIGH confidence - Claude Code pattern verified)

```sql
-- Sessions: conversation containers
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    cwd TEXT,  -- Working directory for context
    metadata JSON  -- Extensible properties
);

-- Messages: individual conversation turns
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    role TEXT NOT NULL,  -- 'user', 'assistant', 'system'
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    token_count INTEGER,  -- For context window tracking
    metadata JSON
);

-- Index for efficient session loading
CREATE INDEX idx_messages_session ON messages(session_id, created_at);

-- Config persistence (optional)
CREATE TABLE config_overrides (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Connection Strategy

For CLI applications, research indicates:

**Single connection (recommended for Cherry2K):**
- CLI invocations are short-lived
- Connection pooling overhead not justified
- Simpler error handling

```rust
pub struct Repository {
    conn: Connection,  // rusqlite::Connection
}

impl Repository {
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        Ok(Self { conn })
    }
}
```

**Note on async:** rusqlite is synchronous. For async contexts, either:
1. Use `spawn_blocking` for database operations
2. Use `async-sqlite` or `sqlx` crate
3. Keep storage operations on dedicated thread

Given Cherry2K's CLI nature, synchronous rusqlite with `spawn_blocking` is simplest.

## Shell Integration Patterns

### ZLE Widget Architecture (HIGH confidence - verified from zsh docs)

```zsh
# Widget definition
function _cherry2k_complete() {
    local input="$BUFFER"

    # Check for trigger prefix
    if [[ "$input" == "* "* ]]; then
        local query="${input#\* }"

        # Clear line, show processing
        zle kill-whole-line
        echo -n "Thinking..."

        # Invoke binary, capture output
        local response
        response=$(cherry2k chat --inline "$query" 2>&1)

        # Display response
        echo "\r\033[K$response"

        # Restore prompt
        zle reset-prompt
    else
        # Pass through to normal behavior
        zle accept-line
    fi
}

# Register widget
zle -N _cherry2k_complete

# Bind to Enter (when buffer starts with "* ")
# Or bind to specific key like Ctrl+G
bindkey '^G' _cherry2k_complete
```

### Performance Considerations

Research shows shell startup performance is critical:

**DO:**
- Keep plugin file minimal (under 50 lines)
- Defer heavy operations to binary
- Use `autoload` for completion functions

**DON'T:**
- Source multiple files on startup
- Run any binary during shell init
- Define many functions inline

```zsh
# GOOD: Minimal plugin file
# cherry2k.plugin.zsh

# Define widget
function _cherry2k_complete() { ... }
zle -N _cherry2k_complete
bindkey '^G' _cherry2k_complete

# Lazy-load completions
fpath=("${0:A:h}/completions" $fpath)
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Heavy Shell Initialization

**What:** Loading conversation history, validating API keys, or running any binary during shell startup.

**Why bad:** Adds 100-300ms to every new shell, frustrates users.

**Instead:** All validation happens on first invocation, not on shell load.

### Anti-Pattern 2: Synchronous Blocking Without Feedback

**What:** Shell hangs with no output while waiting for AI response.

**Why bad:** Users think it's frozen, may Ctrl+C and retry.

**Instead:** Show "Thinking..." immediately, stream response as it arrives.

### Anti-Pattern 3: Binary Executes Shell Commands

**What:** Binary runs shell commands directly via `std::process::Command`.

**Why bad:**
- Loses user's shell context (env vars, aliases)
- Security risk (privilege escalation)
- Unexpected behavior

**Instead:** Binary outputs commands, shell layer executes them.

### Anti-Pattern 4: Monolithic Provider Implementation

**What:** Single file handling all providers with if/else branching.

**Why bad:** Hard to maintain, test, or add new providers.

**Instead:** Trait abstraction with separate files per provider.

### Anti-Pattern 5: Buffering Entire Response

**What:** Waiting for complete AI response before displaying.

**Why bad:**
- Wastes time (responses can take 5-30 seconds)
- Uses more memory
- Worse UX (no feedback during wait)

**Instead:** Stream chunks to terminal as they arrive.

### Anti-Pattern 6: Ephemeral Sessions Only

**What:** No persistence, context lost after each query.

**Why bad:** Users must repeat context, AI gives worse responses.

**Instead:** SQLite persistence with session continuity.

## Scalability Considerations

| Concern              | At Personal Use                     | At Power User                       | At Team Use         |
|----------------------|-------------------------------------|-------------------------------------|---------------------|
| Conversation History | Thousands of rows, no issue         | Tens of thousands, consider pruning | SQLite scales fine  |
| Context Window       | Track tokens, truncate old messages | Same + summarization                | Same                |
| Multiple Providers   | Runtime switching works             | Same                                | Config per user     |
| Concurrent Access    | N/A (single user)                   | Multiple terminals fine             | WAL mode for SQLite |

For Cherry2K's scope (personal terminal assistant), SQLite with default settings handles all realistic usage.

## Build Order Implications

Based on component dependencies:

### Phase 1: Core Infrastructure
1. **Error types** (no dependencies)
2. **Configuration loading** (depends on: error types)
3. **Provider trait definition** (depends on: error types)

### Phase 2: Single Provider Implementation
4. **OpenAI provider** (depends on: trait, config, errors)
5. **Basic CLI** (depends on: provider, config)
6. **Inline output** (depends on: CLI)

This gets a working `cherry2k chat "question"` command.

### Phase 3: Storage Layer
7. **SQLite schema + migrations** (no code dependencies)
8. **Repository implementation** (depends on: schema)
9. **Session management** (depends on: repository)

### Phase 4: Shell Integration
10. **ZLE widget** (depends on: working binary)
11. **Completions** (depends on: binary commands)

### Phase 5: Additional Providers
12. **Anthropic provider** (depends on: trait)
13. **Ollama provider** (depends on: trait)
14. **Provider factory** (depends on: all providers)

### Phase 6: Advanced Features
15. **TUI mode** (depends on: everything above)
16. **File operations** (depends on: core, storage)
17. **Command execution flow** (depends on: core, shell integration)

## Sources

**Architecture Patterns:**
- [Claude Code Session Management](https://deepwiki.com/anthropics/claude-code/3.3-session-and-conversation-management) - Session architecture patterns
- [Zed AI Agent System](https://deepwiki.com/zed-industries/zed/12-ai-agent-system) - Agent Communication Protocol (ACP) patterns
- [aichat GitHub](https://github.com/sigoden/aichat) - Rust terminal AI with multi-provider support

**Provider Abstraction:**
- [rust-genai](https://github.com/jeremychone/rust-genai) - Multi-provider Rust library
- [RLLM](https://github.com/graniet/rllm) - Unified ChatProvider/CompletionProvider traits
- [AxonerAI Medium article](https://medium.com/@mnjkshrm/building-axonerai-a-rust-framework-for-agentic-systems-cea8e8d73ba0) - Trait-based provider design

**Shell Integration:**
- [zsh_codex](https://github.com/tom-doerr/zsh_codex) - ZLE widget for AI completion
- [Oh My Zsh Async Rendering](https://deepwiki.com/ohmyzsh/ohmyzsh/6.3-asynchronous-prompt-rendering) - Subprocess management
- [Zsh Line Editor Documentation](https://zsh.sourceforge.io/Doc/Release/Zsh-Line-Editor.html) - Official ZLE reference

**Storage Patterns:**
- [OpenAI Agents SDK SQLite Sessions](https://openai.github.io/openai-agents-python/sessions/advanced_sqlite_session/) - Session schema patterns
- [Claude Code DeepWiki](https://deepwiki.com/anthropics-claude/claude-code/2.3-session-management) - SQLite session structure

**Async/Streaming:**
- [Tokio Streams Tutorial](https://tokio.rs/tokio/tutorial/streams) - Async streaming patterns
- [Tokio I/O Tutorial](https://tokio.rs/tokio/tutorial/io) - AsyncRead/AsyncWrite

**Anti-Patterns:**
- [AI Coding Anti-Patterns](https://dev.to/lingodotdev/ai-coding-anti-patterns-6-things-to-avoid-for-better-ai-coding-f3e) - Common mistakes
- [LLM Gateway Architecture](https://www.truefoundry.com/blog/llm-gateway) - Abstraction layer patterns

---

*Architecture research: 2026-01-29*