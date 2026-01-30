# Phase 2: Single Provider End-to-End - Context

**Gathered:** 2026-01-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Prove the core AI interaction flow with OpenAI-compatible API. User runs `cherry2k chat "prompt"` and receives a streamed response. Response streams to terminal, API errors surface clearly, Ctrl+C cancels mid-stream. This phase establishes the provider abstraction that Phase 5 will extend to Anthropic and Ollama.

</domain>

<decisions>
## Implementation Decisions

### Streaming display
- Line-buffered output — buffer until newline, then print whole line (cleaner than character-by-character)
- Animated spinner/dots while waiting for first content to arrive
- Ctrl+C prompts "Cancel response? [y/n]" before stopping (not immediate)
- Clean separation — blank line before and after response for visual distinction

### Error presentation
- Errors displayed in boxed/framed visual block — distinct from normal output
- Rate limit errors show detailed info: remaining quota, reset time, and retry suggestion
- Invalid API key errors provide config guidance: "Set OPENAI_API_KEY or check ~/.config/cherry2k/config.toml"
- Network errors show user-friendly message by default + technical details in verbose/debug mode

### Provider trait design
- Streaming-only method — single `complete()` returning stream; non-streaming callers collect the stream
- Extension trait pattern for provider-specific features — base trait covers common denominator, optional extension traits (e.g., `AnthropicExtensions`) for provider-specific capabilities
- Explicit `validate_config()` method — constructor succeeds, caller decides when to validate (matches existing CLAUDE.md design)
- Include `health_check()` method — async ping to confirm provider is reachable (useful for Ollama)

### Output formatting
- Markdown rendering with config toggle — default rendered, `--plain` flag or config option for raw
- Syntax highlighting for code blocks — detect language and apply terminal colors
- Subtle icon prefix (e.g., ✨ or ▶) before response — light indicator, not verbose label
- Continuous flow for long responses — no pagination or breaks

### Claude's Discretion
- Exact spinner animation choice (dots, braille, etc.)
- Box/frame style for errors (unicode borders, ASCII, etc.)
- Which markdown elements to render (headers, bold, lists, etc.)
- Syntax highlighting library choice (syntect, etc.)

</decisions>

<specifics>
## Specific Ideas

- Streaming should feel responsive but not "flickery" — line buffering achieves this
- Error boxes should be visually distinct but not alarming for recoverable errors
- The `validate_config()` pattern allows CLI to validate early on startup while keeping trait flexible

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-single-provider-e2e*
*Context gathered: 2026-01-30*
