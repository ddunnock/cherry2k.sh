---
phase: 02-single-provider-e2e
plan: 02
subsystem: cli-output
tags: [terminal, spinner, markdown, error-display, streaming]

dependency-graph:
  requires: [01-foundation-and-safety]
  provides: [output-utilities, spinner, stream-writer, error-box, markdown-render]
  affects: [02-03-openai-integration]

tech-stack:
  added: [indicatif, termimad, colored]
  patterns: [line-buffered-output, unicode-box-drawing]

key-files:
  created:
    - crates/cli/src/output/mod.rs
    - crates/cli/src/output/spinner.rs
    - crates/cli/src/output/stream_writer.rs
    - crates/cli/src/output/error_box.rs
    - crates/cli/src/output/markdown.rs
  modified:
    - Cargo.toml
    - crates/cli/Cargo.toml
    - crates/cli/src/main.rs

decisions:
  - id: 02-02-01
    choice: "Separate display_error and display_provider_error functions"
    rationale: "Avoids runtime downcasting complexity; cleaner API for callers"
  - id: 02-02-02
    choice: "Unicode box-drawing chars instead of cli-boxes crate"
    rationale: "cli-boxes not available on crates.io; direct Unicode is simpler"
  - id: 02-02-03
    choice: "COLUMNS env var for terminal width detection"
    rationale: "Lightweight approach; can add terminal_size crate later if needed"

metrics:
  duration: 4 min
  completed: 2026-01-30
---

# Phase 02 Plan 02: Output Utilities Summary

Terminal output utilities with spinner, line-buffered streaming, boxed errors with actionable guidance, and markdown rendering via termimad.

## What Was Built

### Output Module (`crates/cli/src/output/`)

Created complete terminal output utilities for the CLI:

1. **ResponseSpinner** (`spinner.rs`)
   - Wraps indicatif ProgressBar for animated waiting state
   - Cyan colored spinner with configurable message
   - 100ms tick interval for smooth animation
   - `start()` / `stop()` / `set_message()` API

2. **StreamWriter** (`stream_writer.rs`)
   - Line-buffered output for streaming AI responses
   - Buffers until newline, then prints complete line
   - Prevents janky character-by-character output
   - `write_chunk()` / `flush()` / `has_buffered_content()` API

3. **Error Box** (`error_box.rs`)
   - Unicode double-line box-drawing frame (red colored)
   - `display_error()` for generic errors
   - `display_provider_error()` with actionable guidance:
     - RateLimited: shows retry time, quota suggestion
     - InvalidApiKey: shows env var name, config file path
     - Unavailable: suggests retry or alternative provider
   - Terminal width detection via COLUMNS env var

4. **Markdown Renderer** (`markdown.rs`)
   - termimad-based rendering with custom skin
   - Colors: Yellow bold, Cyan italic, Green code
   - Plain mode toggle for piped output / no-color environments
   - `render_markdown(text, plain)` API

### Dependencies Added

```toml
indicatif = "0.17"   # Spinner animations
termimad = "0.30"    # Markdown rendering
colored = "3"        # ANSI colors
```

## Commits

| Hash | Message |
|------|---------|
| 4acb91b | chore(02-02): add output dependencies to workspace |
| b7bc4d7 | feat(02-02): add spinner and line-buffered stream writer |
| 64959b3 | feat(02-02): add error box and markdown rendering utilities |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed runtime downcasting for ProviderError**
- **Found during:** Task 3
- **Issue:** `dyn Error` cannot be cast to `dyn Any` for downcasting
- **Fix:** Created separate `display_provider_error()` function instead
- **Impact:** Cleaner API, caller explicitly chooses which function to use

## Test Coverage

All output utilities have unit tests:
- Spinner creation and lifecycle
- StreamWriter line buffering behavior
- Error message formatting for all ProviderError variants
- Markdown plain mode toggle

## Usage Examples

```rust
use cherry2k::output::{ResponseSpinner, StreamWriter, display_provider_error, render_markdown};

// Spinner while waiting
let spinner = ResponseSpinner::new();
spinner.start();
// ... await response ...
spinner.stop();

// Line-buffered streaming
let mut writer = StreamWriter::new();
writer.write_chunk("Hello, ")?;
writer.write_chunk("world!\n")?;  // Prints complete line
writer.flush()?;

// Error display
display_provider_error(&ProviderError::InvalidApiKey { provider: "OpenAI".into() });

// Markdown rendering
let formatted = render_markdown("**bold** text", false);
let plain = render_markdown("**bold** text", true);
```

## Next Phase Readiness

Plan 02-03 (OpenAI Integration) can now use these utilities:
- Spinner while waiting for API response
- StreamWriter for streaming token output
- Error box for API errors with actionable guidance
- Markdown rendering for formatted responses

All verification passed:
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace` (23 CLI tests, all passing)
