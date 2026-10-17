/// Generates the default `config.toml` contents with explanatory comments.
///
/// This is used by `mosaico init` to create a starter config file that
/// users can immediately edit.
pub fn generate_config() -> String {
    r##"# Mosaico configuration
# Location: ~/.config/mosaico/config.toml

# Color theme. Controls border colors and status bar colors.
# Available themes:
#   name = "catppuccin"   flavor = mocha | macchiato | frappe | latte
#   name = "rose-pine"    flavor = main | moon | dawn
#   name = "tokyo-night"  flavor = night | storm | day
[theme]
name = "catppuccin"
flavor = "mocha"

[layout]
# Gap in pixels between windows and screen edges.
gap = 8
# Ratio of space given to the first window in each split (0.0 to 1.0).
ratio = 0.5
# How windows are hidden during workspace switches.
# "cloak" (recommended): invisible, keeps its taskbar icon.
# "hide": fully hidden, removes its taskbar icon.
# "minimize": minimized, keeps its taskbar icon.
hiding = "cloak"
# Default layout algorithm: "bsp", "vertical-stack", or "three-column".
default = "bsp"

[workspaces]
# How a workspace switch is applied across monitors.
# "per-monitor" (default): only the focused monitor switches.
# "global": all monitors switch in lockstep, like Windows virtual desktops.
mode = "per-monitor"
# Whether stepping past the last workspace, with goto-workspace-next or the
# wheel over the bar, goes back to the first, and before the first to the
# last. false stops at either end.
wrap = false

# Per-workspace layout overrides (workspace number 1-8).
# Available layouts: "bsp", "vertical-stack", "three-column".
# Workspaces without an entry use the [layout] default above.
# [workspaces.layouts]
# 1 = "vertical-stack"
# 3 = "three-column"

[borders]
# Border width in pixels around the focused window.
width = 4
# Corner style for borders and tiled windows: "square", "small", or "round".
corner_style = "small"

# Override theme border colors. Each field accepts a hex color
# ("#00b4d8") or a named theme color (blue, green, mauve, teal, etc.).
# focused:   color while a tiled layout is active
# monocle:   color while monocle mode is active
# unfocused: color drawn around unfocused tiled windows. Set to "none"
#            to disable unfocused borders entirely; leave empty (or
#            unset) to use the theme's muted gray default.
# [borders.colors]
# focused = "blue"
# monocle = "green"
# unfocused = "none"

[mouse]
# Move the cursor to the center of the focused window on keyboard navigation.
follows_focus = true
# Automatically focus the window under the cursor without clicking.
focus_follows_mouse = false
# Drop a tiled window you drag by its title bar on another tile to swap the
# two, or on another monitor to put it before the tile under the pointer.
# false: a dropped window goes back to its slot, or joins the end of the
# other monitor's windows.
drag_to_swap = true

[animations]
# Window transitions: windows move to their new place over a short time
# when they swap, move or resize, and when the layout changes, instead of
# jumping there.
enabled = true
# How long a transition takes, in milliseconds (0 to 1000).
duration = 180
# How windows speed up and slow down on the way: "linear", "ease-out-quad",
# "ease-out-cubic", "ease-in-out-cubic", "ease-out-expo", or
# "cubic-bezier(x1, y1, x2, y2)" with all four numbers from 0 to 1.
easing = "ease-out-cubic"
# Snap windows into place while animations are turned off in the system's
# settings ("Animation effects" on Windows), and in a remote session.
follow_system = true

[logging]
# Enable file logging to ~/.config/mosaico/logs/mosaico.log.
enabled = false
# Minimum log level: "debug", "info", "warn", or "error".
level = "info"
# Maximum log file size in MB before rotation.
max_file_mb = 10
"##
    .to_string()
}
