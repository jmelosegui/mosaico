//! Named color themes for the mosaico UI.
//!
//! [`ThemeConfig`] is the user-facing struct that deserializes from the
//! `[theme]` section in `config.toml` (e.g. `name = "catppuccin"`,
//! `flavor = "mocha"`). Call [`ThemeConfig::resolve()`] to get the
//! concrete [`Theme`] used internally for color lookups.

use serde::{Deserialize, Serialize};

use super::bar::BarColors;
use super::palette;

/// User-facing theme configuration.
///
/// Deserializes from the `[theme]` section in `config.toml`:
///
/// ```toml
/// [theme]
/// name = "catppuccin"
/// flavor = "mocha"
/// ```
///
/// The two-field design allows future themes (e.g. `name = "tokyo"`,
/// `flavor = "night"`) without changing the config schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    /// Theme family name (e.g. "catppuccin").
    pub name: String,
    /// Flavor or variant within the theme (e.g. "mocha", "latte").
    pub flavor: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "catppuccin".into(),
            flavor: "mocha".into(),
        }
    }
}

impl ThemeConfig {
    /// Resolves the config into a concrete [`Theme`] for color lookups.
    ///
    /// Unknown name/flavor combinations fall back to Catppuccin Mocha.
    pub fn resolve(&self) -> Theme {
        match self.name.to_ascii_lowercase().as_str() {
            "catppuccin" => match self.flavor.to_ascii_lowercase().as_str() {
                "macchiato" => Theme::Macchiato,
                "frappe" | "frappé" => Theme::Frappe,
                "latte" => Theme::Latte,
                _ => Theme::Mocha,
            },
            _ => Theme::Mocha,
        }
    }
}

/// A resolved color theme used internally for color lookups.
///
/// Obtained from [`ThemeConfig::resolve()`]. All color methods live
/// here so the rest of the codebase doesn't need to know about
/// theme names or flavors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Theme {
    /// Catppuccin Mocha — dark theme with warm pastels.
    #[default]
    Mocha,
    /// Catppuccin Macchiato — dark theme with muted tones.
    Macchiato,
    /// Catppuccin Frappé — mid-tone dark theme.
    Frappe,
    /// Catppuccin Latte — light theme.
    Latte,
}

impl Theme {
    /// Returns the bar color palette for this theme.
    pub fn bar_colors(self) -> BarColors {
        palette::bar_colors(self)
    }

    /// Returns the focused window border color (Catppuccin Blue).
    pub fn border_focused(self) -> &'static str {
        match self {
            Self::Mocha => "#89b4fa",
            Self::Macchiato => "#8aadf4",
            Self::Frappe => "#8caaee",
            Self::Latte => "#1e66f5",
        }
    }

    /// Returns the monocle window border color (Catppuccin Green).
    pub fn border_monocle(self) -> &'static str {
        match self {
            Self::Mocha => "#a6e3a1",
            Self::Macchiato => "#a6da95",
            Self::Frappe => "#a6d189",
            Self::Latte => "#40a02b",
        }
    }

    /// Resolves a named Catppuccin color (e.g. "blue", "green") to its
    /// hex value for this theme. Returns `None` for unknown names.
    ///
    /// This allows users to write `focused = "blue"` instead of a hex
    /// code, and mosaico picks the correct shade for the active theme.
    pub fn named_color(self, name: &str) -> Option<&'static str> {
        palette::named_color(self, name)
    }

    /// Resolves a color value that may be a hex code, a named color,
    /// or empty. Returns the resolved hex string.
    ///
    /// - `""` → returns `fallback`
    /// - `"blue"` → resolves via the theme palette
    /// - `"#89b4fa"` → returned as-is
    pub fn resolve_color<'a>(&self, value: &'a str, fallback: &'a str) -> &'a str {
        if value.is_empty() {
            return fallback;
        }
        if value.starts_with('#') {
            return value;
        }
        self.named_color(value).unwrap_or(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_config_is_catppuccin_mocha() {
        let tc = ThemeConfig::default();
        assert_eq!(tc.name, "catppuccin");
        assert_eq!(tc.flavor, "mocha");
        assert_eq!(tc.resolve(), Theme::Mocha);
    }

    #[test]
    fn resolve_all_catppuccin_flavors() {
        let cases = [
            ("mocha", Theme::Mocha),
            ("macchiato", Theme::Macchiato),
            ("frappe", Theme::Frappe),
            ("latte", Theme::Latte),
        ];
        for (flavor, expected) in cases {
            let tc = ThemeConfig {
                name: "catppuccin".into(),
                flavor: flavor.into(),
            };
            assert_eq!(tc.resolve(), expected, "flavor {flavor}");
        }
    }

    #[test]
    fn resolve_is_case_insensitive() {
        let tc = ThemeConfig {
            name: "Catppuccin".into(),
            flavor: "Latte".into(),
        };
        assert_eq!(tc.resolve(), Theme::Latte);
    }

    #[test]
    fn unknown_theme_falls_back_to_mocha() {
        let tc = ThemeConfig {
            name: "tokyo".into(),
            flavor: "night".into(),
        };
        assert_eq!(tc.resolve(), Theme::Mocha);
    }

    #[test]
    fn unknown_flavor_falls_back_to_mocha() {
        let tc = ThemeConfig {
            name: "catppuccin".into(),
            flavor: "espresso".into(),
        };
        assert_eq!(tc.resolve(), Theme::Mocha);
    }

    #[test]
    fn theme_config_round_trips_through_toml() {
        let tc = ThemeConfig {
            name: "catppuccin".into(),
            flavor: "frappe".into(),
        };
        let toml_str = toml::to_string(&tc).unwrap();
        let parsed: ThemeConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed, tc);
    }

    #[test]
    fn mocha_colors_match_catppuccin() {
        let c = Theme::Mocha.bar_colors();
        assert_eq!(c.background, "#1e1e2e");
        assert_eq!(c.foreground, "#cdd6f4");
        assert_eq!(c.active_workspace, "#89b4fa");
    }

    #[test]
    fn latte_is_light_theme() {
        let c = Theme::Latte.bar_colors();
        assert_eq!(c.background, "#eff1f5");
        assert_eq!(c.foreground, "#4c4f69");
    }

    #[test]
    fn border_colors_match_theme() {
        assert_eq!(Theme::Mocha.border_focused(), "#89b4fa");
        assert_eq!(Theme::Mocha.border_monocle(), "#a6e3a1");
        assert_eq!(Theme::Latte.border_focused(), "#1e66f5");
        assert_eq!(Theme::Latte.border_monocle(), "#40a02b");
    }

    #[test]
    fn resolve_color_handles_all_cases() {
        let t = Theme::Mocha;
        // Empty → fallback
        assert_eq!(t.resolve_color("", "#default"), "#default");
        // Hex → as-is
        assert_eq!(t.resolve_color("#ff0000", "#default"), "#ff0000");
        // Named → resolved
        assert_eq!(t.resolve_color("blue", "#default"), "#89b4fa");
        // Unknown name → as-is
        assert_eq!(t.resolve_color("chartreuse", "#default"), "chartreuse");
    }

    #[test]
    fn each_theme_has_distinct_base() {
        let bases: Vec<String> = [Theme::Mocha, Theme::Macchiato, Theme::Frappe, Theme::Latte]
            .iter()
            .map(|t| t.bar_colors().background.clone())
            .collect();
        for (i, a) in bases.iter().enumerate() {
            for (j, b) in bases.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "themes {i} and {j} share the same background");
                }
            }
        }
    }
}
