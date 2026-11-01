//! The banner under a row being changed, shown when the shortcut
//! pressed already belongs to another binding.

use eframe::egui::{
    Align, Button, Color32, Frame, Layout, Margin, RichText, Stroke, Ui, Widget, vec2,
};

use crate::icons::{self, Icon};
use crate::theme::{self, Colors};

/// What the user chose on the banner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Undecided,
    Cancel,
    Replace,
}

/// Draws a banner saying `shortcut` is already bound to `owner`, with
/// Cancel and Replace buttons.
pub fn draw(ui: &mut Ui, c: &Colors, shortcut: &str, owner: &str) -> Choice {
    let mut choice = Choice::Undecided;
    Frame::new()
        .fill(c.warning.gamma_multiply(0.12))
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            let size = vec2(ui.available_width(), 24.0);
            ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                if button(ui, "Replace", c.accent, c.on_accent).clicked() {
                    choice = Choice::Replace;
                }
                if button(ui, "Cancel", Color32::TRANSPARENT, c.text).clicked() {
                    choice = Choice::Cancel;
                }
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    message(ui, c, shortcut, owner)
                });
            });
        });
    choice
}

fn message(ui: &mut Ui, c: &Colors, shortcut: &str, owner: &str) {
    ui.spacing_mut().item_spacing.x = 4.0;
    let (rect, _) = ui.allocate_exact_size(vec2(15.0, 15.0), eframe::egui::Sense::hover());
    icons::paint(ui.painter(), rect, Icon::Warning, c.warning, 1.5);
    ui.add_space(6.0);
    let said = format!("{shortcut} is already bound to");
    ui.label(RichText::new(said).font(theme::regular(12.0)));
    ui.label(RichText::new(owner).font(theme::bold(12.0)));
}

fn button(ui: &mut Ui, label: &str, fill: Color32, text: Color32) -> eframe::egui::Response {
    let text = RichText::new(label).font(theme::bold(12.0)).color(text);
    let button = Button::new(text)
        .fill(fill)
        .stroke(Stroke::NONE)
        .corner_radius(5)
        .min_size(vec2(0.0, 24.0));
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = vec2(12.0, 4.0);
        button.ui(ui)
    })
    .inner
}
