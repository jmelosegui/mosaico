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
    ///
    /// A `[theme]` without one reads it as empty, which stands for the
    /// default flavor of the family, whichever family that is.
    #[serde(default)]
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
    /// An unknown name falls back to Catppuccin Mocha, and an unknown
    /// flavor to the default flavor of its family.
    pub fn resolve(&self) -> Theme {
        self.check().theme()
    }

    /// Says whether the name and the flavor are a theme and a flavor
    /// mosaico knows, and which theme they stand for.
    ///
    /// [`resolve`](Self::resolve) is built on this answer, so what
    /// `mosaico doctor` reports from it is what the daemon uses.
    pub fn check(&self) -> ThemeCheck {
        match self.name.to_ascii_lowercase().as_str() {
            "catppuccin" => family(CatppuccinFlavor::parse(&self.flavor), Theme::Catppuccin),
            "rose-pine" | "rosepine" | "rose_pine" => {
                family(RosePineFlavor::parse(&self.flavor), Theme::RosePine)
            }
            "tokyo-night" | "tokyonight" | "tokyo_night" => {
                family(TokyoNightFlavor::parse(&self.flavor), Theme::TokyoNight)
            }
            _ => ThemeCheck::UnknownName(Theme::default()),
        }
    }
}

/// What a theme name and flavor stand for: a theme mosaico knows, or
/// the theme it falls back to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeCheck {
    /// The name and the flavor are known, and give this theme.
    Known(Theme),
    /// The name is known but not the flavor, so this theme, with the
    /// default flavor of its family, is used instead.
    UnknownFlavor(Theme),
    /// The name is not a theme mosaico knows, so this theme, the
    /// default one, is used instead.
    UnknownName(Theme),
}

impl ThemeCheck {
    /// The theme mosaico uses: the one known, or the one it falls back to.
    pub fn theme(self) -> Theme {
        match self {
            Self::Known(theme) | Self::UnknownFlavor(theme) | Self::UnknownName(theme) => theme,
        }
    }
}

/// The check of a theme of a known family: `flavor` is the flavor found,
/// or `None` when the family's default flavor is used instead.
fn family<F: Default>(flavor: Option<F>, theme: fn(F) -> Theme) -> ThemeCheck {
    match flavor {
        Some(flavor) => ThemeCheck::Known(theme(flavor)),
        None => ThemeCheck::UnknownFlavor(theme(F::default())),
    }
}

impl CatppuccinFlavor {
    /// The Catppuccin flavor `flavor` names, the default for an empty one.
    fn parse(flavor: &str) -> Option<Self> {
        match flavor.to_ascii_lowercase().as_str() {
            "" => Some(Self::default()),
            "mocha" => Some(Self::Mocha),
            "macchiato" => Some(Self::Macchiato),
            "frappe" | "frappé" => Some(Self::Frappe),
            "latte" => Some(Self::Latte),
            _ => None,
        }
    }
}

impl RosePineFlavor {
    /// The Rosé Pine flavor `flavor` names, the default for an empty one.
    fn parse(flavor: &str) -> Option<Self> {
        match flavor.to_ascii_lowercase().as_str() {
            "" => Some(Self::default()),
            "main" => Some(Self::Main),
            "moon" => Some(Self::Moon),
            "dawn" => Some(Self::Dawn),
            _ => None,
        }
    }
}

impl TokyoNightFlavor {
    /// The Tokyo Night flavor `flavor` names, the default for an empty one.
    fn parse(flavor: &str) -> Option<Self> {
        match flavor.to_ascii_lowercase().as_str() {
            "" => Some(Self::default()),
            "night" => Some(Self::Night),
            "storm" => Some(Self::Storm),
            "day" => Some(Self::Day),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "theme_check_tests.rs"]
mod tests;
