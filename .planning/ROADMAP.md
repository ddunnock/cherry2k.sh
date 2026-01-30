# Roadmap: Cherry2K

## Overview

Cherry2K brings Warp-like AI assistance to any terminal through seamless zsh integration. The roadmap progresses from a safe foundation through single-provider proof-of-concept, storage for session continuity, zsh integration for the inline experience, multi-provider flexibility, command execution flow, and finally file operations. Each phase delivers a coherent, verifiable capability while building toward the full vision of AI assistance without context switching.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation and Safety** - CLI skeleton with safe command execution architecture
- [ ] **Phase 2: Single Provider End-to-End** - OpenAI streaming responses working via CLI
- [ ] **Phase 3: Storage and Session Continuity** - Conversation persistence and session resume
- [ ] **Phase 4: Zsh Integration** - The `* ` prefix inline experience
- [ ] **Phase 5: Multi-Provider Support** - Anthropic, Ollama, and provider switching
- [ ] **Phase 6: Command Execution Flow** - Intent detection and shell command execution
- [ ] **Phase 7: File Operations** - File reading, writing, and editing with diff preview

## Phase Details

### Phase 1: Foundation and Safety
**Goal**: Establish CLI skeleton with security-first command execution architecture
**Depends on**: Nothing (first phase)
**Requirements**: CMD-01
**Success Criteria** (what must be TRUE):
  1. User can run `cherry2k --help` and see available commands
  2. User can run `cherry2k --version` and see version info
  3. Configuration loads from `~/.config/cherry2k/config.toml` or env vars
  4. Command confirmation flow exists (scaffolded for later use)
  5. Error types provide clear, actionable messages
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md - Workspace structure and error types
- [x] 01-02-PLAN.md - Configuration loading with env var overrides
- [x] 01-03-PLAN.md - CLI skeleton with clap and confirmation flow

### Phase 2: Single Provider End-to-End
**Goal**: Prove the core AI interaction flow with OpenAI-compatible API
**Depends on**: Phase 1
**Requirements**: PROV-01
**Success Criteria** (what must be TRUE):
  1. User can run `cherry2k chat "What is Rust?"` and receive a streamed response
  2. Response streams to terminal character-by-character (not buffered)
  3. API errors surface as clear error messages (rate limit, invalid key, network)
  4. User can cancel mid-stream with Ctrl+C
**Plans**: TBD

Plans:
- [ ] 02-01: Provider trait design (OpenAI, Anthropic, Ollama in mind)
- [ ] 02-02: OpenAI provider implementation with streaming
- [ ] 02-03: Terminal output formatting and stream display

### Phase 3: Storage and Session Continuity
**Goal**: Enable conversation context that persists across invocations
**Depends on**: Phase 2
**Requirements**: TERM-03, TERM-04
**Success Criteria** (what must be TRUE):
  1. User can have a multi-turn conversation with context retained
  2. Conversation persists after terminal closes
  3. User can resume a previous session with `cherry2k resume`
  4. Context window managed (old messages summarized or pruned)
**Plans**: TBD

Plans:
- [ ] 03-01: SQLite schema and migrations
- [ ] 03-02: Repository layer with async-safe access
- [ ] 03-03: Session management and context window handling

### Phase 4: Zsh Integration
**Goal**: Deliver the inline `* ` prefix experience that defines Cherry2K
**Depends on**: Phase 3
**Requirements**: TERM-01, TERM-02
**Success Criteria** (what must be TRUE):
  1. User can type `* what is my IP` and see inline response
  2. Response appears in-terminal, not in separate REPL
  3. User returns to normal prompt after response completes
  4. Ctrl+G keybinding triggers AI mode from anywhere in command line
  5. Tab completion works for cherry2k commands
**Plans**: TBD

Plans:
- [ ] 04-01: ZLE widget for `* ` prefix capture
- [ ] 04-02: Shell functions and plugin file
- [ ] 04-03: Keybindings and completions

### Phase 5: Multi-Provider Support
**Goal**: Support OpenAI, Anthropic, and Ollama with seamless switching
**Depends on**: Phase 4
**Requirements**: PROV-02, PROV-03, PROV-04
**Success Criteria** (what must be TRUE):
  1. User can configure Anthropic API and get responses
  2. User can configure Ollama and get local model responses
  3. User can switch providers via config file
  4. User can switch providers in-session with `* /provider anthropic`
  5. Streaming works consistently across all providers
**Plans**: TBD

Plans:
- [ ] 05-01: Anthropic provider implementation
- [ ] 05-02: Ollama provider implementation
- [ ] 05-03: Provider factory and in-session switching

### Phase 6: Command Execution Flow
**Goal**: Enable AI to suggest commands that execute in user's shell
**Depends on**: Phase 5
**Requirements**: INTENT-01, INTENT-02, INTENT-03, CMD-02, CMD-03
**Success Criteria** (what must be TRUE):
  1. AI distinguishes questions from command requests
  2. Questions receive explanatory answers
  3. Command requests show suggested command with "Run this? [y/n]"
  4. Confirmed commands execute in shell context (not subprocess)
  5. Command output is visible to user
  6. Failed commands show error with option for AI explanation
**Plans**: TBD

Plans:
- [ ] 06-01: Intent detection logic
- [ ] 06-02: Command suggestion and confirmation flow
- [ ] 06-03: Shell execution and error handling

### Phase 7: File Operations
**Goal**: Enable AI to read, write, and edit files with user approval
**Depends on**: Phase 6
**Requirements**: INTENT-04, FILE-01, FILE-02, FILE-03, FILE-04
**Success Criteria** (what must be TRUE):
  1. AI can read files when user references them or current directory
  2. AI can propose new file creation with diff preview
  3. AI can propose file edits with diff preview
  4. User must approve file changes before write (safe mode default)
  5. Power users can enable auto-write mode via config
  6. File operations respect directory scope (no writes outside project)
**Plans**: TBD

Plans:
- [ ] 07-01: File reading and context injection
- [ ] 07-02: Diff preview and file write flow
- [ ] 07-03: Safety controls and scope enforcement

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation and Safety | 3/3 | ✓ Complete | 2026-01-30 |
| 2. Single Provider End-to-End | 0/3 | Not started | - |
| 3. Storage and Session Continuity | 0/3 | Not started | - |
| 4. Zsh Integration | 0/3 | Not started | - |
| 5. Multi-Provider Support | 0/3 | Not started | - |
| 6. Command Execution Flow | 0/3 | Not started | - |
| 7. File Operations | 0/3 | Not started | - |

---

## Requirement Coverage

| Requirement | Phase | Description |
|-------------|-------|-------------|
| CMD-01 | Phase 1 | Confirmation flow architecture |
| PROV-01 | Phase 2 | OpenAI-compatible API support |
| TERM-03 | Phase 3 | Conversational context persists |
| TERM-04 | Phase 3 | Follow-up questions work |
| TERM-01 | Phase 4 | `* ` prefix triggers AI |
| TERM-02 | Phase 4 | Inline response, returns to prompt |
| PROV-02 | Phase 5 | Anthropic Claude API support |
| PROV-03 | Phase 5 | Ollama local inference support |
| PROV-04 | Phase 5 | Provider switching |
| INTENT-01 | Phase 6 | Intent detection |
| INTENT-02 | Phase 6 | Questions get explanations |
| INTENT-03 | Phase 6 | Commands get suggestions |
| CMD-02 | Phase 6 | Commands execute in shell |
| CMD-03 | Phase 6 | Command output visible |
| INTENT-04 | Phase 7 | Coding triggers file ops |
| FILE-01 | Phase 7 | File reading |
| FILE-02 | Phase 7 | File creation with preview |
| FILE-03 | Phase 7 | File editing with preview |
| FILE-04 | Phase 7 | Configurable safety mode |

**Coverage:** 19/19 v1 requirements mapped

**Deferred to v2:**
- TUI-01, TUI-02, TUI-03: TUI mode (marked optional in PROJECT.md)

---
*Last updated: 2026-01-30*
