//! The Actions tab: drawing its rows, and what a click on one does.

use eframe::egui::{RichText, Ui};

use super::banner::{self, Choice};
use super::row::{self, Click, Prompt, RowView};
use super::{KeybindingsPage, PROMPT, Row, combo_text, header, labels};
use crate::theme::Colors;
use crate::widgets;

impl KeybindingsPage {
    /// Draws the tab: the filter over the keybindings matching both
    /// `query` and it, a section per group. A search of the window
    /// replaces the filter. A banner asking whether to take a shortcut is
    /// drawn under its row, and what it says goes in `chosen`. Returns
    /// the row clicked, and how.
    pub(super) fn show_actions(
        &mut self,
        ui: &mut Ui,
        c: &Colors,
        query: &str,
        chosen: &mut Choice,
    ) -> Option<(Row, Click)> {
        let searching = !query.trim().is_empty();
        if !searching && header::draw(ui, c, &mut self.filter) {
            self.restore_defaults();
        }
        let filter = if searching { "" } else { self.filter.as_str() };
        let mut clicked = None;
        let mut any = false;
        for group in labels::GROUPS {
            let mut shown: Vec<_> = (0..self.current.len())
                .filter(|&i| labels::group(self.current[i].action) == group)
                .filter(|&i| self.row_matches(i, query) && self.row_matches(i, filter))
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
                        clicked = Some((Row::Action(i), click));
                    }
                    if let (Some(p), true) = (&self.pending, self.capturing == Some(Row::Action(i)))
                    {
                        *chosen = banner::draw(ui, c, &p.shortcut, &p.owners);
                    }
                }
            });
        }
        if !any && !searching && self.doc.is_some() {
            let text = format!("No keybindings match \u{201c}{}\u{201d}.", filter.trim());
            ui.label(RichText::new(text).color(c.dim_text));
        }
        clicked
    }

    pub(super) fn show_row(&self, ui: &mut Ui, c: &Colors, index: usize) -> Click {
        let binding = &self.current[index];
        let combo = match &self.pending {
            Some(p) if self.capturing == Some(Row::Action(index)) => p.shortcut.replace(' ', "+"),
            _ => combo_text(binding),
        };
        let own = combo_text(binding);
        let default = self.default_of(binding.action).map(combo_text);
        let default = default.filter(|d| *d != own);
        let unsaved = self.saved_of(binding.action).map(combo_text).as_ref() != Some(&own);
        let label = labels::label(binding.action);
        let id = binding.action.to_string();
        let keys: Vec<String> = combo.split('+').map(str::to_owned).collect();
        let capturing = self.prompt(Row::Action(index));
        let view = RowView {
            label: &label,
            id: &id,
            keys: &keys,
            unsaved,
            default: default.as_deref(),
            clash: self.clashes[index].as_deref(),
            capturing,
            app: None,
        };
        row::draw(ui, c, &view)
    }

    /// The line the row shows while it waits for a key press, when it
    /// does.
    pub(super) fn prompt(&self, row: Row) -> Option<Prompt<'_>> {
        (self.capturing == Some(row)).then(|| match &self.refused {
            Some(why) => Prompt {
                text: why,
                refused: true,
            },
            None => Prompt {
                text: PROMPT,
                refused: false,
            },
        })
    }

    pub(super) fn click(&mut self, row: Row, click: Click) {
        match click {
            Click::Nothing => {}
            Click::Change => {
                self.capturing = Some(row);
                self.refused = None;
                self.pending = None;
            }
            Click::Cancel => self.stop_capture(),
            Click::Reset => {
                let Row::Action(index) = row else { return };
                let default = self.default_of(self.current[index].action).cloned();
                if let Some(d) = default {
                    self.bind(row, &d.key, &d.modifiers);
                }
            }
            Click::Details => self.toggle_editor(row),
            Click::Remove => self.remove_app(row),
        }
    }

    pub(super) fn row_matches(&self, index: usize, query: &str) -> bool {
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
