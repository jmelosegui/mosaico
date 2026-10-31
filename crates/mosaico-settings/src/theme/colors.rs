//! The theme's [`UiColors`] as the colors egui paints with.

use eframe::egui::Color32;
use mosaico_core::config::{Theme, UiColors};

/// What each part of the window is painted with. See [`UiColors`] for
/// the roles shared with core.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colors {
    pub dark: bool,
    pub title_bar: Color32,
    pub sidebar: Color32,
    pub page: Color32,
    pub card: Color32,
    pub control: Color32,
    pub line: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    /// A selected segment, and buttons that are not the primary one.
    pub raised: Color32,
    /// Behind the selected navigation item.
    pub selected: Color32,
    pub text: Color32,
    pub dim_text: Color32,
    pub faint_text: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub good: Color32,
    pub warning: Color32,
    pub bad: Color32,
}

impl Colors {
    /// The colors of `theme`.
    pub fn of(theme: Theme) -> Self {
        let ui = UiColors::of(theme);
        let hex = |hex: &str| Color32::from_hex(hex).unwrap_or(Color32::MAGENTA);
        let accent = hex(ui.accent);
        let (raised, tint) = if ui.dark {
            (hex(ui.hover), 0.18)
        } else {
            (hex(ui.pressed), 0.14)
        };
        Self {
            dark: ui.dark,
            title_bar: hex(ui.title_bar),
            sidebar: hex(ui.sidebar),
            page: hex(ui.page),
            card: hex(ui.card),
            control: hex(ui.control),
            line: hex(ui.line),
            hover: hex(ui.hover),
            pressed: hex(ui.pressed),
            raised,
            selected: accent.gamma_multiply(tint),
            text: hex(ui.text),
            dim_text: hex(ui.dim_text),
            faint_text: hex(ui.faint_text),
            accent,
            on_accent: hex(ui.on_accent),
            good: hex(ui.good),
            warning: hex(ui.warning),
            bad: hex(ui.bad),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_color_parses() {
        for theme in Theme::ALL {
            let c = Colors::of(theme);
            let all = [c.title_bar, c.page, c.card, c.text, c.accent, c.good, c.bad];
            assert!(!all.contains(&Color32::MAGENTA), "{theme:?}");
        }
    }
}
