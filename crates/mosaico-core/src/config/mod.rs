pub mod keybinding;
mod loader;
pub mod template;

use serde::{Deserialize, Serialize};

pub use keybinding::{Keybinding, Modifier};
pub use loader::{
    config_dir, config_path, keybindings_path, load, load_keybindings, load_rules, rules_path,
};

/// Top-level configuration for Mosaico.
///
/// Loaded from `~/.config/mosaico/config.toml`. Missing sections
/// fall back to defaults thanks to `#[serde(default)]`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Layout algorithm parameters.
    pub layout: LayoutConfig,
    /// Border appearance settings.
    pub borders: BorderConfig,
}

/// Layout algorithm settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    /// Gap in pixels between windows and screen edges.
    pub gap: i32,
    /// Ratio of space given to the first window in each split (0.0–1.0).
    pub ratio: f64,
}

/// Border appearance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BorderConfig {
    /// Border width in pixels.
    pub width: i32,
    /// Hex color for the focused window border (e.g. "#00b4d8").
    pub focused: String,
    /// Hex color for the monocle mode border (e.g. "#2d6a4f").
    pub monocle: String,
}

/// A rule that determines whether a window should be managed (tiled).
///
/// Rules are evaluated in order. The first matching rule wins.
/// If no rule matches, the window is managed by default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowRule {
    /// Match windows with this exact class name (case-insensitive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_class: Option<String>,
    /// Match windows whose title contains this string (case-insensitive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_title: Option<String>,
    /// Whether matching windows should be managed (tiled).
    pub manage: bool,
}

/// Returns the default window rules.
///
/// These exclude window classes that don't behave well when tiled,
/// such as UWP app frames that enforce their own size constraints.
pub fn default_rules() -> Vec<WindowRule> {
    vec![WindowRule {
        match_class: Some("ApplicationFrameWindow".into()),
        match_title: None,
        manage: false,
    }]
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self { gap: 8, ratio: 0.5 }
    }
}

impl Default for BorderConfig {
    fn default() -> Self {
        Self {
            width: 4,
            focused: "#00b4d8".into(),
            monocle: "#2d6a4f".into(),
        }
    }
}

impl Config {
    /// Clamps layout and border values to safe ranges.
    ///
    /// Prevents negative gaps, out-of-range ratios, and excessively
    /// large border widths that could cause rendering artifacts.
    pub fn validate(&mut self) {
        self.layout.gap = self.layout.gap.clamp(0, 200);
        self.layout.ratio = self.layout.ratio.clamp(0.1, 0.9);
        self.borders.width = self.borders.width.clamp(0, 32);
    }
}

/// Evaluates window rules to decide if a window should be managed.
///
/// Returns `true` if the window should be tiled. When no rule matches,
/// defaults to `true`.
pub fn should_manage(class: &str, title: &str, rules: &[WindowRule]) -> bool {
    for rule in rules {
        if matches_rule(class, title, rule) {
            return rule.manage;
        }
    }
    true
}

fn matches_rule(class: &str, title: &str, rule: &WindowRule) -> bool {
    if let Some(ref mc) = rule.match_class
        && !class.eq_ignore_ascii_case(mc)
    {
        return false;
    }
    if let Some(ref mt) = rule.match_title
        && !title
            .to_ascii_lowercase()
            .contains(&mt.to_ascii_lowercase())
    {
        return false;
    }
    rule.match_class.is_some() || rule.match_title.is_some()
}

/// Wrapper for deserializing the keybindings file.
///
/// The file contains a top-level `[[keybinding]]` array of tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeybindingsFile {
    #[serde(default = "keybinding::defaults")]
    keybinding: Vec<Keybinding>,
}

/// Wrapper for deserializing the rules file.
///
/// The file contains a top-level `[[rule]]` array of tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RulesFile {
    #[serde(default = "default_rules")]
    rule: Vec<WindowRule>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        // Arrange / Act
        let config = Config::default();

        // Assert
        assert_eq!(config.layout.gap, 8);
        assert_eq!(config.borders.width, 4);
    }

    #[test]
    fn default_keybindings_are_not_empty() {
        // Act
        let bindings = keybinding::defaults();

        // Assert
        assert!(!bindings.is_empty());
    }

    #[test]
    fn partial_toml_uses_defaults_for_missing_sections() {
        // Arrange
        let toml_str = "[layout]\ngap = 16\n";

        // Act
        let config: Config = toml::from_str(toml_str).unwrap();

        // Assert
        assert_eq!(config.layout.gap, 16);
        assert_eq!(config.layout.ratio, 0.5);
    }

    #[test]
    fn rule_excludes_by_class() {
        // Arrange
        let rules = vec![WindowRule {
            match_class: Some("TaskManager".into()),
            match_title: None,
            manage: false,
        }];

        // Act / Assert
        assert!(!should_manage("TaskManager", "Task Manager", &rules));
        assert!(should_manage("Notepad", "Untitled", &rules));
    }

    #[test]
    fn rule_excludes_by_title_substring() {
        // Arrange
        let rules = vec![WindowRule {
            match_class: None,
            match_title: Some("settings".into()),
            manage: false,
        }];

        // Act / Assert
        assert!(!should_manage("App", "Windows Settings", &rules));
        assert!(should_manage("App", "My Document", &rules));
    }

    #[test]
    fn first_matching_rule_wins() {
        // Arrange
        let rules = vec![
            WindowRule {
                match_class: Some("Chrome".into()),
                match_title: None,
                manage: false,
            },
            WindowRule {
                match_class: Some("Chrome".into()),
                match_title: None,
                manage: true,
            },
        ];

        // Act / Assert
        assert!(!should_manage("Chrome", "Google", &rules));
    }

    #[test]
    fn no_rules_defaults_to_manage() {
        // Act / Assert
        assert!(should_manage("Any", "Window", &[]));
    }

    #[test]
    fn validate_clamps_extreme_values() {
        // Arrange
        let mut config = Config {
            layout: LayoutConfig {
                gap: -50,
                ratio: 2.0,
            },
            borders: BorderConfig {
                width: 999,
                ..Default::default()
            },
        };

        // Act
        config.validate();

        // Assert
        assert_eq!(config.layout.gap, 0);
        assert!((config.layout.ratio - 0.9).abs() < f64::EPSILON);
        assert_eq!(config.borders.width, 32);
    }

    #[test]
    fn default_rules_exclude_uwp_frame() {
        // Arrange
        let rules = default_rules();

        // Act / Assert
        assert!(!should_manage("ApplicationFrameWindow", "Settings", &rules));
    }
}
