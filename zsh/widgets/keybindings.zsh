# Cherry2K - Keybinding Configuration
# ZLE widgets for keyboard shortcuts
#
# This file contains keybinding handlers, primarily Ctrl+G for quick AI mode entry.

# ============================================================================
# Ctrl+G Handler
# ============================================================================

# Handle Ctrl+G to enter AI mode from anywhere
# - Empty prompt: enter AI mode with "* " prefix (ready to type query)
# - Non-empty prompt: prepend "* " to existing text and enter AI mode
_cherry2k_ctrl_g_handler() {
    if [[ $_CHERRY2K_AI_MODE -eq 1 ]]; then
        # Already in AI mode: exit
        _cherry2k_exit_ai_mode
        BUFFER=""
        zle -R
        return
    fi

    # Not in AI mode: enter AI mode
    if [[ -z "$BUFFER" ]]; then
        # Empty prompt: set up "* " and enter AI mode
        BUFFER="* "
        CURSOR=2
        _cherry2k_enter_ai_mode
    else
        # Non-empty: prepend "* " and enter AI mode
        BUFFER="* $BUFFER"
        CURSOR=$((CURSOR + 2))
        _cherry2k_enter_ai_mode
    fi
}

# ============================================================================
# Keybinding Setup
# ============================================================================

# Setup keybindings for all keymaps
_cherry2k_setup_keybindings() {
    # Register the widget
    zle -N _cherry2k_ctrl_g_handler

    # Bind Ctrl+G in main/emacs keymap
    bindkey '^G' _cherry2k_ctrl_g_handler

    # Bind Ctrl+G in vi keymaps (if available)
    # Use -M flag with silent failure in case vi mode not enabled
    bindkey -M viins '^G' _cherry2k_ctrl_g_handler 2>/dev/null
    bindkey -M vicmd '^G' _cherry2k_ctrl_g_handler 2>/dev/null
}
