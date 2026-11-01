//! One binding of the Keybindings page: what it does, its shortcut as
//! keycaps, and the button to rebind it.

use eframe::egui::{Align, Color32, Frame, Layout, Margin, RichText, Sense, Stroke, Ui, vec2};

use crate::icons::Icon;
use crate::theme::{self, Colors};
use crate::widgets;

/// What a row shows of a binding.
pub struct RowView<'a> {
    pub label: &'a str,
    /// The action as `keybindings.toml` names it.
    pub id: &'a str,
    /// The keys of the shortcut, as `["Alt", "Shift", "J"]`.
    pub keys: &'a [String],
    /// The shortcut was edited and not applied.
    pub unsaved: bool,
    /// The default shortcut, when the binding is not on it.
    pub default: Option<&'a str>,
    /// What else the shortcut is bound to, when something is.
    pub clash: Option<&'a str>,
    /// What the row says while the binding is being changed.
    pub capturing: Option<Prompt<'a>>,
}

/// The line a row shows while it waits for a key press.
#[derive(Clone, Copy)]
pub struct Prompt<'a> {
    pub text: &'a str,
    /// The text says why the last press was refused.
    pub refused: bool,
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
    let fill = if view.capturing.is_some() {
        c.selected
    } else {
        Color32::TRANSPARENT
    };
    Frame::new()
        .fill(fill)
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            let size = vec2(ui.available_width(), 26.0);
            ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                click = controls(ui, c, view);
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    title(ui, c, view);
                });
            });
        });
    click
}

fn controls(ui: &mut Ui, c: &Colors, view: &RowView) -> Click {
    let mut click = Click::Nothing;
    let capturing = view.capturing.is_some();
    let (icon_color, name) = if capturing {
        (c.accent, format!("Cancel changing {}", view.label))
    } else {
        (c.faint_text, format!("Change {}", view.label))
    };
    if widgets::icon_button(ui, c, Icon::Edit, &name, icon_color).clicked() {
        click = if capturing {
            Click::Cancel
        } else {
            Click::Change
        };
    }
    match view.capturing {
        Some(prompt) => {
            shortcut(ui, c, view, true);
            let color = if prompt.refused { c.warning } else { c.accent };
            let text = RichText::new(prompt.text)
                .font(theme::regular(11.0))
                .color(color);
            ui.label(text);
        }
        None => {
            shortcut(ui, c, view, false);
            if let Some(default) = view.default {
                let name = format!("Reset {} to {default}", view.label);
                if widgets::icon_button(ui, c, Icon::Reset, &name, c.accent).clicked() {
                    click = Click::Reset;
                }
            }
            if let Some(clash) = view.clash {
                clash_note(ui, c, clash);
            }
        }
    }
    click
}

fn clash_note(ui: &mut Ui, c: &Colors, text: &str) {
    ui.label(
        RichText::new("Conflict")
            .font(theme::bold(11.0))
            .color(c.warning),
    );
    ui.label(
        RichText::new(text)
            .font(theme::regular(11.0))
            .color(c.warning),
    );
}

/// The keycaps, in an accent outline while the binding is being changed.
/// The row lays out right to left, so the keys go in reversed.
fn shortcut(ui: &mut Ui, c: &Colors, view: &RowView, capturing: bool) {
    let keys: Vec<&str> = view.keys.iter().rev().map(String::as_str).collect();
    if capturing {
        Frame::new()
            .corner_radius(6)
            .stroke(Stroke::new(1.0, c.accent))
            .fill(c.control)
            .inner_margin(Margin::symmetric(6, 3))
            .show(ui, |ui| widgets::keycaps(ui, c, &keys, false));
    } else {
        widgets::keycaps(ui, c, &keys, view.unsaved);
    }
}

fn title(ui: &mut Ui, c: &Colors, view: &RowView) {
    ui.spacing_mut().item_spacing.x = 6.0;
    ui.label(RichText::new(view.label).font(theme::regular(13.0)));
    let id = RichText::new(view.id)
        .font(theme::regular(11.0))
        .color(c.faint_text);
    ui.label(id);
    if view.unsaved {
        let (rect, _) = ui.allocate_exact_size(vec2(6.0, 6.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 3.0, c.accent);
    }
}
