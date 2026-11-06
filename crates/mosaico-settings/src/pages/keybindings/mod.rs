//! The Keybindings page: the hotkeys of `keybindings.toml`, each with
//! the keys that trigger it, and the applications keys start.

mod app_editor;
mod apps;
mod banner;
mod capture;
mod edits;
mod header;
pub(super) mod labels;
mod row;
mod rows;

use eframe::egui::{RichText, Ui};
use mosaico_core::config::{
    ConfigStore, Keybinding, KeybindingsDocument, LaunchBinding, Modifier, keybinding,
};

use self::app_editor::Editor;
use self::banner::Choice;
use super::{FilePage, Page};
use crate::theme::Colors;

const PROMPT: &str = "Press the new shortcut";

/// A row of the page: a keybinding or a launch binding, by its position
/// among its kind in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Action(usize),
    Launch(usize),
}

/// A launch binding as the file holds it, or why it cannot be read.
type Launch = Result<LaunchBinding, String>;

/// A shortcut pressed that another binding already has.
struct Pending {
    key: String,
    modifiers: Vec<Modifier>,
    /// The bindings that have it.
    others: Vec<Row>,
    /// The shortcut as the banner says it: `Alt Shift H`.
    shortcut: String,
    /// What the other bindings do.
    owners: String,
}

/// The Keybindings page, editing `keybindings.toml`.
pub struct KeybindingsPage {
    store: ConfigStore,
    /// The file open for editing, unless it cannot be.
    doc: Option<KeybindingsDocument>,
    problem: Option<String>,
    /// The bindings as the file holds them.
    saved: Vec<Keybinding>,
    /// The bindings with the edits made so far.
    current: Vec<Keybinding>,
    /// For each binding, what else its combination is bound to.
    clashes: Vec<Option<String>>,
    /// The launch bindings as the file holds them.
    saved_launches: Vec<Launch>,
    /// The launch bindings with the edits made so far.
    launches: Vec<Launch>,
    /// For each launch binding, what else its combination is bound to.
    launch_clashes: Vec<Option<String>>,
    /// The launch binding whose editor is open.
    editing: Option<Editor>,
    defaults: Vec<Keybinding>,
    /// The row waiting for a key press.
    capturing: Option<Row>,
    /// Why the last key press was not taken.
    refused: Option<String>,
    /// A shortcut pressed that is bound already, waiting for the user to
    /// replace it or cancel.
    pending: Option<Pending>,
    /// What the filter field holds.
    filter: String,
}

impl KeybindingsPage {
    /// The page for the file of `store`.
    pub fn new(store: ConfigStore) -> Self {
        let mut page = Self {
            store,
            doc: None,
            problem: None,
            saved: Vec::new(),
            current: Vec::new(),
            clashes: Vec::new(),
            saved_launches: Vec::new(),
            launches: Vec::new(),
            launch_clashes: Vec::new(),
            editing: None,
            defaults: keybinding::defaults(),
            capturing: None,
            refused: None,
            pending: None,
            filter: String::new(),
        };
        page.load();
        page
    }

    fn load(&mut self) {
        match self.store.open_keybindings() {
            Ok(doc) => {
                self.saved = doc.bindings();
                self.saved_launches = launches_of(&doc);
                self.doc = Some(doc);
                self.problem = None;
            }
            Err(e) => {
                self.doc = None;
                self.saved.clear();
                self.saved_launches.clear();
                self.problem = Some(e.to_string());
            }
        }
        self.refresh();
    }

    /// Reads the bindings and their clashes from the open file.
    fn refresh(&mut self) {
        let Some(doc) = &self.doc else {
            self.current.clear();
            self.clashes.clear();
            self.launches.clear();
            self.launch_clashes.clear();
            self.capturing = None;
            self.editing = None;
            return;
        };
        self.current = doc.bindings();
        self.launches = launches_of(doc);
        self.clashes = vec![None; self.current.len()];
        self.launch_clashes = vec![None; self.launches.len()];
        for conflict in doc.conflicts() {
            let rows = (conflict.bindings.iter().map(|&i| Row::Action(i)))
                .chain(conflict.launches.iter().map(|&i| Row::Launch(i)));
            let rows: Vec<Row> = rows.collect();
            for &row in &rows {
                let others: Vec<_> = (rows.iter().filter(|&&o| o != row))
                    .map(|&o| self.owner(o))
                    .collect();
                let clash = Some(format!("Also bound to {}.", others.join(", ")));
                match row {
                    Row::Action(i) => self.clashes[i] = clash,
                    Row::Launch(i) => self.launch_clashes[i] = clash,
                }
            }
        }
        let gone = |row| match row {
            Row::Action(i) => i >= self.current.len(),
            Row::Launch(i) => i >= self.launches.len(),
        };
        if self.capturing.is_some_and(gone) {
            self.capturing = None;
        }
        if self
            .editing
            .as_ref()
            .is_some_and(|e| e.index >= self.launches.len())
        {
            self.editing = None;
        }
    }

    /// What the binding of `row` does, as a clash names it.
    fn owner(&self, row: Row) -> String {
        match row {
            Row::Action(i) => self.current.get(i).map(|b| labels::label(b.action)),
            Row::Launch(i) => match self.launches.get(i) {
                Some(Ok(launch)) => Some(apps::label(launch)),
                _ => Some(format!("launch binding {}", i + 1)),
            },
        }
        .unwrap_or_default()
    }
}

fn launches_of(doc: &KeybindingsDocument) -> Vec<Launch> {
    let launches = doc.launches().into_iter();
    launches.map(|l| l.map_err(|e| e.to_string())).collect()
}

/// The combination of `binding` as `Alt+Shift+J`, or what the file says
/// when its key is not one.
fn combo_text(binding: &Keybinding) -> String {
    match binding.combo() {
        Ok(combo) => combo.to_string(),
        Err(_) => format!("{} (unknown key)", binding.key),
    }
}

impl FilePage for KeybindingsPage {
    fn page(&self) -> Page {
        Page::Keybindings
    }

    fn changes(&self) -> usize {
        let moved = (self.current.iter())
            .filter(|now| {
                let was = self.saved_of(now.action).map(combo_text);
                was.is_none_or(|was| was != combo_text(now))
            })
            .count();
        let dropped = (self.saved.iter())
            .filter(|was| !self.current.iter().any(|now| now.action == was.action))
            .count();
        moved + dropped + self.launch_changes()
    }

    fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    fn matches(&self, query: &str) -> usize {
        let actions = (0..self.current.len()).filter(|&i| self.row_matches(i, query));
        actions.count() + self.apps_matching(query, "")
    }

    fn show(&mut self, ui: &mut Ui, c: &Colors, query: &str) {
        self.take_key_press(ui);
        // A search of the window replaces the page's own filter.
        let searching = !query.trim().is_empty();
        if !searching && header::draw(ui, c, &mut self.filter) {
            self.restore_defaults();
        }
        let filter = if searching {
            String::new()
        } else {
            self.filter.clone()
        };
        let mut chosen = Choice::Undecided;
        let actions = self.show_actions(ui, c, query, &filter, &mut chosen);
        let mut any = actions.shown;
        let mut clicked = actions.clicked;
        if self.doc.is_some() {
            let apps = self.show_apps(ui, c, query, &filter, &mut chosen);
            any |= apps.shown;
            clicked = clicked.or(apps.clicked);
        }
        if !any && self.doc.is_some() {
            let text = format!("No keybindings match \u{201c}{}\u{201d}.", filter.trim());
            ui.label(RichText::new(text).color(c.dim_text));
        }
        match chosen {
            Choice::Undecided => {}
            Choice::Cancel => self.stop_capture(),
            Choice::Replace => self.replace(),
        }
        if let Some((row, click)) = clicked {
            self.click(row, click);
        }
    }

    fn apply(&mut self) -> Result<(), String> {
        let Some(doc) = &mut self.doc else {
            return Err(self.problem.clone().unwrap_or_default());
        };
        let clashes = doc.conflicts();
        if !clashes.is_empty() {
            let names: Vec<_> = clashes.iter().map(|c| c.combo.to_string()).collect();
            return Err(format!(
                "keybindings are bound twice: {}. Give one of each a different key",
                names.join(", ")
            ));
        }
        doc.save().map_err(|e| e.to_string())?;
        self.saved = doc.bindings();
        self.saved_launches = launches_of(doc);
        Ok(())
    }

    fn revert(&mut self) {
        self.stop_capture();
        self.editing = None;
        self.load();
    }

    /// Follows the file while nothing is edited. An open editor stays
    /// open, and starts over from the file when it changed under it.
    fn reload(&mut self) {
        self.load();
        if let Some(editor) = &self.editing
            && let Some(Ok(launch)) = self.launches.get(editor.index)
            && editor.applied_to(launch) != *launch
        {
            self.editing = Some(Editor::of(editor.index, launch));
        }
    }
}
