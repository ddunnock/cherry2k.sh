# Cherry2K - AI Mode Widget
# ZLE widget for prefix detection and AI mode state management
#
# This widget wraps self-insert to detect "* " prefix and switch to AI mode.
# In AI mode, the prompt changes to cherry emoji to indicate the user is
# composing an AI query.

# ============================================================================
# State Variables
# ============================================================================

# AI mode state (0 = normal, 1 = AI mode)
typeset -g _CHERRY2K_AI_MODE=0

# Saved prompt for restoration when exiting AI mode
typeset -g _CHERRY2K_SAVED_PROMPT=""

# Original self-insert widget reference
typeset -g _CHERRY2K_ORIGINAL_SELF_INSERT=""

# Original backward-delete-char widget reference
typeset -g _CHERRY2K_ORIGINAL_BACKWARD_DELETE=""

# ============================================================================
# AI Mode Transition Functions
# ============================================================================

# Enter AI mode - called when user types "* " prefix
_cherry2k_enter_ai_mode() {
    # Already in AI mode, do nothing
    if [[ $_CHERRY2K_AI_MODE -eq 1 ]]; then
        return 0
    fi

    # Set AI mode flag
    _CHERRY2K_AI_MODE=1

    # Save original prompt for restoration
    _CHERRY2K_SAVED_PROMPT="$PROMPT"

    # Strip "* " from the left buffer (visual only)
    LBUFFER="${LBUFFER#\* }"

    # Set cherry emoji prompt
    PROMPT=$'\U1F352 '

    # Bind Enter to AI mode accept handler
    bindkey '^M' _cherry2k_ai_mode_accept

    # Trigger redisplay
    zle -R
}

# Exit AI mode - called when user backspaces past prefix
_cherry2k_exit_ai_mode() {
    # Not in AI mode, do nothing
    if [[ $_CHERRY2K_AI_MODE -eq 0 ]]; then
        return 0
    fi

    # Clear AI mode flag
    _CHERRY2K_AI_MODE=0

    # Restore original prompt
    PROMPT="$_CHERRY2K_SAVED_PROMPT"
    _CHERRY2K_SAVED_PROMPT=""

    # Restore original Enter keybinding
    bindkey '^M' accept-line

    # Trigger redisplay
    zle -R
}

# ============================================================================
# ZLE Widget Wrappers
# ============================================================================

# Self-insert wrapper - detects "* " prefix
_cherry2k_self_insert_wrapper() {
    # Call original self-insert first
    zle .self-insert "$@"

    # Not in AI mode: check for "* " prefix activation
    if [[ $_CHERRY2K_AI_MODE -eq 0 ]]; then
        # Detect exactly "* " (asterisk + space)
        if [[ "$BUFFER" == "* " ]]; then
            _cherry2k_enter_ai_mode
        fi
    fi
}

# Backward-delete wrapper - detects exit from AI mode
_cherry2k_backward_delete_wrapper() {
    # Call original backward-delete-char first
    zle .backward-delete-char "$@"

    # In AI mode: check if buffer is empty (user deleted everything)
    if [[ $_CHERRY2K_AI_MODE -eq 1 ]]; then
        # If buffer is empty, exit AI mode
        if [[ -z "$BUFFER" ]]; then
            _cherry2k_exit_ai_mode
        fi
    fi
}

# ============================================================================
# Ctrl+G Handler
# ============================================================================

# Handle Ctrl+G to toggle AI mode
_cherry2k_ctrl_g_handler() {
    if [[ $_CHERRY2K_AI_MODE -eq 0 ]]; then
        # Not in AI mode: enter AI mode
        if [[ -z "$BUFFER" ]]; then
            # Empty prompt: just enter AI mode with empty buffer
            _CHERRY2K_AI_MODE=1
            _CHERRY2K_SAVED_PROMPT="$PROMPT"
            PROMPT=$'\U1F352 '
            # Bind Enter to AI mode accept handler
            bindkey '^M' _cherry2k_ai_mode_accept
            zle -R
        else
            # Non-empty: prepend "* " and enter AI mode
            local current_buffer="$BUFFER"
            BUFFER=""
            _CHERRY2K_AI_MODE=1
            _CHERRY2K_SAVED_PROMPT="$PROMPT"
            PROMPT=$'\U1F352 '
            # Bind Enter to AI mode accept handler
            bindkey '^M' _cherry2k_ai_mode_accept
            BUFFER="$current_buffer"
            CURSOR="${#BUFFER}"
            zle -R
        fi
    else
        # In AI mode: exit
        _cherry2k_exit_ai_mode
        BUFFER=""
        zle -R
    fi
}

# ============================================================================
# Context Collection
# ============================================================================

# Collect shell context as JSON and write to temp file
# Returns: Path to temp file (caller is responsible for cleanup)
# Dependency: jq must be installed for JSON escaping
_cherry2k_collect_context() {
    local context_depth=${CHERRY2K_CONTEXT_DEPTH:-10}
    local context_file=$(mktemp)

    # Start JSON structure
    {
        echo "{"

        # Current directory
        echo "  \"pwd\": $(echo "$PWD" | jq -Rs .),"

        # Shell info
        echo "  \"shell\": $(echo "$SHELL" | jq -Rs .),"
        echo "  \"zsh_version\": $(echo "$ZSH_VERSION" | jq -Rs .),"

        # Command history (last N commands with timestamps if available)
        echo "  \"history\": ["

        # Use fc to get history with timestamps
        # -r = reverse (newest first), -l = list, -i = ISO timestamp
        local first=1
        fc -rli -${context_depth} 2>/dev/null | while IFS= read -r line; do
            # Parse format: "N  YYYY-MM-DD HH:MM  command" or "N  command"
            # Try to extract timestamp and command
            if [[ $line =~ ^[[:space:]]*[0-9]+[[:space:]]+([0-9-]+\ [0-9:]+)[[:space:]]+(.*)$ ]]; then
                local timestamp="${match[1]}"
                local command="${match[2]}"
                [[ $first -eq 0 ]] && echo ","
                echo -n "    {\"timestamp\":$(echo "$timestamp" | jq -Rs .),\"command\":$(echo "$command" | jq -Rs .)}"
                first=0
            elif [[ $line =~ ^[[:space:]]*[0-9]+[[:space:]]+(.*)$ ]]; then
                # No timestamp, just command
                local command="${match[1]}"
                [[ $first -eq 0 ]] && echo ","
                echo -n "    {\"command\":$(echo "$command" | jq -Rs .)}"
                first=0
            fi
        done
        echo ""
        echo "  ],"

        # Environment (filtered - no secrets)
        echo "  \"env\": {"
        echo "    \"USER\": $(echo "$USER" | jq -Rs .),"
        echo "    \"HOME\": $(echo "$HOME" | jq -Rs .),"
        echo "    \"TERM\": $(echo "$TERM" | jq -Rs .)"
        echo "  }"

        echo "}"
    } > "$context_file"

    # Return path to temp file
    echo "$context_file"
}

# ============================================================================
# AI Mode Accept (Enter Handler)
# ============================================================================

# Handle Enter press in AI mode - invokes AI and displays response
_cherry2k_ai_mode_accept() {
    # If not in AI mode, call original accept-line
    if [[ $_CHERRY2K_AI_MODE -eq 0 ]]; then
        zle .accept-line
        return
    fi

    # Extract query from buffer
    local query="$BUFFER"

    # If query is empty, exit AI mode and accept empty line
    if [[ -z "$query" ]]; then
        _cherry2k_exit_ai_mode
        zle .accept-line
        return
    fi

    # History prevention: Clear buffer before accept-line so history sees empty line
    local saved_query="$query"
    BUFFER=""
    zle .accept-line

    # Print newline to move query visually up
    print ""

    # Collect context to temp file
    local context_file=$(_cherry2k_collect_context)

    # Setup SIGINT trap for cleanup during streaming
    local _cherry2k_context_file_for_cleanup="$context_file"
    trap '_cherry2k_cleanup_on_sigint "$_cherry2k_context_file_for_cleanup"' INT

    # Invoke cherry2k chat with context file
    cherry2k chat --context-file="$context_file" "$saved_query"
    local exit_code=$?

    # Cleanup: remove context file, restore trap
    rm -f "$context_file"
    trap - INT

    # Exit AI mode
    _cherry2k_exit_ai_mode

    # Reset prompt
    zle .reset-prompt 2>/dev/null || true

    return $exit_code
}

# Cleanup handler for SIGINT during AI invocation
_cherry2k_cleanup_on_sigint() {
    local context_file="$1"
    print "\n^C (cancelled)"
    rm -f "$context_file"
    trap - INT
    _cherry2k_exit_ai_mode
    zle .reset-prompt 2>/dev/null || true
}

# ============================================================================
# Plugin Initialization
# ============================================================================

# Initialize the plugin - register widgets and keybindings
_cherry2k_plugin_init() {
    # Store original widget references (use .self-insert for builtin)
    _CHERRY2K_ORIGINAL_SELF_INSERT="${widgets[self-insert]}"
    _CHERRY2K_ORIGINAL_BACKWARD_DELETE="${widgets[backward-delete-char]}"

    # Initialize state
    _CHERRY2K_AI_MODE=0
    _CHERRY2K_SAVED_PROMPT=""

    # Register custom widgets
    zle -N self-insert _cherry2k_self_insert_wrapper
    zle -N backward-delete-char _cherry2k_backward_delete_wrapper
    zle -N _cherry2k_ctrl_g_handler
    zle -N _cherry2k_ai_mode_accept

    # Bind Ctrl+G to toggle AI mode
    bindkey '^G' _cherry2k_ctrl_g_handler

    # Also bind for vim mode if active
    if [[ -n "$KEYMAP" ]]; then
        bindkey -M viins '^G' _cherry2k_ctrl_g_handler 2>/dev/null
        bindkey -M vicmd '^G' _cherry2k_ctrl_g_handler 2>/dev/null
    fi
}
