use serde::{Deserialize, Serialize};

use super::{CatppuccinFlavor, RosePineFlavor, Theme, TokyoNightFlavor};

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

impl RosePineFlavor {
    fn resolve(flavor: &str) -> Self {
        match flavor.to_ascii_lowercase().as_str() {
            "moon" => Self::Moon,
            "dawn" => Self::Dawn,
            _ => Self::Main,
        }
    }
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
