//! The colors a theme offers a border, and what a border setting shows.

use crate::config::palette;
use crate::config::{Color, Theme};

impl Theme {
    /// The names of the theme's palette colors, such as `"mauve"`, in the
    /// order the palette lists them. Each resolves with
    /// [`Theme::named_color`].
    pub fn color_names(self) -> Vec<&'static str> {
        palette::color_names(self)
    }

    /// The color the border setting at `key` has when it is empty, such
    /// as `"borders.colors.focused"`. `None` for any other key.
    pub fn border_default(self, key: &str) -> Option<&'static str> {
        match key {
            "borders.colors.focused" => Some(self.border_focused()),
            "borders.colors.monocle" => Some(self.border_monocle()),
            "borders.colors.unfocused" => Some(self.border_unfocused()),
            _ => None,
        }
    }

    /// The color the setting at `key` has when it is empty: a border of
    /// `config.toml`, such as `"borders.colors.focused"`, or a color of
    /// `bar.toml`, such as `"colors.background"`. `None` for any other
    /// key.
    pub fn default_color(self, key: &str) -> Option<String> {
        if let Some(color) = self.border_default(key) {
            return Some(color.into());
        }
        let bar = self.bar_colors();
        let color = match key.strip_prefix("colors.")? {
            "background" => bar.background,
            "foreground" => bar.foreground,
            "active_workspace" => bar.active_workspace,
            "active_workspace_text" => bar.active_workspace_text,
            "inactive_workspace" => bar.inactive_workspace,
            "separator" => bar.separator,
            "accent" => bar.accent,
            "widget_background" => bar.widget_background,
            "pill_border" => bar.pill_border,
            _ => return None,
        };
        Some(color)
    }

    /// The name the palette gives `hex`, such as `"blue"`, if it lists
    /// that color.
    pub fn color_name(self, hex: &str) -> Option<&'static str> {
        self.color_names().into_iter().find(|name| {
            self.named_color(name)
                .is_some_and(|color| color.eq_ignore_ascii_case(hex.trim()))
        })
    }

    /// The color a setting holding `value` is drawn with: `fallback` when
    /// it is empty, the palette's color for a name, or the hex itself.
    /// `None` for `"none"` and for text that is no color.
    pub fn swatch(self, value: &str, fallback: &str) -> Option<Color> {
        let value = value.trim();
        if value.eq_ignore_ascii_case("none") {
            return None;
        }
        Color::from_hex(self.resolve_color(value, fallback))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CatppuccinFlavor, RosePineFlavor};

    const MOCHA: Theme = Theme::Catppuccin(CatppuccinFlavor::Mocha);

    #[test]
    fn color_names_are_those_the_theme_resolves() {
        for theme in Theme::ALL {
            let names = theme.color_names();
            assert!(!names.is_empty(), "{theme:?}");
            for name in names {
                assert!(theme.named_color(name).is_some(), "{theme:?} {name}");
            }
        }
    }

    #[test]
    fn color_names_follow_the_theme() {
        assert!(MOCHA.color_names().contains(&"mauve"));
        let pine = Theme::RosePine(RosePineFlavor::Main);
        assert!(pine.color_names().contains(&"iris"));
        assert!(!MOCHA.color_names().contains(&"iris"));
    }

    #[test]
    fn color_name_is_the_palette_name_of_a_hex() {
        assert_eq!(MOCHA.color_name("#89B4FA"), Some("blue"));
        assert_eq!(MOCHA.color_name("#123456"), None);
    }

    #[test]
    fn border_default_is_the_themes_color_for_each_border() {
        assert_eq!(
            MOCHA.border_default("borders.colors.focused"),
            Some(MOCHA.border_focused())
        );
        assert_eq!(
            MOCHA.border_default("borders.colors.monocle"),
            Some(MOCHA.border_monocle())
        );
        assert_eq!(
            MOCHA.border_default("borders.colors.unfocused"),
            Some(MOCHA.border_unfocused())
        );
        assert_eq!(MOCHA.border_default("borders.width"), None);
    }

    #[test]
    fn swatch_resolves_names_hex_and_the_fallback() {
        let blue = Color::from_hex("#89b4fa");
        assert_eq!(MOCHA.swatch("blue", "#000000"), blue);
        assert_eq!(MOCHA.swatch("#89B4FA", "#000000"), blue);
        assert_eq!(MOCHA.swatch("", "#89b4fa"), blue);
    }

    #[test]
    fn swatch_is_none_for_none_and_for_text_that_is_no_color() {
        assert_eq!(MOCHA.swatch("none", "#89b4fa"), None);
        assert_eq!(MOCHA.swatch("chartreuse", "#89b4fa"), None);
        assert_eq!(MOCHA.swatch("#12", "#89b4fa"), None);
    }

    #[test]
    fn default_color_covers_the_borders_and_the_bar() {
        assert_eq!(
            MOCHA.default_color("borders.colors.focused").as_deref(),
            Some(MOCHA.border_focused())
        );
        assert_eq!(
            MOCHA.default_color("colors.active_workspace_text"),
            Some(MOCHA.bar_colors().active_workspace_text)
        );
        assert_eq!(MOCHA.default_color("colors.nothing"), None);
        assert_eq!(MOCHA.default_color("layout.gap"), None);
    }
}
