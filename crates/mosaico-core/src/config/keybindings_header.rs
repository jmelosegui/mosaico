pub(super) fn header() -> &'static str {
    r##"# Mosaico keybindings
# Location: ~/.config/mosaico/keybindings.toml
#
# Each [[keybinding]] entry maps a key combination to an action.
#
# Available actions:
#   focus-left, focus-right, focus-up, focus-down,
#   move-left, move-right, move-up, move-down,
#   goto-workspace-1 .. goto-workspace-8,
#   goto-workspace-next, goto-workspace-prev (with [workspaces] wrap in
#   config.toml, past the last workspace they go round to the first),
#   send-to-workspace-1 .. send-to-workspace-8,
#   retile, toggle-monocle, close-focused, minimize-focused,
#   toggle-float, cycle-float, grow-width, shrink-width, grow-height,
#   shrink-height
#
# Available modifiers: alt, shift, ctrl, win
#
# Key names: A-Z, 0-9, F1-F24, Enter, Space, Tab, Escape,
#            Left, Right, Up, Down, Minus, Plus, Comma, Period

# Focus: Alt + H/J/K/L (vim-style spatial navigation)
"##
}
