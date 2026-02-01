# Requirements: Cherry2K

## v1 Requirements

### Terminal Integration

| ID | Requirement | Priority |
|----|-------------|----------|
| TERM-01 | `* ` prefix triggers AI from any terminal prompt | Must |
| TERM-02 | AI responds inline, returns user to prompt when done | Must |
| TERM-03 | Conversational context persists within session | Must |
| TERM-04 | Follow-up questions work without repeating context | Must |

### Intent Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| INTENT-01 | AI distinguishes questions from command requests from coding tasks | Must |
| INTENT-02 | Questions get explanations | Must |
| INTENT-03 | Command requests get suggested commands | Must |
| INTENT-04 | Coding requests trigger file operations | Must |

### Command Execution

| ID | Requirement | Priority |
|----|-------------|----------|
| CMD-01 | Suggested commands show "Run this? [y/n]" confirmation | Must |
| CMD-02 | Confirmed commands execute in user's shell context | Must |
| CMD-03 | Command output visible to user | Must |

### File Operations

| ID | Requirement | Priority |
|----|-------------|----------|
| FILE-01 | AI can read files in current directory and subdirectories | Must |
| FILE-02 | AI can write new files with diff preview and approval | Must |
| FILE-03 | AI can edit existing files with diff preview and approval | Must |
| FILE-04 | Configurable: safe mode (ask) vs auto-write mode | Must |

### Multi-Provider Support

| ID | Requirement | Priority |
|----|-------------|----------|
| PROV-01 | OpenAI-compatible API support (configurable base URL for z.ai, etc.) | Must |
| PROV-02 | Anthropic Claude API support | Must |
| PROV-03 | Ollama local inference support | Must |
| PROV-04 | Provider switching via config or in-session command | Must |

## v2 Requirements (Deferred)

### TUI Mode

| ID | Requirement | Priority |
|----|-------------|----------|
| TUI-01 | `cherry2k` or `* /tui` opens full-screen interface | Should |
| TUI-02 | Richer display for longer interactions | Should |
| TUI-03 | Same capabilities as inline mode | Should |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CMD-01 | Phase 1 | Complete |
| PROV-01 | Phase 2 | Complete |
| TERM-03 | Phase 3 | Complete |
| TERM-04 | Phase 3 | Complete |
| TERM-01 | Phase 4 | Complete |
| TERM-02 | Phase 4 | Complete |
| PROV-02 | Phase 5 | Complete |
| PROV-03 | Phase 5 | Complete |
| PROV-04 | Phase 5 | Complete |
| INTENT-01 | Phase 6 | Complete |
| INTENT-02 | Phase 6 | Complete |
| INTENT-03 | Phase 6 | Complete |
| CMD-02 | Phase 6 | Complete |
| CMD-03 | Phase 6 | Complete |
| INTENT-04 | Phase 7 | Pending |
| FILE-01 | Phase 7 | Pending |
| FILE-02 | Phase 7 | Pending |
| FILE-03 | Phase 7 | Pending |
| FILE-04 | Phase 7 | Pending |

**Coverage:** 19/19 v1 requirements mapped to phases

---
*Last updated: 2026-02-01*