//! Named color themes for the mosaico UI.
//!
//! [`ThemeConfig`] is the user-facing struct that deserializes from the
//! `[theme]` section in `config.toml` (e.g. `name = "catppuccin"`,
//! `flavor = "mocha"`). Call [`ThemeConfig::resolve()`] to get the
//! concrete [`Theme`] used internally for color lookups.

#[path = "theme_colors.rs"]
mod theme_colors;
#[path = "theme_config.rs"]
mod theme_config;
#[path = "theme_list.rs"]
mod theme_list;

pub use theme_config::{ThemeCheck, ThemeConfig};

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

impl Theme {
    /// The theme's family as `[theme] name` spells it, such as
    /// `"rose-pine"`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Catppuccin(_) => "catppuccin",
            Self::RosePine(_) => "rose-pine",
            Self::TokyoNight(_) => "tokyo-night",
        }
    }

    /// The theme's flavor as `[theme] flavor` spells it, such as
    /// `"moon"`.
    pub fn flavor_name(self) -> &'static str {
        match self {
            Self::Catppuccin(f) => match f {
                CatppuccinFlavor::Mocha => "mocha",
                CatppuccinFlavor::Macchiato => "macchiato",
                CatppuccinFlavor::Frappe => "frappe",
                CatppuccinFlavor::Latte => "latte",
            },
            Self::RosePine(f) => match f {
                RosePineFlavor::Main => "main",
                RosePineFlavor::Moon => "moon",
                RosePineFlavor::Dawn => "dawn",
            },
            Self::TokyoNight(f) => match f {
                TokyoNightFlavor::Night => "night",
                TokyoNightFlavor::Storm => "storm",
                TokyoNightFlavor::Day => "day",
            },
        }
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

#[cfg(test)]
#[path = "theme_tests.rs"]
mod tests;
