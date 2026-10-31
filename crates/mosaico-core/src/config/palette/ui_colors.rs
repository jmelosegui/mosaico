//! The colors of an app's surfaces, text and accents for a theme, so an
//! app such as the settings window matches the borders and the bar.

use crate::config::theme::Theme;

use super::named_color;
use super::surfaces::surfaces;

/// What each part of an app's window is painted with, as `#rrggbb`.
///
/// Dark flavors raise cards above the page. Light flavors make cards the
/// lightest surface, on a slightly darker page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiColors {
    /// Whether the flavor is dark, so text is light on dark surfaces.
    pub dark: bool,
    /// The title bar, the darkest surface.
    pub title_bar: &'static str,
    /// The navigation pane beside the page.
    pub sidebar: &'static str,
    /// Behind the cards of a page.
    pub page: &'static str,
    /// A group of settings.
    pub card: &'static str,
    /// Fields, drop-downs and switches off, set apart from the card.
    pub control: &'static str,
    /// Card outlines and separators.
    pub line: &'static str,
    /// A control or item under the pointer.
    pub hover: &'static str,
    /// A control being pressed, or the selected item.
    pub pressed: &'static str,
    /// Labels and values.
    pub text: &'static str,
    /// Help under a label, and secondary text.
    pub dim_text: &'static str,
    /// Placeholders and disabled text.
    pub faint_text: &'static str,
    /// Primary buttons, switches on and the selection, the theme's
    /// focused border color.
    pub accent: &'static str,
    /// Text on [`UiColors::accent`].
    pub on_accent: &'static str,
    /// Something that works, such as a running daemon.
    pub good: &'static str,
    /// Something to look at, such as an unsaved change.
    pub warning: &'static str,
    /// Something that failed.
    pub bad: &'static str,
}

impl UiColors {
    /// The colors for `theme`.
    pub fn of(theme: Theme) -> Self {
        let s = surfaces(theme);
        let named = |name| named_color(theme, name).unwrap_or(s.text);
        let (page, card, control, line, hover, pressed) = if s.dark {
            (
                s.base, s.surface0, s.base, s.surface1, s.surface1, s.surface2,
            )
        } else {
            (
                s.mantle, s.base, s.mantle, s.surface0, s.surface0, s.surface1,
            )
        };
        Self {
            dark: s.dark,
            title_bar: s.crust,
            sidebar: s.mantle,
            page,
            card,
            control,
            line,
            hover,
            pressed,
            text: s.text,
            dim_text: s.subtext,
            faint_text: s.overlay,
            accent: theme.border_focused(),
            on_accent: s.base,
            good: named("green"),
            warning: named("yellow"),
            bad: named("red"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::theme::CatppuccinFlavor;

    fn luma(hex: &str) -> u32 {
        let v = u32::from_str_radix(&hex[1..], 16).expect("hex color");
        (v >> 16 & 0xff) * 3 + (v >> 8 & 0xff) * 6 + (v & 0xff)
    }

    #[test]
    fn dark_flavors_raise_cards_and_light_ones_make_them_lightest() {
        for theme in Theme::ALL {
            let c = UiColors::of(theme);
            if c.dark {
                assert!(luma(c.card) > luma(c.page), "{theme:?}");
                assert!(luma(c.text) > luma(c.card), "{theme:?}");
            } else {
                assert!(luma(c.card) >= luma(c.page), "{theme:?}");
                assert!(luma(c.text) < luma(c.card), "{theme:?}");
            }
            assert!(luma(c.title_bar) <= luma(c.page), "{theme:?}");
        }
    }

    #[test]
    fn the_accent_is_the_focused_border_color() {
        let mocha = Theme::Catppuccin(CatppuccinFlavor::Mocha);
        assert_eq!(UiColors::of(mocha).accent, "#89b4fa");
        assert_eq!(UiColors::of(mocha).page, "#1e1e2e");
    }
}
