# Phase 6: Command Execution Flow - Research

**Researched:** 2026-01-31
**Domain:** Shell command execution, intent detection, process management
**Confidence:** HIGH

## Summary

This phase implements the command execution flow for Cherry2K: detecting user intent (question vs command request), presenting suggested commands with bash syntax highlighting, executing approved commands with real-time streaming output, and handling errors/signals properly.

The existing codebase already has the foundation: `confirm.rs` provides `[y/n/e]` confirmation, `StreamWriter` handles line-buffered output, `signal.rs` manages Ctrl+C with `CancellationToken`, and `error_box.rs` displays formatted errors. The main additions are: (1) intent detection via system prompts, (2) shell command execution with `tokio::process::Command`, (3) stdout/stderr streaming with proper coloring, and (4) signal forwarding to child processes.

**Primary recommendation:** Use `tokio::process::Command` for async command execution with piped stdout/stderr. Stream output line-by-line using `BufReader` + `AsyncBufReadExt`. For Ctrl+C propagation, use the `signal-child` crate to send SIGINT to child processes. Intent detection is best handled via system prompts with explicit markers (`!` and `/run`) as fallbacks.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1.49+ | Async process spawning | Already in use, `tokio::process::Command` provides async child process management |
| signal-child | 1.0.6 | SIGINT to child | Simple trait extension for `std::process::Child`, works with tokio Child via id() |
| nix | 0.31+ | Signal handling (fallback) | Low-level Unix APIs if signal-child insufficient |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio-process-stream | 0.5+ | Stream wrapper | Alternative to manual BufReader if cleaner API preferred |
| termimad | 0.30 | Code block rendering | Already in use for markdown; has bash syntax highlighting |
| colored | 3.x | ANSI colors | Already in use; stderr red coloring |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tokio::process | portable-pty | PTY gives full terminal emulation but adds complexity; not needed for simple command execution |
| signal-child | nix::sys::signal::kill | nix is lower-level, requires Pid handling; signal-child is simpler |
| Manual streaming | tokio-process-stream | tokio-process-stream is cleaner but adds dependency; manual is more flexible |

**Installation:**
```bash
cargo add signal-child
# tokio, termimad, colored already in workspace
```

## Architecture Patterns

### Recommended Project Structure
```
crates/cli/src/
├── commands/
│   ├── chat.rs           # Modify to detect intent and route
│   └── execute.rs        # NEW: Shell command execution
├── intent/               # NEW: Intent detection module
│   ├── mod.rs
│   ├── detector.rs       # System prompt + heuristics
│   └── types.rs          # Intent enum (Question, Command, etc.)
├── output/
│   ├── stream_writer.rs  # Extend for stderr coloring
│   └── command_output.rs # NEW: Command result display
└── confirm.rs            # Already has Edit support
```

### Pattern 1: Intent Detection via System Prompt
**What:** Use the AI provider itself to classify intent, with explicit markers as override
**When to use:** Always - this is the primary detection mechanism
**Example:**
```rust
// System prompt addition for intent detection
const INTENT_SYSTEM_PROMPT: &str = r#"
You are a terminal assistant. Classify user requests as:
- QUESTION: User wants information/explanation
- COMMAND: User wants you to suggest a shell command to run

When user wants a command, respond with EXACTLY this format:
```bash
<command here>
```

If the request is ambiguous between question and command, prefer COMMAND.

Explicit markers override classification:
- `!` prefix or `/run` prefix = always COMMAND mode
- `?` suffix = always QUESTION mode
"#;

// In chat.rs, parse AI response for code blocks
fn parse_response_for_command(response: &str) -> Option<String> {
    // Look for ```bash or ```sh code blocks
    let re = regex::Regex::new(r"```(?:bash|sh)\n([\s\S]*?)\n```").ok()?;
    re.captures(response).map(|c| c[1].trim().to_string())
}
```

### Pattern 2: Async Command Execution with Streaming
**What:** Spawn command with piped stdout/stderr, stream line-by-line
**When to use:** When executing approved commands
**Example:**
```rust
// Source: https://docs.rs/tokio/latest/tokio/process/struct.Command.html
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::process::Stdio;

pub async fn execute_command(cmd: &str) -> Result<ExitStatus, Error> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)  // Clean up if we're dropped
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            line = stdout_reader.next_line() => {
                match line? {
                    Some(line) => println!("{}", line),
                    None => break,
                }
            }
            line = stderr_reader.next_line() => {
                match line? {
                    Some(line) => eprintln!("{}", line.red()),
                    None => {}
                }
            }
        }
    }

    child.wait().await
}
```

### Pattern 3: Ctrl+C Signal Forwarding
**What:** Forward SIGINT to child process when user presses Ctrl+C
**When to use:** During command execution
**Example:**
```rust
// Using signal-child crate
use signal_child::Signalable;
use tokio::signal;

pub async fn execute_with_interrupt(cmd: &str) -> Result<ExitStatus, Error> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Get the raw Child for signal sending
    let child_id = child.id().expect("child has id");

    // Spawn signal handler
    let interrupt_handle = tokio::spawn(async move {
        signal::ctrl_c().await.ok();
        // Send SIGINT to child process group
        unsafe {
            libc::kill(-(child_id as i32), libc::SIGINT);
        }
    });

    // ... stream output ...

    let status = child.wait().await?;
    interrupt_handle.abort();  // Cancel signal handler

    Ok(status)
}
```

### Pattern 4: Exit Code Display
**What:** Show clear success/failure indicators with exit codes
**When to use:** After command completion
**Example:**
```rust
use colored::Colorize;

pub fn display_exit_status(status: ExitStatus) {
    match status.code() {
        Some(0) => {
            println!("{} Command completed (exit 0)", "".green());
        }
        Some(code) => {
            println!("{} Command failed (exit {})", "".red(), code);
        }
        None => {
            // Process was terminated by signal
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                if let Some(sig) = status.signal() {
                    println!("{} Terminated by signal {}", "".yellow(), sig);
                }
            }
        }
    }
}
```

### Anti-Patterns to Avoid
- **Blocking stdin read during execution:** Don't block on stdin while command runs; use async
- **Ignoring stderr:** Always capture and display stderr, colored differently
- **Automatic command execution:** Never execute without user confirmation
- **Raw PTY for simple commands:** PTY adds complexity; use pipes unless interactive terminal needed
- **Swallowing exit codes:** Always show exit code, especially on failure

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Process signal sending | Manual libc calls | signal-child crate | Platform handling, error management |
| Async line reading | Custom buffering | BufReader + AsyncBufReadExt | Tokio-optimized, handles edge cases |
| ANSI color output | Manual escape codes | colored crate | Already in use, respects NO_COLOR |
| Markdown code blocks | Custom parser | termimad (already used) | Handles bash syntax highlighting |
| Intent classification | Rule-based NLP | LLM system prompt | AI handles nuance better than rules |

**Key insight:** The existing codebase already has most utilities (StreamWriter, confirm, error_box). Reuse and extend rather than replace.

## Common Pitfalls

### Pitfall 1: Zombie Processes on Drop
**What goes wrong:** Child process continues running after parent drops handle
**Why it happens:** Default tokio::process behavior doesn't kill on drop
**How to avoid:** Use `.kill_on_drop(true)` on Command builder
**Warning signs:** Process still running after CLI exits (check `ps aux`)

### Pitfall 2: Blocking Main Thread with Signals
**What goes wrong:** Signal handler blocks async runtime
**Why it happens:** Using synchronous signal handling in async context
**How to avoid:** Use `tokio::signal::ctrl_c()` which is async
**Warning signs:** CLI becomes unresponsive during command execution

### Pitfall 3: Mixed stdout/stderr Ordering
**What goes wrong:** Output appears in wrong order
**Why it happens:** Buffering differences between stdout and stderr
**How to avoid:** Use `tokio::select!` to handle both streams concurrently
**Warning signs:** Error messages appear after subsequent stdout lines

### Pitfall 4: Intent Detection False Positives
**What goes wrong:** Questions interpreted as commands (or vice versa)
**Why it happens:** Ambiguous natural language, no explicit markers
**How to avoid:** Default to command mode (action-oriented), provide explicit markers
**Warning signs:** AI suggesting commands when user wanted explanation

### Pitfall 5: Losing Partial Output on Interrupt
**What goes wrong:** Output before Ctrl+C is lost
**Why it happens:** Not flushing buffers before exit
**How to avoid:** Flush StreamWriter on interrupt, similar to existing pattern in chat.rs
**Warning signs:** Truncated output when user cancels

### Pitfall 6: Shell Injection
**What goes wrong:** User input executed as shell commands
**Why it happens:** Passing AI-generated commands directly to shell
**How to avoid:** Always require user confirmation, display exact command
**Warning signs:** Unexpected command execution, security alerts

## Code Examples

Verified patterns from official sources:

### Command Execution with Streaming (tokio docs pattern)
```rust
// Source: https://docs.rs/tokio/latest/tokio/process/index.html
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::process::Stdio;

async fn run_streaming(cmd: &str) -> std::io::Result<std::process::ExitStatus> {
    let mut child = Command::new("sh")
        .args(["-c", cmd])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");

    // Spawn task to read stderr
    let stderr_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            eprintln!("\x1b[31m{}\x1b[0m", line); // Red
        }
    });

    // Read stdout in main task
    let mut stdout_reader = BufReader::new(stdout).lines();
    while let Ok(Some(line)) = stdout_reader.next_line().await {
        println!("{}", line);
    }

    stderr_handle.await.ok();
    child.wait().await
}
```

### Exit Status Handling (std::process docs)
```rust
// Source: https://doc.rust-lang.org/std/process/struct.ExitStatus.html
use std::process::ExitStatus;

fn display_result(status: ExitStatus) {
    if status.success() {
        println!("\x1b[32m\u{2713}\x1b[0m Command completed (exit 0)");
    } else {
        match status.code() {
            Some(code) => {
                println!("\x1b[31m\u{2717}\x1b[0m Command failed (exit {})", code);
            }
            None => {
                println!("\x1b[33m!\x1b[0m Process terminated by signal");
            }
        }
    }
}
```

### Confirmation Flow (existing pattern)
```rust
// Source: crates/cli/src/confirm.rs - extend for command edit
use crate::confirm::{confirm_command, ConfirmResult};

async fn suggest_and_maybe_execute(command: &str) -> Result<()> {
    // Display with bash highlighting (termimad)
    println!("Suggested command:");
    println!("```bash\n{}\n```", command);

    match confirm_command(command)? {
        ConfirmResult::Yes => {
            execute_command(command).await?;
        }
        ConfirmResult::No => {
            println!("Command cancelled.");
        }
        ConfirmResult::Edit => {
            // Open $EDITOR or readline for edit
            let edited = edit_command(command)?;
            if confirm("Run edited command?", false)? == ConfirmResult::Yes {
                execute_command(&edited).await?;
            }
        }
    }
    Ok(())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| PTY for all commands | Pipes + PTY only when needed | 2024 | Simpler, fewer dependencies |
| Rule-based intent | LLM-based intent detection | 2024-2025 | Better handling of ambiguous requests |
| sync process::Command | tokio::process::Command | Tokio 1.x | Non-blocking, better resource usage |
| Manual signal handling | signal-child/nix crates | 2023+ | Platform abstraction, fewer bugs |

**Deprecated/outdated:**
- `std::process::Command` in async contexts: Use tokio::process instead
- `ctrlc` crate for complex signal handling: Use `tokio::signal` + nix
- Manual ANSI escapes: Use `colored` crate for portability

## Open Questions

Things that couldn't be fully resolved:

1. **Edit command UX**
   - What we know: `ConfirmResult::Edit` exists, need to implement edit flow
   - What's unclear: Use $EDITOR vs inline readline vs simple prompt?
   - Recommendation: Start with simple re-prompt, add $EDITOR in future

2. **Process group handling for pipelines**
   - What we know: `kill(-pgid, SIGINT)` kills process group
   - What's unclear: How to handle complex pipelines (`cmd1 | cmd2 | cmd3`)
   - Recommendation: Use `-c` flag with `sh`, let shell handle pipeline signals

3. **Windows compatibility**
   - What we know: signal-child is Unix-only, tokio::process works on Windows
   - What's unclear: How to handle Ctrl+C on Windows
   - Recommendation: Defer Windows support, document Unix-only for now

## Sources

### Primary (HIGH confidence)
- [tokio::process::Command](https://docs.rs/tokio/latest/tokio/process/struct.Command.html) - Command spawning, kill_on_drop, piping
- [std::process::ExitStatus](https://doc.rust-lang.org/std/process/struct.ExitStatus.html) - Exit code handling
- [signal-child crate](https://docs.rs/signal-child) - SIGINT to child processes
- [nix::sys::signal::kill](https://docs.rs/nix/latest/nix/sys/signal/fn.kill.html) - Low-level signal API

### Secondary (MEDIUM confidence)
- [tokio-process-stream](https://docs.rs/tokio-process-stream) - Stream wrapper alternative
- [Fixing Ctrl+C in Terminal Apps](https://www.fiveonefour.com/blog/Fixing-ctrl-c-in-terminal-apps-child-process-management) - Best practices article
- [Shell_GPT](https://github.com/TheR1D/shell_gpt) - Reference implementation for command suggestion UX

### Tertiary (LOW confidence)
- Intent detection approaches from web search - need validation with actual implementation
- Windows signal handling - not verified, deferred

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - tokio::process is well-documented, signal-child is small/auditable
- Architecture: HIGH - Patterns align with existing codebase (StreamWriter, confirm.rs)
- Pitfalls: MEDIUM - Based on documentation and articles, needs validation
- Intent detection: MEDIUM - LLM-based approach is standard but prompt needs tuning

**Research date:** 2026-01-31
**Valid until:** 2026-03-01 (60 days - stable domain, tokio process API unlikely to change)
