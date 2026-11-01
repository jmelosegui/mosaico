//! The editor that opens under a rule's row.

use eframe::egui::{Align, Frame, Layout, Margin, RichText, Ui};
use mosaico_core::config::{RuleAction, WindowRule};

use crate::theme::{self, Colors};
use crate::widgets;

/// Draws the editor of `rule`, number `n`. `was` is the rule as saved, to
/// outline what was edited. Returns whether Done was clicked.
pub fn show(
    ui: &mut Ui,
    c: &Colors,
    n: usize,
    rule: &mut WindowRule,
    was: Option<&WindowRule>,
) -> bool {
    let mut done = false;
    Frame::new()
        .fill(c.selected)
        .inner_margin(Margin {
            left: 40,
            right: 16,
            top: 4,
            bottom: 14,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 12.0;
            fields(ui, c, n, rule, was);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let text = RichText::new("Matching windows")
                    .font(theme::regular(12.0))
                    .color(c.dim_text);
                ui.label(text);
                decision(ui, c, rule, was);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    done = super::row::outlined_button(ui, c, "Done").clicked();
                });
            });
        });
    done
}

fn fields(ui: &mut Ui, c: &Colors, n: usize, rule: &mut WindowRule, was: Option<&WindowRule>) {
    let gap = 12.0;
    let width = (ui.available_width() - 2.0 * gap) / 3.0;
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        let was_class = was.map(|w| &w.match_class);
        field(
            ui,
            c,
            ("Class is", n, width),
            &mut rule.match_class,
            was_class,
        );
        let was_title = was.map(|w| &w.match_title);
        let title = ("Title contains", n, width);
        field(ui, c, title, &mut rule.match_title, was_title);
        let was_path = was.map(|w| &w.match_path);
        let path = ("Executable path ends with", n, width);
        field(ui, c, path, &mut rule.match_path, was_path);
    });
}

/// A line of text for an optional match, which is left out while empty.
fn field(
    ui: &mut Ui,
    c: &Colors,
    (name, n, width): (&str, usize, f32),
    value: &mut Option<String>,
    was: Option<&Option<String>>,
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 5.0;
        ui.set_width(width);
        let label = RichText::new(name)
            .font(theme::regular(11.0))
            .color(c.dim_text);
        ui.label(label);
        let shown = value.clone().unwrap_or_default();
        let mut text = shown.clone();
        let changed = was.is_none_or(|w| w != value);
        let label = format!("{name} of rule {n}");
        // The text frame has 6 points of margin on each side.
        widgets::text(ui, c, &label, width - 14.0, &mut text, "Any", changed);
        if text != shown {
            *value = (!text.is_empty()).then_some(text);
        }
    });
}

/// Whether the matching windows are tiled, float or are left alone.
fn decision(ui: &mut Ui, c: &Colors, rule: &mut WindowRule, was: Option<&WindowRule>) {
    let changed = was.is_none_or(|w| w.action != rule.action);
    let options = RuleAction::ALL.map(|action| (action, action_label(action)));
    widgets::segmented(ui, c, &options, &mut rule.action, changed);
}

/// What the window calls `action`.
pub(super) fn action_label(action: RuleAction) -> &'static str {
    match action {
        RuleAction::Tile => "Tile",
        RuleAction::Float => "Float",
        RuleAction::Ignore => "Don't tile",
    }
}
