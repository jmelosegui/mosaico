//! The Keybindings page: the hotkeys of `keybindings.toml`, each with
//! the keys that trigger it.

mod capture;
mod labels;
mod row;

use eframe::egui::{RichText, Ui};
use mosaico_core::config::{ConfigStore, Keybinding, KeybindingsDocument, keybinding};

use self::capture::Pressed;
use self::row::{Click, RowView};
use super::{FilePage, Page};
use crate::theme::Colors;
use crate::widgets;

const PROMPT: &str = "Press the new key combination, or Esc to cancel.";

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

    fn bind(&mut self, index: usize, key: &str, modifiers: &[mosaico_core::config::Modifier]) {
        let Some(doc) = &mut self.doc else { return };
        match doc.set_key(index, key, modifiers) {
            Ok(()) => {
                self.stop_capture();
                self.refresh();
            }
            Err(_) => self.refused = Some(format!("{key} can't be used as a hotkey.")),
        }
    }

    fn stop_capture(&mut self) {
        self.capturing = None;
        self.refused = None;
    }

    fn default_of(&self, action: mosaico_core::Action) -> Option<&Keybinding> {
        self.defaults.iter().find(|d| d.action == action)
    }

    fn show_row(&self, ui: &mut Ui, c: &Colors, index: usize) -> Click {
        let binding = &self.current[index];
        let combo = combo_text(binding);
        let default = self
            .default_of(binding.action)
            .map(combo_text)
            .filter(|d| *d != combo);
        let unsaved = self.saved.get(index).map(combo_text).as_ref() != Some(&combo);
        let label = labels::label(binding.action);
        let capturing =
            (self.capturing == Some(index)).then(|| self.refused.as_deref().unwrap_or(PROMPT));
        let view = RowView {
            label: &label,
            combo: &combo,
            unsaved,
            default: default.as_deref(),
            clash: self.clashes[index].as_deref(),
            capturing,
        };
        row::draw(ui, c, &view)
    }

    fn click(&mut self, index: usize, click: Click) {
        match click {
            Click::Nothing => {}
            Click::Change => {
                self.capturing = Some(index);
                self.refused = None;
            }
            Click::Cancel => self.stop_capture(),
            Click::Reset => {
                let default = self.default_of(self.current[index].action).cloned();
                if let Some(d) = default {
                    self.bind(index, &d.key, &d.modifiers);
                }
            }
        }
    }

    fn matches(&self, index: usize, query: &str) -> bool {
        let binding = &self.current[index];
        let query = query.trim().to_lowercase();
        let texts = [
            labels::label(binding.action),
            binding.action.to_string(),
            combo_text(binding),
        ];
        texts.iter().any(|t| t.to_lowercase().contains(&query))
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
        let moved = (self.current.iter().zip(&self.saved))
            .filter(|(now, was)| combo_text(now) != combo_text(was))
            .count();
        moved + self.current.len().abs_diff(self.saved.len())
    }

    fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    fn show(&mut self, ui: &mut Ui, c: &Colors, query: &str) {
        self.take_key_press(ui);
        let mut clicked = None;
        let mut any = false;
        for group in labels::GROUPS {
            let shown: Vec<_> = (0..self.current.len())
                .filter(|&i| labels::group(self.current[i].action) == group)
                .filter(|&i| self.matches(i, query))
                .collect();
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
                }
            });
        }
        if !any && self.doc.is_some() {
            let text = format!("No keybindings match \u{201c}{}\u{201d}.", query.trim());
            ui.label(RichText::new(text).color(c.dim_text));
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
