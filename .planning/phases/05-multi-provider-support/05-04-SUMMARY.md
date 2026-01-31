---
phase: 05
plan: 04
subsystem: cli
tags: [cli, provider, zsh, slash-commands]
dependency-graph:
  requires: [05-03]
  provides: [provider-cli-commands, slash-commands, session-provider-switching]
  affects: []
tech-stack:
  added: []
  patterns: [state-file-persistence, slash-command-detection]
file-tracking:
  key-files:
    created:
      - crates/cli/src/commands/provider.rs
    modified:
      - crates/cli/src/commands/chat.rs
      - crates/cli/src/commands/mod.rs
      - crates/cli/src/main.rs
      - crates/cli/Cargo.toml
      - crates/storage/src/context.rs
      - zsh/widgets/ai-mode.zsh
decisions:
  - id: state-file-provider
    description: "In-session provider switching uses state file at ~/.local/state/cherry2k/active_provider"
  - id: dyn-aiprovider-context
    description: "prepare_context now accepts &dyn AiProvider for ProviderFactory compatibility"
metrics:
  duration: 5 min
  completed: 2026-01-31
---

# Phase 05 Plan 04: CLI Integration + Slash Commands Summary

**One-liner:** Provider switching via CLI commands and zsh slash commands with state file persistence.

## What Was Built

### Provider Commands Module (`crates/cli/src/commands/provider.rs`)
- `run_list()` - Lists all configured providers with active marker
- `run_current()` - Shows current provider and model
- `run_switch()` - Switches active provider via state file
- State file stored at `~/.local/state/cherry2k/active_provider`

### Chat Command Updates (`crates/cli/src/commands/chat.rs`)
- Replaced hardcoded `OpenAiProvider` with `ProviderFactory`
- Reads `active_provider` state file to respect in-session switches
- Falls back to config's `default_provider` if no override

### CLI Integration (`crates/cli/src/main.rs`)
- Added `provider` subcommand with `--list` flag
- Usage: `cherry2k provider [NAME] [--list]`

### Zsh Slash Commands (`zsh/widgets/ai-mode.zsh`)
- `/provider` - Show current provider
- `/provider <name>` - Switch to provider
- `/providers` - List all providers
- `/help` - Show available commands

## Key Changes

| File | Change |
|------|--------|
| `crates/cli/src/commands/provider.rs` | New module (197 lines) |
| `crates/cli/src/commands/chat.rs` | Use ProviderFactory |
| `crates/cli/src/main.rs` | Add Provider command |
| `crates/storage/src/context.rs` | Accept `&dyn AiProvider` |
| `zsh/widgets/ai-mode.zsh` | Slash command handling |

## Decisions Made

1. **State file for provider switching**: Using `~/.local/state/cherry2k/active_provider` allows in-session switching that persists across chat invocations without modifying the config file.

2. **`&dyn AiProvider` for prepare_context**: Changed generic signature to concrete trait object to support ProviderFactory's dynamic dispatch.

3. **Slash commands in AI mode only**: Slash commands are detected in `_cherry2k_ai_mode_accept`, so they only work when in AI mode (after typing `* `).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] DummyProvider test incompatibility**
- **Found during:** Task 3 verification
- **Issue:** Storage crate's DummyProvider used old async trait syntax without BoxFuture
- **Fix:** Updated to use `BoxFuture<'_, ...>` pattern from 05-03
- **Files modified:** `crates/storage/src/context.rs`
- **Commit:** 88d1b86

## Test Results

```
171 tests passed (42 cli + 63 core + 66 storage)
0 failures
Clippy: clean
```

## CLI Examples

```bash
# List providers
$ cherry2k provider --list
Available providers:
  * anthropic (claude-sonnet-4-20250514) [active]
    ollama (llama3.2)
    openai (gpt-4o)

# Show current
$ cherry2k provider
Currently using: anthropic (claude-sonnet-4-20250514)

# Switch provider
$ cherry2k provider openai
Switched to: openai (gpt-4o)
```

## Next Phase Readiness

Phase 05 is now complete. All plans executed:
- 05-01: Anthropic provider
- 05-02: Ollama provider
- 05-03: Provider factory
- 05-04: CLI integration + slash commands

**Phase 05 deliverables complete:**
- Multi-provider architecture with OpenAI, Anthropic, Ollama
- Provider factory for dynamic dispatch
- CLI and slash commands for provider management
- In-session provider switching
