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

[borders]
# Border width in pixels around the focused window.
width = 4
# Override theme border colors (hex or named: blue, green, mauve, etc.):
# focused = "blue"
# monocle = "green"

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

/// Generates the default `keybindings.toml` contents with explanatory comments.
///
/// This is used by `mosaico init` to create a starter keybindings file.
pub fn generate_keybindings() -> String {
    r##"# Mosaico keybindings
# Location: ~/.config/mosaico/keybindings.toml
#
# Each [[keybinding]] entry maps a key combination to an action.
#
# Available actions:
#   focus-left, focus-right, focus-up, focus-down,
#   move-left, move-right, move-up, move-down,
#   goto-workspace-1 .. goto-workspace-8,
#   send-to-workspace-1 .. send-to-workspace-8,
#   retile, toggle-monocle, close-focused, minimize-focused
#
# Available modifiers: alt, shift, ctrl, win
#
# Key names: A-Z, 0-9, F1-F24, Enter, Space, Tab, Escape,
#            Left, Right, Up, Down, Minus, Plus, Comma, Period

# Focus: Alt + H/J/K/L (vim-style spatial navigation)
[[keybinding]]
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

# Workspaces: Alt + 1..8 (switch), Alt + Shift + 1..8 (send window)
[[keybinding]]
action = "goto-workspace-1"
key = "1"
modifiers = ["alt"]

[[keybinding]]
action = "goto-workspace-2"
key = "2"
modifiers = ["alt"]

[[keybinding]]
action = "goto-workspace-3"
key = "3"
modifiers = ["alt"]

[[keybinding]]
action = "goto-workspace-4"
key = "4"
modifiers = ["alt"]

[[keybinding]]
action = "goto-workspace-5"
key = "5"
modifiers = ["alt"]

[[keybinding]]
action = "goto-workspace-6"
key = "6"
modifiers = ["alt"]

[[keybinding]]
action = "goto-workspace-7"
key = "7"
modifiers = ["alt"]

[[keybinding]]
action = "goto-workspace-8"
key = "8"
modifiers = ["alt"]

[[keybinding]]
action = "send-to-workspace-1"
key = "1"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "send-to-workspace-2"
key = "2"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "send-to-workspace-3"
key = "3"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "send-to-workspace-4"
key = "4"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "send-to-workspace-5"
key = "5"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "send-to-workspace-6"
key = "6"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "send-to-workspace-7"
key = "7"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "send-to-workspace-8"
key = "8"
modifiers = ["alt", "shift"]
"##
    .to_string()
}

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
color = \"green\"  # custom color for text and border (hex or named)\n"
        .to_string()
}

/// Generates the default `rules.toml` contents with explanatory comments.
///
/// This is used by `mosaico init` to create a starter rules file.
pub fn generate_rules() -> String {
    r#"# Mosaico window rules
# Location: ~/.config/mosaico/rules.toml
#
# Rules control which windows are tiled. They are evaluated in order;
# the first matching rule wins. Unmatched windows are tiled by default.
#
# Fields:
#   match_class  - exact class name match (case-insensitive)
#   match_title  - substring match on the window title (case-insensitive)
#   manage       - true to tile, false to ignore
#
# Tip: run `mosaico debug list` to see each window's class name.

# UWP apps (Settings, Store, etc.) enforce their own size constraints
# and don't behave well when tiled.
[[rule]]
match_class = "ApplicationFrameWindow"
manage = false

# GPG passphrase prompt — small modal dialog, should not be tiled.
[[rule]]
match_title = "pinentry"
manage = false

# Add your own rules below. Examples:
#
# [[rule]]
# match_class = "TaskManagerWindow"
# manage = false
#
# [[rule]]
# match_title = "Popup"
# manage = false
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_template_parses_as_valid_config() {
        // Arrange
        let toml_str = generate_config();

        // Act
        let result: Result<crate::Config, _> = toml::from_str(&toml_str);

        // Assert
        assert!(
            result.is_ok(),
            "config template is not valid TOML: {result:?}"
        );
    }

    #[test]
    fn config_template_matches_default_values() {
        // Arrange
        let toml_str = generate_config();

        // Act
        let mut config: crate::Config = toml::from_str(&toml_str).unwrap();
        config.validate();

        // Assert
        let mut defaults = crate::Config::default();
        defaults.validate();
        assert_eq!(config.theme, defaults.theme);
        assert_eq!(config.theme.name, "catppuccin");
        assert_eq!(config.theme.flavor, "mocha");
        assert_eq!(config.layout.gap, defaults.layout.gap);
        assert_eq!(config.layout.ratio, defaults.layout.ratio);
        assert_eq!(config.borders.width, defaults.borders.width);
        // Border colors should resolve from the default Mocha theme.
        assert_eq!(config.borders.focused, defaults.borders.focused);
        assert_eq!(config.borders.monocle, defaults.borders.monocle);
    }

    #[test]
    fn keybindings_template_parses_correctly() {
        // Arrange
        let toml_str = generate_keybindings();

        // Act
        let result: Result<super::super::KeybindingsFile, _> = toml::from_str(&toml_str);

        // Assert
        assert!(
            result.is_ok(),
            "keybindings template is not valid TOML: {result:?}"
        );
    }

    #[test]
    fn keybindings_template_matches_defaults() {
        // Arrange
        let toml_str = generate_keybindings();

        // Act
        let file: super::super::KeybindingsFile = toml::from_str(&toml_str).unwrap();

        // Assert
        let defaults = crate::config::keybinding::defaults();
        assert_eq!(file.keybinding.len(), defaults.len());
    }

    #[test]
    fn rules_template_parses_correctly() {
        // Arrange
        let toml_str = generate_rules();

        // Act
        let result: Result<super::super::RulesFile, _> = toml::from_str(&toml_str);

        // Assert
        assert!(
            result.is_ok(),
            "rules template is not valid TOML: {result:?}"
        );
    }

    #[test]
    fn rules_template_matches_defaults() {
        // Arrange
        let toml_str = generate_rules();

        // Act
        let file: super::super::RulesFile = toml::from_str(&toml_str).unwrap();

        // Assert
        let defaults = crate::config::default_rules();
        assert_eq!(file.rule.len(), defaults.len());
    }

    #[test]
    fn bar_template_parses_as_valid_bar_config() {
        // Arrange
        let toml_str = generate_bar();

        // Act
        let result: Result<crate::BarConfig, _> = toml::from_str(&toml_str);

        // Assert
        assert!(result.is_ok(), "bar template is not valid TOML: {result:?}");
    }

    #[test]
    fn bar_template_matches_default_values() {
        // Arrange
        let toml_str = generate_bar();

        // Act
        let mut config: crate::BarConfig = toml::from_str(&toml_str).unwrap();
        config.validate();

        // Assert
        let mut defaults = crate::BarConfig::default();
        defaults.validate();
        assert_eq!(config.height, defaults.height);
        assert_eq!(config.font, defaults.font);
        assert_eq!(config.font_size, defaults.font_size);
        assert_eq!(config.padding, defaults.padding);
        assert_eq!(config.pill_padding, defaults.pill_padding);
        assert_eq!(config.pill_radius, defaults.pill_radius);
        assert_eq!(config.item_gap, defaults.item_gap);
        assert_eq!(config.separator, defaults.separator);
        assert_eq!(config.background_opacity, defaults.background_opacity);
        assert_eq!(config.monitors, defaults.monitors);
        assert_eq!(config.left.len(), defaults.left.len());
        assert_eq!(config.right.len(), defaults.right.len());
    }
}
