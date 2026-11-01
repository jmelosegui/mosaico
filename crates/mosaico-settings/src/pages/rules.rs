//! The Rules page: the user's window rules, from `user-rules.toml`, over
//! the community rules, from `rules.toml`, which are shown and not edited.

mod card;
mod community;
mod state;

use eframe::egui::{RichText, Ui};
use mosaico_core::config::{ConfigStore, WindowRule};

use super::{FilePage, Page};
use crate::theme::{self, Colors};
use crate::widgets;
use card::{Ask, Place};
use state::Draft;

pub struct RulesPage {
    store: ConfigStore,
    draft: Draft,
    community: Vec<WindowRule>,
    community_problem: Option<String>,
    problem: Option<String>,
}

impl RulesPage {
    pub fn new(store: &ConfigStore) -> Self {
        let mut page = Self {
            store: store.clone(),
            draft: Draft::default(),
            community: Vec::new(),
            community_problem: None,
            problem: None,
        };
        page.reload();
        page
    }

    fn your_rules(&mut self, ui: &mut Ui, c: &Colors, query: &str) {
        ui.label(
            RichText::new("Your rules")
                .font(theme::bold(13.0))
                .color(c.dim_text),
        );
        ui.label(
            RichText::new("The first rule that matches a window decides what happens to it.")
                .font(theme::regular(11.0))
                .color(c.faint_text),
        );
        ui.add_space(8.0);
        let last = self.draft.entries.len().saturating_sub(1);
        let mut ask = None;
        for i in 0..self.draft.entries.len() {
            if !self.shown(i, query) {
                continue;
            }
            let place = Place {
                number: i + 1,
                first: i == 0,
                last: i == last,
                was: self.draft.was(i).cloned(),
                changed: self.draft.is_changed(i),
            };
            let rule = &mut self.draft.entries[i].rule;
            if let Some(what) = card::show(ui, c, &place, rule) {
                ask = Some((i, what));
            }
        }
        match ask {
            Some((i, Ask::Up)) => self.draft.move_up(i),
            Some((i, Ask::Down)) => self.draft.move_down(i),
            Some((i, Ask::Delete)) => self.draft.remove(i),
            None => {}
        }
        if self.draft.entries.is_empty() {
            let text = "No rules of your own yet.";
            ui.label(RichText::new(text).color(c.dim_text));
            ui.add_space(8.0);
        }
        if widgets::button(ui, c, "Add rule", false, true).clicked() {
            self.draft.add();
        }
        ui.add_space(24.0);
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
        self.your_rules(ui, c, query);
        let problem = self.community_problem.as_deref();
        community::show(ui, c, &self.community, query, problem);
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
        Ok(())
    }

    fn revert(&mut self) {
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
