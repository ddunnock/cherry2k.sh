# Project Research Summary

**Project:** Cherry2K - Zsh Terminal AI Assistant
**Domain:** Terminal AI Assistant with Shell Integration
**Researched:** 2026-01-29
**Confidence:** HIGH

## Executive Summary

Cherry2K is a zsh-integrated AI assistant that brings Warp-like AI capabilities to any terminal. Research across 2026 industry sources reveals a mature ecosystem with clear technology choices and well-documented pitfalls. The Rust async ecosystem has settled on Tokio + reqwest as the standard stack, rusqlite handles SQLite persistence, and ratatui is the de facto TUI library. The recommended approach is to build a lightweight provider abstraction directly on reqwest rather than adopting evolving AI client libraries — the APIs are simple REST endpoints, and direct implementation gives full control over streaming behavior and error handling.

The key differentiation opportunity lies in Cherry2K's **inline zsh integration approach** (responses appear in-flow with normal terminal usage) combined with **multi-provider flexibility** (OpenAI, Anthropic, Ollama) and **intent-aware responses** (automatically detecting whether user wants a command, an answer, or a file modification). Competitors like Copilot CLI require context-switching to separate prompts; Cherry2K's `* ` prefix and ZLE widget integration maintain terminal flow.

The critical risks are security-related: AI-suggested commands must never auto-execute without sandboxing, and ZLE widget state management has subtle foot-guns that cause terminal corruption. The research strongly recommends: (1) default to confirmation-before-execution, (2) design the provider abstraction with all three providers in mind from day one, and (3) use tokio-rusqlite or spawn_blocking for SQLite operations to avoid async runtime starvation.

## Key Findings

### Recommended Stack

The Rust ecosystem has clear winners in every category. Use Tokio 1.49+ as the async runtime (async-std was discontinued March 2025), reqwest 0.13 for HTTP with native SSE streaming support, rusqlite 0.38 with the bundled feature for zero runtime SQLite dependencies, and clap 4.5 for CLI parsing. For error handling, thiserror 2.0 in library crates and anyhow 2.0 in the binary crate.

**Core technologies:**
- **Tokio 1.49**: Async runtime — de facto standard, LTS releases, required by reqwest and most ecosystem crates
- **reqwest 0.13**: HTTP client — built-in SSE streaming via `bytes_stream()`, JSON support, rustls TLS
- **rusqlite 0.38**: SQLite persistence — bundles SQLite 3.51.1, no runtime dependencies, 40M+ downloads
- **clap 4.5**: CLI framework — derive macros, shell completion generation, subcommand support
- **thiserror 2.0 / anyhow 2.0**: Error handling — typed errors in library code, contextual errors in binary

**Build your own provider abstraction.** Do not use rust-genai or async-openai. The AI APIs are simple REST + SSE endpoints (~100 lines per provider), and direct implementation avoids dependency on rapidly-evolving AI libraries while giving full control.

### Expected Features

**Must have (table stakes):**
- Natural language to command — the core value proposition
- Command explanation — low complexity, high value
- Confirmation before execution — non-negotiable safety requirement (LITL attack research)
- Streaming responses — expected UX, all modern AI tools stream
- Conversation context — enables follow-up questions within session
- Basic ZLE integration — the `* ` prefix and Ctrl+G shortcut

**Should have (differentiators):**
- Intent detection (question vs command vs file task) — eliminates mode-switching friction
- Inline responses (not separate REPL) — maintains terminal flow, key differentiator vs Copilot CLI
- Local-first with Ollama — addresses 2026's loudest concerns: privacy and cost
- Error explanation — proactive analysis when commands fail

**Defer (v2+):**
- Multi-file awareness — complex indexing, high token cost
- Autonomous/agent mode — safety concerns, not MVP priority
- MCP integration — ecosystem still maturing
- TUI mode — can start CLI-only
- Shell command suggestions (real-time) — performance complexity

### Architecture Approach

The architecture follows the established pattern: a compiled Rust binary handles AI communication and heavy lifting, while pure zsh scripts (ZLE widgets, shell functions) provide user-facing integration. This separation is critical — shell startup remains fast (no binary execution during init), the binary can be tested independently, and shell scripts stay portable. The binary communicates with shell via stdin/stdout (subprocess communication), never executing shell commands directly (prevents privilege escalation).

**Major components:**
1. **ZLE Widget Layer** (zsh/) — captures `* ` prefix, invokes binary, displays streamed output
2. **CLI Layer** (crates/cli) — parses args, routes commands, formats terminal output
3. **Core Library** (crates/core) — provider abstraction, conversation logic, configuration
4. **Provider Implementations** (crates/core/provider/) — API-specific request/response handling
5. **Storage Layer** (crates/storage) — SQLite persistence for conversations and sessions

### Critical Pitfalls

1. **Auto-execute without sandboxing** — AI-suggested commands must never run without confirmation. Implement OS-level sandboxing (macOS seatbelt), default to ask-mode, maintain a dangerous command blocklist. This is Phase 1 foundational work.

2. **ZLE widget state corruption** — Calling `zle` commands when ZLE is not active (in subshells, async callbacks) causes "widgets can only be called when ZLE is active" errors and terminal corruption. Check ZLE active state before any `zle` call, never call ZLE from subshells.

3. **Blocking async runtime with sync code** — rusqlite operations on tokio worker threads cause starvation under load. Use tokio-rusqlite (dedicated thread per connection) or spawn_blocking. This is a production-only bug that tests miss.

4. **Provider abstraction leakage** — Designing abstraction based on one provider (usually OpenAI) leads to provider-specific conditionals scattered throughout. Design the trait with OpenAI, Anthropic, AND Ollama docs open simultaneously.

5. **SQLite busy/lock errors** — Without busy_timeout, concurrent writes fail immediately. Always set `PRAGMA busy_timeout=5000`, use `BEGIN IMMEDIATE` for write transactions, keep transactions short.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Foundation and Safety

**Rationale:** Security and terminal integration must be foundational, not bolted on. Research shows teams that defer security architecture face major rework.

**Delivers:** Working CLI skeleton with safe command execution flow
- Error types (thiserror)
- Configuration loading (toml + directories crate)
- Provider trait definition (designed for all 3 providers)
- Basic CLI structure (clap)
- Confirmation-before-execution flow

**Addresses features:** Confirmation before execution (table stakes)
**Avoids pitfalls:** Auto-execute without sandboxing (#1)

### Phase 2: Single Provider Working End-to-End

**Rationale:** Prove the core flow works before adding complexity. OpenAI-compatible API (covers OpenAI and many local models) is best first target.

**Delivers:** User can type `cherry2k chat "question"` and get streamed response
- OpenAI provider implementation
- Streaming response handling (reqwest bytes_stream)
- Terminal output formatting

**Addresses features:** Natural language processing, streaming responses
**Uses stack:** reqwest 0.13 (stream feature), serde_json

### Phase 3: Storage and Session Continuity

**Rationale:** Persistence enables conversation context, which users expect for follow-ups. Must address async/SQLite pitfalls before multi-provider complexity.

**Delivers:** Conversation history persists, sessions resume across invocations
- SQLite schema and migrations
- Repository implementation (tokio-rusqlite or spawn_blocking)
- Session management with $CHERRY2K_SESSION env var
- Context window management (token tracking, sliding window)

**Addresses features:** Conversation context, session persistence
**Avoids pitfalls:** Async blocking (#4), SQLite busy errors (#6), Context explosion (#9)

### Phase 4: Zsh Integration

**Rationale:** The inline experience is Cherry2K's key differentiator. With core flow working, integrate into zsh.

**Delivers:** User can type `* question` in zsh and see inline response
- ZLE widget (captures `* ` prefix)
- Shell functions (cherry2k-config, etc.)
- Keybinding setup (Ctrl+G)
- Completions

**Addresses features:** Inline responses, tab/keybinding integration
**Avoids pitfalls:** ZLE state corruption (#3), CUTBUFFER persistence (#11)

### Phase 5: Multi-Provider Support

**Rationale:** Provider abstraction must be validated against real API differences. Anthropic has different message sequencing, Ollama uses NDJSON not SSE.

**Delivers:** User can switch between OpenAI, Anthropic, and Ollama
- Anthropic provider implementation
- Ollama provider implementation
- Provider factory (model-name-based inference)
- Streaming interruption handling

**Addresses features:** Cross-platform model support, local-first (Ollama)
**Avoids pitfalls:** Provider abstraction leakage (#5), streaming interruption (#7)

### Phase 6: Command Execution Flow

**Rationale:** With safety architecture in place and providers working, add the "generate and run commands" feature.

**Delivers:** AI suggests commands, user confirms, command executes in shell context
- Intent detection (question vs command vs file)
- Command output to shell (binary outputs, shell executes)
- Error explanation (capture stderr, send to LLM)

**Addresses features:** Natural language to command, command explanation, error explanation
**Implements:** Architecture pattern 3 (command execution flow)

### Phase 7: File Operations (Optional/Stretch)

**Rationale:** File operations add significant complexity and security surface. Defer if time-constrained.

**Delivers:** AI can create/modify files with diff preview
- Diff preview before apply
- Path validation and scope enforcement
- File write with user confirmation

**Addresses features:** File diff preview (differentiator)
**Avoids pitfalls:** File operation scope creep (#8), silent failures (#2)

### Phase Ordering Rationale

- **Safety first (Phase 1):** Research documents real incidents of AI tools deleting files, exfiltrating credentials. Sandboxing architecture must be foundational.
- **Vertical slice (Phase 2):** Prove entire flow works with one provider before horizontal expansion.
- **Storage before shell (Phase 3 before 4):** Session continuity requires persistence. Shell integration depends on working binary.
- **Multi-provider after core (Phase 5):** Provider abstraction design informed by actual implementation experience with OpenAI.
- **Execution last (Phase 6-7):** Most complex features with highest security stakes built on solid foundation.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 5 (Multi-Provider):** Needs API-level research on Anthropic vs Ollama message formats, streaming protocols, error codes. Research recommends reading all three provider docs before trait design.
- **Phase 4 (Zsh Integration):** ZLE widget edge cases may need deeper research during implementation. Review zsh-autosuggestions and powerlevel10k issues.

Phases with standard patterns (skip research-phase):
- **Phase 2 (Single Provider):** Well-documented OpenAI API, many Rust examples available.
- **Phase 3 (Storage):** Standard SQLite patterns, tokio-rusqlite has good docs.
- **Phase 6 (Command Execution):** Pattern documented in architecture research.

## Confidence Assessment

| Area         | Confidence  | Notes                                                                                     |
|--------------|-------------|-------------------------------------------------------------------------------------------|
| Stack        | HIGH        | Verified against crates.io, docs.rs, multiple 2026 guides. Clear ecosystem winners.       |
| Features     | HIGH        | Cross-referenced Warp, Copilot CLI, Claude Code, and 2026 reviews. Consistent findings.   |
| Architecture | HIGH        | Patterns verified against aichat, Claude Code, zsh_codex. Well-established.               |
| Pitfalls     | HIGH        | Documented real incidents, official docs, issue trackers. Critical pitfalls well-sourced. |

**Overall confidence:** HIGH

### Gaps to Address

- **Ollama streaming format:** Research sources conflict on NDJSON vs SSE. Verify against actual Ollama 0.5.x during Phase 5.
- **reqwest 0.13 breaking changes:** Verify `query` and `form` features are explicitly added if used. May affect API call construction.
- **TUI as separate binary vs feature flag:** Decision deferred. Consider `cherry2k-tui` as optional binary to reduce compile time if TUI not needed. Address during Phase 7 planning if TUI added.
- **macOS seatbelt vs Linux bubblewrap:** Sandboxing implementation is OS-specific. Needs platform-specific research during Phase 1 detailed planning.

## Sources

### Primary (HIGH confidence)
- [crates.io](https://crates.io) — Verified tokio 1.49, reqwest 0.13, rusqlite 0.38, clap 4.5.54
- [docs.rs](https://docs.rs) — API documentation for all crates
- [Warp All Features](https://www.warp.dev/all-features) — Feature landscape verification
- [GitHub Copilot CLI Docs](https://docs.github.com/en/copilot/concepts/agents/about-copilot-cli) — Competitor analysis
- [Zsh Line Editor Documentation](https://zsh.sourceforge.io/Doc/Release/Zsh-Line-Editor.html) — ZLE widget patterns
- [tokio-rusqlite](https://docs.rs/tokio-rusqlite/latest/tokio_rusqlite/) — Async SQLite patterns

### Secondary (MEDIUM confidence)
- [aichat GitHub](https://github.com/sigoden/aichat) — Architecture patterns for Rust terminal AI
- [rust-genai](https://github.com/jeremychone/rust-genai) — Provider abstraction patterns
- [Claude Code DeepWiki](https://deepwiki.com/anthropics/claude-code) — Session management patterns
- [powerlevel10k issues](https://github.com/romkatv/powerlevel10k/issues/631) — ZLE async pitfalls

### Tertiary (LOW confidence)
- [thiserror/anyhow 2.0](https://dev.to/leapcell/rust-error-handling-compared-anyhow-vs-thiserror-vs-snafu-2003) — 2025 guide, verify against crates.io
- [IEEE Spectrum AI Coding](https://spectrum.ieee.org/ai-coding-degrades) — Silent failure patterns

---
*Research completed: 2026-01-29*
*Ready for roadmap: yes*