/// Generates the default `config.toml` contents with explanatory comments.
///
/// This is used by `mosaico init` to create a starter config file that
/// users can immediately edit.
pub fn generate_config() -> String {
    r##"# Mosaico configuration
# Location: ~/.config/mosaico/config.toml

[layout]
# Gap in pixels between windows and screen edges.
gap = 8
# Ratio of space given to the first window in each split (0.0 to 1.0).
ratio = 0.5

[borders]
# Border width in pixels around the focused window.
width = 4
# Hex color for the focused window border.
focused = "#00b4d8"
# Hex color for the border in monocle mode.
monocle = "#2d6a4f"
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
#   focus-next, focus-prev, swap-next, swap-prev, retile,
#   focus-monitor-next, focus-monitor-prev,
#   move-to-monitor-next, move-to-monitor-prev, toggle-monocle
#
# Available modifiers: alt, shift, ctrl, win
#
# Key names: A-Z, 0-9, F1-F24, Enter, Space, Tab, Escape,
#            Left, Right, Up, Down, Minus, Plus, Comma, Period

# Focus: Alt + J/K (next/prev window)
[[keybinding]]
action = "focus-next"
key = "J"
modifiers = ["alt"]

[[keybinding]]
action = "focus-prev"
key = "K"
modifiers = ["alt"]

# Focus across monitors: Alt + L/H (next/prev monitor)
[[keybinding]]
action = "focus-monitor-next"
key = "L"
modifiers = ["alt"]

[[keybinding]]
action = "focus-monitor-prev"
key = "H"
modifiers = ["alt"]

# Swap windows: Alt + Shift + J/K
[[keybinding]]
action = "swap-next"
key = "J"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "swap-prev"
key = "K"
modifiers = ["alt", "shift"]

# Move window across monitors: Alt + Shift + L/H
[[keybinding]]
action = "move-to-monitor-next"
key = "L"
modifiers = ["alt", "shift"]

[[keybinding]]
action = "move-to-monitor-prev"
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
"##
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
        let config: crate::Config = toml::from_str(&toml_str).unwrap();

        // Assert
        let defaults = crate::Config::default();
        assert_eq!(config.layout.gap, defaults.layout.gap);
        assert_eq!(config.layout.ratio, defaults.layout.ratio);
        assert_eq!(config.borders.width, defaults.borders.width);
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
}
