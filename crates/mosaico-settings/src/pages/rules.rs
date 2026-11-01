//! The Rules page: the user's window rules, from `user-rules.toml`, over
//! the community rules, from `rules.toml`, which are shown and not edited.

mod community;
mod editor;
mod row;
mod state;

use eframe::egui::{Align, Layout, Rect, RichText, Sense, Ui, vec2};
use mosaico_core::config::{ConfigStore, WindowRule};

use super::{FilePage, Page};
use crate::icons::{self, Icon};
use crate::theme::{self, Colors};
use crate::widgets;
use state::Draft;

pub struct RulesPage {
    store: ConfigStore,
    draft: Draft,
    community: Vec<WindowRule>,
    community_problem: Option<String>,
    problem: Option<String>,
    /// The rule whose editor is open.
    editing: Option<usize>,
}

impl RulesPage {
    pub fn new(store: &ConfigStore) -> Self {
        let mut page = Self {
            store: store.clone(),
            draft: Draft::default(),
            community: Vec::new(),
            community_problem: None,
            problem: None,
            editing: None,
        };
        page.reload();
        page
    }

    fn intro(&mut self, ui: &mut Ui, c: &Colors) {
        // The sketch puts the intro 12 points under the file name, closer
        // than the header leaves before a section, in a row as tall as
        // the button.
        ui.add_space(-6.0);
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = 21.0;
            ui.horizontal(|ui| {
                let text = "Rules are checked from the top and the first match wins. A window no rule matches is tiled.";
                let text = RichText::new(text)
                    .font(theme::regular(12.0))
                    .color(c.dim_text);
                ui.label(text);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if row::add_button(ui, c, "Add rule").clicked() {
                        self.editing = Some(self.draft.add());
                        ui.ctx().request_repaint();
                    }
                });
            });
        });
        ui.add_space(14.0);
    }

    fn your_rules(&mut self, ui: &mut Ui, c: &Colors, query: &str) {
        heading(ui, c, None, "Your rules", "user-rules.toml");
        let mut ask = None;
        let mut rects = Vec::new();
        let mut dragged = None;
        let mut closed = false;
        let shown: Vec<_> = (0..self.draft.entries.len())
            .filter(|&i| self.shown(i, query))
            .collect();
        widgets::card(ui, c, |ui| {
            for (place, &i) in shown.iter().enumerate() {
                let editing = self.editing == Some(i);
                let n = i + 1;
                let rule = self.draft.entries[i].rule.clone();
                let row = row::user(ui, c, n, &rule, editing);
                rects.push((i, row.rect));
                if let Some(what) = row.ask {
                    ask = Some((i, what));
                }
                if row.grip.is_some_and(|grip| grip.drag_stopped()) {
                    dragged = Some(i);
                }
                if editing {
                    let was = self.draft.was(i).cloned();
                    let rule = &mut self.draft.entries[i].rule;
                    closed = editor::show(ui, c, n, rule, was.as_ref());
                }
                if place + 1 < shown.len() {
                    widgets::divider(ui, c);
                }
            }
            if shown.is_empty() {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    let text = "No rules of your own yet.";
                    ui.label(RichText::new(text).color(c.dim_text));
                });
                ui.add_space(12.0);
            }
        });
        if closed {
            self.editing = None;
            ui.ctx().request_repaint();
        }
        if let Some(from) = dragged {
            self.drop(ui, from, &rects);
        }
        if ask.is_some() || dragged.is_some() {
            ui.ctx().request_repaint();
        }
        match ask {
            Some((i, row::Ask::Edit)) if self.editing == Some(i) => self.editing = None,
            Some((i, row::Ask::Edit)) => self.editing = Some(i),
            Some((i, row::Ask::Delete)) => self.delete(i),
            _ => {}
        }
        ui.add_space(14.0);
    }

    /// Moves the rule dragged from `from` to the row under the pointer.
    fn drop(&mut self, ui: &Ui, from: usize, rects: &[(usize, Rect)]) {
        let Some(pointer) = ui.ctx().pointer_interact_pos() else {
            return;
        };
        let under = rects.iter().find(|(_, r)| r.y_range().contains(pointer.y));
        let Some(&(to, _)) = under else {
            return;
        };
        self.draft.move_to(from, to);
        if self.editing == Some(from) {
            self.editing = Some(to);
        }
    }

    fn delete(&mut self, index: usize) {
        self.draft.remove(index);
        self.editing = match self.editing {
            Some(at) if at == index => None,
            Some(at) if at > index => Some(at - 1),
            other => other,
        };
    }

    /// Whether the rule at `index` matches `query`. A rule with nothing to
    /// match yet is always shown, as it was just added.
    fn shown(&self, index: usize, query: &str) -> bool {
        let rule = &self.draft.entries[index].rule;
        let blank = rule.match_class.is_none() && rule.match_title.is_none();
        let blank = blank && rule.match_path.is_none();
        blank || community::matches(rule, query)
    }
}

impl FilePage for RulesPage {
    fn page(&self) -> Page {
        Page::Rules
    }

    fn changes(&self) -> usize {
        self.draft.changes()
    }

    fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    fn show(&mut self, ui: &mut Ui, c: &Colors, query: &str) {
        self.intro(ui, c);
        self.your_rules(ui, c, query);
        let problem = self.community_problem.as_deref();
        if let Some(rule) = community::show(ui, c, &self.community, query, problem) {
            self.editing = Some(self.draft.push(rule));
            ui.ctx().request_repaint();
        }
    }

    fn apply(&mut self) -> Result<(), String> {
        self.draft.check()?;
        let mut doc = self.store.open_user_rules().map_err(|e| e.to_string())?;
        if doc.rules() != self.draft.saved() {
            return Err("user-rules.toml changed meanwhile; revert to see it".into());
        }
        let slots: Vec<_> = self
            .draft
            .entries
            .iter()
            .map(|e| (e.origin, e.rule.clone()))
            .collect();
        doc.arrange(&slots).map_err(|e| e.to_string())?;
        doc.save().map_err(|e| e.to_string())?;
        self.draft.reset(doc.rules());
        self.editing = None;
        Ok(())
    }

    fn revert(&mut self) {
        self.editing = None;
        let saved = self.draft.saved().to_vec();
        self.draft.reset(saved);
    }

    fn reload(&mut self) {
        match self.store.open_user_rules() {
            Ok(doc) => {
                self.draft.reset(doc.rules());
                self.problem = None;
            }
            Err(e) => {
                self.draft.reset(Vec::new());
                self.problem = Some(e.to_string());
            }
        }
        let len = self.draft.entries.len();
        self.editing = self.editing.filter(|&at| at < len);
        self.community_problem = None;
        self.community = Vec::new();
        if self.store.rules_path().exists() {
            match self.store.try_load_rules() {
                Ok(rules) => self.community = rules,
                Err(e) => self.community_problem = Some(e.to_string()),
            }
        }
    }
}

/// A section's title, over its card, with a dim `note` after it.
fn heading(ui: &mut Ui, c: &Colors, icon: Option<Icon>, title: &str, note: &str) {
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = 20.0;
        ui.horizontal(|ui| heading_row(ui, c, icon, title, note));
    });
    ui.add_space(2.0);
}

fn heading_row(ui: &mut Ui, c: &Colors, icon: Option<Icon>, title: &str, note: &str) {
    ui.spacing_mut().item_spacing.x = 8.0;
    if let Some(icon) = icon {
        let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
        icons::paint(ui.painter(), rect, icon, c.dim_text, 1.5);
    }
    let title = RichText::new(title)
        .font(theme::bold(13.0))
        .color(c.dim_text);
    ui.label(title);
    let note = RichText::new(note)
        .font(theme::regular(11.0))
        .color(c.faint_text);
    ui.label(note);
}
