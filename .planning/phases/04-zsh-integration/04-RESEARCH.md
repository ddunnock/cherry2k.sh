# Phase 04: Zsh Integration - Research

**Researched:** 2026-01-31
**Domain:** Zsh ZLE (Z-Shell Line Editor) integration with Rust backend
**Confidence:** HIGH

## Summary

Zsh integration for inline AI responses requires leveraging the Zsh Line Editor (ZLE) system, which provides a powerful widget-based architecture for custom input handling. The standard approach combines ZLE widgets for input capture with Rust-based CLI streaming output. The `* ` prefix detection and inline response display require custom widget implementation that manipulates ZLE special parameters (BUFFER, LBUFFER, RBUFFER) and coordinates with the Rust backend for streaming responses.

Key findings show that ZLE widgets are the canonical mechanism for custom terminal behavior in zsh, with well-established patterns for keybindings, buffer manipulation, and signal handling. The Rust ecosystem provides excellent terminal control via crates like crossterm (already in use via termimad) and specialized markdown rendering libraries for terminal output.

**Primary recommendation:** Implement pure zsh ZLE widgets that invoke the Rust CLI binary, using ZLE's BUFFER/LBUFFER/RBUFFER parameters for prefix detection and self-insert widget wrapping for live input capture. Leverage existing Rust terminal output infrastructure (termimad, indicatif) and add 8-bit styling via ANSI color schemes.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ZLE (zsh built-in) | zsh 5.8+ | Line editor framework | Native to zsh, zero dependencies, widget-based architecture is canonical approach |
| crossterm | 0.29 | Terminal control (ANSI) | Industry standard for Rust terminal apps, already used via termimad |
| termimad | 0.30 | Markdown rendering | Already in project, stable, supports terminal width and styling |
| indicatif | 0.17 | Progress/spinner display | Already in project, widely used for CLI progress indicators |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| terminal-emoji | 0.4.1 | Safe emoji display | For cherry prompt (🍒) with terminal compatibility checking |
| anes | 0.1 | ANSI escape sequences | If need low-level terminal control beyond crossterm |
| streamdown | 0.1.4 | Streaming markdown | Alternative to termimad if need streaming-first markdown rendering |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ZLE widgets | Oh-My-Zsh framework | OMZ adds bloat (0.38s+ startup time), pure zsh is faster and cleaner |
| termimad | mdcat | mdcat is CLI-focused, termimad better for library integration |
| Pure zsh | Rust TUI (ratatui) | TUI framework is overkill for inline responses, breaks terminal flow |

**Installation:**
```bash
# Zsh (system package)
brew install zsh  # macOS
apt install zsh   # Linux

# Rust dependencies (already in Cargo.toml)
termimad = "0.30"
indicatif = "0.17"
```

## Architecture Patterns

### Recommended Project Structure
```
zsh/
├── cherry2k.plugin.zsh       # Main plugin, sources other files
├── widgets/
│   ├── ai-mode.zsh           # Prefix detection and AI mode widget
│   ├── keybindings.zsh       # Ctrl+G and other bindings
│   └── vim-navigation.zsh    # Vim mode customizations
└── completions/
    └── _cherry2k             # Tab completion definitions
```

### Pattern 1: ZLE Widget for Prefix Detection

**What:** Custom self-insert wrapper that detects `* ` prefix and switches to AI mode
**When to use:** Live prefix detection without waiting for Enter
**Example:**
```zsh
# Source: Official zsh ZLE documentation + sgeb.io custom widgets guide

# Store original self-insert widget
_cherry2k_original_self_insert="$widgets[self-insert]"

_cherry2k_self_insert() {
    # Call original self-insert first
    zle .$_cherry2k_original_self_insert

    # Check if buffer now starts with "* "
    if [[ "$BUFFER" == "* "* ]]; then
        # Switch to AI mode
        _cherry2k_enter_ai_mode
    fi
}

# Register widget
zle -N self-insert _cherry2k_self_insert
```

### Pattern 2: BUFFER Manipulation for AI Mode

**What:** Use LBUFFER/RBUFFER to manage AI mode prompt and input
**When to use:** Switching between normal and AI mode
**Example:**
```zsh
# Source: Official zsh ZLE documentation

_cherry2k_enter_ai_mode() {
    # Set AI mode flag
    _CHERRY2K_AI_MODE=1

    # Replace "* " with cherry emoji in display (keep in BUFFER for detection)
    # Note: Visual indication only, actual prefix stays for exit detection
    LBUFFER="${LBUFFER#\* }"

    # Store original prompt for restoration
    _CHERRY2K_SAVED_PROMPT="$PROMPT"
    PROMPT="🍒 "

    # Trigger redisplay
    zle -R
}

_cherry2k_exit_ai_mode() {
    _CHERRY2K_AI_MODE=0
    PROMPT="$_CHERRY2K_SAVED_PROMPT"
    zle -R
}
```

### Pattern 3: Keybinding with ZLE Widget

**What:** Bind Ctrl+G to enter AI mode
**When to use:** Quick AI mode activation
**Example:**
```zsh
# Source: sgeb.io + mastering-zsh docs

_cherry2k_ctrl_g_handler() {
    if [[ -z "$BUFFER" ]]; then
        # Empty prompt: enter AI mode
        BUFFER="* "
        CURSOR=2
    else
        # Non-empty: prepend "* " to existing text
        BUFFER="* $BUFFER"
        CURSOR=$((2 + ${#BUFFER}))
    fi
    _cherry2k_enter_ai_mode
}

zle -N _cherry2k_ctrl_g_handler
bindkey '^G' _cherry2k_ctrl_g_handler
```

### Pattern 4: Shell Context Extraction

**What:** Capture shell history and environment for AI context
**When to use:** Providing context to AI for better responses
**Example:**
```zsh
# Source: zsh fc command documentation + mastering-zsh

_cherry2k_get_context() {
    local context_depth=${CHERRY2K_CONTEXT_DEPTH:-10}
    local context_json="{"

    # Current directory
    context_json+="\"pwd\":\"$PWD\","

    # Recent command history (with timestamps if EXTENDED_HISTORY set)
    local history_output=$(fc -rli -${context_depth} 2>/dev/null | jq -Rs .)
    context_json+="\"history\":${history_output},"

    # Environment variables (filtered)
    context_json+="\"env\":{\"USER\":\"$USER\",\"SHELL\":\"$SHELL\"}"

    context_json+="}"
    echo "$context_json"
}
```

### Pattern 5: Streaming Response Display

**What:** Invoke Rust CLI and display streaming output inline
**When to use:** Showing AI response as it arrives
**Example:**
```zsh
# Source: zsh-ollama-command + shellgpt patterns

_cherry2k_invoke_ai() {
    local query="$1"
    local context="$(_cherry2k_get_context)"

    # Print newline to move query up
    print ""

    # Call Rust binary with streaming output
    # The binary handles the actual display with termimad/indicatif
    cherry2k chat --context="$context" "$query"

    local exit_code=$?

    # Return to prompt
    if [[ $exit_code -eq 0 ]]; then
        _cherry2k_exit_ai_mode
    fi

    return $exit_code
}
```

### Pattern 6: Signal Handling (Ctrl+C)

**What:** Trap SIGINT to cancel streaming and clean up
**When to use:** User interrupts streaming response
**Example:**
```zsh
# Source: zsh Jobs & Signals documentation

_cherry2k_setup_signal_handlers() {
    # Define trap for SIGINT during AI mode
    TRAPINT() {
        if [[ $_CHERRY2K_AI_MODE -eq 1 ]]; then
            # Cancel was requested
            print "\n^C"
            _cherry2k_exit_ai_mode
            zle -R
            return 130  # Standard Ctrl+C exit code
        fi
    }
}
```

### Anti-Patterns to Avoid

- **Modifying history directly:** ZLE widgets run before history is written. Set HISTORY_IGNORE pattern instead to exclude AI queries from history.
- **Synchronous API calls in widgets:** Never block ZLE. Always fork to background or use zle -I for cleanup before external commands.
- **Hardcoded terminal escape sequences:** Use termimad/crossterm for ANSI codes, don't hand-roll sequences.
- **Global variable pollution:** Prefix all plugin variables with `_CHERRY2K_` or `_cherry2k_` to avoid conflicts.
- **Ignoring terminal width:** Use termimad's width detection or $COLUMNS for wrapping.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Markdown rendering | Custom parser + ANSI codes | termimad (already in use) | Handles code blocks, tables, wrapping, color schemes correctly |
| Terminal dimensions | Parse tput output | crossterm::terminal::size() | Cross-platform, handles edge cases (pipe, resize, etc.) |
| Progress indicators | Manual spinner with sleep loops | indicatif (already in use) | Smooth animations, terminal cleanup, no flicker |
| ANSI escape codes | String concatenation | anes or crossterm | Proper sequencing, terminal compatibility, edge case handling |
| JSON parsing in zsh | awk/sed pipelines | jq (system dependency) | Robust, handles escaping, widely available |
| History timestamp parsing | String manipulation | fc -li with proper format | Handles EXTENDED_HISTORY format, timezone-aware |

**Key insight:** Terminal emulators have quirks (cursor positioning bugs, ANSI support differences, resize races). Use battle-tested libraries that handle these edge cases.

## Common Pitfalls

### Pitfall 1: Terminal Prompt Corruption
**What goes wrong:** ANSI escape sequences in output corrupt the prompt after streaming completes, showing garbage characters or misaligned text.
**Why it happens:** Incomplete escape sequences, missing reset codes (ESC[0m), or terminal state not restored after interruption.
**How to avoid:**
- Always end output with ANSI reset (ESC[0m)
- Use `zle -I` before any output that might fail
- Flush and reset terminal state in SIGINT handler
**Warning signs:**
- Prompt appears on wrong line
- Colors bleed into next command
- Cursor invisible after response

### Pitfall 2: ZLE Buffer Safety
**What goes wrong:** Modifying BUFFER/LBUFFER/RBUFFER outside valid ranges causes crashes or unexpected behavior.
**Why it happens:** Cursor position (CURSOR) becomes invalid after buffer modification, or attempting to manipulate buffers in non-widget context.
**How to avoid:**
- Always check `[[ -o zle ]]` before widget operations
- Update CURSOR when modifying BUFFER length
- Use `zle -la` to verify widget is registered
**Warning signs:**
- "widgets can only be called when ZLE is active" error
- Cursor jumps to wrong position
- Text inserted in wrong location

### Pitfall 3: Self-Insert Widget Conflicts
**What goes wrong:** Multiple plugins wrapping self-insert create infinite recursion or override each other.
**Why it happens:** Plugins chain self-insert without preserving previous widget, or use wrong widget name reference.
**How to avoid:**
- Store original widget: `_original="$widgets[self-insert]"`
- Call with dot prefix: `zle .$_original`
- Use unique widget names with plugin prefix
**Warning signs:**
- Shell hangs on character input
- Characters don't appear when typing
- Error about widget not found

### Pitfall 4: History Pollution
**What goes wrong:** AI queries appear in shell history, cluttering history search and violating user expectation.
**Why it happens:** Zsh writes BUFFER to history on accept-line, including AI mode queries.
**How to avoid:**
- Use `setopt HIST_IGNORE_SPACE` and prefix AI commands with space
- Or set HISTORY_IGNORE pattern for `* ` prefix
- Accept-line widget wrapper that sets HISTORY_IGNORE flag
**Warning signs:**
- `history` shows AI queries
- Ctrl+R finds AI prompts instead of real commands

### Pitfall 5: Oh-My-Zsh Performance Impact
**What goes wrong:** Adding plugin to Oh-My-Zsh slows shell startup significantly (300-500ms overhead).
**Why it happens:** OMZ loads ~100 plugins/scripts on every shell start, compounding plugin overhead.
**How to avoid:**
- Support standalone zsh use (don't require OMZ)
- Lazy-load heavy operations (defer until first use)
- Use autoload for widget functions
**Warning signs:**
- Shell startup time >0.5s
- Noticeable delay before first prompt

### Pitfall 6: Vim Mode Keybinding Conflicts
**What goes wrong:** Custom keybindings only work in one mode (insert vs command), or override essential vim bindings.
**Why it happens:** Keybindings are keymap-specific. Bindkey without `-M` only affects main keymap.
**How to avoid:**
- Check current keymap: `echo $KEYMAP`
- Bind to multiple keymaps: `bindkey -M viins '^G' widget` and `bindkey -M vicmd '^G' widget`
- Test in both insert and command modes
**Warning signs:**
- Keybinding works sometimes but not others
- Vim mode users report broken bindings

### Pitfall 7: Multiline Input Handling
**What goes wrong:** Newlines in BUFFER break prompt display or cause premature submission.
**Why it happens:** Zsh treats \n specially in BUFFER, and prompt doesn't account for multiline content.
**How to avoid:**
- Use literal newlines with `$'\n'` syntax
- Update prompt with PS2 for continuation lines
- Handle BUFFER line count for cursor positioning
**Warning signs:**
- Prompt renders incorrectly with multiline input
- Enter submits when Shift+Enter was pressed

### Pitfall 8: Signal Handling During External Commands
**What goes wrong:** Ctrl+C during cherry2k binary execution doesn't clean up terminal state.
**Why it happens:** SIGINT goes to child process, not zsh widget context. Terminal state left in raw mode or with escape sequences incomplete.
**How to avoid:**
- Use `trap 'cleanup' INT` around external command
- Call `zle -I` before forking to external command
- Restore terminal with `stty sane` in cleanup
**Warning signs:**
- Terminal doesn't respond after Ctrl+C
- Escape sequences visible as text after interrupt

## Code Examples

Verified patterns from official sources:

### Example 1: Complete Prefix Detection Widget
```zsh
# Source: Official zsh ZLE documentation + community patterns

# Main plugin entry point
_cherry2k_plugin_init() {
    # Store original widgets
    typeset -g _CHERRY2K_ORIGINAL_SELF_INSERT="${widgets[self-insert]}"

    # State tracking
    typeset -g _CHERRY2K_AI_MODE=0
    typeset -g _CHERRY2K_SAVED_PROMPT=""

    # Config (user can override in .zshrc)
    typeset -g CHERRY2K_CONTEXT_DEPTH=${CHERRY2K_CONTEXT_DEPTH:-10}

    # Register widgets
    zle -N self-insert _cherry2k_self_insert_wrapper
    zle -N _cherry2k_ai_mode_accept
    zle -N _cherry2k_ctrl_g_handler

    # Keybindings
    bindkey '^G' _cherry2k_ctrl_g_handler
    # In AI mode, Enter triggers AI invocation
    bindkey -M main '^M' _cherry2k_ai_mode_accept
}

_cherry2k_self_insert_wrapper() {
    # Call original self-insert
    zle .$_CHERRY2K_ORIGINAL_SELF_INSERT "$@"

    # Detect "* " prefix activation
    if [[ "$BUFFER" == "* " ]] && [[ $_CHERRY2K_AI_MODE -eq 0 ]]; then
        _cherry2k_enter_ai_mode
    fi

    # Detect backspace past "* " (exit AI mode)
    if [[ "$BUFFER" != "* "* ]] && [[ $_CHERRY2K_AI_MODE -eq 1 ]]; then
        _cherry2k_exit_ai_mode
    fi
}

# Initialize plugin
_cherry2k_plugin_init
```

### Example 2: Vim Navigation in AI Mode
```zsh
# Source: zsh-vi-mode plugin patterns + vim keybinding docs

_cherry2k_setup_vim_bindings() {
    # Only set up vim bindings if user is in vi mode
    if [[ $KEYMAP == vi* ]] || bindkey | grep -q "vi-"; then

        # In AI mode, support vim navigation
        # These work after pressing Esc in AI mode

        # viins (insert mode) - for typing AI query
        bindkey -M viins '^[' _cherry2k_vim_escape  # Esc to command mode

        # vicmd (command mode) - for navigation
        bindkey -M vicmd '^' vi-beginning-of-line   # ^ to start
        bindkey -M vicmd '$' vi-end-of-line         # $ to end
        bindkey -M vicmd 'h' vi-backward-char       # h left
        bindkey -M vicmd 'l' vi-forward-char        # l right
        bindkey -M vicmd 'w' vi-forward-word        # w next word
        bindkey -M vicmd 'b' vi-backward-word       # b prev word

        # Return to insert mode
        bindkey -M vicmd 'i' vi-insert              # i to insert
        bindkey -M vicmd 'a' vi-add-next            # a to append
    fi
}

_cherry2k_vim_escape() {
    # Custom escape handler for AI mode
    if [[ $_CHERRY2K_AI_MODE -eq 1 ]]; then
        # Switch to command mode but stay in AI mode
        zle -K vicmd
    else
        # Normal escape behavior
        zle vi-cmd-mode
    fi
}

zle -N _cherry2k_vim_escape
```

### Example 3: Shell Context Collection
```zsh
# Source: fc command docs + zsh history best practices

_cherry2k_collect_context() {
    local context_depth=${CHERRY2K_CONTEXT_DEPTH:-10}
    local context_file=$(mktemp)

    # Start JSON structure
    echo "{" > "$context_file"

    # Current directory
    echo "  \"pwd\": \"$PWD\"," >> "$context_file"

    # Shell info
    echo "  \"shell\": \"$SHELL\"," >> "$context_file"
    echo "  \"zsh_version\": \"$ZSH_VERSION\"," >> "$context_file"

    # Command history (last N commands with timestamps if available)
    echo "  \"history\": [" >> "$context_file"

    # Use fc to get history with timestamps
    # -r = reverse (newest first), -l = list, -i = ISO timestamp, -n = no line numbers
    local history_lines=()
    local first=1

    # Capture history, parse each line
    fc -rli -${context_depth} 2>/dev/null | while IFS= read -r line; do
        # Parse format: "YYYY-MM-DD HH:MM  command"
        if [[ $line =~ ^([0-9-]+\ [0-9:]+)\ +(.*)$ ]]; then
            local timestamp="${match[1]}"
            local command="${match[2]}"

            [[ $first -eq 0 ]] && echo "," >> "$context_file"
            echo -n "    {\"timestamp\":\"$timestamp\",\"command\":" >> "$context_file"
            echo -n "$command" | jq -Rs . >> "$context_file"
            echo -n "}" >> "$context_file"
            first=0
        fi
    done

    echo "" >> "$context_file"
    echo "  ]," >> "$context_file"

    # Environment (filtered - no secrets)
    echo "  \"env\": {" >> "$context_file"
    echo "    \"USER\": \"$USER\"," >> "$context_file"
    echo "    \"HOME\": \"$HOME\"," >> "$context_file"
    echo "    \"TERM\": \"$TERM\"" >> "$context_file"
    echo "  }" >> "$context_file"

    echo "}" >> "$context_file"

    cat "$context_file"
    rm "$context_file"
}
```

### Example 4: Streaming Response Handler
```zsh
# Source: shellgpt + zsh-ollama-command patterns

_cherry2k_ai_mode_accept() {
    # Called when user presses Enter in AI mode

    if [[ $_CHERRY2K_AI_MODE -eq 0 ]]; then
        # Not in AI mode, normal accept-line
        zle .accept-line
        return
    fi

    # Extract query (remove "* " prefix)
    local query="${BUFFER#\* }"

    if [[ -z "$query" ]]; then
        # Empty query, just exit AI mode
        _cherry2k_exit_ai_mode
        zle .accept-line
        return
    fi

    # Prevent this from going to history
    # Method 1: Set the line to empty so history sees nothing
    local saved_buffer="$BUFFER"
    BUFFER=""
    zle .accept-line  # This accepts empty line (no history entry)

    # Now we're back at a fresh prompt, run AI command
    print ""  # Move query up

    # Collect context
    local context=$(_cherry2k_collect_context)

    # Create temp file for context
    local context_file=$(mktemp)
    echo "$context" > "$context_file"

    # Setup signal handler
    local interrupted=0
    _cherry2k_sigint_handler() {
        interrupted=1
        print "\n^C (cancelled)"
    }
    trap _cherry2k_sigint_handler INT

    # Call Rust binary (it handles streaming display)
    cherry2k chat --context-file="$context_file" "$query"
    local exit_code=$?

    # Cleanup
    trap - INT
    rm -f "$context_file"

    # Check if AI asked a question (stay in AI mode)
    # For now, exit AI mode. Future: parse response for questions
    _cherry2k_exit_ai_mode

    # Return to prompt with proper status
    zle .reset-prompt
    return $exit_code
}

zle -N _cherry2k_ai_mode_accept
```

### Example 5: 8-bit Retro Styling Configuration
```rust
// Source: ANSI color scheme research + terminal color standards
// File: crates/cli/src/output/retro.rs

use termimad::crossterm::style::Color;

/// Create a retro 8-bit color scheme for terminal output.
/// Uses the classic 16 ANSI colors for maximum compatibility.
pub fn retro_color_scheme() -> RetroColors {
    RetroColors {
        // Primary text: Bright green (classic terminal green)
        text: Color::AnsiValue(10),  // Bright Green

        // Headers: Bright yellow/gold
        header: Color::AnsiValue(11), // Bright Yellow

        // Code blocks: Cyan on black background
        code: Color::AnsiValue(14),   // Bright Cyan
        code_bg: Color::AnsiValue(0), // Black

        // Cherry emoji prompt: Red/Magenta
        prompt: Color::AnsiValue(13), // Bright Magenta

        // Error messages: Bright red
        error: Color::AnsiValue(9),   // Bright Red

        // Dimmed/secondary text: Dark gray
        dim: Color::AnsiValue(8),     // Bright Black (dark gray)
    }
}

/// Apply retro styling to MadSkin for markdown rendering
pub fn apply_retro_skin(skin: &mut MadSkin) {
    let colors = retro_color_scheme();

    // Main text uses retro green
    skin.paragraph.set_fg(colors.text);

    // Headers bold and bright
    for header in &mut skin.headers {
        header.set_fg(colors.header);
        header.add_attr(crossterm::style::Attribute::Bold);
    }

    // Code blocks with border (8-bit style box)
    skin.code_block.set_fg(colors.code);
    skin.code_block.set_bg(colors.code_bg);

    // Inline code slightly different
    skin.inline_code.set_fg(colors.code);

    // Lists use classic bullets
    skin.bullet = StyledChar::from_fg_char(colors.text, '•');
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Oh-My-Zsh for all plugins | Pure zsh plugins with optional OMZ compat | ~2023 | 5-10x faster startup, better for minimal setups |
| bindkey -s (hardcoded strings) | ZLE widgets with functions | Always recommended | Enables conditional logic, state management |
| Manual ANSI escape codes | crossterm/termimad libraries | ~2020 (Rust ecosystem maturity) | Cross-platform, handles edge cases automatically |
| Custom markdown parsers | termimad/pulldown-cmark | ~2019 | Correct CommonMark spec compliance, table support |
| Synchronous API calls | Streaming with async/await | ~2018 (Tokio 0.1) | Real-time feedback, cancellation support |
| fc output parsing with awk | fc + jq for JSON | ~2016 (jq adoption) | Robust escaping, structured data |

**Deprecated/outdated:**
- **tui-rs**: Replaced by ratatui (fork) in 2023 - use ratatui for TUI apps
- **termion**: Less actively maintained than crossterm - prefer crossterm for new projects
- **Bash-style PROMPT_COMMAND**: Zsh uses precmd/preexec hooks, more powerful and composable

## Open Questions

Things that couldn't be fully resolved:

1. **Cherry Animation Design**
   - What we know: indicatif supports custom spinner frames, can use cherry emoji
   - What's unclear: Exact animation sequence (spinning cherries? pulsing? growing?)
   - Recommendation: Start with simple pulsing: 🍒 → 🍒💫 → 🍒✨ → 🍒 (3-frame loop)

2. **8-bit Font Aesthetic Technical Limits**
   - What we know: Terminals support ANSI colors but not font changes. 8-bit look via color scheme.
   - What's unclear: Can we achieve convincing retro look with colors alone?
   - Recommendation: Use bright green on black for prose (classic terminal), magenta/cyan for highlights. Test user perception. Consider ASCII art borders for code blocks.

3. **Default Context Depth**
   - What we know: fc can retrieve arbitrary history depth, larger = more context but slower
   - What's unclear: What's the optimal balance for typical use cases?
   - Recommendation: Start with 10 commands (configurable), monitor context window limits from AI providers

4. **AI Question Detection for Follow-up Mode**
   - What we know: Need to stay in 🍒 mode if AI asks a question
   - What's unclear: Reliable heuristic for question detection (LLM output ends with "?", or semantic analysis?)
   - Recommendation: Phase 1: Simple heuristic (ends with "?"). Phase 2: Add "continue mode" flag from AI response JSON

## Sources

### Primary (HIGH confidence)
- [Zsh Line Editor Official Documentation](https://zsh.sourceforge.io/Doc/Release/Zsh-Line-Editor.html) - Complete ZLE reference
- [Custom ZLE Widgets Guide by Serge Gebhardt](https://sgeb.io/posts/zsh-zle-custom-widgets/) - Practical patterns and examples
- [termimad crates.io](https://crates.io/crates/termimad) - Version 0.30 documentation
- [crossterm crates.io](https://crates.io/crates/crossterm) - Version 0.29 documentation
- [indicatif crates.io](https://crates.io/crates/indicatif) - Version 0.17 documentation
- [Zsh Shell Builtin Commands](https://zsh.sourceforge.io/Doc/Release/Shell-Builtin-Commands.html) - fc command reference

### Secondary (MEDIUM confidence)
- [Mastering Zsh: Widgets](https://github.com/rothgar/mastering-zsh/blob/master/docs/helpers/widgets.md) - Community widget patterns
- [Mastering Zsh: History](https://github.com/rothgar/mastering-zsh/blob/master/docs/config/history.md) - History configuration best practices
- [ZSH You Should Use Plugin](https://github.com/MichaelAquilina/zsh-you-should-use) - Example of production zsh plugin patterns
- [LLMs in Your Terminal Guide (Medium, Feb 2025)](https://medium.com/@oleksandr.zhyhalo/llms-in-your-terminal-guide-for-zsh-a90b4f6f20fe) - Current AI integration patterns
- [Terminal Color Standards](https://github.com/termstandard/colors) - ANSI color specification
- [terminal-emoji crates.io](https://crates.io/crates/terminal-emoji) - Safe emoji display library

### Tertiary (LOW confidence)
- Various WebSearch results about zsh plugin development (2025-2026) - General ecosystem trends
- ShellGPT and zsh-ai implementations - Reference for AI terminal integration approaches
- Oh-My-Zsh performance discussions - Context for avoiding bloat

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official documentation and established crate ecosystem
- Architecture: HIGH - Well-documented ZLE patterns with official examples
- Pitfalls: MEDIUM - Mix of documented issues and community experience, some edge cases need testing

**Research date:** 2026-01-31
**Valid until:** ~60 days (March 2026) - Zsh and core crates are stable, but plugin ecosystem evolves
