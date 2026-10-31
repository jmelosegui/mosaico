pub(super) fn entries() -> &'static str {
    r##"[[keybinding]]
action = "focus-down"
key = "J"
modifiers = ["alt"]

[[keybinding]]
action = "focus-up"
key = "K"
modifiers = ["alt"]

[[keybinding]]
action = "focus-right"
key = "L"
modifiers = ["alt"]

[[keybinding]]
action = "focus-left"
key = "H"
modifiers = ["alt"]

# Move: Alt + Shift + H/J/K/L (swap or cross-monitor)
[[keybinding]]
action = "move-down"
key = "J"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "move-up"
key = "K"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "move-right"
key = "L"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "move-left"
key = "H"
modifiers = ["alt", "shift"]

# Re-apply layout: Alt + Shift + R
[[keybinding]]
action = "retile"
key = "R"
modifiers = ["alt", "shift"]

# Toggle monocle mode: Alt + T
[[keybinding]]
action = "toggle-monocle"
key = "T"
modifiers = ["alt"]

# Float the focused window above the tiles, or tile it again: Alt + F
[[keybinding]]
action = "toggle-float"
key = "F"
modifiers = ["alt"]

# Focus the next floating window: Alt + Shift + F
[[keybinding]]
action = "cycle-float"
key = "F"
modifiers = ["alt", "shift"]

# Resize the focused floating window: Alt + Period/Comma for its width,
# Alt + Shift + Period/Comma for its height
[[keybinding]]
action = "grow-width"
key = "Period"
modifiers = ["alt"]

[[keybinding]]
action = "shrink-width"
key = "Comma"
modifiers = ["alt"]

[[keybinding]]
action = "grow-height"
key = "Period"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "shrink-height"
key = "Comma"
modifiers = ["alt", "shift"]

# Close focused window: Alt + Q
[[keybinding]]
action = "close-focused"
key = "Q"
modifiers = ["alt"]

# Minimize focused window: Alt + M
[[keybinding]]
action = "minimize-focused"
key = "M"
modifiers = ["alt"]

# Cycle layout: Alt + N
[[keybinding]]
action = "cycle-layout"
key = "N"
modifiers = ["alt"]

# Pause/unpause all hotkeys: Alt + Shift + P
[[keybinding]]
action = "toggle-pause"
key = "P"
modifiers = ["alt", "shift"]
"##
}
