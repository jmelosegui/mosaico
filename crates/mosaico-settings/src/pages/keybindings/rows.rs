//! Drawing one row of the page, and what a click on it does.

use eframe::egui::Ui;

use super::row::{self, Click, Prompt, RowView};
use super::{KeybindingsPage, PROMPT, combo_text, labels};
use crate::theme::Colors;

impl KeybindingsPage {
    pub(super) fn show_row(&self, ui: &mut Ui, c: &Colors, index: usize) -> Click {
        let binding = &self.current[index];
        let combo = match &self.pending {
            Some(p) if self.capturing == Some(index) => p.shortcut.replace(' ', "+"),
            _ => combo_text(binding),
        };
        let own = combo_text(binding);
        let default = self.default_of(binding.action).map(combo_text);
        let default = default.filter(|d| *d != own);
        let unsaved = self.saved_of(binding.action).map(combo_text).as_ref() != Some(&own);
        let label = labels::label(binding.action);
        let id = binding.action.to_string();
        let keys: Vec<String> = combo.split('+').map(str::to_owned).collect();
        let capturing = (self.capturing == Some(index)).then(|| match &self.refused {
            Some(why) => Prompt {
                text: why,
                refused: true,
            },
            None => Prompt {
                text: PROMPT,
                refused: false,
            },
        });
        let view = RowView {
            label: &label,
            id: &id,
            keys: &keys,
            unsaved,
            default: default.as_deref(),
            clash: self.clashes[index].as_deref(),
            capturing,
        };
        row::draw(ui, c, &view)
    }

    pub(super) fn click(&mut self, index: usize, click: Click) {
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

    pub(super) fn matches(&self, index: usize, query: &str) -> bool {
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
