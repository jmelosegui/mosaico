//! One binding of the Keybindings page: what it does, its key
//! combination as keycaps, and the buttons to change it.

use eframe::egui::{Frame, Margin, Response, RichText, Role, Stroke, Ui, WidgetInfo};

use crate::theme::{self, Colors};
use crate::widgets::{self, RowState};

/// What a row shows of a binding.
pub struct RowView<'a> {
    pub label: &'a str,
    /// The combination, as `Alt+Shift+J`.
    pub combo: &'a str,
    /// The combination was edited and not applied.
    pub unsaved: bool,
    /// The default combination, when the binding is not on it.
    pub default: Option<&'a str>,
    /// What else the combination is bound to, when something is.
    pub clash: Option<&'a str>,
    /// The line under the label while the binding is being changed.
    pub capturing: Option<&'a str>,
}

/// What the user did on a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Click {
    Nothing,
    Change,
    Cancel,
    Reset,
}

/// Draws a row and reports what was clicked.
pub fn draw(ui: &mut Ui, c: &Colors, view: &RowView) -> Click {
    let mut click = Click::Nothing;
    let state = RowState {
        unsaved: view.unsaved,
        default: view.default.map(str::to_owned),
    };
    let help = view.capturing.or(view.clash);
    let reset = widgets::row(ui, c, view.label, help, &state, |ui| {
        if view.capturing.is_some() {
            let cancel = named_button(ui, c, "Cancel", &format!("Cancel changing {}", view.label));
            if cancel.clicked() {
                click = Click::Cancel;
            }
            prompt(ui, c);
        } else {
            let change = named_button(ui, c, "Change", &format!("Change {}", view.label));
            if change.clicked() {
                click = Click::Change;
            }
            for part in view.combo.split('+').rev() {
                keycap(ui, c, part, view.unsaved);
            }
            if view.clash.is_some() {
                let text = RichText::new("Conflict")
                    .font(theme::bold(11.0))
                    .color(c.warning);
                ui.label(text);
            }
        }
    });
    if reset {
        click = Click::Reset;
    }
    click
}

fn named_button(ui: &mut Ui, c: &Colors, text: &str, name: &str) -> Response {
    let response = widgets::button(ui, c, text, false, true);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
    response
}

fn keycap(ui: &mut Ui, c: &Colors, text: &str, changed: bool) {
    let stroke = Stroke::new(1.0, if changed { c.accent } else { c.line });
    Frame::new()
        .fill(c.control)
        .stroke(stroke)
        .corner_radius(5)
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).font(theme::bold(12.0)));
        });
}

fn prompt(ui: &mut Ui, c: &Colors) {
    Frame::new()
        .stroke(Stroke::new(1.0, c.accent))
        .corner_radius(5)
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            let text = RichText::new("Press the new keys…")
                .font(theme::regular(12.0))
                .color(c.accent);
            ui.label(text);
        });
}
