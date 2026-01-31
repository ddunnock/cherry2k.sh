# Cherry2K - Vim Mode Navigation
# Support for vi-mode users in AI mode
#
# This file provides vim keybinding support for AI mode input.
# Standard vim navigation (^, $, h, l, w, b) already works natively in vi mode.
# We only need to handle Esc specially to stay in AI mode when switching to command mode.

# ============================================================================
# Vim Escape Handler
# ============================================================================

# Handle Escape in AI mode - switch to command mode but stay in AI mode
_cherry2k_vim_escape() {
    if [[ $_CHERRY2K_AI_MODE -eq 1 ]]; then
        # In AI mode: switch to command mode but don't exit AI mode
        zle -K vicmd
    else
        # Not in AI mode: standard vi-cmd-mode behavior
        zle vi-cmd-mode
    fi
}

# ============================================================================
# Vim Mode Setup
# ============================================================================

# Setup vim bindings for AI mode (only if user has vi mode enabled)
_cherry2k_setup_vim_bindings() {
    # Check if vi mode is active
    # Method 1: Check if 'vi' option is set
    # Method 2: Check if viins keymap exists and is configured
    if [[ -o vi ]] || bindkey -lL main 2>/dev/null | grep -q 'viins'; then
        # Register the vim escape widget
        zle -N _cherry2k_vim_escape

        # Bind Escape in vi insert mode to our handler
        # This allows Esc to switch to command mode while staying in AI mode
        bindkey -M viins '^[' _cherry2k_vim_escape

        # Standard vim navigation in command mode already works:
        # ^ (vi-first-non-blank), $ (vi-end-of-line), h, l, w, b, etc.
        # No additional bindings needed for these.
    fi
}
