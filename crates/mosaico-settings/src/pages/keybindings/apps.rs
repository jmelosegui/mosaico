//! The Applications tab of the Keybindings page: the `[[launch]]`
//! bindings, each starting a program from a shortcut, or bringing an open
//! window of it into view.

use eframe::egui::{Align, Frame, Layout, Margin, RichText, Ui};
use mosaico_core::config::{LaunchBinding, WindowMatch, join_args};

use super::app_editor::{self, Editor, EditorClick, Shortcut};
use super::banner::{self, Choice};
use super::row::{self, Click, RowView};
use super::{KeybindingsPage, Row};
use crate::icons::Icon;
use crate::pages::rules::row::add_button;
use crate::theme::{self, Colors};
use crate::widgets;

const EMPTY: &str = "No applications yet. Add one to start it with a shortcut.";

/// What names an application: its command, or a placeholder while it has
/// none.
pub(super) fn label(launch: &LaunchBinding) -> String {
    if launch.command.trim().is_empty() {
        "New application".into()
    } else {
        launch.command.clone()
    }
}

/// The combination of `launch` as `Alt+Shift+T`, nothing while it has no
/// key, or what the file says when its key is not one.
fn combo_text(launch: &LaunchBinding) -> String {
    if launch.key.trim().is_empty() {
        return String::new();
    }
    match launch.combo() {
        Ok(combo) => combo.to_string(),
        Err(_) => format!("{} (unknown key)", launch.key),
    }
}

impl KeybindingsPage {
    /// How many launch bindings differ from the file.
    pub(super) fn launch_changes(&self) -> usize {
        let len = self.launches.len().max(self.saved_launches.len());
        (0..len)
            .filter(|&i| self.launches.get(i) != self.saved_launches.get(i))
            .count()
    }

    /// How many applications match `query`.
    pub(super) fn apps_matching(&self, query: &str) -> usize {
        (0..self.launches.len())
            .filter(|&i| self.app_matches(i, query))
            .count()
    }

    fn app_matches(&self, index: usize, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        let texts = match &self.launches[index] {
            Ok(launch) => vec![label(launch), join_args(&launch.args), combo_text(launch)],
            Err(problem) => vec![problem.clone()],
        };
        texts.iter().any(|t| t.to_lowercase().contains(&query))
    }

    /// Draws the tab: the applications matching `query`, all of them when
    /// it is empty, and nothing when a search finds none. A banner asking
    /// whether to take a shortcut is drawn under its row, and what it says
    /// goes in `chosen`. Returns the row clicked, and how.
    pub(super) fn show_apps(
        &mut self,
        ui: &mut Ui,
        c: &Colors,
        query: &str,
        chosen: &mut Choice,
    ) -> Option<(Row, Click)> {
        self.doc.as_ref()?;
        let shown: Vec<usize> = (0..self.launches.len())
            .filter(|&i| self.app_matches(i, query))
            .collect();
        if !query.trim().is_empty() && shown.is_empty() {
            return None;
        }
        let mut add = false;
        widgets::heading(ui, c, "Applications", |ui| {
            add = add_button(ui, c, "Add application").clicked();
        });
        let mut clicked = None;
        let mut done = false;
        widgets::card(ui, c, |ui| {
            for (n, &i) in shown.iter().enumerate() {
                if n > 0 {
                    widgets::divider(ui, c);
                }
                let click = self.show_app(ui, c, i);
                if click != Click::Nothing {
                    clicked = Some((Row::Launch(i), click));
                }
                if let (Some(p), true) = (&self.pending, self.capturing == Some(Row::Launch(i))) {
                    *chosen = banner::draw(ui, c, &p.shortcut, &p.owners);
                }
                match self.show_editor(ui, c, i) {
                    EditorClick::Nothing => {}
                    EditorClick::Done => done = true,
                    EditorClick::Shortcut => {
                        let row = Row::Launch(i);
                        let click = if self.capturing == Some(row) {
                            Click::Cancel
                        } else {
                            Click::Change
                        };
                        clicked = Some((row, click));
                    }
                }
            }
            if shown.is_empty() {
                Frame::new()
                    .inner_margin(Margin::symmetric(16, 12))
                    .show(ui, |ui| ui.label(RichText::new(EMPTY).color(c.dim_text)));
            }
        });
        ui.add_space(16.0);
        if done {
            self.editing = None;
        }
        if add {
            self.add_app();
            ui.ctx().request_repaint();
        }
        clicked
    }

    fn show_app(&self, ui: &mut Ui, c: &Colors, index: usize) -> Click {
        let launch = match &self.launches[index] {
            Ok(launch) => launch,
            Err(problem) => return broken(ui, c, index, problem),
        };
        let row = Row::Launch(index);
        let keys = self.keys(index, launch);
        let name = label(launch);
        let args = join_args(&launch.args);
        let view = RowView {
            label: &name,
            id: &args,
            keys: &keys,
            unsaved: self.launches.get(index) != self.saved_launches.get(index),
            default: None,
            clash: self.launch_clashes[index].as_deref(),
            capturing: self.prompt(row),
            app: Some(self.editing.as_ref().is_some_and(|e| e.index == index)),
        };
        row::draw(ui, c, &view)
    }

    /// The keys of the application at `index`, as `["Alt", "T"]`: the
    /// shortcut waiting for an answer while its row asks about one.
    fn keys(&self, index: usize, launch: &LaunchBinding) -> Vec<String> {
        let combo = match &self.pending {
            Some(p) if self.capturing == Some(Row::Launch(index)) => p.shortcut.replace(' ', "+"),
            _ => combo_text(launch),
        };
        combo
            .split('+')
            .filter(|k| !k.is_empty())
            .map(str::to_owned)
            .collect()
    }

    /// Draws the editor of the application at `index` when it is open,
    /// and writes what was typed into the file. Returns what was clicked.
    fn show_editor(&mut self, ui: &mut Ui, c: &Colors, index: usize) -> EditorClick {
        let Some(Ok(current)) = self.launches.get(index).cloned() else {
            return EditorClick::Nothing;
        };
        let was = self.saved_launches.get(index).cloned().and_then(Result::ok);
        let keys = self.keys(index, &current);
        let shortcut = Shortcut {
            keys: &keys,
            capturing: self.capturing == Some(Row::Launch(index)),
        };
        let Some(editor) = self.editing.as_mut().filter(|e| e.index == index) else {
            return EditorClick::Nothing;
        };
        let name = format!("application {}", index + 1);
        let click = app_editor::show(ui, c, &name, &shortcut, editor, was.as_ref());
        let edited = editor.applied_to(&current);
        if edited != current
            && let Some(doc) = &mut self.doc
            && doc.set_launch(index, &edited).is_ok()
        {
            self.refresh();
        }
        click
    }

    /// Opens the editor of the application of `row`, or closes it when it
    /// is open.
    pub(super) fn toggle_editor(&mut self, row: Row) {
        let Row::Launch(index) = row else { return };
        if self.editing.as_ref().is_some_and(|e| e.index == index) {
            self.editing = None;
        } else if let Some(Ok(launch)) = self.launches.get(index) {
            self.editing = Some(Editor::of(index, launch));
        }
    }

    pub(super) fn remove_app(&mut self, row: Row) {
        let Row::Launch(index) = row else { return };
        let Some(doc) = &mut self.doc else { return };
        if doc.remove_launch(index).is_err() {
            return;
        }
        self.editing = match self.editing.take() {
            Some(e) if e.index == index => None,
            Some(mut e) if e.index > index => {
                e.index -= 1;
                Some(e)
            }
            other => other,
        };
        self.stop_capture();
        self.refresh();
    }

    /// Adds an application with nothing filled in yet, and opens its
    /// editor.
    fn add_app(&mut self) {
        let Some(doc) = &mut self.doc else { return };
        let blank = LaunchBinding {
            command: String::new(),
            args: Vec::new(),
            key: String::new(),
            modifiers: Vec::new(),
            focus: false,
            windows: WindowMatch::default(),
        };
        if doc.add_launch(&blank).is_err() {
            return;
        }
        self.refresh();
        let index = self.launches.len() - 1;
        self.editing = Some(Editor::of(index, &blank));
    }
}

/// A launch binding of the file that cannot be read: why, and a button to
/// remove it.
fn broken(ui: &mut Ui, c: &Colors, index: usize, problem: &str) -> Click {
    let mut click = Click::Nothing;
    Frame::new()
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let name = format!("Remove launch binding {}", index + 1);
                if widgets::icon_button(ui, c, Icon::Trash, &name, c.faint_text).clicked() {
                    click = Click::Remove;
                }
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let text = RichText::new(problem)
                        .font(theme::regular(12.0))
                        .color(c.warning);
                    ui.label(text);
                });
            });
        });
    click
}
