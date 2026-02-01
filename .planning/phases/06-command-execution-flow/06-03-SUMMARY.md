---
phase: 06-command-execution-flow
plan: 03
subsystem: cli-output
tags: [termimad, syntax-highlighting, confirmation, user-input]

dependency-graph:
  requires: ["06-01", "06-02"]
  provides: ["command-display", "edit-flow", "confirm-exports"]
  affects: ["06-04"]

tech-stack:
  added: []
  patterns: ["termimad-markdown-rendering", "stdin-line-editing"]

files:
  created:
    - crates/cli/src/output/command_display.rs
  modified:
    - crates/cli/src/output/mod.rs
    - crates/cli/src/confirm.rs
    - crates/cli/src/lib.rs

decisions:
  - id: "06-03-01"
    choice: "MadSkin::default() for syntax highlighting"
    rationale: "termimad provides built-in bash code block rendering"
  - id: "06-03-02"
    choice: "Module-level #![allow(dead_code)] for confirm.rs"
    rationale: "Public API not yet used by binary, will be used in 06-04"
  - id: "06-03-03"
    choice: "Empty edit input returns original command"
    rationale: "Convenient UX - press Enter to keep suggestion unchanged"

metrics:
  duration: "3 min"
  completed: "2026-02-01"
---

# Phase 06 Plan 03: Command Display & Edit Flow Summary

Syntax highlighting for suggested commands using termimad, plus edit flow for user command modification.

## What Was Built

### Command Display Module
- `display_suggested_command(command, context)` function
- Renders commands in markdown code blocks with bash syntax highlighting
- Optional context text displayed before command block
- Uses termimad's MadSkin for terminal rendering

### Edit Command Flow
- `edit_command(original)` function for interactive command editing
- Displays current command and prompts for replacement
- Empty input preserves original command
- Returns edited or original command string

### Module Exports
- Added `pub mod confirm` to lib.rs
- Exported `display_suggested_command` from output module
- Module-level `#![allow(dead_code)]` for API not yet used by binary

## Key Files

| File | Purpose |
|------|---------|
| `crates/cli/src/output/command_display.rs` | Command display with syntax highlighting |
| `crates/cli/src/confirm.rs` | edit_command function added |
| `crates/cli/src/output/mod.rs` | Export display_suggested_command |
| `crates/cli/src/lib.rs` | Export confirm module |

## Decisions Made

1. **MadSkin::default() for highlighting**: termimad provides built-in bash code block rendering without additional configuration

2. **Module-level dead_code allow**: The confirm module is public API that will be used in 06-04 CLI integration, so we allow dead_code at module level rather than per-function

3. **Empty edit returns original**: User-friendly UX where pressing Enter without typing keeps the suggested command

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed collapsible_if clippy warning**
- **Found during:** Task 3 verification
- **Issue:** Nested if statements in command_display.rs
- **Fix:** Combined into `if let Some(ctx) = context && !ctx.is_empty()`
- **Files modified:** crates/cli/src/output/command_display.rs
- **Commit:** 82fc782

## Verification

- `cargo check -p cherry2k` passes
- `cargo clippy -p cherry2k -- -D warnings` passes
- All 56 library tests pass
- Pre-existing doctest failures in retro.rs (unrelated to this plan)

## Next Phase Readiness

Plan 06-04 (CLI Integration) can now:
- Use `cherry2k::output::display_suggested_command` for command presentation
- Use `cherry2k::confirm::{confirm_command, edit_command, ConfirmResult}` for user confirmation
- Integrate with intent detection and command execution from 06-01/06-02
