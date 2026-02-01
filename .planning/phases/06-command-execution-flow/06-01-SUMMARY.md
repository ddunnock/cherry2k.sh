---
phase: 06-command-execution-flow
plan: 01
subsystem: cli
tags: [intent-detection, response-parsing, regex]
dependency-graph:
  requires: []
  provides: [Intent, DetectedCommand, detect_intent, parse_command_from_response]
  affects: [06-03, 06-04]
tech-stack:
  added: [regex]
  patterns: [LazyLock for static regex compilation]
key-files:
  created:
    - crates/cli/src/intent/mod.rs
    - crates/cli/src/intent/types.rs
    - crates/cli/src/intent/detector.rs
  modified:
    - crates/cli/src/main.rs
    - crates/cli/Cargo.toml
    - Cargo.toml
decisions:
  - key: regex-pattern
    choice: "```(?:bash|sh|shell)\\n([\\s\\S]*?)\\n```"
    why: Matches bash/sh/shell code blocks, captures content, non-greedy
  - key: empty-handling
    choice: Empty or whitespace-only code blocks return Question intent
    why: Prevents false positives from malformed responses
  - key: first-block-wins
    choice: Only first matching code block is extracted
    why: Simple, predictable behavior; multiple commands are rare
metrics:
  duration: 2.5 min
  completed: 2026-02-01
---

# Phase 06 Plan 01: Intent Detection Summary

Regex-based parser extracts commands from bash/sh/shell code blocks in AI responses, distinguishing command suggestions from explanatory answers.

## What Was Built

### Intent Types (`types.rs`)
- `Intent` enum with `Question` and `Command(DetectedCommand)` variants
- `DetectedCommand` struct holding extracted command and optional context text
- Convenience constructors: `new()` and `with_context()`

### Intent Detector (`detector.rs`)
- `detect_intent(response: &str) -> Intent` - main entry point
- `parse_command_from_response(response: &str) -> Option<DetectedCommand>` - parsing logic
- Static regex compiled once via `LazyLock`
- Captures text before code block as context

### Module Exports (`mod.rs`)
- Re-exports `Intent`, `DetectedCommand`, `detect_intent`, `parse_command_from_response`
- Wired into `main.rs` as `pub mod intent`

## Key Implementation Details

1. **Regex Pattern**: `r"```(?:bash|sh|shell)\n([\s\S]*?)\n```"`
   - Matches `bash`, `sh`, or `shell` language tags
   - Non-greedy capture for multi-line content
   - Requires newline after opening fence (standard markdown)

2. **Edge Case Handling**:
   - Empty code blocks → `Intent::Question`
   - Whitespace-only blocks → `Intent::Question`
   - Non-shell code blocks (python, js) → `Intent::Question`
   - Multiple code blocks → First one wins

3. **Context Extraction**: Text before the code block is trimmed and stored as context, enabling the UI to show AI's explanation alongside the command.

## Test Coverage

12 tests covering:
- Bash/sh/shell code blocks detected correctly
- Python/JavaScript blocks ignored
- Multi-line commands preserved
- Context text captured
- Empty/whitespace blocks handled
- First-block-wins behavior verified

## Commits

| Hash | Description |
|------|-------------|
| 9164f0b | Create intent detection types |
| a546c30 | Implement intent detection from AI responses |

## Deviations from Plan

None - plan executed exactly as written.

## Next Phase Readiness

The intent module provides the foundation for command execution flow:
- Plan 06-02 (Command Executor) can now use `DetectedCommand` for execution
- Plan 06-03 (Command Display) can use `Intent` to route responses
- Plan 06-04 (CLI Integration) will wire these together in chat flow
