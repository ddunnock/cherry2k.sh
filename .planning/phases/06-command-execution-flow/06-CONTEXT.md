# Phase 6: Command Execution Flow - Context

**Gathered:** 2026-01-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Enable AI to suggest commands that execute in user's shell. AI distinguishes questions from command requests, presents suggested commands for confirmation, executes in shell context, and handles errors. File operations are a separate phase.

</domain>

<decisions>
## Implementation Decisions

### Intent Detection
- Hybrid approach: Natural language inference + optional explicit markers
- When ambiguous, default to command suggestion (action-oriented)
- Explicit markers: Both `!` prefix and `/run` prefix force command mode
- No automatic explanation of command reasoning — user can ask if needed

### Command Presentation
- Code block with bash syntax highlighting for suggested commands
- No special warnings for dangerous commands — user always confirms
- Sometimes offer alternatives when valid options exist
- Confirmation prompt: `[y]es / [n]o / [e]dit` — user can modify command before running

### Execution and Feedback
- Stream output in real-time as command runs
- No progress indicator needed — output stream is feedback
- Ctrl+C propagates directly to running command (no confirmation)
- Show exit code on completion: `✓ Command completed (exit 0)`

### Error Handling
- Don't automatically offer to explain errors — user can ask
- Stderr displayed in red/highlighted styling
- Suggest fixes for obvious errors (e.g., "Permission denied — try with sudo?")
- Always show exit code on failure: `✗ Command failed (exit 1)`

### Claude's Discretion
- Exact intent detection heuristics
- When to offer alternatives vs single command
- Which errors are "obvious" enough to suggest fixes
- Implementation of shell context execution

</decisions>

<specifics>
## Specific Ideas

- Already have confirmation flow architecture from Phase 1 (CMD-01)
- Existing `display_error` utilities can be extended for stderr highlighting
- StreamWriter pattern from Phase 2 can be reused for real-time output

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-command-execution-flow*
*Context gathered: 2026-01-31*
