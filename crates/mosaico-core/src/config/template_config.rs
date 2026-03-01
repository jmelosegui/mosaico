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

[borders]
# Border width in pixels around the focused window.
width = 4
# Corner style for borders and tiled windows: "square", "small", or "round".
corner_style = "small"
# Override theme border colors (hex or named: blue, green, mauve, etc.):
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
