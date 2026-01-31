# Cherry2K - AI Mode Widget
# ZLE widget for prefix detection and AI mode state management
#
# This widget wraps self-insert to detect "* " prefix and switch to AI mode.
# In AI mode, the prompt changes to cherry emoji to indicate the user is
# composing an AI query.

# ============================================================================
# State Variables
# ============================================================================

typeset -g _CHERRY2K_AI_MODE=0           # 0 = normal, 1 = AI mode
typeset -g _CHERRY2K_SAVED_PROMPT=""     # Original prompt for restoration

# ============================================================================
# AI Mode Transition Functions
# ============================================================================

# Enter AI mode - called when user types "* " prefix
_cherry2k_enter_ai_mode() {
    [[ $_CHERRY2K_AI_MODE -eq 1 ]] && return 0

    _CHERRY2K_AI_MODE=1
    _CHERRY2K_SAVED_PROMPT="$PROMPT"
    LBUFFER="${LBUFFER#\* }"
    PROMPT=$'\U1F352 '
    bindkey '^M' _cherry2k_ai_mode_accept
    zle -R
}

# Exit AI mode - called when user backspaces past prefix
_cherry2k_exit_ai_mode() {
    [[ $_CHERRY2K_AI_MODE -eq 0 ]] && return 0

    _CHERRY2K_AI_MODE=0
    PROMPT="$_CHERRY2K_SAVED_PROMPT"
    _CHERRY2K_SAVED_PROMPT=""
    bindkey '^M' accept-line
    zle -R
}

# ============================================================================
# ZLE Widget Wrappers
# ============================================================================

# Self-insert wrapper - detects "* " prefix to enter AI mode
_cherry2k_self_insert_wrapper() {
    zle .self-insert "$@"

    if [[ $_CHERRY2K_AI_MODE -eq 0 && "$BUFFER" == "* " ]]; then
        _cherry2k_enter_ai_mode
    fi
}

# Backward-delete wrapper - exits AI mode when buffer becomes empty
_cherry2k_backward_delete_wrapper() {
    zle .backward-delete-char "$@"

    if [[ $_CHERRY2K_AI_MODE -eq 1 && -z "$BUFFER" ]]; then
        _cherry2k_exit_ai_mode
    fi
}

# ============================================================================
# Context Collection
# ============================================================================

# Collect shell context as JSON. Returns path to temp file (caller must cleanup).
# Requires: jq
_cherry2k_collect_context() {
    local context_depth=${CHERRY2K_CONTEXT_DEPTH:-10}
    local context_file
    context_file=$(mktemp)

    # Start JSON structure
    {
        echo "{"

        # Current directory
        echo "  \"pwd\": $(echo "$PWD" | jq -Rs .),"

        # Shell info
        echo "  \"shell\": $(echo "$SHELL" | jq -Rs .),"
        echo "  \"zsh_version\": $(echo "$ZSH_VERSION" | jq -Rs .),"

        # Command history (last N commands, fc -rli: reverse, list, ISO timestamp)
        echo "  \"history\": ["
        local first=1
        fc -rli -${context_depth} 2>/dev/null | while IFS= read -r line; do
            # Format: "N  YYYY-MM-DD HH:MM  command" or "N  command"
            if [[ $line =~ ^[[:space:]]*[0-9]+[[:space:]]+([0-9-]+\ [0-9:]+)[[:space:]]+(.*)$ ]]; then
                local timestamp="${match[1]}" command="${match[2]}"
                [[ $first -eq 0 ]] && echo ","
                echo -n "    {\"timestamp\":$(echo "$timestamp" | jq -Rs .),\"command\":$(echo "$command" | jq -Rs .)}"
                first=0
            elif [[ $line =~ ^[[:space:]]*[0-9]+[[:space:]]+(.*)$ ]]; then
                local command="${match[1]}"
                [[ $first -eq 0 ]] && echo ","
                echo -n "    {\"command\":$(echo "$command" | jq -Rs .)}"
                first=0
            fi
        done
        echo ""
        echo "  ],"

        # Environment (safe subset only)
        echo "  \"env\": {"
        echo "    \"USER\": $(echo "$USER" | jq -Rs .),"
        echo "    \"HOME\": $(echo "$HOME" | jq -Rs .),"
        echo "    \"TERM\": $(echo "$TERM" | jq -Rs .)"
        echo "  }"
        echo "}"
    } > "$context_file"

    echo "$context_file"
}

# ============================================================================
# AI Mode Accept (Enter Handler)
# ============================================================================

# Handle Enter press in AI mode - invokes AI and displays response
_cherry2k_ai_mode_accept() {
    if [[ $_CHERRY2K_AI_MODE -eq 0 ]]; then
        zle .accept-line
        return
    fi

    local query="$BUFFER"
    if [[ -z "$query" ]]; then
        _cherry2k_exit_ai_mode
        zle .accept-line
        return
    fi

    # Clear buffer before accept-line to prevent history recording
    BUFFER=""
    zle .accept-line
    print ""

    local context_file
    context_file=$(_cherry2k_collect_context)
    trap '_cherry2k_cleanup_on_sigint "$context_file"' INT

    cherry2k chat --context-file="$context_file" "$query"
    local exit_code=$?

    rm -f "$context_file"
    trap - INT
    _cherry2k_exit_ai_mode
    zle .reset-prompt 2>/dev/null || true

    return $exit_code
}

# SIGINT handler - cleanup and exit AI mode
_cherry2k_cleanup_on_sigint() {
    print "\n^C (cancelled)"
    rm -f "$1"
    trap - INT
    _cherry2k_exit_ai_mode
    zle .reset-prompt 2>/dev/null || true
}

# ============================================================================
# Plugin Initialization
# ============================================================================

# Initialize the plugin - register widgets and keybindings
_cherry2k_plugin_init() {
    _CHERRY2K_AI_MODE=0
    _CHERRY2K_SAVED_PROMPT=""

    # Register custom widgets
    zle -N self-insert _cherry2k_self_insert_wrapper
    zle -N backward-delete-char _cherry2k_backward_delete_wrapper
    zle -N _cherry2k_ai_mode_accept

    _cherry2k_setup_keybindings
    _cherry2k_setup_vim_bindings
}
