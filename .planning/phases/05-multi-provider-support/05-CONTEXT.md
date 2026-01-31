# Phase 5: Multi-Provider Support - Context

**Gathered:** 2026-01-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Support OpenAI, Anthropic, and Ollama with seamless switching. Users can configure multiple providers, switch between them in-session, and select models. This phase does NOT include intent detection or command execution (Phase 6).

</domain>

<decisions>
## Implementation Decisions

### Provider Switching UX
- Immediate switch when user runs `* /provider <name>` — brief confirmation message
- Fresh start on provider switch — new provider doesn't see previous messages (history stays in DB)
- `/providers` command shows all configured providers and which is active
- `/provider` (no args) shows current provider and model: "Currently using: anthropic (claude-sonnet-4-20250514)"

### Configuration Experience
- Separate TOML sections: `[providers.openai]`, `[providers.anthropic]`, `[providers.ollama]`
- Validate all configured API keys on startup (hit provider endpoints)
- Invalid key: warn and continue — "Warning: anthropic key invalid" but other providers remain usable
- Explicit default required: `default_provider = "anthropic"` in config

### Model Selection
- Per-provider model config: `[providers.anthropic] model = "claude-sonnet-4-20250514"`
- `/model <name>` command switches model within current provider
- `/models` command fetches and displays available models from provider API
- Ollama uses same commands — `/models` lists locally installed models

### Error & Fallback Behavior
- No automatic fallback — show error, user manually switches with `/provider`
- Rate limit errors show retry time: "Rate limited. Retry in 42s or /provider to switch"
- Ollama not running: clear error with hint "Ollama not running. Start with: ollama serve"
- Provider-specific error styling — each provider can format errors with its own look

### Claude's Discretion
- Exact API validation approach (which endpoint to hit)
- Error message wording details
- Internal provider factory architecture
- Streaming implementation details per provider

</decisions>

<specifics>
## Specific Ideas

- Keep switching fast — no unnecessary confirmation dialogs
- Model lists should be useful (show context window size if API provides it)
- Ollama experience should feel like cloud providers, not second-class

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-multi-provider-support*
*Context gathered: 2026-01-31*
