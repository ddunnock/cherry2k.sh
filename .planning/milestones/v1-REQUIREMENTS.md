# Requirements Archive: v1 Cherry2K MVP

**Archived:** 2026-01-31
**Status:** ✅ SHIPPED

This is the archived requirements specification for v1.
For current requirements, see `.planning/REQUIREMENTS.md` (created for next milestone).

---

## v1 Requirements

### Terminal Integration

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| TERM-01 | `* ` prefix triggers AI from any terminal prompt | Must | ✓ Complete |
| TERM-02 | AI responds inline, returns user to prompt when done | Must | ✓ Complete |
| TERM-03 | Conversational context persists within session | Must | ✓ Complete |
| TERM-04 | Follow-up questions work without repeating context | Must | ✓ Complete |

### Intent Detection

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| INTENT-01 | AI distinguishes questions from command requests from coding tasks | Must | ✓ Complete |
| INTENT-02 | Questions get explanations | Must | ✓ Complete |
| INTENT-03 | Command requests get suggested commands | Must | ✓ Complete |
| INTENT-04 | Coding requests trigger file operations | Must | ✓ Complete |

### Command Execution

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| CMD-01 | Suggested commands show "Run this? [y/n]" confirmation | Must | ✓ Complete |
| CMD-02 | Confirmed commands execute in user's shell context | Must | ✓ Complete |
| CMD-03 | Command output visible to user | Must | ✓ Complete |

### File Operations

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FILE-01 | AI can read files in current directory and subdirectories | Must | ✓ Complete |
| FILE-02 | AI can write new files with diff preview and approval | Must | ✓ Complete |
| FILE-03 | AI can edit existing files with diff preview and approval | Must | ✓ Complete |
| FILE-04 | Configurable: safe mode (ask) vs auto-write mode | Must | ✓ Complete |

### Multi-Provider Support

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| PROV-01 | OpenAI-compatible API support (configurable base URL for z.ai, etc.) | Must | ✓ Complete |
| PROV-02 | Anthropic Claude API support | Must | ✓ Complete |
| PROV-03 | Ollama local inference support | Must | ✓ Complete |
| PROV-04 | Provider switching via config or in-session command | Must | ✓ Complete |

---

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CMD-01 | Phase 1 | ✓ Complete |
| PROV-01 | Phase 2 | ✓ Complete |
| TERM-03 | Phase 3 | ✓ Complete |
| TERM-04 | Phase 3 | ✓ Complete |
| TERM-01 | Phase 4 | ✓ Complete |
| TERM-02 | Phase 4 | ✓ Complete |
| PROV-02 | Phase 5 | ✓ Complete |
| PROV-03 | Phase 5 | ✓ Complete |
| PROV-04 | Phase 5 | ✓ Complete |
| INTENT-01 | Phase 6 | ✓ Complete |
| INTENT-02 | Phase 6 | ✓ Complete |
| INTENT-03 | Phase 6 | ✓ Complete |
| CMD-02 | Phase 6 | ✓ Complete |
| CMD-03 | Phase 6 | ✓ Complete |
| INTENT-04 | Phase 7 | ✓ Complete |
| FILE-01 | Phase 7 | ✓ Complete |
| FILE-02 | Phase 7 | ✓ Complete |
| FILE-03 | Phase 7 | ✓ Complete |
| FILE-04 | Phase 7 | ✓ Complete |

**Coverage:** 19/19 v1 requirements shipped

---

## Milestone Summary

**Shipped:** 19 of 19 v1 requirements

**Adjusted:** None — all requirements implemented as originally specified

**Dropped:** None

**Deferred to v2:**
- TUI-01, TUI-02, TUI-03: TUI mode (marked optional in PROJECT.md)

---

*Archived: 2026-01-31 as part of v1 milestone completion*
