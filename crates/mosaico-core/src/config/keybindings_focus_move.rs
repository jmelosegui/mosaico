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
"##
}
