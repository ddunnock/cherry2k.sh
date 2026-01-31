# Phase 5: Multi-Provider Support - Research

**Researched:** 2026-01-31
**Domain:** Multi-provider AI API integration (Anthropic, Ollama)
**Confidence:** HIGH

## Summary

This phase extends the existing OpenAI provider implementation to support Anthropic Claude API and Ollama local models. The research reveals that all three providers use streaming APIs but with different formats: OpenAI and Anthropic use Server-Sent Events (SSE), while Ollama uses newline-delimited JSON (NDJSON). The existing provider trait and reqwest-eventsource foundation are well-suited for extension.

**Key findings:**
- Anthropic uses SSE with different event structure than OpenAI (content_block_delta vs choices)
- Ollama uses NDJSON streaming format, not SSE, requiring different parsing approach
- Provider factory pattern with HashMap<String, Box<dyn AiProvider>> enables dynamic provider switching
- Health check endpoints differ significantly: OpenAI uses /models, Anthropic uses /messages with minimal payload, Ollama uses /api/version or /api/tags
- Configuration already structured correctly with separate provider tables in TOML

**Primary recommendation:** Implement Anthropic provider using reqwest-eventsource (same as OpenAI), implement Ollama provider with line-by-line JSON parsing, create ProviderFactory with HashMap registry for runtime switching.

## Standard Stack

The established libraries/tools for multi-provider AI API integration:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | 0.12+ | HTTP client | Industry standard async HTTP client, used by existing OpenAI provider |
| reqwest-eventsource | 0.7+ | SSE streaming | Handles Server-Sent Events for OpenAI and Anthropic APIs |
| serde_json | 1.0+ | JSON parsing | Standard JSON parser, needed for all provider responses |
| async-stream | 0.3+ | Stream generation | Already used for OpenAI, enables ergonomic async stream creation |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| futures | 0.3+ | Stream utilities | Already in use, provides StreamExt for stream composition |
| tokio | 1.49+ | Async runtime | Already workspace dependency, handles async operations |
| toml | 0.9+ | Configuration | Already in use, supports nested provider tables |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Manual NDJSON parsing | ndjson-stream crate | Manual parsing is simpler for Ollama's format (one JSON per line), avoiding extra dependency |
| Dynamic dispatch with Box<dyn> | Enum-based provider | Box<dyn> enables runtime provider switching without rebuilding, enum requires compile-time knowledge |
| HashMap factory | match-based factory | HashMap enables dynamic provider registration, match requires code changes for new providers |

**Installation:**
```bash
# All core dependencies already in workspace
# No new crate dependencies required for Phase 5
```

## Architecture Patterns

### Recommended Project Structure
```
crates/core/src/provider/
├── mod.rs                 # Provider registry and factory
├── trait.rs               # AiProvider trait (existing)
├── types.rs               # Shared request/response types (existing)
├── sse.rs                 # SSE parsing utilities (existing)
├── openai.rs              # OpenAI implementation (existing)
├── anthropic.rs           # NEW: Anthropic implementation
└── ollama.rs              # NEW: Ollama implementation
```

### Pattern 1: Provider Factory with HashMap Registry
**What:** Dynamic provider lookup using string identifier
**When to use:** Runtime provider switching, user-configurable providers
**Example:**
```rust
// Source: Based on Rust factory pattern best practices
pub struct ProviderFactory {
    providers: HashMap<String, Box<dyn AiProvider>>,
}

impl ProviderFactory {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: String, provider: Box<dyn AiProvider>) {
        self.providers.insert(id, provider);
    }

    pub fn get(&self, id: &str) -> Option<&dyn AiProvider> {
        self.providers.get(id).map(|b| b.as_ref())
    }
}
```

### Pattern 2: Provider-Specific SSE Event Parsing
**What:** Each provider parses its own SSE event structure
**When to use:** SSE-based providers (OpenAI, Anthropic)
**Example:**
```rust
// Source: Anthropic streaming docs + existing OpenAI pattern
fn parse_anthropic_sse_chunk(data: &str) -> Option<String> {
    if data == "[DONE]" {
        return None;
    }

    let parsed: serde_json::Value = serde_json::from_str(data).ok()?;

    // Anthropic uses content_block_delta events
    if parsed["type"] == "content_block_delta" {
        return parsed["delta"]["text"].as_str().map(String::from);
    }

    None
}
```

### Pattern 3: NDJSON Streaming for Ollama
**What:** Parse newline-delimited JSON responses
**When to use:** Ollama API (uses NDJSON, not SSE)
**Example:**
```rust
// Source: Ollama API docs
use futures::stream::StreamExt;

async fn parse_ollama_stream(response: reqwest::Response) -> impl Stream<Item = Result<String, ProviderError>> {
    let stream = response.bytes_stream();

    try_stream! {
        let mut buffer = Vec::new();

        for await bytes in stream {
            let bytes = bytes?;
            buffer.extend_from_slice(&bytes);

            while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                let line = buffer.drain(..=newline_pos).collect::<Vec<_>>();
                let line_str = String::from_utf8_lossy(&line[..line.len()-1]);

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line_str) {
                    if let Some(text) = json["response"].as_str() {
                        if !text.is_empty() {
                            yield text.to_string();
                        }
                    }

                    // Check if done
                    if json["done"].as_bool() == Some(true) {
                        break;
                    }
                }
            }
        }
    }
}
```

### Pattern 4: Health Check with Provider-Specific Endpoints
**What:** Each provider validates connectivity using appropriate endpoint
**When to use:** Startup validation, provider availability checking
**Example:**
```rust
// Source: Anthropic API docs, Ollama API docs, existing OpenAI pattern
impl AiProvider for AnthropicProvider {
    async fn health_check(&self) -> Result<(), ProviderError> {
        // Anthropic: Use /v1/models endpoint
        let url = "https://api.anthropic.com/v1/models";
        let response = self.client.get(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await?;

        match response.status().as_u16() {
            200..=299 => Ok(()),
            401 => Err(ProviderError::InvalidApiKey { provider: "anthropic".into() }),
            _ => Err(ProviderError::Unavailable {
                provider: "anthropic".into(),
                reason: format!("Status {}", response.status())
            }),
        }
    }
}

impl AiProvider for OllamaProvider {
    async fn health_check(&self) -> Result<(), ProviderError> {
        // Ollama: Use /api/version endpoint
        let url = format!("{}/api/version", self.host);
        let response = self.client.get(&url).send().await
            .map_err(|e| {
                // Connection refused = Ollama not running
                if e.is_connect() {
                    ProviderError::Unavailable {
                        provider: "ollama".into(),
                        reason: "Ollama not running. Start with: ollama serve".into(),
                    }
                } else {
                    ProviderError::RequestFailed(e.to_string())
                }
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ProviderError::Unavailable {
                provider: "ollama".into(),
                reason: format!("Status {}", response.status()),
            })
        }
    }
}
```

### Anti-Patterns to Avoid
- **Single SSE parser for all providers:** OpenAI and Anthropic have different event structures; use provider-specific parsing
- **Treating Ollama like SSE:** Ollama uses NDJSON, not SSE; attempting to parse with reqwest-eventsource will fail
- **Hardcoded provider list:** Use HashMap factory to allow dynamic provider registration
- **Shared error messages:** Provider-specific error formatting enhances UX (e.g., "Ollama not running" vs generic "unavailable")

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SSE parsing | Custom event parser | reqwest-eventsource | Handles reconnection, Last-Event-ID, edge cases in SSE spec |
| HTTP client | Custom TCP + HTTP | reqwest | Connection pooling, timeouts, redirects, compression |
| Async streaming | Manual futures | async-stream crate | Clean syntax, error propagation, already in use |
| JSON streaming | Custom buffer management | Lines iterator + serde_json | NDJSON format is simple, custom parsing adds complexity |

**Key insight:** Streaming APIs have subtle edge cases (partial chunks, reconnection, errors mid-stream). Existing libraries handle these robustly.

## Common Pitfalls

### Pitfall 1: Assuming SSE Format is Universal
**What goes wrong:** Attempting to parse Ollama responses with reqwest-eventsource fails silently or with cryptic errors
**Why it happens:** Both OpenAI and Anthropic use SSE, easy to assume all streaming APIs do
**How to avoid:** Check API documentation for response format; Ollama explicitly uses "newline-delimited JSON"
**Warning signs:** Connection succeeds but no chunks are yielded, or parse errors in SSE library

### Pitfall 2: Not Handling Provider-Unavailable vs Invalid-Key
**What goes wrong:** User sees "invalid API key" when Ollama isn't running, or "unavailable" when key is wrong
**Why it happens:** Different HTTP status codes (401 vs connection refused), different error semantics per provider
**How to avoid:** Map connection errors (Ollama) to Unavailable with helpful message, map 401 to InvalidApiKey
**Warning signs:** User reports confusing error messages, tries to fix wrong thing

### Pitfall 3: Forgetting Anthropic Version Header
**What goes wrong:** Anthropic API returns 400 errors about missing anthropic-version header
**Why it happens:** Unlike OpenAI (just Authorization), Anthropic requires version header for API stability
**How to avoid:** Always include "anthropic-version: 2023-06-01" header in Anthropic requests
**Warning signs:** 400 errors from Anthropic API even with valid key

### Pitfall 4: Not Testing with Empty/Missing Provider Config
**What goes wrong:** Panic or unwrap errors when user hasn't configured a provider
**Why it happens:** Config uses Option<ProviderConfig>, easy to forget to handle None case
**How to avoid:** validate_config() should be called before registration; factory.get() returns Option
**Warning signs:** Crashes on startup when TOML doesn't have [providers.anthropic] section

### Pitfall 5: Rate Limit Header Parsing Differences
**What goes wrong:** retry_after_secs incorrect because header format differs between providers
**Why it happens:** OpenAI uses Retry-After (integer seconds), Anthropic may use different header names
**How to avoid:** Parse rate limit headers per-provider; default to 60 seconds if header missing/malformed
**Warning signs:** User retries too soon or waits unnecessarily long after rate limit

### Pitfall 6: Buffer Accumulation in NDJSON Parsing
**What goes wrong:** Memory grows unbounded when parsing Ollama responses
**Why it happens:** Buffer is filled but lines aren't drained, especially if no newlines found
**How to avoid:** Always drain buffer up to newline position; consider max buffer size (e.g., 1MB)
**Warning signs:** Memory usage grows during long streaming responses, OOM errors

## Code Examples

Verified patterns from official sources:

### Anthropic Streaming Request
```rust
// Source: https://platform.claude.com/docs/en/api/streaming
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt};

async fn anthropic_streaming_request(
    client: &Client,
    api_key: &str,
    messages: Vec<Message>,
) -> Result<EventSource, ProviderError> {
    let url = "https://api.anthropic.com/v1/messages";

    let body = serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 1024,
        "messages": messages,
        "stream": true,
    });

    let event_source = client.post(url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .eventsource()?;

    Ok(event_source)
}

// Parse Anthropic SSE events
fn parse_anthropic_event(data: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(data).ok()?;

    // Event types: message_start, content_block_start, content_block_delta,
    //              content_block_stop, message_delta, message_stop, ping
    match json["type"].as_str()? {
        "content_block_delta" => {
            // Extract text delta
            json["delta"]["text"].as_str().map(String::from)
        }
        "message_stop" => None,  // End of stream
        _ => None,  // Ignore other event types
    }
}
```

### Ollama Chat Streaming
```rust
// Source: https://github.com/ollama/ollama/blob/main/docs/api.md
use reqwest::Client;

async fn ollama_chat_stream(
    client: &Client,
    host: &str,
    model: &str,
    messages: Vec<Message>,
) -> Result<reqwest::Response, ProviderError> {
    let url = format!("{}/api/chat", host);

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
    });

    let response = client.post(&url)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(ProviderError::RequestFailed(
            format!("HTTP {}", response.status())
        ));
    }

    Ok(response)
}

// Parse NDJSON chunks
async fn parse_ollama_chunks(response: reqwest::Response) -> impl Stream<Item = Result<String, ProviderError>> {
    let mut lines = response.bytes_stream();
    let mut buffer = Vec::new();

    try_stream! {
        while let Some(chunk) = lines.next().await {
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);

            // Process complete lines
            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                let line = buffer.drain(..=pos).collect::<Vec<_>>();
                let text = String::from_utf8_lossy(&line[..line.len()-1]);

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    // Yield response chunk
                    if let Some(msg) = json["message"]["content"].as_str() {
                        if !msg.is_empty() {
                            yield msg.to_string();
                        }
                    }

                    // Check if stream is done
                    if json["done"].as_bool() == Some(true) {
                        break;
                    }
                }
            }
        }
    }
}
```

### Provider Factory Registration
```rust
// Source: Rust factory pattern best practices
use std::collections::HashMap;

pub struct ProviderFactory {
    providers: HashMap<String, Box<dyn AiProvider>>,
}

impl ProviderFactory {
    pub fn from_config(config: &Config) -> Result<Self, ConfigError> {
        let mut factory = Self::new();

        // Register OpenAI if configured
        if let Some(ref cfg) = config.openai {
            let provider = OpenAiProvider::new(cfg.clone());
            provider.validate_config()?;
            factory.register("openai".to_string(), Box::new(provider));
        }

        // Register Anthropic if configured
        if let Some(ref cfg) = config.anthropic {
            let provider = AnthropicProvider::new(cfg.clone());
            provider.validate_config()?;
            factory.register("anthropic".to_string(), Box::new(provider));
        }

        // Register Ollama if configured
        if let Some(ref cfg) = config.ollama {
            let provider = OllamaProvider::new(cfg.clone());
            provider.validate_config()?;
            factory.register("ollama".to_string(), Box::new(provider));
        }

        Ok(factory)
    }

    pub fn get(&self, id: &str) -> Option<&dyn AiProvider> {
        self.providers.get(id).map(|boxed| boxed.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }
}
```

### Model Listing Endpoints
```rust
// Source: Anthropic and Ollama API docs

// Anthropic: GET /v1/models
async fn anthropic_list_models(client: &Client, api_key: &str) -> Result<Vec<String>, ProviderError> {
    let response = client.get("https://api.anthropic.com/v1/models")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;
    let models = json["data"].as_array()
        .ok_or_else(|| ProviderError::ParseError("Expected array".into()))?
        .iter()
        .filter_map(|m| m["id"].as_str().map(String::from))
        .collect();

    Ok(models)
}

// Ollama: GET /api/tags
async fn ollama_list_models(client: &Client, host: &str) -> Result<Vec<String>, ProviderError> {
    let url = format!("{}/api/tags", host);
    let response = client.get(&url).send().await?;

    let json: serde_json::Value = response.json().await?;
    let models = json["models"].as_array()
        .ok_or_else(|| ProviderError::ParseError("Expected models array".into()))?
        .iter()
        .filter_map(|m| m["name"].as_str().map(String::from))
        .collect();

    Ok(models)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single provider in config | Multi-provider with [providers.X] tables | TOML best practices 2020+ | Clean separation of provider configs |
| Enum-based provider selection | HashMap factory with dynamic dispatch | Rust trait object patterns | Runtime provider switching without recompilation |
| Manual SSE parsing | reqwest-eventsource | Library stable 2022+ | Robust handling of SSE edge cases |
| Anthropic "claude-3" models | "claude-sonnet-4-5" naming | Nov 2024 Claude 4 release | Versioned model IDs with snapshot dates |
| Ollama "llama2" | "llama3.3" and "gemma3" models | 2024-2025 model releases | More capable open models available |

**Deprecated/outdated:**
- Claude 3 model IDs (claude-3-opus-20240229): Use Claude 4.5 series (claude-sonnet-4-5-20250929)
- #[async_trait] macro: Native async traits since Rust 1.75+ (already adopted in Phase 2)
- API version "2023-01-01" for Anthropic: Current version is "2023-06-01"

## Open Questions

Things that couldn't be fully resolved:

1. **Model name validation**
   - What we know: Anthropic has /v1/models endpoint, Ollama has /api/tags
   - What's unclear: Should we validate model names at config load or defer to API?
   - Recommendation: Validate at runtime (first completion request), store valid models from /models endpoint if needed

2. **Provider switching during active stream**
   - What we know: User context says "fresh start" when switching providers
   - What's unclear: Should in-progress stream be cancelled gracefully or immediately?
   - Recommendation: Immediate cancellation (drop stream), next message goes to new provider

3. **Retry-After header format consistency**
   - What we know: OpenAI uses integer seconds, Anthropic confirmed uses integer seconds
   - What's unclear: Does Ollama send rate limit headers?
   - Recommendation: Default to 60 seconds if header missing, test with actual rate limit responses

4. **Anthropic model name format stability**
   - What we know: Current format is "claude-sonnet-4-5-20250929" with snapshot date
   - What's unclear: Will config "claude-sonnet-4-5" resolve to latest snapshot?
   - Recommendation: Use full snapshot IDs in config for stability, allow shorthand in /model command

## Sources

### Primary (HIGH confidence)
- [Anthropic Streaming API Docs](https://platform.claude.com/docs/en/api/streaming) - SSE event structure, headers, endpoints
- [Anthropic API Overview](https://platform.claude.com/docs/en/api/overview) - Authentication, base URL, error codes
- [Ollama API Documentation](https://github.com/ollama/ollama/blob/main/docs/api.md) - NDJSON format, endpoints
- [Ollama List Models Endpoint](https://docs.ollama.com/api/tags) - /api/tags response structure
- Existing codebase: crates/core/src/provider/openai.rs, trait.rs - Proven patterns

### Secondary (MEDIUM confidence)
- [Anthropic Rate Limits Guide](https://www.aifreeapi.com/en/posts/claude-api-429-error-fix) - 429 error handling, retry-after
- [Ollama Connection Troubleshooting](https://markaicode.com/fix-ollama-api-connection-refused-error-troubleshooting/) - Error detection patterns
- [Anthropic Model Overview 2026](https://platform.claude.com/docs/en/about-claude/models/overview) - Current model naming
- [Ollama Models 2026](https://skywork.ai/blog/llm/ollama-models-list-2025-100-models-compared/) - Available models
- [Rust Factory Pattern](https://codesignal.com/learn/courses/creational-patterns-in-rust/lessons/factory-method-pattern-in-rust-a-guide-to-flexible-object-creation) - HashMap with trait objects
- [TOML Nested Tables](https://lib.rs/crates/toml) - Serde deserialization patterns

### Tertiary (LOW confidence)
- WebSearch results on SSE reconnection patterns - General guidance, needs official source verification
- WebSearch on NDJSON parsing libraries - Verified manual parsing is simpler for this use case

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All dependencies already in use, verified with existing OpenAI provider
- Architecture: HIGH - Anthropic and Ollama official docs provide complete API specifications
- Pitfalls: MEDIUM-HIGH - Based on official docs + common patterns, some edge cases not tested

**Research date:** 2026-01-31
**Valid until:** 2026-03-15 (45 days - APIs are relatively stable, model names may change)
