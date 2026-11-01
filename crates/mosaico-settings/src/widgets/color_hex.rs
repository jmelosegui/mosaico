//! The hex entry of a color picker's popup.

use eframe::egui::{Align, Id, Layout, RichText, Ui};
use mosaico_core::config::Color;

use super::color::{ColorPicker, hex};
use super::color_popup::caption;
use crate::theme::{self, Colors};

/// A line for a `#rrggbb` color, which takes the value only once it is one.
pub(super) fn hex_entry(ui: &mut Ui, c: &Colors, picker: &ColorPicker, id: Id, value: &mut String) {
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
    ui.horizontal(|ui| {
        caption(ui, "Hex", false, c.dim_text);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            super::text(ui, c, &label, 108.0, &mut text, "#rrggbb", false);
        });
    });
    let parsed = Color::from_hex(&text);
    if let (true, Some(color)) = (text != before, parsed) {
        *value = hex(color);
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
