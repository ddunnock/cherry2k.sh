# Phase 4: Zsh Integration - Context

**Gathered:** 2026-01-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver the inline `* ` prefix experience that defines Cherry2K. Users type `* what is my IP` and get responses inline in their terminal, then return to their normal prompt. Also includes Ctrl+G keybinding for quick AI mode entry and tab completion for cherry2k commands.

</domain>

<decisions>
## Implementation Decisions

### Prefix Capture
- AI mode activates on `* ` prefix detection (not on Enter)
- Visual indicator: cherry emoji `🍒` replaces the normal prompt while in AI mode
- Multi-line input: Shift+Enter for newlines, Enter submits
- Exit AI mode: Backspace past `* ` returns to normal prompt naturally

### Response Display
- Response streams directly above the query line (natural terminal flow — query moves down)
- Visual style: 8-bit aesthetic for prose, markdown code blocks wrapped in boxes
- Loading indicator: Cherry-themed animation while waiting for first token
- Prompt return: Automatic — returns to normal `$` prompt unless AI asked a question, then stays in `🍒` mode for follow-up

### Keybinding Behavior
- Ctrl+G enters AI mode (behavior at empty vs non-empty prompt is Claude's discretion)
- Ctrl+C cancels during streaming (not Ctrl+G)
- Vim bindings in AI mode: esc then navigation keys (`^` for beginning, `$` for end, etc.)

### Shell State Handling
- AI queries do NOT appear in shell history (keep history clean)
- Exit codes reflect errors: 0 on success, non-zero on API errors or cancellation
- Full shell context available to AI: pwd, last N commands, and their outputs
- Context depth is configurable via config file

### Claude's Discretion
- Exact Ctrl+G behavior when prompt is empty vs has text
- Default number of commands to capture for context
- Specifics of cherry animation design
- How to achieve 8-bit text aesthetic within terminal constraints
- Vim binding scope and completeness

</decisions>

<specifics>
## Specific Ideas

- "It's be nice since this is called Cherry2K to use the cherry emoji" — `🍒` for AI mode prompt
- "Would be kind of neat if we could make it be another font, like 80's 8-bit style, except for any markdown which we could wrap in a box" — retro aesthetic for responses
- "If we could have vim bindings that would be ideal, so that for example, I can esc shift+^ to go to the beginning of the line" — vim navigation in AI input

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-zsh-integration*
*Context gathered: 2026-01-31*
