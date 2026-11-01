//! A color picker: a button with the color's swatch and name that opens
//! the theme's palette, a hex entry and the choices that stand for no
//! color of its own.

use eframe::egui::{
    Color32, CursorIcon, Frame, Margin, Popup, PopupCloseBehavior, RectAlign, Response, Role,
    Sense, Stroke, Ui, WidgetInfo, epaint::Shadow, vec2,
};
use mosaico_core::config::{Color, Theme};

use super::color_button::{Button, paint_button};
use super::color_popup::{self, WIDTH};
use crate::theme::Colors;

const BUTTON: f32 = 190.0;

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

pub(super) fn paint(c: Color) -> Color32 {
    Color32::from_rgb(c.r, c.g, c.b)
}

/// `c` as `#rrggbb`.
pub(super) fn hex(c: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

/// How a value reads on the button, and whether it is dimmed because it
/// is the theme's.
fn caption(picker: &ColorPicker, value: &str) -> (String, bool) {
    if value.is_empty() {
        let text = match picker.theme.color_name(picker.fallback) {
            Some(name) => format!("Theme default ({name})"),
            None => "Theme default".into(),
        };
        (text, true)
    } else if value.eq_ignore_ascii_case("none") {
        ("None".into(), false)
    } else {
        (value.to_owned(), false)
    }
}

/// Edits `value`, text as `config.toml` holds it: empty for the theme's
/// color, `"none"`, a palette name or `#rrggbb`.
pub fn color(ui: &mut Ui, c: &Colors, picker: &ColorPicker, value: &mut String) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(BUTTON, 26.0), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, picker.label));
    let open = Popup::is_id_open(ui.ctx(), Popup::default_response_id(&response));
    if ui.is_rect_visible(rect) {
        let (text, dim) = caption(picker, value);
        let swatch = picker.theme.swatch(value, picker.fallback);
        let button = Button {
            rect,
            text: &text,
            dim,
            swatch,
            outlined: picker.changed || open,
            open,
        };
        paint_button(ui, c, &button);
    }
    let response = response.on_hover_cursor(CursorIcon::PointingHand);

    let popup = Popup::from_toggle_button_response(&response)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .align(RectAlign::BOTTOM_END)
        .gap(6.0)
        .frame(popover(c))
        .width(WIDTH);
    let mut changed = false;
    popup.show(|ui| changed = color_popup::menu(ui, c, picker, value));
    let mut response = response;
    if changed {
        response.mark_changed();
    }
    response
}

/// The frame of a popover: a card with a shadow.
fn popover(c: &Colors) -> Frame {
    let alpha = if c.dark { 0x80 } else { 0x33 };
    Frame::new()
        .fill(c.card)
        .stroke(Stroke::new(1.0, c.line))
        .corner_radius(8)
        .inner_margin(Margin::same(12))
        .shadow(Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(alpha),
        })
}
