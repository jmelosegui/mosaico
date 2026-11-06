//! The editor that opens under an application's row: what it starts and
//! whether a key press brings an open window of it into view instead.

use eframe::egui::{Align, Frame, Label, Layout, Margin, RichText, Ui, vec2};
use mosaico_core::config::{LaunchBinding, join_args, split_args};

use crate::pages::rules::row::{named_outlined_button, outlined_button};
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

/// The shortcut of the application being edited, as the editor shows it.
pub struct Shortcut<'a> {
    /// Its keys, as `["Alt", "Shift", "T"]`, none while it has no shortcut.
    pub keys: &'a [String],
    /// Its row is waiting for a key press.
    pub capturing: bool,
}

/// What was clicked in the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorClick {
    Nothing,
    Done,
    /// Record a shortcut, or stop recording it.
    Shortcut,
}

const COMMAND_HELP: &str =
    "A program such as wt or msedge, a full path to one, or a link such as https://github.com.";
const ARGS_HELP: &str =
    "Passed to the command, separated by spaces. Put one that holds spaces in double quotes.";
const PATH_HELP: &str = "Empty matches the command's name. Set it when the windows belong \
    to another program, as WindowsTerminal.exe for wt.";
const FOCUS_HELP: &str = "Off: every press starts a new one. On: a press brings an open \
    window forward, and the next press moves to another.";
/// The space between two fields side by side.
const GAP: f32 = 12.0;

/// Draws `editor`, for the application named `name` with `shortcut`,
/// outlining what differs from `was`, the binding as saved. Returns what
/// was clicked.
pub fn show(
    ui: &mut Ui,
    c: &Colors,
    name: &str,
    shortcut: &Shortcut,
    editor: &mut Editor,
    was: Option<&LaunchBinding>,
) -> EditorClick {
    let mut click = EditorClick::Nothing;
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
            if shortcut_line(ui, c, name, shortcut) {
                click = EditorClick::Shortcut;
            }
            fields(ui, c, name, editor, was);
            focus(ui, c, name, editor, was);
            let row = vec2(ui.available_width(), 26.0);
            ui.allocate_ui_with_layout(row, Layout::right_to_left(Align::Center), |ui| {
                if outlined_button(ui, c, "Done").clicked() {
                    click = EditorClick::Done;
                }
            });
        });
    click
}

/// The focus checkbox and, only while it is ticked, the path the
/// windows to focus end with, which nothing else uses.
fn focus(ui: &mut Ui, c: &Colors, name: &str, editor: &mut Editor, was: Option<&LaunchBinding>) {
    let width = (ui.available_width() - 24.0 - GAP) / 2.0;
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 4.0;
        let text = "Focus an open window of it instead of starting another";
        widgets::checkbox(ui, c, text, &mut editor.focus);
        Frame::new()
            .inner_margin(Margin {
                left: 24,
                ..Margin::ZERO
            })
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 12.0;
                ui.add(Label::new(help(c, FOCUS_HELP)).wrap());
                if editor.focus {
                    let was_path = was.map(|w| w.windows.match_path.clone().unwrap_or_default());
                    let changed = was_path.is_none_or(|w| w != editor.path.trim());
                    let path = (
                        "Its windows' path ends with",
                        "The command's name",
                        PATH_HELP,
                    );
                    field(ui, c, (path, name, width), &mut editor.path, changed);
                }
            });
    });
}

/// The keys of the shortcut and the button that records it. Returns
/// whether the button was clicked.
fn shortcut_line(ui: &mut Ui, c: &Colors, name: &str, shortcut: &Shortcut) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        ui.label(RichText::new("Shortcut").font(theme::regular(13.0)));
        if shortcut.keys.is_empty() {
            ui.label(RichText::new("None yet").color(c.faint_text));
        } else {
            let keys: Vec<&str> = shortcut.keys.iter().map(String::as_str).collect();
            widgets::keycaps(ui, c, &keys, false);
        }
        let (text, label, hint) = if shortcut.capturing {
            (
                "Cancel",
                format!("Stop recording the shortcut of {name}"),
                "Press the keys together, holding Ctrl, Alt or Shift. Esc cancels.",
            )
        } else {
            (
                "Record",
                format!("Record the shortcut of {name}"),
                "Click Record, then press the keys that should start it.",
            )
        };
        clicked = named_outlined_button(ui, c, text, &label).clicked();
        ui.add(Label::new(help(c, hint)).wrap());
    });
    clicked
}

/// A line of help under a field.
fn help(c: &Colors, text: &str) -> RichText {
    RichText::new(text)
        .font(theme::regular(11.0))
        .color(c.faint_text)
}

/// The command and its arguments, side by side.
fn fields(ui: &mut Ui, c: &Colors, name: &str, editor: &mut Editor, was: Option<&LaunchBinding>) {
    let width = (ui.available_width() - GAP) / 2.0;
    let was_args = was.map(|w| join_args(&w.args));
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = GAP;
        let changed = was.is_none_or(|w| w.command != editor.command.trim());
        let command = ("Command", "A program, a path or a link", COMMAND_HELP);
        field(ui, c, (command, name, width), &mut editor.command, changed);
        let changed = was_args.is_none_or(|w| split_args(&w) != split_args(&editor.args));
        let args = ("Arguments", "None", ARGS_HELP);
        field(ui, c, (args, name, width), &mut editor.args, changed);
    });
}

/// A labelled line of text with a line of help under it, named for
/// assistive technology after the field and the application.
fn field(
    ui: &mut Ui,
    c: &Colors,
    ((title, hint, help_text), name, width): ((&str, &str, &str), &str, f32),
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
        ui.add(Label::new(help(c, help_text)).wrap());
    });
}
