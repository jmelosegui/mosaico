//! The Keybindings page: the hotkeys of `keybindings.toml`, each with
//! the keys that trigger it.

mod banner;
mod capture;
mod edits;
mod header;
pub(super) mod labels;
mod row;
mod rows;

use eframe::egui::{RichText, Ui};
use mosaico_core::config::{ConfigStore, Keybinding, KeybindingsDocument, Modifier, keybinding};

use self::banner::Choice;
use self::capture::Pressed;
use self::row::Click;
use super::{FilePage, Page};
use crate::theme::Colors;
use crate::widgets;

const PROMPT: &str = "Press the new shortcut";

/// A shortcut pressed that another binding already has.
struct Pending {
    key: String,
    modifiers: Vec<Modifier>,
    /// The bindings that have it.
    others: Vec<usize>,
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
    defaults: Vec<Keybinding>,
    /// The binding waiting for a key press.
    capturing: Option<usize>,
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
                self.doc = Some(doc);
                self.problem = None;
            }
            Err(e) => {
                self.doc = None;
                self.saved.clear();
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
            self.capturing = None;
            return;
        };
        self.current = doc.bindings();
        self.clashes = vec![None; self.current.len()];
        for conflict in doc.conflicts() {
            for &i in &conflict.bindings {
                let others: Vec<_> = (conflict.bindings.iter().filter(|&&o| o != i))
                    .filter_map(|&o| self.current.get(o))
                    .map(|b| labels::label(b.action))
                    .collect();
                self.clashes[i] = Some(format!("Also bound to {}.", others.join(", ")));
            }
        }
        if self.capturing.is_some_and(|i| i >= self.current.len()) {
            self.capturing = None;
        }
    }

    fn take_key_press(&mut self, ui: &Ui) {
        let Some(index) = self.capturing else { return };
        let events = ui.input(|i| i.events.clone());
        match capture::pressed(&events) {
            None => {}
            Some(Pressed::Cancel) => self.stop_capture(),
            Some(Pressed::Refused(why)) => self.refused = Some(why.into()),
            Some(Pressed::Combo { key, modifiers }) => self.bind(index, &key, &modifiers),
        }
    }

    fn stop_capture(&mut self) {
        self.capturing = None;
        self.refused = None;
        self.pending = None;
    }
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
        moved + dropped
    }

    fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    fn matches(&self, query: &str) -> usize {
        (0..self.current.len())
            .filter(|&i| self.row_matches(i, query))
            .count()
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
        let mut clicked = None;
        let mut chosen = Choice::Undecided;
        let mut any = false;
        for group in labels::GROUPS {
            let mut shown: Vec<_> = (0..self.current.len())
                .filter(|&i| labels::group(self.current[i].action) == group)
                .filter(|&i| self.row_matches(i, query) && self.row_matches(i, &filter))
                .collect();
            shown.sort_by_key(|&i| labels::rank(self.current[i].action));
            if shown.is_empty() {
                continue;
            }
            any = true;
            widgets::section(ui, c, group, |ui| {
                for (n, &i) in shown.iter().enumerate() {
                    if n > 0 {
                        widgets::divider(ui, c);
                    }
                    let click = self.show_row(ui, c, i);
                    if click != Click::Nothing {
                        clicked = Some((i, click));
                    }
                    if let (Some(p), true) = (&self.pending, self.capturing == Some(i)) {
                        chosen = banner::draw(ui, c, &p.shortcut, &p.owners);
                    }
                }
            });
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
        if let Some((index, click)) = clicked {
            self.click(index, click);
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
        Ok(())
    }

    fn revert(&mut self) {
        self.stop_capture();
        self.load();
    }

    fn reload(&mut self) {
        self.load();
    }
}
