//! The community rules, read-only.

use eframe::egui::{CollapsingHeader, RichText, Ui};
use mosaico_core::config::WindowRule;

use crate::theme::{self, Colors};

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

/// Lists the rules matching `query` under a heading that opens and
/// closes them; searching opens it.
pub fn show(ui: &mut Ui, c: &Colors, rules: &[WindowRule], query: &str, problem: Option<&str>) {
    let shown: Vec<_> = rules.iter().filter(|r| matches(r, query)).collect();
    let title = format!("Community rules ({})", shown.len());
    let mut header = CollapsingHeader::new(RichText::new(title).font(theme::bold(13.0)));
    if !query.trim().is_empty() {
        header = header.open(Some(true));
    }
    header.show(ui, |ui| {
        let note = |ui: &mut Ui, text: &str| {
            ui.label(
                RichText::new(text)
                    .font(theme::regular(11.0))
                    .color(c.faint_text),
            );
        };
        note(
            ui,
            "Downloaded for everyone, in rules.toml. Yours above come first.",
        );
        if let Some(problem) = problem {
            note(ui, problem);
        }
        for rule in shown {
            let line = RichText::new(describe(rule))
                .font(theme::regular(12.0))
                .color(c.dim_text);
            ui.label(line);
        }
    });
}
