//! The popup of a color picker: the theme's colors, the one picked and
//! what else can be done with the setting.

use eframe::egui::{
    Align, Button, Color32, CornerRadius, Layout, Rect, RichText, Role, Sense, Stroke, StrokeKind,
    Ui, WidgetInfo, vec2,
};
use mosaico_core::config::Color;

use super::color::{ColorPicker, hex, paint};
use super::color_hex::hex_entry;
use crate::icons::{self, Icon};
use crate::theme::{self, Colors};

/// How wide the popup's content is.
pub const WIDTH: f32 = 230.0;
const SWATCH: f32 = 26.0;
const PER_LINE: usize = 7;

pub(super) fn caption(ui: &mut Ui, text: &str, bold: bool, color: Color32) {
    let font = if bold {
        theme::bold(12.0)
    } else {
        theme::regular(12.0)
    };
    ui.label(RichText::new(text).font(font).color(color));
}

fn palette_swatch(ui: &mut Ui, c: &Colors, name: &str, rgb: Option<Color>, on: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(SWATCH, SWATCH), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
    let response = response.on_hover_text(name);
    let line = if on {
        Stroke::new(2.0, c.text)
    } else if response.hovered() {
        Stroke::new(1.0, c.faint_text)
    } else {
        Stroke::NONE
    };
    let fill = rgb.map_or(Color32::TRANSPARENT, paint);
    ui.painter()
        .rect(rect, CornerRadius::same(6), fill, line, StrokeKind::Inside);
    response.clicked()
}

/// A line of the popup that is a button, with `icon` before it.
fn action(ui: &mut Ui, c: &Colors, label: &str, icon: Option<Icon>, color: Color32) -> bool {
    let font = theme::regular(12.0);
    let galley = ui.painter().layout_no_wrap(label.into(), font, color);
    let lead = if icon.is_some() { 21.0 } else { 0.0 };
    let size = vec2(WIDTH, 18.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    let painter = ui.painter();
    if let Some(icon) = icon {
        let at = Rect::from_center_size(rect.left_center() + vec2(6.5, 0.0), vec2(13.0, 13.0));
        icons::paint(painter, at, icon, c.accent, 1.5);
    }
    let pos = rect.left_center() + vec2(lead, -galley.size().y / 2.0);
    painter.galley_with_override_text_color(pos, galley, color);
    response
        .on_hover_cursor(eframe::egui::CursorIcon::PointingHand)
        .clicked()
}

fn none_choice(ui: &mut Ui, c: &Colors, on: bool) -> bool {
    let text = RichText::new("None")
        .font(theme::regular(12.0))
        .color(c.text);
    let fill = if on { c.selected } else { Color32::TRANSPARENT };
    let response = ui.add(
        Button::new(text)
            .fill(fill)
            .corner_radius(5)
            .min_size(vec2(WIDTH, 24.0)),
    );
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, "None"));
    response.clicked()
}

/// Draws the popup and edits `value`; whether it changed.
pub fn menu(ui: &mut Ui, c: &Colors, picker: &ColorPicker, value: &mut String) -> bool {
    ui.set_width(WIDTH);
    ui.spacing_mut().item_spacing = vec2(6.0, 10.0);
    let before = value.clone();
    let hex_id = ui.make_persistent_id(("color-hex", picker.label));
    let none = value == "none";
    let shown = picker.theme.swatch(value, picker.fallback);
    let current = shown.map(hex);
    let name = current
        .as_deref()
        .map(|hex| picker.theme.color_name(hex).unwrap_or("Custom"));
    let mut picked = false;

    caption(ui, "Theme colors", true, c.dim_text);
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        for line in picker.theme.color_names().chunks(PER_LINE) {
            ui.horizontal(|ui| {
                for &swatch in line {
                    let rgb = picker.theme.swatch(swatch, "");
                    if palette_swatch(ui, c, swatch, rgb, name == Some(swatch)) {
                        *value = swatch.into();
                        picked = true;
                    }
                }
            });
        }
    });
    if picker.allow_none && none_choice(ui, c, none) {
        *value = "none".into();
        picked = true;
    }
    ui.horizontal(|ui| {
        let (name, code) = match (name, &current) {
            (Some(name), Some(code)) => (name, code.to_uppercase()),
            _ => ("None", String::new()),
        };
        caption(ui, name, true, c.text);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            caption(ui, &code, false, c.faint_text);
        });
    });
    super::divider(ui, c);
    if !picked {
        hex_entry(ui, c, picker, hex_id, value);
        if !value.is_empty() {
            let label = format!("Use the theme default ({})", default_name(picker));
            if action(ui, c, &label, Some(Icon::Reset), c.accent) {
                value.clear();
                picked = true;
            }
        }
    }
    if picked {
        ui.data_mut(|d| d.remove::<String>(hex_id));
        ui.close();
    }
    *value != before
}

/// The name of the color the picker has when it is empty, or its hex.
pub fn default_name<'a>(picker: &'a ColorPicker) -> &'a str {
    picker
        .theme
        .color_name(picker.fallback)
        .unwrap_or(picker.fallback)
}
