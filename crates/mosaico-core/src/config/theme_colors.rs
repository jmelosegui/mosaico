use crate::config::bar::BarColors;
use crate::config::palette;

use super::{CatppuccinFlavor, RosePineFlavor, Theme, TokyoNightFlavor};

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
