/// Generates the default `bar.toml` contents with explanatory comments.
///
/// This is used by `mosaico init` to create a starter bar configuration file.
pub fn generate_bar() -> String {
    "# Mosaico status bar\n\
# Location: ~/.config/mosaico/bar.toml\n\
\n\
# Whether the status bar is displayed.\n\
enabled = true\n\
# Bar height in pixels. Increase for high-DPI displays.\n\
height = 64\n\
# Font family name. A Nerd Font is required for widget icons.\n\
# See https://www.nerdfonts.com/ for installation instructions.\n\
font = \"CaskaydiaCove Nerd Font\"\n\
# Font size in pixels.\n\
font_size = 24\n\
# Render bar text in bold.\n\
font_bold = true\n\
# Render bar text in italic.\n\
font_italic = false\n\
# Render bar text with underline.\n\
font_underline = false\n\
# Horizontal padding at the bar edges in pixels.\n\
padding = 8\n\
# Horizontal padding inside each widget pill in pixels.\n\
pill_padding = 12\n\
# Corner radius for pill backgrounds (0 = square corners).\n\
pill_radius = 4\n\
# Border width for pill backgrounds in pixels. 0 = no border.\n\
pill_border_width = 2\n\
# Gap between pills in pixels.\n\
item_gap = 10\n\
# Gap between individual workspace number pills in pixels.\n\
workspace_gap = 4\n\
# Separator string between widget groups (empty string = no separator).\n\
separator = \"\"\n\
# Background opacity as a percentage (0 = fully transparent, 100 = opaque).\n\
background_opacity = 0\n\
# Which monitors to show the bar on (0-indexed). Empty = all monitors.\n\
# Example: monitors = [0] shows the bar only on the primary monitor.\n\
monitors = []\n\
\n\
# Override individual theme colors (hex or named: blue, green, mauve, etc.).\n\
# [colors]\n\
# background = \"#1e1e2e\"\n\
# foreground = \"#89b4fa\"\n\
# active_workspace = \"#435375\"\n\
# active_workspace_text = \"#cdd6f4\"\n\
# inactive_workspace = \"#89b4fa\"\n\
# separator = \"#45475a\"\n\
# accent = \"#a6e3a1\"\n\
# widget_background = \"#313244\"\n\
# pill_border = \"#89b4fa\"\n\
\n\
# Left-side widgets (rendered left-to-right).\n\
# Set enabled = false to hide a widget without removing it.\n\
# Set icon = \"\" to hide the icon, or use any Nerd Font glyph.\n\
\n\
[[left]]\n\
type = \"workspaces\"\n\
# enabled = true\n\
\n\
[[left]]\n\
type = \"active_window\"\n\
# enabled = true\n\
\n\
[[left]]\n\
type = \"layout\"\n\
# enabled = true\n\
icon = \"\\uF009\"\n\
\n\
# Right-side widgets (rendered right-to-left).\n\
# Set enabled = false to hide a widget without removing it.\n\
\n\
[[right]]\n\
type = \"clock\"\n\
# enabled = true\n\
format = \"%H:%M:%S\"\n\
icon = \"\\uF017\"\n\
\n\
[[right]]\n\
type = \"date\"\n\
# enabled = true\n\
format = \"%A %d %B %Y\"\n\
icon = \"\\uF073\"\n\
\n\
[[right]]\n\
type = \"ram\"\n\
# enabled = true\n\
icon = \"\\uF2DB\"\n\
\n\
[[right]]\n\
type = \"cpu\"\n\
# enabled = true\n\
icon = \"\\uF085\"\n\
\n\
[[right]]\n\
type = \"update\"\n\
# enabled = true  # auto-hidden when no update is available\n\
icon = \"\\uF019\"\n\
color = \"green\"  # custom color for text and border (hex or named)\n\
\n\
# Media widget -- shows currently playing track from Spotify, YouTube Music, etc.\n\
# Auto-hidden when nothing is playing.\n\
# [[right]]\n\
# type = \"media\"\n\
# icon = \"\\uF001\"\n\
# max_length = 40  # truncate long titles with \"...\"\n"
        .to_string()
}
