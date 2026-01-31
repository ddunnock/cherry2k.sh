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
            zle -R
        else
            # Non-empty: prepend "* " and enter AI mode
            local current_buffer="$BUFFER"
            BUFFER=""
            _CHERRY2K_AI_MODE=1
            _CHERRY2K_SAVED_PROMPT="$PROMPT"
            PROMPT=$'\U1F352 '
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

    # Bind Ctrl+G to toggle AI mode
    bindkey '^G' _cherry2k_ctrl_g_handler

    # Also bind for vim mode if active
    if [[ -n "$KEYMAP" ]]; then
        bindkey -M viins '^G' _cherry2k_ctrl_g_handler 2>/dev/null
        bindkey -M vicmd '^G' _cherry2k_ctrl_g_handler 2>/dev/null
    fi
}
