/// Generates the default `config.toml` contents with explanatory comments.
///
/// This is used by `mosaico init` to create a starter config file that
/// users can immediately edit.
pub fn generate_config() -> String {
    r##"# Mosaico configuration
# Location: ~/.config/mosaico/config.toml

# Color theme. Controls border colors and status bar colors.
# Available: name = "catppuccin", flavor = mocha | macchiato | frappe | latte
[theme]
name = "catppuccin"
flavor = "mocha"

[layout]
# Gap in pixels between windows and screen edges.
gap = 8
# Ratio of space given to the first window in each split (0.0 to 1.0).
ratio = 0.5
# How windows are hidden during workspace switches.
# "cloak" (recommended): invisible via DWM, keeps taskbar icon.
# "hide": SW_HIDE, removes taskbar icon.
# "minimize": SW_MINIMIZE, keeps taskbar icon but shows as minimized.
hiding = "cloak"
# Default layout algorithm: "bsp", "vertical-stack", or "three-column".
default = "bsp"

[workspaces]
# How a workspace switch is applied across monitors.
# "per-monitor" (default): only the focused monitor switches.
# "global": all monitors switch in lockstep, like Windows virtual desktops.
mode = "per-monitor"

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

# Override theme border colors. Both fields accept a hex color
# ("#00b4d8") or a named theme color (blue, green, mauve, teal, etc.).
# focused: color while a tiled layout is active
# monocle: color while monocle mode is active
# [borders.colors]
# focused = "blue"
# monocle = "green"

[mouse]
# Move the cursor to the center of the focused window on keyboard navigation.
follows_focus = true
# Automatically focus the window under the cursor without clicking.
focus_follows_mouse = false

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
