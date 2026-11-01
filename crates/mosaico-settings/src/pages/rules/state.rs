//! The rules being edited: what is saved, and the list as it is now.

use mosaico_core::config::{RuleAction, WindowRule};

/// A rule of the list, and the rule of the file it comes from.
#[derive(Debug, Clone)]
pub struct Entry {
    pub origin: Option<usize>,
    pub rule: WindowRule,
}

/// The rules of the file and the list being edited from them.
#[derive(Debug, Default)]
pub struct Draft {
    saved: Vec<WindowRule>,
    pub entries: Vec<Entry>,
}

impl Draft {
    pub fn new(saved: Vec<WindowRule>) -> Self {
        let entry = |(i, rule): (usize, &WindowRule)| Entry {
            origin: Some(i),
            rule: rule.clone(),
        };
        let entries = saved.iter().enumerate().map(entry).collect();
        Self { saved, entries }
    }

    pub fn saved(&self) -> &[WindowRule] {
        &self.saved
    }

    /// The saved rule the entry at `index` comes from.
    pub fn was(&self, index: usize) -> Option<&WindowRule> {
        self.entries[index].origin.and_then(|i| self.saved.get(i))
    }

    /// Whether the entry at `index` is new, moved or edited.
    pub fn is_changed(&self, index: usize) -> bool {
        let entry = &self.entries[index];
        entry.origin != Some(index) || self.was(index) != Some(&entry.rule)
    }

    /// How many entries are changed, and how many rules were removed.
    pub fn changes(&self) -> usize {
        let changed = (0..self.entries.len())
            .filter(|&i| self.is_changed(i))
            .count();
        let kept = self.entries.iter().filter(|e| e.origin.is_some()).count();
        changed + self.saved.len().saturating_sub(kept)
    }

    /// Adds a rule that matches nothing yet, to tile.
    pub fn add(&mut self) {
        let rule = WindowRule {
            match_class: None,
            match_title: None,
            match_path: None,
            action: RuleAction::Tile,
        };
        self.entries.push(Entry { origin: None, rule });
    }

    pub fn remove(&mut self, index: usize) {
        self.entries.remove(index);
    }

    pub fn move_up(&mut self, index: usize) {
        if index > 0 {
            self.entries.swap(index, index - 1);
        }
    }

    pub fn move_down(&mut self, index: usize) {
        if index + 1 < self.entries.len() {
            self.entries.swap(index, index + 1);
        }
    }

    /// Starts again from `saved`, dropping the edits.
    pub fn reset(&mut self, saved: Vec<WindowRule>) {
        *self = Self::new(saved);
    }

    /// What is wrong with the first rule that cannot be saved.
    pub fn check(&self) -> Result<(), String> {
        for (i, entry) in self.entries.iter().enumerate() {
            let place = i + 1;
            entry
                .rule
                .check()
                .map_err(|e| format!("rule {place}: {e}"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
