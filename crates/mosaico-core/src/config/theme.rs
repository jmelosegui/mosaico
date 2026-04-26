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
/// The two-field design lets the same config syntax address every
/// theme family (Catppuccin, Rosé Pine, Tokyo Night, ...) without
/// changing the schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    /// Theme family name (e.g. "catppuccin", "rose-pine", "tokyo-night").
    pub name: String,
    /// Flavor or variant within the theme (e.g. "mocha", "moon", "night").
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
            "catppuccin" => Theme::Catppuccin(CatppuccinFlavor::resolve(&self.flavor)),
            "rose-pine" | "rosepine" | "rose_pine" => {
                Theme::RosePine(RosePineFlavor::resolve(&self.flavor))
            }
            "tokyo-night" | "tokyonight" | "tokyo_night" => {
                Theme::TokyoNight(TokyoNightFlavor::resolve(&self.flavor))
            }
            _ => Theme::default(),
        }
    }
}

/// A resolved color theme used internally for color lookups.
///
/// Each variant pairs a theme family with a flavor of that family,
/// so invalid combinations (e.g. Rosé Pine with a Mocha flavor) are
/// unrepresentable at the type level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Catppuccin (Mocha, Macchiato, Frappé, Latte).
    Catppuccin(CatppuccinFlavor),
    /// Rosé Pine (Main, Moon, Dawn).
    RosePine(RosePineFlavor),
    /// Tokyo Night (Night, Storm, Day).
    TokyoNight(TokyoNightFlavor),
}

impl Default for Theme {
    fn default() -> Self {
        Self::Catppuccin(CatppuccinFlavor::Mocha)
    }
}

/// Catppuccin flavors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CatppuccinFlavor {
    /// Mocha -- dark theme with warm pastels (default).
    #[default]
    Mocha,
    /// Macchiato -- dark theme with muted tones.
    Macchiato,
    /// Frappé -- mid-tone dark theme.
    Frappe,
    /// Latte -- light theme.
    Latte,
}

impl CatppuccinFlavor {
    fn resolve(flavor: &str) -> Self {
        match flavor.to_ascii_lowercase().as_str() {
            "macchiato" => Self::Macchiato,
            "frappe" | "frappé" => Self::Frappe,
            "latte" => Self::Latte,
            _ => Self::Mocha,
        }
    }
}

/// Rosé Pine flavors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RosePineFlavor {
    /// Main -- the standard dark variant (default).
    #[default]
    Main,
    /// Moon -- a slightly lighter dark variant.
    Moon,
    /// Dawn -- the light variant.
    Dawn,
}

impl RosePineFlavor {
    fn resolve(flavor: &str) -> Self {
        match flavor.to_ascii_lowercase().as_str() {
            "moon" => Self::Moon,
            "dawn" => Self::Dawn,
            _ => Self::Main,
        }
    }
}

/// Tokyo Night flavors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TokyoNightFlavor {
    /// Night -- the standard dark variant (default).
    #[default]
    Night,
    /// Storm -- darker variant with a bluish tint.
    Storm,
    /// Day -- the light variant.
    Day,
}

impl TokyoNightFlavor {
    fn resolve(flavor: &str) -> Self {
        match flavor.to_ascii_lowercase().as_str() {
            "storm" => Self::Storm,
            "day" => Self::Day,
            _ => Self::Night,
        }
    }
}

impl Theme {
    /// Returns the bar color palette for this theme.
    pub fn bar_colors(self) -> BarColors {
        palette::bar_colors(self)
    }

    /// Returns the focused window border color.
    ///
    /// Catppuccin uses Blue, Rosé Pine uses Iris (matching the bar
    /// foreground), Tokyo Night uses its primary Blue accent. The
    /// exact hex shifts per flavor so the highlight reads well
    /// against each base background.
    pub fn border_focused(self) -> &'static str {
        match self {
            Self::Catppuccin(f) => match f {
                CatppuccinFlavor::Mocha => "#89b4fa",
                CatppuccinFlavor::Macchiato => "#8aadf4",
                CatppuccinFlavor::Frappe => "#8caaee",
                CatppuccinFlavor::Latte => "#1e66f5",
            },
            Self::RosePine(f) => match f {
                // Iris matches the bar foreground so the focused
                // window outline visually ties into the bar.
                RosePineFlavor::Main | RosePineFlavor::Moon => "#c4a7e7",
                RosePineFlavor::Dawn => "#907aa9",
            },
            Self::TokyoNight(f) => match f {
                TokyoNightFlavor::Night | TokyoNightFlavor::Storm => "#7aa2f7",
                TokyoNightFlavor::Day => "#2e7de9",
            },
        }
    }

    /// Returns the monocle window border color.
    ///
    /// Catppuccin uses Green, Rosé Pine uses Foam, Tokyo Night uses
    /// Green.
    pub fn border_monocle(self) -> &'static str {
        match self {
            Self::Catppuccin(f) => match f {
                CatppuccinFlavor::Mocha => "#a6e3a1",
                CatppuccinFlavor::Macchiato => "#a6da95",
                CatppuccinFlavor::Frappe => "#a6d189",
                CatppuccinFlavor::Latte => "#40a02b",
            },
            Self::RosePine(f) => match f {
                RosePineFlavor::Main | RosePineFlavor::Moon => "#9ccfd8",
                RosePineFlavor::Dawn => "#56949f",
            },
            Self::TokyoNight(f) => match f {
                TokyoNightFlavor::Night | TokyoNightFlavor::Storm => "#9ece6a",
                TokyoNightFlavor::Day => "#587539",
            },
        }
    }

    /// Returns the unfocused window border color.
    ///
    /// Chosen to be visible but understated against each theme's base
    /// background. Catppuccin uses Overlay0, Rosé Pine uses Subtle,
    /// Tokyo Night uses Comment.
    pub fn border_unfocused(self) -> &'static str {
        match self {
            Self::Catppuccin(f) => match f {
                CatppuccinFlavor::Mocha => "#6c7086",
                CatppuccinFlavor::Macchiato => "#6e738d",
                CatppuccinFlavor::Frappe => "#737994",
                CatppuccinFlavor::Latte => "#9ca0b0",
            },
            Self::RosePine(f) => match f {
                RosePineFlavor::Main | RosePineFlavor::Moon => "#908caa",
                RosePineFlavor::Dawn => "#797593",
            },
            Self::TokyoNight(f) => match f {
                TokyoNightFlavor::Night | TokyoNightFlavor::Storm => "#565f89",
                TokyoNightFlavor::Day => "#848cb5",
            },
        }
    }

    /// Resolves a named color (e.g. `"blue"`, `"pine"`) to its hex
    /// value for this theme. Returns `None` for unknown names.
    ///
    /// This lets users write `focused = "blue"` instead of a hex
    /// code and pick up the right shade for whichever theme they
    /// happen to be using.
    pub fn named_color(self, name: &str) -> Option<&'static str> {
        palette::named_color(self, name)
    }

    /// Resolves a color value that may be a hex code, a named color,
    /// or empty. Returns the resolved hex string.
    ///
    /// - `""` returns `fallback`
    /// - `"blue"` resolves via the theme palette
    /// - `"#89b4fa"` is returned as-is
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
        assert_eq!(tc.resolve(), Theme::Catppuccin(CatppuccinFlavor::Mocha));
    }

    #[test]
    fn resolve_all_catppuccin_flavors() {
        let cases = [
            ("mocha", CatppuccinFlavor::Mocha),
            ("macchiato", CatppuccinFlavor::Macchiato),
            ("frappe", CatppuccinFlavor::Frappe),
            ("latte", CatppuccinFlavor::Latte),
        ];
        for (flavor, expected) in cases {
            let tc = ThemeConfig {
                name: "catppuccin".into(),
                flavor: flavor.into(),
            };
            assert_eq!(tc.resolve(), Theme::Catppuccin(expected), "flavor {flavor}");
        }
    }

    #[test]
    fn resolve_rose_pine_flavors() {
        let cases = [
            ("main", RosePineFlavor::Main),
            ("moon", RosePineFlavor::Moon),
            ("dawn", RosePineFlavor::Dawn),
        ];
        for (flavor, expected) in cases {
            let tc = ThemeConfig {
                name: "rose-pine".into(),
                flavor: flavor.into(),
            };
            assert_eq!(tc.resolve(), Theme::RosePine(expected), "flavor {flavor}");
        }
    }

    #[test]
    fn resolve_tokyo_night_flavors() {
        let cases = [
            ("night", TokyoNightFlavor::Night),
            ("storm", TokyoNightFlavor::Storm),
            ("day", TokyoNightFlavor::Day),
        ];
        for (flavor, expected) in cases {
            let tc = ThemeConfig {
                name: "tokyo-night".into(),
                flavor: flavor.into(),
            };
            assert_eq!(tc.resolve(), Theme::TokyoNight(expected), "flavor {flavor}");
        }
    }

    #[test]
    fn rose_pine_name_aliases() {
        for name in ["rose-pine", "rosepine", "rose_pine", "Rose-Pine"] {
            let tc = ThemeConfig {
                name: name.into(),
                flavor: "main".into(),
            };
            assert_eq!(tc.resolve(), Theme::RosePine(RosePineFlavor::Main));
        }
    }

    #[test]
    fn tokyo_night_name_aliases() {
        for name in ["tokyo-night", "tokyonight", "tokyo_night", "Tokyo-Night"] {
            let tc = ThemeConfig {
                name: name.into(),
                flavor: "night".into(),
            };
            assert_eq!(tc.resolve(), Theme::TokyoNight(TokyoNightFlavor::Night));
        }
    }

    #[test]
    fn resolve_is_case_insensitive() {
        let tc = ThemeConfig {
            name: "Catppuccin".into(),
            flavor: "Latte".into(),
        };
        assert_eq!(tc.resolve(), Theme::Catppuccin(CatppuccinFlavor::Latte));
    }

    #[test]
    fn unknown_theme_falls_back_to_default() {
        let tc = ThemeConfig {
            name: "tokyo".into(), // missing the -night
            flavor: "night".into(),
        };
        assert_eq!(tc.resolve(), Theme::default());
    }

    #[test]
    fn unknown_flavor_falls_back_to_family_default() {
        let tc = ThemeConfig {
            name: "catppuccin".into(),
            flavor: "espresso".into(),
        };
        assert_eq!(tc.resolve(), Theme::Catppuccin(CatppuccinFlavor::Mocha));

        let tc = ThemeConfig {
            name: "rose-pine".into(),
            flavor: "midnight".into(),
        };
        assert_eq!(tc.resolve(), Theme::RosePine(RosePineFlavor::Main));
    }

    #[test]
    fn theme_config_round_trips_through_toml() {
        let tc = ThemeConfig {
            name: "rose-pine".into(),
            flavor: "moon".into(),
        };
        let toml_str = toml::to_string(&tc).unwrap();
        let parsed: ThemeConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed, tc);
    }

    #[test]
    fn mocha_bar_colors_match_catppuccin() {
        let c = Theme::Catppuccin(CatppuccinFlavor::Mocha).bar_colors();
        assert_eq!(c.background, "#1e1e2e");
        assert_eq!(c.foreground, "#89b4fa");
        assert_eq!(c.active_workspace, "#435375");
        assert_eq!(c.pill_border, "#89b4fa");
    }

    #[test]
    fn latte_is_light_theme() {
        let c = Theme::Catppuccin(CatppuccinFlavor::Latte).bar_colors();
        assert_eq!(c.background, "#eff1f5");
        assert_eq!(c.foreground, "#1e66f5");
    }

    #[test]
    fn rose_pine_main_bar_features_iris_and_rose() {
        // The bar foreground and the focused window border share
        // Iris (purple) so the focused window visually ties into the
        // bar's lead color.
        let c = Theme::RosePine(RosePineFlavor::Main).bar_colors();
        assert_eq!(c.background, "#191724"); // Base
        assert_eq!(c.foreground, "#c4a7e7"); // Iris
        assert_eq!(c.accent, "#ebbcba"); // Rose
        assert_eq!(
            Theme::RosePine(RosePineFlavor::Main).border_focused(),
            c.foreground
        );
    }

    #[test]
    fn tokyo_night_bar_uses_blue_accent() {
        let c = Theme::TokyoNight(TokyoNightFlavor::Night).bar_colors();
        assert_eq!(c.background, "#1a1b26");
        assert_eq!(c.foreground, "#7aa2f7");
    }

    #[test]
    fn border_focused_matches_per_flavor() {
        assert_eq!(
            Theme::Catppuccin(CatppuccinFlavor::Mocha).border_focused(),
            "#89b4fa"
        );
        // Rose Pine matches its bar foreground (Iris).
        assert_eq!(
            Theme::RosePine(RosePineFlavor::Main).border_focused(),
            "#c4a7e7"
        );
        assert_eq!(
            Theme::TokyoNight(TokyoNightFlavor::Night).border_focused(),
            "#7aa2f7"
        );
    }

    #[test]
    fn rose_pine_focus_border_matches_bar_foreground() {
        // Every Rose Pine flavor's focused border equals its bar
        // foreground so the highlight is consistent across the UI.
        for flavor in [
            RosePineFlavor::Main,
            RosePineFlavor::Moon,
            RosePineFlavor::Dawn,
        ] {
            let theme = Theme::RosePine(flavor);
            assert_eq!(
                theme.border_focused(),
                theme.bar_colors().foreground,
                "flavor {flavor:?}"
            );
        }
    }

    #[test]
    fn border_unfocused_distinct_from_focused_for_every_theme() {
        let themes = [
            Theme::Catppuccin(CatppuccinFlavor::Mocha),
            Theme::Catppuccin(CatppuccinFlavor::Macchiato),
            Theme::Catppuccin(CatppuccinFlavor::Frappe),
            Theme::Catppuccin(CatppuccinFlavor::Latte),
            Theme::RosePine(RosePineFlavor::Main),
            Theme::RosePine(RosePineFlavor::Moon),
            Theme::RosePine(RosePineFlavor::Dawn),
            Theme::TokyoNight(TokyoNightFlavor::Night),
            Theme::TokyoNight(TokyoNightFlavor::Storm),
            Theme::TokyoNight(TokyoNightFlavor::Day),
        ];
        for t in themes {
            assert_ne!(t.border_focused(), t.border_unfocused(), "theme {t:?}");
            assert_ne!(t.border_monocle(), t.border_unfocused(), "theme {t:?}");
        }
    }

    #[test]
    fn resolve_color_handles_all_cases() {
        let t = Theme::Catppuccin(CatppuccinFlavor::Mocha);
        assert_eq!(t.resolve_color("", "#default"), "#default");
        assert_eq!(t.resolve_color("#ff0000", "#default"), "#ff0000");
        assert_eq!(t.resolve_color("blue", "#default"), "#89b4fa");
        assert_eq!(t.resolve_color("chartreuse", "#default"), "chartreuse");
    }

    #[test]
    fn each_theme_has_distinct_base() {
        let bases: Vec<String> = [
            Theme::Catppuccin(CatppuccinFlavor::Mocha),
            Theme::Catppuccin(CatppuccinFlavor::Macchiato),
            Theme::Catppuccin(CatppuccinFlavor::Frappe),
            Theme::Catppuccin(CatppuccinFlavor::Latte),
            Theme::RosePine(RosePineFlavor::Main),
            Theme::RosePine(RosePineFlavor::Moon),
            Theme::RosePine(RosePineFlavor::Dawn),
            Theme::TokyoNight(TokyoNightFlavor::Night),
            Theme::TokyoNight(TokyoNightFlavor::Storm),
            Theme::TokyoNight(TokyoNightFlavor::Day),
        ]
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
