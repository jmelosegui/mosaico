//! What the Keybindings page does to the open file.

use eframe::egui::Ui;
use mosaico_core::Action;
use mosaico_core::config::{Keybinding, Modifier};
use mosaico_core::hotkey::{Key, KeyCombo};

use super::capture::{self, Pressed};
use super::{KeybindingsPage, Pending, Row};

impl KeybindingsPage {
    /// Binds the row being changed to the combination pressed, if one was.
    pub(super) fn take_key_press(&mut self, ui: &Ui) {
        let Some(row) = self.capturing else { return };
        let events = ui.input(|i| i.events.clone());
        match capture::pressed(&events) {
            None => {}
            Some(Pressed::Cancel) => self.stop_capture(),
            Some(Pressed::Refused(why)) => self.refused = Some(why.into()),
            Some(Pressed::Combo { key, modifiers }) => self.bind(row, &key, &modifiers),
        }
    }

    pub(super) fn stop_capture(&mut self) {
        self.capturing = None;
        self.refused = None;
        self.pending = None;
    }

    /// Binds the row to the combination, unless another binding has it,
    /// in which case the user is asked.
    pub(super) fn bind(&mut self, row: Row, key: &str, modifiers: &[Modifier]) {
        let Ok(parsed) = key.parse::<Key>() else {
            self.refused = Some(format!("{key} can't be used as a hotkey."));
            return;
        };
        let combo = KeyCombo {
            modifiers: modifiers.iter().copied().collect(),
            key: parsed,
        };
        let others: Vec<Row> = self
            .all_rows()
            .filter(|&other| other != row && self.combo_of(other) == Some(combo))
            .collect();
        if others.is_empty() {
            self.set(row, key, modifiers);
            return;
        }
        let owners: Vec<_> = others.iter().map(|&o| self.owner(o)).collect();
        self.capturing = Some(row);
        self.refused = None;
        self.pending = Some(Pending {
            key: key.into(),
            modifiers: modifiers.to_vec(),
            others,
            shortcut: combo.to_string().replace('+', " "),
            owners: owners.join(", "),
        });
    }

    fn set(&mut self, row: Row, key: &str, modifiers: &[Modifier]) {
        let Some(doc) = &mut self.doc else { return };
        let set = match row {
            Row::Action(i) => doc.set_key(i, key, modifiers),
            Row::Launch(i) => doc.set_launch_key(i, key, modifiers),
        };
        match set {
            Ok(()) => {
                self.stop_capture();
                self.refresh();
            }
            Err(_) => self.refused = Some(format!("{key} can't be used as a hotkey.")),
        }
    }

    /// Takes the shortcut from the bindings that had it, for the one
    /// being changed. A launch binding that loses its shortcut is removed,
    /// as a keybinding is, since a launch binding needs a key.
    pub(super) fn replace(&mut self) {
        let (Some(row), Some(pending)) = (self.capturing, self.pending.take()) else {
            return;
        };
        let Some(doc) = &mut self.doc else { return };
        let set = match row {
            Row::Action(i) => doc.set_key(i, &pending.key, &pending.modifiers),
            Row::Launch(i) => doc.set_launch_key(i, &pending.key, &pending.modifiers),
        };
        if set.is_err() {
            return;
        }
        let mut others = pending.others;
        // Later positions first, so the earlier ones stay where they are.
        others.sort_unstable_by_key(|&o| std::cmp::Reverse(position(o)));
        for other in others {
            let _ = match other {
                Row::Action(i) => doc.remove(i),
                Row::Launch(i) => doc.remove_launch(i),
            };
            if matches!(other, Row::Launch(_)) {
                self.editing = None;
            }
        }
        self.stop_capture();
        self.refresh();
    }

    /// Puts every action back on its default shortcut, adding the ones
    /// the file lost. Launch bindings have no default and are left alone.
    pub(super) fn restore_defaults(&mut self) {
        let Some(doc) = &mut self.doc else { return };
        for default in &self.defaults {
            let at = (self.current.iter()).position(|b| b.action == default.action);
            let _ = match at {
                Some(i) => doc.set_key(i, &default.key, &default.modifiers),
                None => doc.add(default.action, &default.key, &default.modifiers),
            };
        }
        self.stop_capture();
        self.refresh();
    }

    pub(super) fn default_of(&self, action: Action) -> Option<&Keybinding> {
        self.defaults.iter().find(|d| d.action == action)
    }

    pub(super) fn saved_of(&self, action: Action) -> Option<&Keybinding> {
        self.saved.iter().find(|b| b.action == action)
    }

    fn all_rows(&self) -> impl Iterator<Item = Row> {
        let actions = (0..self.current.len()).map(Row::Action);
        actions.chain((0..self.launches.len()).map(Row::Launch))
    }

    /// The combination the binding of `row` is on, if it has one.
    fn combo_of(&self, row: Row) -> Option<KeyCombo> {
        match row {
            Row::Action(i) => self.current.get(i)?.combo().ok(),
            Row::Launch(i) => self.launches.get(i)?.as_ref().ok()?.combo().ok(),
        }
    }
}

/// Where `row` is among the rows of its kind.
fn position(row: Row) -> usize {
    match row {
        Row::Action(i) | Row::Launch(i) => i,
    }
}
