# Cherry2K - Vim Mode Navigation
# Escape handling for vi-mode users in AI mode.
# Standard vim navigation (^, $, h, l, w, b) works natively.

# Escape in AI mode switches to command mode without exiting AI mode
_cherry2k_vim_escape() {
    if [[ $_CHERRY2K_AI_MODE -eq 1 ]]; then
        zle -K vicmd
    else
        zle vi-cmd-mode
    fi
}

# Setup vim bindings if vi mode is enabled
_cherry2k_setup_vim_bindings() {
    if [[ -o vi ]] || bindkey -lL main 2>/dev/null | grep -q 'viins'; then
        zle -N _cherry2k_vim_escape
        bindkey -M viins '^[' _cherry2k_vim_escape
    fi
}
