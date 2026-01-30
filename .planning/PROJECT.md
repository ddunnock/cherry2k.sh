# Cherry2K

## What This Is

Cherry2K is a zsh-integrated AI assistant that brings Warp-like AI capabilities to any terminal. Users type `* ` followed by a question or request, and the AI responds inline - understanding whether they want command help, code written, or files edited. It's seamless AI assistance without leaving the terminal flow.

## Core Value

Seamless AI assistance without context switching. You stay in your terminal, in your flow, and get help exactly when you need it.

## Requirements

### Validated

(None yet — ship to validate)

### Active

**Terminal Integration:**
- [ ] `* ` prefix triggers AI from any terminal prompt
- [ ] AI responds inline, returns user to prompt when done
- [ ] Conversational context persists within session
- [ ] Follow-up questions work without repeating context

**Intent Detection:**
- [ ] AI distinguishes questions from command requests from coding tasks
- [ ] Questions get explanations
- [ ] Command requests get suggested commands
- [ ] Coding requests trigger file operations

**Command Execution:**
- [ ] Suggested commands show "Run this? [y/n]" confirmation
- [ ] Confirmed commands execute in user's shell context
- [ ] Command output visible to user

**File Operations:**
- [ ] AI can read files in current directory and subdirectories
- [ ] AI can write new files with diff preview and approval
- [ ] AI can edit existing files with diff preview and approval
- [ ] Configurable: safe mode (ask) vs auto-write mode

**Multi-Provider Support:**
- [ ] OpenAI-compatible API support (configurable base URL for z.ai, etc.)
- [ ] Anthropic Claude API support
- [ ] Ollama local inference support
- [ ] Provider switching via config or in-session command

**TUI Mode (Optional):**
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

**Current State:**
- Project structure defined, no implementation exists
- Standards and conventions established
- Provider abstraction designed but not built

## Constraints

- **Language**: Rust 1.75+ — defined in CLAUDE.md, supports native async traits
- **Shell**: Zsh only for v1 — bash/fish possible later but not now
- **Persistence**: SQLite — simple, embedded, no external database needed
- **Security**: API keys in env vars or 0600 config file — never in code or logs
- **Quality**: 80% test coverage minimum — enforced via CI

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| `* ` prefix for AI invocation | Single character + space is fast to type, unlikely to conflict | — Pending |
| Inline responses (not REPL mode) | Matches Warp's minimal-friction UX | — Pending |
| OpenAI-compatible API as primary | Supports z.ai and other compatible services | — Pending |
| Configurable safety (diff preview) | Safe by default, power users can disable | — Pending |
| TUI as optional mode | Covers use cases where inline isn't enough | — Pending |

---
*Last updated: 2026-01-29 after initialization*