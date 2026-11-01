//! The community rules, read-only.

use eframe::egui::{RichText, Ui};
use mosaico_core::config::WindowRule;

use super::row;
use crate::icons::Icon;
use crate::theme::{self, Colors};
use crate::widgets;

/// What a rule matches on, as one line.
pub fn describe(rule: &WindowRule) -> String {
    let matches = [
        ("class", &rule.match_class),
        ("title", &rule.match_title),
        ("path", &rule.match_path),
    ];
    let parts: Vec<String> = matches
        .iter()
        .filter_map(|(name, value)| {
            value
                .as_ref()
                .map(|v| format!("{name} \u{201c}{v}\u{201d}"))
        })
        .collect();
    format!("{}: {}", rule.action.name(), parts.join(", "))
}

/// Whether the rule's patterns or action contain `query`, ignoring case.
pub fn matches(rule: &WindowRule, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    describe(rule).to_lowercase().contains(&query)
}

/// Lists the rules matching `query` under their heading, dimmed as they
/// are read-only. Returns the rule whose Override was clicked.
pub fn show(
    ui: &mut Ui,
    c: &Colors,
    rules: &[WindowRule],
    query: &str,
    problem: Option<&str>,
) -> Option<WindowRule> {
    let note = "rules.toml, read-only. Downloaded from jmelosegui/mosaico-rules.";
    super::heading(ui, c, Some(Icon::Lock), "Community rules", note);
    if let Some(problem) = problem {
        let text = RichText::new(problem)
            .font(theme::regular(11.0))
            .color(c.warning);
        ui.label(text);
    }
    let shown: Vec<_> = rules.iter().filter(|r| matches(r, query)).collect();
    if shown.is_empty() {
        return None;
    }
    let mut chosen = None;
    ui.scope(|ui| {
        ui.set_opacity(0.8);
        widgets::card(ui, c, |ui| {
            for (i, rule) in shown.iter().enumerate() {
                if i > 0 {
                    widgets::divider(ui, c);
                }
                if row::community(ui, c, i + 1, rule).is_some() {
                    chosen = Some((*rule).clone());
                }
            }
        });
    });
    chosen
}
