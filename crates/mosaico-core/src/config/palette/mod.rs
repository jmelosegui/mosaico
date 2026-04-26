//! Color palettes for each supported theme family.
//!
//! Public dispatchers route lookups to the right family module based
//! on the resolved [`Theme`] variant.

mod catppuccin;
mod rose_pine;
mod tokyo_night;

use super::bar::BarColors;
use super::theme::Theme;

/// Resolves a named theme color (e.g. `"blue"`, `"pine"`) to its hex
/// value for the given theme. Returns `None` for unknown names.
///
/// Common color names like `"red"`, `"blue"`, `"green"`, `"yellow"`
/// resolve in every theme family so `focused = "blue"` works
/// regardless of which theme the user picks; family-specific names
/// (`pine`, `iris`, `magenta`, ...) only resolve under their own
/// family.
pub fn named_color(theme: Theme, name: &str) -> Option<&'static str> {
    let table: &[(&str, &str)] = match theme {
        Theme::Catppuccin(f) => catppuccin::table(f),
        Theme::RosePine(f) => rose_pine::table(f),
        Theme::TokyoNight(f) => tokyo_night::table(f),
    };
    table
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, hex)| *hex)
}

/// Returns the bar color palette for the given theme.
pub fn bar_colors(theme: Theme) -> BarColors {
    match theme {
        Theme::Catppuccin(f) => catppuccin::bar_colors(f),
        Theme::RosePine(f) => rose_pine::bar_colors(f),
        Theme::TokyoNight(f) => tokyo_night::bar_colors(f),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::theme::{CatppuccinFlavor, RosePineFlavor, TokyoNightFlavor};

    #[test]
    fn named_color_resolves_blue_for_each_family() {
        // Common name "blue" works under every theme family so
        // portable configs (focused = "blue") look right regardless
        // of the theme the user picks.
        assert!(named_color(Theme::Catppuccin(CatppuccinFlavor::Mocha), "blue").is_some());
        assert!(named_color(Theme::RosePine(RosePineFlavor::Main), "blue").is_some());
        assert!(named_color(Theme::TokyoNight(TokyoNightFlavor::Night), "blue").is_some());
    }

    #[test]
    fn named_color_is_case_insensitive() {
        let mocha = Theme::Catppuccin(CatppuccinFlavor::Mocha);
        assert_eq!(named_color(mocha, "Blue"), Some("#89b4fa"));
        assert_eq!(named_color(mocha, "GREEN"), Some("#a6e3a1"));

        let pine = Theme::RosePine(RosePineFlavor::Main);
        assert_eq!(named_color(pine, "PINE"), named_color(pine, "pine"));
    }

    #[test]
    fn named_color_returns_none_for_unknown() {
        let mocha = Theme::Catppuccin(CatppuccinFlavor::Mocha);
        assert_eq!(named_color(mocha, "chartreuse"), None);
        // Hex codes are not resolved as named colors.
        assert_eq!(named_color(mocha, "#89b4fa"), None);
    }

    #[test]
    fn family_specific_names_do_not_leak_across_families() {
        // `pine` is a Rosé Pine name, not a Catppuccin one.
        assert!(named_color(Theme::RosePine(RosePineFlavor::Main), "pine").is_some());
        assert!(named_color(Theme::Catppuccin(CatppuccinFlavor::Mocha), "pine").is_none());

        // `mauve` resolves under both Catppuccin (native) and Tokyo
        // Night (alias for purple).
        assert!(named_color(Theme::Catppuccin(CatppuccinFlavor::Mocha), "mauve").is_some());
        assert!(named_color(Theme::TokyoNight(TokyoNightFlavor::Night), "mauve").is_some());
    }

    #[test]
    fn common_aliases_resolve_in_every_theme() {
        // The four most-used names work under every supported theme.
        // Distinct themes return distinct shades.
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
        for name in ["red", "green", "blue", "yellow"] {
            for theme in themes {
                assert!(
                    named_color(theme, name).is_some(),
                    "{name} should resolve in {theme:?}"
                );
            }
        }
    }
}
