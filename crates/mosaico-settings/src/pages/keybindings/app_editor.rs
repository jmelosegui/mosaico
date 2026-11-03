//! The editor that opens under an application's row: what it starts and
//! whether a key press brings an open window of it into view instead.

use eframe::egui::{Align, Frame, Layout, Margin, RichText, Ui};
use mosaico_core::config::{LaunchBinding, join_args, split_args};

use crate::pages::rules::row::outlined_button;
use crate::theme::{self, Colors};
use crate::widgets;

/// The fields of the launch binding being edited, as typed, so a space
/// typed at the end of the arguments stays until the next one.
#[derive(Debug, Clone)]
pub struct Editor {
    /// The launch binding, by its position in the file.
    pub index: usize,
    command: String,
    args: String,
    path: String,
    focus: bool,
}

impl Editor {
    pub fn of(index: usize, binding: &LaunchBinding) -> Self {
        Self {
            index,
            command: binding.command.clone(),
            args: join_args(&binding.args),
            path: binding.windows.match_path.clone().unwrap_or_default(),
            focus: binding.focus,
        }
    }

    /// `binding` with the fields of the editor. Its key combination, and
    /// the class and title it matches, are not edited here.
    pub fn applied_to(&self, binding: &LaunchBinding) -> LaunchBinding {
        let mut edited = binding.clone();
        edited.command = self.command.trim().to_owned();
        edited.args = split_args(&self.args);
        let path = self.path.trim();
        edited.windows.match_path = (!path.is_empty()).then(|| path.to_owned());
        edited.focus = self.focus;
        edited
    }
}

/// Draws `editor`, for the application named `name`, outlining what
/// differs from `was`, the binding as saved. Returns whether Done was
/// clicked.
pub fn show(
    ui: &mut Ui,
    c: &Colors,
    name: &str,
    editor: &mut Editor,
    was: Option<&LaunchBinding>,
) -> bool {
    let mut done = false;
    Frame::new()
        .fill(c.selected)
        .inner_margin(Margin {
            left: 40,
            right: 16,
            top: 4,
            bottom: 14,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 12.0;
            fields(ui, c, name, editor, was);
            ui.horizontal(|ui| {
                let text = "Focus an open window of it instead of starting another";
                widgets::checkbox(ui, c, text, &mut editor.focus);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    done = outlined_button(ui, c, "Done").clicked();
                });
            });
        });
    done
}

fn fields(ui: &mut Ui, c: &Colors, name: &str, editor: &mut Editor, was: Option<&LaunchBinding>) {
    let gap = 12.0;
    let width = (ui.available_width() - 2.0 * gap) / 3.0;
    let was_args = was.map(|w| join_args(&w.args));
    let was_path = was.map(|w| w.windows.match_path.clone().unwrap_or_default());
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        let changed = was.is_none_or(|w| w.command != editor.command.trim());
        let command = ("Command", "A program, a path or a link");
        field(ui, c, (command, name, width), &mut editor.command, changed);
        let changed = was_args.is_none_or(|w| split_args(&w) != split_args(&editor.args));
        let args = ("Arguments", "None");
        field(ui, c, (args, name, width), &mut editor.args, changed);
        let changed = was_path.is_none_or(|w| w != editor.path.trim());
        let path = ("Its windows' path ends with", "The command's name");
        field(ui, c, (path, name, width), &mut editor.path, changed);
    });
}

/// A labelled line of text, named for assistive technology after the
/// field and the application.
fn field(
    ui: &mut Ui,
    c: &Colors,
    ((title, hint), name, width): ((&str, &str), &str, f32),
    value: &mut String,
    changed: bool,
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 5.0;
        ui.set_width(width);
        let label = RichText::new(title)
            .font(theme::regular(11.0))
            .color(c.dim_text);
        ui.label(label);
        let label = format!("{title} of {name}");
        // The text frame has 6 points of margin on each side.
        widgets::text(ui, c, &label, width - 14.0, value, hint, changed);
    });
}
