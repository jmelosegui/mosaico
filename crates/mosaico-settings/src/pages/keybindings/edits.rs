//! What the Keybindings page does to the open file.

use mosaico_core::Action;
use mosaico_core::config::{Keybinding, Modifier};

use super::{KeybindingsPage, Pending, combo_text, labels};

impl KeybindingsPage {
    /// Binds the binding at `index` to the combination, unless another
    /// binding has it, in which case the user is asked.
    pub(super) fn bind(&mut self, index: usize, key: &str, modifiers: &[Modifier]) {
        let Some(action) = self.current.get(index).map(|b| b.action) else {
            return;
        };
        let probe = Keybinding {
            action,
            key: key.into(),
            modifiers: modifiers.to_vec(),
        };
        let Ok(combo) = probe.combo() else {
            self.refused = Some(format!("{key} can't be used as a hotkey."));
            return;
        };
        let others: Vec<usize> = (0..self.current.len())
            .filter(|&i| i != index)
            .filter(|&i| self.current[i].combo().is_ok_and(|c| c == combo))
            .collect();
        if others.is_empty() {
            self.set(index, key, modifiers);
            return;
        }
        let owners: Vec<_> = others
            .iter()
            .map(|&i| labels::label(self.current[i].action))
            .collect();
        self.capturing = Some(index);
        self.refused = None;
        self.pending = Some(Pending {
            key: key.into(),
            modifiers: modifiers.to_vec(),
            others,
            shortcut: combo_text(&probe).replace('+', " "),
            owners: owners.join(", "),
        });
    }

    fn set(&mut self, index: usize, key: &str, modifiers: &[Modifier]) {
        let Some(doc) = &mut self.doc else { return };
        match doc.set_key(index, key, modifiers) {
            Ok(()) => {
                self.stop_capture();
                self.refresh();
            }
            Err(_) => self.refused = Some(format!("{key} can't be used as a hotkey.")),
        }
    }

    /// Takes the shortcut from the bindings that had it, for the one
    /// being changed.
    pub(super) fn replace(&mut self) {
        let (Some(index), Some(pending)) = (self.capturing, self.pending.take()) else {
            return;
        };
        let Some(doc) = &mut self.doc else { return };
        if doc
            .set_key(index, &pending.key, &pending.modifiers)
            .is_err()
        {
            return;
        }
        let mut others = pending.others;
        others.sort_unstable_by(|a, b| b.cmp(a));
        for other in others {
            let _ = doc.remove(other);
        }
        self.stop_capture();
        self.refresh();
    }

    /// Puts every action back on its default shortcut, adding the ones
    /// the file lost.
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
}
