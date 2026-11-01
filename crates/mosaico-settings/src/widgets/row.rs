//! Sections of settings: a heading over a card of rows, each with a
//! label, help and a control.

use eframe::egui::{Align, Frame, Layout, Margin, Response, RichText, Sense, Stroke, Ui, vec2};

use crate::icons::{self, Icon};
use crate::theme::{self, Colors};

/// How a row's setting differs from what is saved and from the default.
#[derive(Debug, Clone, Default)]
pub struct RowState {
    /// The setting was edited and not applied.
    pub unsaved: bool,
    /// The default, as shown, when the setting is not at its default.
    pub default: Option<String>,
}

/// A heading over a card. `rows` adds the rows, with [`divider`]s
/// between them.
pub fn section(ui: &mut Ui, c: &Colors, title: &str, rows: impl FnOnce(&mut Ui)) {
    heading(ui, c, title, |_| {});
    card(ui, c, rows);
    ui.add_space(16.0);
}

/// The title of a section, with `right` adding controls at its far end,
/// such as a button.
pub fn heading(ui: &mut Ui, c: &Colors, title: &str, right: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(title)
                .font(theme::bold(13.0))
                .color(c.dim_text),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), right);
    });
    ui.add_space(2.0);
}

/// A card of rows, which `rows` adds with [`divider`]s between them.
pub fn card(ui: &mut Ui, c: &Colors, rows: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(c.card)
        .stroke(Stroke::new(1.0, c.line))
        .corner_radius(8)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            rows(ui);
        });
}

/// A line between two rows of a card.
pub fn divider(ui: &mut Ui, c: &Colors) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, c.line);
}

/// A row for one setting. `control` adds the control on the right.
/// Returns whether the reset button, shown when the setting is not at its
/// default, was clicked.
pub fn row(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    help: Option<&str>,
    state: &RowState,
    control: impl FnOnce(&mut Ui),
) -> bool {
    let mut reset = false;
    Frame::new()
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            let size = vec2(ui.available_width(), 26.0);
            ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                control(ui);
                if state.default.is_some() {
                    reset = reset_button(ui, c).clicked();
                }
                ui.add_space(8.0);
                let controls = ui.min_rect().height();
                let help = match (help, &state.default) {
                    (Some(help), Some(default)) => Some(format!("{help}  Default: {default}.")),
                    (Some(help), None) => Some(help.to_owned()),
                    (None, Some(default)) => Some(format!("Default: {default}.")),
                    (None, None) => None,
                };
                let Some(help) = help else {
                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        ui.add_space(((controls - LABEL_HEIGHT) / 2.0).max(0.0));
                        title(ui, c, label, state.unsaved);
                    });
                    return;
                };
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.y = 3.0;
                    title(ui, c, label, state.unsaved);
                    ui.label(
                        RichText::new(help)
                            .font(theme::regular(11.0))
                            .color(c.dim_text),
                    );
                });
            });
        });
    reset
}

/// The height of a row's label line.
const LABEL_HEIGHT: f32 = 16.0;

fn title(ui: &mut Ui, c: &Colors, label: &str, unsaved: bool) {
    ui.spacing_mut().interact_size.y = 0.0;
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).font(theme::bold(13.0)));
        if unsaved {
            let (rect, _) = ui.allocate_exact_size(vec2(6.0, 6.0), Sense::hover());
            ui.painter().circle_filled(rect.center(), 3.0, c.accent);
        }
    });
}

fn reset_button(ui: &mut Ui, c: &Colors) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(26.0, 26.0), Sense::click());
    let fill = if response.hovered() {
        c.hover
    } else {
        c.control
    };
    ui.painter().rect_filled(rect, 5.0, fill);
    let icon = eframe::egui::Rect::from_center_size(rect.center(), vec2(14.0, 14.0));
    icons::paint(ui.painter(), icon, Icon::Reset, c.accent, 1.5);
    response
        .on_hover_text("Reset to the default")
        .on_hover_cursor(eframe::egui::CursorIcon::PointingHand)
}
