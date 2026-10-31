//! One rule of the user's list, as a card.

use eframe::egui::{Button, Frame, Margin, Response, RichText, Role, Stroke, Ui, WidgetInfo};
use mosaico_core::config::{RuleAction, WindowRule};

use crate::theme::{self, Colors};
use crate::widgets;

/// What the user asked of a card besides editing the rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ask {
    Up,
    Down,
    Delete,
}

/// How the card sits in the list.
pub struct Place {
    /// The rule's number, from 1.
    pub number: usize,
    pub first: bool,
    pub last: bool,
    /// The rule as saved, if it is saved, to mark what was edited.
    pub was: Option<WindowRule>,
    /// The rule is new, moved or edited.
    pub changed: bool,
}

/// Draws the card of `rule`, editing it.
pub fn show(ui: &mut Ui, c: &Colors, place: &Place, rule: &mut WindowRule) -> Option<Ask> {
    let mut ask = None;
    let outline = if place.changed { c.accent } else { c.line };
    let n = place.number;
    Frame::new()
        .fill(c.card)
        .stroke(Stroke::new(1.0, outline))
        .corner_radius(8)
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Rule {n}")).font(theme::bold(13.0)));
                ui.with_layout(
                    eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                    |ui| ask = buttons(ui, c, place),
                );
            });
            fields(ui, c, place, rule);
        });
    ui.add_space(8.0);
    ask
}

fn buttons(ui: &mut Ui, c: &Colors, place: &Place) -> Option<Ask> {
    let n = place.number;
    let mut ask = None;
    let delete = small(ui, c, "Delete", &format!("Delete rule {n}"), true);
    let down = small(ui, c, "Down", &format!("Move rule {n} down"), !place.last);
    let up = small(ui, c, "Up", &format!("Move rule {n} up"), !place.first);
    for (response, what) in [(delete, Ask::Delete), (down, Ask::Down), (up, Ask::Up)] {
        if response.clicked() {
            ask = Some(what);
        }
    }
    ask
}

fn small(ui: &mut Ui, c: &Colors, text: &str, label: &str, enabled: bool) -> Response {
    let text = RichText::new(text).font(theme::regular(12.0)).color(c.text);
    let button = Button::new(text).fill(c.raised).corner_radius(5);
    let response = ui.add_enabled(enabled, button);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, label));
    response
}

fn fields(ui: &mut Ui, c: &Colors, place: &Place, rule: &mut WindowRule) {
    let n = place.number;
    let was = place.was.as_ref();
    eframe::egui::Grid::new(("rule-fields", n))
        .num_columns(2)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            let was_class = was.map(|w| &w.match_class);
            field(ui, c, ("Class", n), &mut rule.match_class, was_class);
            ui.end_row();
            let was_title = was.map(|w| &w.match_title);
            field(
                ui,
                c,
                ("Title contains", n),
                &mut rule.match_title,
                was_title,
            );
            ui.end_row();
            let was_path = was.map(|w| &w.match_path);
            field(ui, c, ("Program path", n), &mut rule.match_path, was_path);
            ui.end_row();
            action(ui, c, n, rule, was.map(|w| w.action));
            ui.end_row();
        });
}

/// A line of text for an optional match, which is left out while empty.
fn field(
    ui: &mut Ui,
    c: &Colors,
    (name, n): (&str, usize),
    value: &mut Option<String>,
    was: Option<&Option<String>>,
) {
    ui.label(
        RichText::new(name)
            .font(theme::regular(12.0))
            .color(c.dim_text),
    );
    let shown = value.clone().unwrap_or_default();
    let mut text = shown.clone();
    let changed = was.is_none_or(|w| w != value);
    let label = format!("{name} of rule {n}");
    widgets::text(ui, c, &label, 360.0, &mut text, "any", changed);
    if text != shown {
        *value = (!text.is_empty()).then_some(text);
    }
}

fn action(ui: &mut Ui, c: &Colors, n: usize, rule: &mut WindowRule, was: Option<RuleAction>) {
    ui.label(
        RichText::new("Action")
            .font(theme::regular(12.0))
            .color(c.dim_text),
    );
    let options = [
        (RuleAction::Tile, "Tile"),
        (RuleAction::Float, "Float"),
        (RuleAction::Ignore, "Ignore"),
    ];
    let changed = was != Some(rule.action);
    let label = format!("Action of rule {n}");
    widgets::dropdown(ui, c, &label, 110.0, &options, &mut rule.action, changed);
}
