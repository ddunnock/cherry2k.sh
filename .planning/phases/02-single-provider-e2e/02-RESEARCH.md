# Phase 2: Single Provider End-to-End - Research

**Researched:** 2026-01-30
**Domain:** Rust HTTP streaming, terminal formatting, async patterns
**Confidence:** HIGH

## Summary

This phase implements streaming OpenAI API integration with terminal display. The research confirms the standard Rust stack for this domain is well-established and mature as of 2026:

**Core finding:** Build directly on `reqwest 0.12` + `eventsource-stream` for SSE parsing, avoiding heavyweight client libraries like `async-openai` to maintain control over the provider abstraction. Use `termimad` for markdown rendering, `indicatif` for spinners, and `tokio::signal::ctrl_c()` for graceful cancellation. The Rust 2024 edition (Edition 2024) enables native async traits without `async-trait` crate.

**Architecture decision validated:** The provider trait design from CONTEXT.md (streaming-only `complete()` method, explicit `validate_config()`, extension traits for provider-specific features) aligns perfectly with Rust 1.75+ native async trait capabilities and standard Stream patterns.

**Key implementation pattern:** Line-buffered output via collecting chunks until newline matches the CONTEXT.md decision and avoids terminal flicker. SSE parsing should use `eventsource-stream` crate to handle the `data:` prefix protocol correctly. OpenAI's `[DONE]` message is a special SSE event that signals stream completion.

**Primary recommendation:** Use `reqwest-eventsource` (built on `eventsource-stream`) for SSE handling with automatic retry support. Implement provider trait returning `Pin<Box<dyn Stream<Item = Result<String, ProviderError>> + Send>>` using `async-stream::stream!` macro for clean yield-based stream construction.

## Standard Stack

The established libraries/tools for streaming HTTP + terminal UI in Rust:

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.12.26 | HTTP client with streaming | De facto standard, maintained by seanmonstar, tokio integration, 0.12+ has mature streaming API |
| reqwest-eventsource | 0.6.0 | SSE wrapper for reqwest | Built on eventsource-stream, handles retries, clean Stream interface |
| eventsource-stream | 0.2.x | SSE protocol parser | Low-level SSE parsing, used by reqwest-eventsource |
| tokio | 1.49.0 | Async runtime | Required by reqwest, project already uses 1.49 |
| tokio-stream | 0.1.x | Stream utilities | Official tokio stream extensions (StreamExt trait) |
| async-stream | 0.3.x | Stream construction macro | Provides `stream!` and `try_stream!` macros for yield-based streams |

### Terminal Output

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| termimad | 0.34.1 | Terminal markdown rendering | Main markdown display, supports styling, wrapping, minimal dependencies |
| syntect | 5.x | Syntax highlighting | Code block highlighting (used by termimad/mdcat for language detection) |
| indicatif | 0.18.3 | Progress/spinner | Waiting spinner before first content arrives, thread-safe for async |
| cli-boxes | 1.x | Unicode box drawing | Error display frames, 9 predefined styles (SINGLE, DOUBLE, ROUND, etc.) |
| colored | 2.x | ANSI terminal colors | Error highlighting, respects NO_COLOR env var |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| futures | 0.3.x | Stream traits + utilities | Stream combinators (map, filter, etc.), project already depends on it |
| tokio-util | 0.7.x | CancellationToken | Graceful shutdown coordination for Ctrl+C handling |
| serde_json | 1.0.149 | JSON parsing | OpenAI API request/response (project already has) |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| reqwest-eventsource | async-openai (full client) | async-openai provides complete OpenAI SDK but locks us into their abstractions; we need provider-agnostic trait |
| termimad | mdcat (CLI tool) | mdcat is a binary, not a library; termimad gives more control over formatting |
| indicatif | Custom spinner | Indicatif is battle-tested, thread-safe, well-maintained; no reason to build custom |
| native async traits | async-trait crate | Rust 1.75+ has native async trait support; async-trait only needed for <1.75 compatibility |

**Installation:**

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "stream"] }
reqwest-eventsource = "0.6"
tokio = { version = "1.49", features = ["full"] }
tokio-stream = "0.1"
tokio-util = { version = "0.7", features = ["rt"] }
async-stream = "0.3"
futures = "0.3"
termimad = "0.34"
indicatif = "0.18"
cli-boxes = "1"
colored = "2"
serde_json = "1.0"
```

## Architecture Patterns

### Recommended Project Structure

```
crates/core/src/
├── provider/
│   ├── mod.rs              # ProviderFactory, re-exports
│   ├── trait.rs            # AiProvider trait + CompletionStream type alias
│   ├── openai.rs           # OpenAiProvider implementation
│   ├── types.rs            # CompletionRequest, CompletionResponse
│   └── sse.rs              # SSE parsing utilities (OpenAI format)
crates/cli/src/
├── commands/
│   └── chat.rs             # `cherry2k chat` command
├── output/
│   ├── mod.rs
│   ├── markdown.rs         # Markdown rendering via termimad
│   ├── spinner.rs          # Indicatif spinner wrapper
│   ├── stream_writer.rs    # Line-buffered stream display
│   └── error_box.rs        # Boxed error display
└── signal.rs               # Ctrl+C handling
```

### Pattern 1: Provider Trait with Native Async

**What:** Provider abstraction returning streaming responses
**When to use:** All AI provider implementations
**Example:**

```rust
// Source: Rust 1.75+ native async traits
use futures::Stream;
use std::pin::Pin;

pub type CompletionStream = Pin<Box<dyn Stream<Item = Result<String, ProviderError>> + Send>>;

pub trait AiProvider: Send + Sync {
    /// Stream completion chunks. Non-streaming callers collect the stream.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionStream, ProviderError>;

    /// Provider identifier for logging/config
    fn provider_id(&self) -> &'static str;

    /// Validate config (constructor succeeds, caller decides when to validate)
    fn validate_config(&self) -> Result<(), ConfigError>;

    /// Health check - async ping to confirm reachable
    async fn health_check(&self) -> Result<(), ProviderError>;
}
```

**Rationale:** Matches CONTEXT.md decisions exactly. Rust 1.75+ native async traits avoid `async-trait` macro overhead. Streaming-only method keeps API surface small.

### Pattern 2: SSE Streaming with reqwest-eventsource

**What:** Parse OpenAI SSE format with automatic retries
**When to use:** OpenAI provider implementation
**Example:**

```rust
// Source: https://docs.rs/reqwest-eventsource/
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt};
use async_stream::stream;

impl OpenAiProvider {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionStream, ProviderError> {
        let mut es = self.client
            .post(&self.base_url)
            .json(&req.to_openai_format())
            .eventsource()?;

        let stream = stream! {
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => continue,
                    Ok(Event::Message(msg)) => {
                        if msg.data == "[DONE]" {
                            break;
                        }
                        // Parse JSON chunk
                        let chunk: OpenAIChunk = serde_json::from_str(&msg.data)?;
                        if let Some(content) = chunk.choices[0].delta.content {
                            yield Ok(content);
                        }
                    }
                    Err(e) => yield Err(ProviderError::from(e)),
                }
            }
        };

        Ok(Box::pin(stream))
    }
}
```

**Rationale:** `reqwest-eventsource` handles SSE protocol details (`data:` prefix, retries). OpenAI sends `data: [DONE]` as completion signal. `async-stream::stream!` provides clean yield syntax.

### Pattern 3: Line-Buffered Terminal Output

**What:** Buffer streaming text until newline, then print whole line
**When to use:** Streaming response display (matches CONTEXT.md decision)
**Example:**

```rust
// Source: Rust stdout behavior (line-buffered by default)
use std::io::{self, Write};

pub struct StreamWriter {
    buffer: String,
    stdout: io::Stdout,
}

impl StreamWriter {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            stdout: io::stdout(),
        }
    }

    pub fn write_chunk(&mut self, chunk: &str) -> io::Result<()> {
        self.buffer.push_str(chunk);

        // Print complete lines, keep remainder in buffer
        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = &self.buffer[..=newline_pos];
            self.stdout.write_all(line.as_bytes())?;
            self.stdout.flush()?;
            self.buffer.drain(..=newline_pos);
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            self.stdout.write_all(self.buffer.as_bytes())?;
            self.buffer.clear();
        }
        self.stdout.flush()
    }
}
```

**Rationale:** Matches CONTEXT.md line-buffering decision. Prevents character-by-character flicker. Manual flush handles incomplete lines at stream end.

### Pattern 4: Graceful Cancellation with tokio::select!

**What:** Ctrl+C prompts confirmation before stopping stream
**When to use:** Long-running streaming operations
**Example:**

```rust
// Source: https://tokio.rs/tokio/topics/shutdown
use tokio::signal;
use tokio_util::sync::CancellationToken;

pub async fn run_chat(provider: &dyn AiProvider, prompt: &str) -> Result<(), Error> {
    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    // Spawn Ctrl+C handler
    tokio::spawn(async move {
        signal::ctrl_c().await.ok();
        eprintln!("\n\nCancel response? [y/n]: ");
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() && input.trim() == "y" {
            cancel_clone.cancel();
        }
    });

    let mut stream = provider.complete(request).await?;
    let mut writer = StreamWriter::new();

    loop {
        tokio::select! {
            chunk = stream.next() => {
                match chunk {
                    Some(Ok(text)) => writer.write_chunk(&text)?,
                    Some(Err(e)) => return Err(e),
                    None => break,
                }
            }
            _ = cancel_token.cancelled() => {
                eprintln!("\nCancelled by user");
                break;
            }
        }
    }

    writer.flush()?;
    Ok(())
}
```

**Rationale:** Matches CONTEXT.md requirement for confirmation before cancel (not immediate). `tokio::select!` races stream chunks vs cancellation. `CancellationToken` provides clean async coordination.

### Pattern 5: Boxed Error Display

**What:** Visually distinct error frames in terminal
**When to use:** All error output to user
**Example:**

```rust
// Source: https://docs.rs/cli-boxes/
use cli_boxes::{BoxChars, BorderStyle};
use colored::Colorize;

pub fn display_error(error: &dyn std::error::Error) {
    let box_chars = BorderStyle::DOUBLE.chars();
    let message = format_error_message(error);
    let width = 60;

    // Top border
    println!("{}", format!("{}{}{}",
        box_chars.top_left,
        box_chars.top.to_string().repeat(width),
        box_chars.top_right
    ).red());

    // Error content (wrapped)
    for line in message.lines() {
        println!("{} {:<width$} {}",
            box_chars.left.to_string().red(),
            line,
            box_chars.right.to_string().red(),
            width = width - 2
        );
    }

    // Bottom border
    println!("{}", format!("{}{}{}",
        box_chars.bottom_left,
        box_chars.bottom.to_string().repeat(width),
        box_chars.bottom_right
    ).red());
}

fn format_error_message(error: &dyn std::error::Error) -> String {
    match error.downcast_ref::<ProviderError>() {
        Some(ProviderError::RateLimited { provider, retry_after_secs }) => {
            format!(
                "Rate Limited by {}\n\n\
                 Retry after: {} seconds\n\
                 Suggestion: Wait and try again, or use a different provider",
                provider, retry_after_secs
            )
        }
        Some(ProviderError::InvalidApiKey { provider }) => {
            format!(
                "Invalid API Key for {}\n\n\
                 Set OPENAI_API_KEY environment variable or check:\n\
                 ~/.config/cherry2k/config.toml",
                provider
            )
        }
        _ => error.to_string(),
    }
}
```

**Rationale:** Matches CONTEXT.md boxed error requirement. Different error types get custom messages with actionable guidance. `cli-boxes` provides Unicode borders, `colored` adds red highlight.

### Anti-Patterns to Avoid

- **Don't use async-trait crate**: Rust 1.75+ has native support, macro overhead unnecessary
- **Don't buffer entire response**: Stream incrementally for responsiveness
- **Don't use character-by-character output**: Line buffering prevents flicker (per CONTEXT.md)
- **Don't skip API key validation**: Call `validate_config()` early in CLI startup
- **Don't ignore SSE protocol details**: OpenAI uses `data:` prefix, `[DONE]` message - use proper parser
- **Don't block async with stdin**: Ctrl+C handling must spawn separate task for stdin read

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SSE parsing | Manual `data:` line parsing | `reqwest-eventsource` or `eventsource-stream` | SSE has retry protocol, event IDs, connection management - easy to get wrong |
| Markdown rendering | Regex-based text transformation | `termimad` | Terminal width wrapping, style preservation, nested structures (lists, code blocks) complex |
| Syntax highlighting | Language detection + ANSI codes | `syntect` (via termimad) | 100+ languages, theme support, proper token parsing |
| Progress spinners | Custom animation loops | `indicatif` | Thread-safe, multiple concurrent spinners, automatic cleanup |
| Terminal color detection | Manual $TERM parsing | `colored` crate | Handles NO_COLOR, CLICOLOR_FORCE, terminal capability detection |
| Stream combinators | Manual async loop logic | `tokio-stream::StreamExt` | Correct Pin handling, fused streams, error propagation |

**Key insight:** Streaming, terminal formatting, and signal handling have subtle edge cases. Use battle-tested crates that handle these properly. The Rust ecosystem has mature solutions for all these problems.

## Common Pitfalls

### Pitfall 1: Stream Pinning Forgotten

**What goes wrong:** Compilation error `stream does not implement Unpin` when using `.next().await`
**Why it happens:** Streams must be pinned before iteration because async state machines are self-referential
**How to avoid:** Use `tokio::pin!(stream)` macro before iteration:

```rust
let stream = provider.complete(request).await?;
tokio::pin!(stream);
while let Some(chunk) = stream.next().await { /* ... */ }
```

**Warning signs:** Compiler error mentioning "Unpin" or "Pin<&mut T>"

### Pitfall 2: SSE Event Parsing Errors Ignored

**What goes wrong:** Stream stops silently on malformed JSON chunks
**Why it happens:** OpenAI occasionally sends empty delta content or non-standard events
**How to avoid:** Handle `serde_json` parse errors gracefully, log and skip bad chunks:

```rust
match serde_json::from_str::<OpenAIChunk>(&msg.data) {
    Ok(chunk) => {
        if let Some(content) = chunk.choices.get(0).and_then(|c| c.delta.content.as_ref()) {
            yield Ok(content.clone());
        }
    }
    Err(e) => {
        tracing::warn!("Failed to parse SSE chunk: {}", e);
        continue; // Skip malformed chunk, don't break stream
    }
}
```

**Warning signs:** Stream stops after first few chunks, no error reported

### Pitfall 3: Stdout Buffering Not Flushed

**What goes wrong:** Last line of output doesn't appear until program exits
**Why it happens:** Line-buffering only flushes on `\n`, final chunk may not have newline
**How to avoid:** Always flush after stream completes:

```rust
while let Some(chunk) = stream.next().await {
    writer.write_chunk(&chunk?)?;
}
writer.flush()?; // CRITICAL: flush remaining buffer
```

**Warning signs:** Output appears incomplete, final sentence missing

### Pitfall 4: Blocking stdin in Async Context

**What goes wrong:** Program hangs when reading Ctrl+C confirmation from stdin
**Why it happens:** `std::io::stdin().read_line()` is blocking, pauses entire async runtime
**How to avoid:** Spawn separate blocking task for stdin:

```rust
tokio::task::spawn_blocking(|| {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    input.trim() == "y"
}).await
```

**Warning signs:** Terminal freezes after Ctrl+C, no response to input

### Pitfall 5: reqwest Runtime Panic

**What goes wrong:** `reqwest::Error` with message "not currently running on the Tokio runtime"
**Why it happens:** reqwest requires Tokio 1.x runtime, fails if used outside
**How to avoid:** Ensure `#[tokio::main]` on binary entry point, don't use `futures::block_on`
**Warning signs:** Panic on first HTTP request, mentions runtime context

### Pitfall 6: API Key in Error Messages

**What goes wrong:** API key leaked in logs or terminal output
**Why it happens:** Error context includes full request headers
**How to avoid:** Strip sensitive headers before error conversion:

```rust
// BAD: includes Authorization header
Err(reqwest_error) => Err(ProviderError::from(reqwest_error))

// GOOD: sanitize first
Err(e) => {
    if e.is_status() && e.status() == Some(StatusCode::UNAUTHORIZED) {
        Err(ProviderError::InvalidApiKey { provider: "openai" })
    } else {
        Err(ProviderError::Network(sanitize_error(e)))
    }
}
```

**Warning signs:** API keys visible in error output, security scanner alerts

## Code Examples

Verified patterns from official sources:

### Creating Async Stream with Yields

```rust
// Source: https://docs.rs/async-stream/
use async_stream::try_stream;
use futures::Stream;

fn openai_stream(response: Response) -> impl Stream<Item = Result<String, ProviderError>> {
    try_stream! {
        let mut es = EventSource::new(response)?;

        while let Some(event) = es.next().await {
            match event? {
                Event::Message(msg) => {
                    if msg.data == "[DONE]" {
                        break;
                    }
                    let chunk: OpenAIChunk = serde_json::from_str(&msg.data)?;
                    if let Some(content) = chunk.choices[0].delta.content.as_ref() {
                        yield content.clone();
                    }
                }
                Event::Open => continue,
            }
        }
    }
}
```

### Terminal Markdown Rendering

```rust
// Source: https://docs.rs/termimad/
use termimad::{MadSkin, StyledChar};

pub fn render_markdown(text: &str, plain: bool) -> String {
    if plain {
        return text.to_string();
    }

    let mut skin = MadSkin::default();
    skin.bold.set_fg(termimad::crossterm::style::Color::Yellow);
    skin.italic.set_fg(termimad::crossterm::style::Color::Cyan);
    skin.inline_code.set_fg(termimad::crossterm::style::Color::Green);

    skin.term_text(text).to_string()
}
```

### Spinner During Stream Wait

```rust
// Source: https://docs.rs/indicatif/
use indicatif::ProgressBar;
use std::time::Duration;

pub async fn stream_with_spinner(
    provider: &dyn AiProvider,
    request: CompletionRequest,
) -> Result<(), Error> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("✨ Waiting for response...");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let mut stream = provider.complete(request).await?;

    // Stop spinner on first chunk
    let mut first_chunk = true;
    while let Some(chunk) = stream.next().await {
        if first_chunk {
            spinner.finish_and_clear();
            first_chunk = false;
        }
        print!("{}", chunk?);
    }

    Ok(())
}
```

### OpenAI Base URL Configuration

```rust
// Source: OpenAI-compatible provider patterns
pub struct OpenAiConfig {
    pub api_key: String,
    pub base_url: String, // Default: "https://api.openai.com/v1"
    pub model: String,
}

impl OpenAiConfig {
    pub fn for_provider(provider: &str) -> Result<Self, ConfigError> {
        let base_url = match provider {
            "openai" => "https://api.openai.com/v1",
            "groq" => "https://api.groq.com/openai/v1",
            "z.ai" => "https://api.z.ai/api/coding/paas/v4",
            _ => return Err(ConfigError::InvalidProvider(provider.into())),
        };

        Ok(Self {
            api_key: std::env::var(format!("{}_API_KEY", provider.to_uppercase()))?,
            base_url: base_url.into(),
            model: "gpt-4".into(), // Default, can override
        })
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| async-trait crate | Native async traits | Rust 1.75 (Dec 2023) | No macro overhead, better error messages, RPITIT support |
| futures::StreamExt | tokio-stream::StreamExt | Tokio 1.0+ | Better tokio integration, simpler imports |
| Manual SSE parsing | reqwest-eventsource | 2024 | Handles retries, event IDs, reconnection automatically |
| tui-rs | Ratatui | 2023 | Active maintenance, but NOT needed for this phase (simple output) |
| ansi_term | colored | 2020+ | NO_COLOR support, better maintained |

**Deprecated/outdated:**
- **async-trait macro**: Native support in Rust 1.75+ eliminates need
- **Manual `impl Future` for streams**: Use `async-stream::stream!` macro instead
- **futures::io for stdin**: Use `tokio::io` or `spawn_blocking` for std::io
- **Custom SSE parsers**: Use `eventsource-stream` or `reqwest-eventsource`

## Open Questions

Things that couldn't be fully resolved:

1. **OpenAI SSE chunk schema stability**
   - What we know: Current format is `{"choices": [{"delta": {"content": "..."}}]}`
   - What's unclear: Does OpenAI guarantee this schema won't change? Should we version detection?
   - Recommendation: Parse defensively (`.get(0).and_then(...)`), log schema changes, test with mock server

2. **Terminal width detection for markdown wrapping**
   - What we know: `termimad` uses `crossterm::terminal::size()` for width detection
   - What's unclear: How to handle width changes mid-stream (terminal resize)?
   - Recommendation: Detect width once at stream start, don't try to handle resize (complexity not worth it for Phase 2)

3. **Ctrl+C signal handling on Windows**
   - What we know: `tokio::signal::ctrl_c()` works on Unix and Windows
   - What's unclear: Does prompt-before-cancel pattern work well on Windows terminals?
   - Recommendation: Test on Windows, may need platform-specific handling (defer to Phase 8 if needed)

## Sources

### Primary (HIGH confidence)

- **Tokio Official Docs** - [Streams Tutorial](https://tokio.rs/tokio/tutorial/streams) - Stream patterns, pinning
- **Tokio Official Docs** - [Graceful Shutdown](https://tokio.rs/tokio/topics/shutdown) - Ctrl+C handling
- **reqwest-eventsource** - [docs.rs](https://docs.rs/reqwest-eventsource/) - API examples, version 0.6.0
- **async-stream** - [docs.rs](https://docs.rs/async-stream) - `stream!` and `try_stream!` macros
- **termimad** - [docs.rs](https://docs.rs/termimad) - Markdown rendering API, version 0.34.1
- **indicatif** - [docs.rs](https://docs.rs/indicatif) - Spinner and progress bar API, version 0.18.3
- **cli-boxes** - [docs.rs](https://docs.rs/cli-boxes/) - Box drawing characters
- **Rust Book** - [Async Traits](https://doc.rust-lang.org/book/ch17-05-traits-for-async.html) - Native async trait patterns

### Secondary (MEDIUM confidence)

- **GitHub: reqwest** - [seanmonstar/reqwest](https://github.com/seanmonstar/reqwest) - Version 0.12.26, tokio compatibility
- **GitHub: async-stream** - [tokio-rs/async-stream](https://github.com/tokio-rs/async-stream) - Stream macro examples
- **GitHub: termimad** - [Canop/termimad](https://github.com/Canop/termimad) - Terminal markdown rendering
- **OpenAI API Docs** - [Chat Streaming](https://platform.openai.com/docs/api-reference/chat-streaming) - SSE format (403 on fetch, but search results confirmed)
- **WebSearch** - [Groq API Base URL](https://community.groq.com/t/what-is-the-base-url-path-for-groq-api/487) - `https://api.groq.com/openai/v1`
- **WebSearch** - [Z.AI OpenAI compatibility](https://docs.z.ai/devpack/tool/others) - `https://api.z.ai/api/coding/paas/v4`

### Tertiary (LOW confidence)

- **WebSearch** - OpenAI streaming format details from community sources (not official docs)
- **WebSearch** - SSE pitfalls from Rocket/Axum discussions (server-side, but hints at client pitfalls)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries confirmed via official docs, versions verified
- Architecture: HIGH - Patterns validated against official Tokio/Rust documentation and CONTEXT.md decisions
- Pitfalls: MEDIUM - Based on GitHub issues and community reports, not all personally verified
- OpenAI specifics: MEDIUM - Official docs inaccessible (403), but community sources consistent

**Research date:** 2026-01-30
**Valid until:** ~2026-03-30 (30 days for stable Rust ecosystem, libraries mature)

**Research constraints from CONTEXT.md:**
- Line-buffered output (not character-by-character) ✓ Confirmed as best practice
- Streaming-only provider method ✓ Aligns with async Stream patterns
- Extension trait pattern ✓ Validated as idiomatic Rust
- Explicit `validate_config()` method ✓ Common in Rust config patterns
- Markdown rendering with toggle ✓ termimad + `--plain` flag pattern
- Boxed error display ✓ cli-boxes provides clean implementation

**Decisions locked by CONTEXT.md:**
- All streaming display decisions (spinner, line buffering, Ctrl+C confirmation)
- Provider trait design (streaming-only, extension traits, validate_config, health_check)
- Error presentation (boxed frames, detailed rate limit info, config guidance)
- Output formatting (markdown with toggle, syntax highlighting, icon prefix, continuous flow)

**Research focused on:** HOW to implement these decisions with standard Rust libraries, not WHETHER to implement them.
