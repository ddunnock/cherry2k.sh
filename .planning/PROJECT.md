# Cherry2K

## What This Is

Cherry2K is a zsh-integrated AI assistant that brings Warp-like AI capabilities to any terminal. Users type `* ` followed by a question or request, and the AI responds inline - understanding whether they want command help, code written, or files edited. It's seamless AI assistance without leaving the terminal flow.

## Core Value

Seamless AI assistance without context switching. You stay in your terminal, in your flow, and get help exactly when you need it.

## Requirements

### Validated

**v1 — Terminal Integration:**
- ✓ `* ` prefix triggers AI from any terminal prompt — v1
- ✓ AI responds inline, returns user to prompt when done — v1
- ✓ Conversational context persists within session — v1
- ✓ Follow-up questions work without repeating context — v1

**v1 — Intent Detection:**
- ✓ AI distinguishes questions from command requests from coding tasks — v1
- ✓ Questions get explanations — v1
- ✓ Command requests get suggested commands — v1
- ✓ Coding requests trigger file operations — v1

**v1 — Command Execution:**
- ✓ Suggested commands show "Run this? [y/n]" confirmation — v1
- ✓ Confirmed commands execute in user's shell context — v1
- ✓ Command output visible to user — v1

**v1 — File Operations:**
- ✓ AI can read files in current directory and subdirectories — v1
- ✓ AI can write new files with diff preview and approval — v1
- ✓ AI can edit existing files with diff preview and approval — v1
- ✓ Configurable: safe mode (ask) vs auto-write mode — v1

**v1 — Multi-Provider Support:**
- ✓ OpenAI-compatible API support (configurable base URL for z.ai, etc.) — v1
- ✓ Anthropic Claude API support — v1
- ✓ Ollama local inference support — v1
- ✓ Provider switching via config or in-session command — v1

### Active

**TUI Mode (Optional v2):**
- [ ] `cherry2k` or `* /tui` opens full-screen interface
- [ ] Richer display for longer interactions
- [ ] Same capabilities as inline mode

### Out of Scope

- Full terminal replacement (like Warp itself) — this is a plugin, not a new terminal
- Agentic loops that run without user awareness — always show what's happening
- Browser or GUI interfaces — terminal-native only
- Multi-user or cloud sync — local-first, single-user

## Context

**Prior Art:**
- Warp terminal's AI features (the inspiration)
- Claude Code's file reading/writing patterns
- Shell completion systems (zsh's ZLE widgets)

**Technical Foundation:**
- Architecture documented in `.planning/codebase/ARCHITECTURE.md`
- Stack defined in `.planning/codebase/STACK.md`
- Quality standards in `.claude/standards/`

**Current State (v1 shipped):**
- 10,563 lines of Rust across 3 crates (core, storage, cli)
- 151 tests passing with zero clippy warnings
- 7 phases, 24 plans completed in 3 days
- Production-ready for zsh users

## Constraints

- **Language**: Rust 1.75+ — defined in CLAUDE.md, supports native async traits
- **Shell**: Zsh only for v1 — bash/fish possible later but not now
- **Persistence**: SQLite — simple, embedded, no external database needed
- **Security**: API keys in env vars or 0600 config file — never in code or logs
- **Quality**: 80% test coverage minimum — enforced via CI
- **Dependencies**: jq required for shell context JSON escaping

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| `* ` prefix for AI invocation | Single character + space is fast to type, unlikely to conflict | ✓ Good — intuitive and non-conflicting |
| Inline responses (not REPL mode) | Matches Warp's minimal-friction UX | ✓ Good — seamless terminal experience |
| OpenAI-compatible API as primary | Supports z.ai and other compatible services | ✓ Good — works with multiple providers |
| Configurable safety (diff preview) | Safe by default, power users can disable | ✓ Good — balances safety and flexibility |
| TUI as optional mode | Covers use cases where inline isn't enough | — Deferred to v2 |
| Provider-agnostic trait architecture | Enable easy addition of new providers | ✓ Good — 3 providers work seamlessly |
| BoxFuture for dyn-compatible traits | Enables Box<dyn AiProvider> in ProviderFactory | ✓ Good — solves async trait object limitation |
| SQLite with tokio-rusqlite | Session persistence without async starvation | ✓ Good — reliable and performant |
| Git-based project scope detection | Secure file operations bounded to git root | ✓ Good — prevents writes outside project |
| JSON temp file for shell context | Handles escaping and large history reliably | ✓ Good — robust context passing |

---
*Last updated: 2026-01-31 after v1 milestone*
