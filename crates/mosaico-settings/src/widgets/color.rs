//! A color picker: a swatch that opens the theme's palette, a hex entry
//! and the choices that stand for no color of its own.

use eframe::egui::{
    Button, Color32, CornerRadius, Id, Popup, PopupCloseBehavior, Rect, Response, RichText, Role,
    Sense, Stroke, StrokeKind, Ui, WidgetInfo, vec2,
};
use mosaico_core::config::{Color, Theme};

use crate::theme::{self, Colors};

const SWATCH: f32 = 22.0;
const WIDTH: f32 = 236.0;

/// What a [`color`] picker edits and shows.
pub struct ColorPicker<'a> {
    /// Names the picker for assistive technology.
    pub label: &'a str,
    /// The theme being edited, whose palette is offered.
    pub theme: Theme,
    /// The hex shown when the value is empty.
    pub fallback: &'a str,
    /// Whether `"none"` is offered.
    pub allow_none: bool,
    /// Whether the setting differs from the saved one.
    pub changed: bool,
}

fn paint(c: Color) -> Color32 {
    Color32::from_rgb(c.r, c.g, c.b)
}

/// How a value reads on the picker: "Theme default", "None", a name or hex.
fn caption(value: &str) -> &str {
    if value.is_empty() {
        "Theme default"
    } else {
        value
    }
}

/// Edits `value`, text as `config.toml` holds it: empty for the theme's
/// color, `"none"`, a palette name or `#rrggbb`.
pub fn color(ui: &mut Ui, c: &Colors, picker: &ColorPicker, value: &mut String) -> Response {
    let swatch = picker.theme.swatch(value, picker.fallback);
    let stroke = if picker.changed {
        Stroke::new(1.0, c.accent)
    } else {
        Stroke::NONE
    };
    let text = RichText::new(caption(value))
        .font(theme::regular(12.0))
        .color(c.text);
    ui.spacing_mut().button_padding = vec2(34.0, 4.0);
    let response = ui.add(
        Button::new(text)
            .fill(c.control)
            .stroke(stroke)
            .corner_radius(5)
            .min_size(vec2(150.0, 26.0)),
    );
    let chip = Rect::from_min_size(
        response.rect.left_center() + vec2(6.0, -9.0),
        vec2(18.0, 18.0),
    );
    paint_chip(ui, c, chip, swatch);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, picker.label));

    let popup = Popup::from_toggle_button_response(&response)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .width(WIDTH);
    let mut changed = false;
    popup.show(|ui| changed = menu(ui, c, picker, value));
    let mut response = response;
    if changed {
        response.mark_changed();
    }
    response
}

fn paint_chip(ui: &Ui, c: &Colors, rect: Rect, color: Option<Color>) {
    let painter = ui.painter();
    match color {
        Some(color) => {
            painter.rect(
                rect,
                4,
                paint(color),
                Stroke::new(1.0, c.line),
                StrokeKind::Inside,
            );
        }
        None => {
            painter.rect_stroke(rect, 4, Stroke::new(1.0, c.faint_text), StrokeKind::Inside);
            painter.line_segment(
                [rect.left_bottom(), rect.right_top()],
                Stroke::new(1.0, c.faint_text),
            );
        }
    }
}

/// A row of the popup, which is on when it is the value; whether it was clicked.
fn choice(ui: &mut Ui, c: &Colors, label: &str, on: bool) -> bool {
    let text = RichText::new(label)
        .font(theme::regular(12.0))
        .color(c.text);
    let fill = if on { c.selected } else { Color32::TRANSPARENT };
    let button = Button::new(text)
        .fill(fill)
        .corner_radius(5)
        .min_size(vec2(WIDTH, 26.0))
        .wrap_mode(eframe::egui::TextWrapMode::Extend);
    let response = ui.add(button);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
    response.clicked()
}

fn palette_swatch(ui: &mut Ui, c: &Colors, name: &str, rgb: Option<Color>, on: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(SWATCH, SWATCH), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
    let response = response.on_hover_text(name);
    let line = if on || response.hovered() {
        Stroke::new(2.0, c.accent)
    } else {
        Stroke::new(1.0, c.line)
    };
    let fill = rgb.map_or(Color32::TRANSPARENT, paint);
    ui.painter()
        .rect(rect, CornerRadius::same(5), fill, line, StrokeKind::Inside);
    response.clicked()
}

fn menu(ui: &mut Ui, c: &Colors, picker: &ColorPicker, value: &mut String) -> bool {
    ui.set_width(WIDTH);
    let before = value.clone();
    let hex_id = ui.make_persistent_id(("color-hex", picker.label));
    let mut picked = false;
    if choice(ui, c, "Theme default", value.is_empty()) {
        value.clear();
        picked = true;
    }
    if picker.allow_none && choice(ui, c, "None", value == "none") {
        *value = "none".into();
        picked = true;
    }
    ui.add_space(6.0);
    ui.label(
        RichText::new(format!("{} palette", picker.theme.label()))
            .font(theme::regular(11.0))
            .color(c.faint_text),
    );
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        for name in picker.theme.color_names() {
            let rgb = picker.theme.swatch(name, "");
            if palette_swatch(ui, c, name, rgb, value.eq_ignore_ascii_case(name)) {
                *value = name.into();
                picked = true;
            }
        }
    });
    ui.add_space(6.0);
    if picked {
        ui.data_mut(|d| d.remove::<String>(hex_id));
        ui.close();
    } else {
        hex_entry(ui, c, picker, hex_id, value);
    }
    *value != before
}

/// A line for a `#rrggbb` color, which takes the value only once it is one.
fn hex_entry(ui: &mut Ui, c: &Colors, picker: &ColorPicker, id: Id, value: &mut String) {
    let mut text = ui
        .data_mut(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| {
            if value.starts_with('#') {
                value.clone()
            } else {
                String::new()
            }
        });
    let before = text.clone();
    let label = format!("{} hex", picker.label);
    super::text(ui, c, &label, 90.0, &mut text, "#RRGGBB", false);
    let parsed = Color::from_hex(&text);
    if let (true, Some(hex)) = (text != before, parsed) {
        *value = format!("#{:02x}{:02x}{:02x}", hex.r, hex.g, hex.b);
    }
    if !text.is_empty() && parsed.is_none() {
        ui.label(
            RichText::new("Use six hex digits, like #89b4fa.")
                .font(theme::regular(11.0))
                .color(c.bad),
        );
    }
    ui.data_mut(|d| d.insert_temp(id, text));
}
