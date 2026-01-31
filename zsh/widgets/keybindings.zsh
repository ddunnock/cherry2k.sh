# Cherry2K - Keybinding Configuration
# ZLE widgets for keyboard shortcuts
#
# This file contains keybinding handlers, primarily Ctrl+G for quick AI mode entry.

# ============================================================================
# Ctrl+G Handler
# ============================================================================

# Toggle AI mode. If in AI mode, exit. Otherwise enter with current buffer.
_cherry2k_ctrl_g_handler() {
    if [[ $_CHERRY2K_AI_MODE -eq 1 ]]; then
        _cherry2k_exit_ai_mode
        BUFFER=""
        zle -R
        return
    fi

    # Enter AI mode, prepending "* " trigger
    BUFFER="* $BUFFER"
    CURSOR=$((${#BUFFER}))
    _cherry2k_enter_ai_mode
}

# ============================================================================
# Keybinding Setup
# ============================================================================

# Register Ctrl+G in main and vi keymaps
_cherry2k_setup_keybindings() {
    zle -N _cherry2k_ctrl_g_handler
    bindkey '^G' _cherry2k_ctrl_g_handler
    bindkey -M viins '^G' _cherry2k_ctrl_g_handler 2>/dev/null
    bindkey -M vicmd '^G' _cherry2k_ctrl_g_handler 2>/dev/null
}
