# Cherry2K - Zsh Terminal AI Assistant
# Main plugin entry point
#
# Installation:
#   Add to .zshrc: source /path/to/cherry2k/zsh/cherry2k.plugin.zsh
#
# Usage:
#   Type "* " at the prompt to enter AI mode (cherry emoji prompt)
#   Backspace to exit AI mode
#   Ctrl+G to toggle AI mode directly

# Guard against double-sourcing
if [[ -n "$_CHERRY2K_LOADED" ]]; then
    return 0
fi
typeset -g _CHERRY2K_LOADED=1

# Plugin directory (for relative sourcing)
typeset -g _CHERRY2K_PLUGIN_DIR="${0:A:h}"

# Source widget files
source "${_CHERRY2K_PLUGIN_DIR}/widgets/ai-mode.zsh"

# Initialize plugin
_cherry2k_plugin_init
