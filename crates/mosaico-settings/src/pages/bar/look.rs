//! The colors and measures the bar preview is drawn with, from the
//! settings being edited.

use eframe::egui::Color32;
use mosaico_core::config::schema::SettingValue;
use mosaico_core::config::{BarWidget, Theme};

use super::state::BarState;

/// The colors and measures the bar is drawn with.
pub(super) struct Look {
    pub(super) background: Color32,
    pub(super) text: Color32,
    pub(super) pill: Color32,
    pub(super) border: Color32,
    pub(super) active: Color32,
    pub(super) active_text: Color32,
    pub(super) inactive_text: Color32,
    pub(super) theme: Theme,
    pub(super) height: f32,
    pub(super) font: f32,
    pub(super) edge: f32,
    pub(super) pad: f32,
    pub(super) radius: f32,
    pub(super) border_width: f32,
    pub(super) gap: f32,
    pub(super) workspace_gap: f32,
}

impl Look {
    pub(super) fn of(edit: &BarState, theme: Theme) -> Self {
        let color = |key: &str| {
            let Some(SettingValue::Text(value)) = edit.get(key).cloned() else {
                return Color32::TRANSPARENT;
            };
            let fallback = theme.default_color(key).unwrap_or_default();
            theme
                .swatch(&value, &fallback)
                .map_or(Color32::TRANSPARENT, |c| Color32::from_rgb(c.r, c.g, c.b))
        };
        let number = |key: &str, scale: f32| match edit.get(key) {
            Some(SettingValue::Int(n)) => *n as f32 * scale,
            _ => 0.0,
        };
        let opacity = number("background_opacity", 0.01).clamp(0.0, 1.0);
        Self {
            background: color("colors.background").gamma_multiply(opacity),
            text: color("colors.foreground"),
            pill: color("colors.widget_background"),
            border: color("colors.pill_border"),
            active: color("colors.active_workspace"),
            active_text: color("colors.active_workspace_text"),
            inactive_text: color("colors.inactive_workspace"),
            theme,
            height: number("height", 0.5).clamp(16.0, 48.0),
            font: number("font_size", 0.5).clamp(8.0, 14.0),
            edge: number("padding", 0.5),
            pad: number("pill_padding", 0.5),
            radius: number("pill_radius", 0.5),
            border_width: number("pill_border_width", 0.5),
            gap: number("item_gap", 0.5),
            workspace_gap: number("workspace_gap", 0.5),
        }
    }
}

/// The text color of widget, or the bar's own when it sets none.
pub(super) fn widget_color(widget: &BarWidget, look: &Look) -> Color32 {
    look.theme
        .swatch(&widget.color, "")
        .map_or(look.text, |c| Color32::from_rgb(c.r, c.g, c.b))
}
