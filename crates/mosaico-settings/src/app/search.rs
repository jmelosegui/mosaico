//! The search of the window: its field, and what matches on every page,
//! one page under another.

use eframe::egui::{
    Align, Align2, CursorIcon, Frame, Key, Margin, Response, RichText, Role, Sense, Stroke,
    TextEdit, Ui, WidgetInfo, vec2,
};

use super::SettingsApp;
use crate::icons::{self, Icon};
use crate::pages::{self, Page};
use crate::theme::{self, Colors};

impl SettingsApp {
    /// Whether the window shows a search rather than a page.
    pub(super) fn searching(&self) -> bool {
        !self.query.trim().is_empty()
    }

    /// How many rows of each page match the search, in the order of the
    /// navigation.
    pub(super) fn counts(&self) -> [usize; Page::ALL.len()] {
        Page::ALL.map(|page| {
            self.files
                .matches(page, &self.query)
                .unwrap_or_else(|| pages::count(page, &self.query))
        })
    }

    /// The pages with a match, each under its name, scrolling to the one
    /// picked in the navigation.
    pub(super) fn results(&mut self, ui: &mut Ui, c: &Colors) {
        let query = self.query.clone();
        let mut first = true;
        for (page, found) in Page::ALL.into_iter().zip(self.counts()) {
            if found == 0 {
                continue;
            }
            if !first {
                ui.add_space(8.0);
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                ui.painter()
                    .hline(rect.x_range(), rect.center().y, Stroke::new(1.0, c.line));
                ui.add_space(20.0);
            }
            first = false;
            let heading = heading(ui, c, page);
            if self.scroll_to == Some(page) {
                heading.scroll_to_me(Some(Align::TOP));
            }
            self.body(ui, c, page, &query);
        }
        self.scroll_to = None;
        if first {
            let text = format!("No settings match \u{201c}{}\u{201d}.", query.trim());
            ui.label(RichText::new(text).color(c.dim_text));
        }
    }
}

/// A page's icon and name over its results, beside the file it edits.
fn heading(ui: &mut Ui, c: &Colors, page: Page) -> Response {
    let response = ui
        .horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
            icons::paint(ui.painter(), rect, page.icon(), c.accent, 1.5);
            ui.add_space(4.0);
            ui.label(RichText::new(page.label()).font(theme::bold(15.0)));
            ui.label(
                RichText::new(page.subtitle())
                    .font(theme::regular(11.0))
                    .color(c.faint_text),
            );
        })
        .response;
    ui.add_space(12.0);
    response
}

/// The search field. Escape, or the cross at its end, clears it.
pub(super) fn field(ui: &mut Ui, c: &Colors, query: &mut String) {
    Frame::new()
        .fill(c.control)
        .corner_radius(5)
        .inner_margin(Margin::symmetric(8, 0))
        .show(ui, |ui| {
            ui.set_height(28.0);
            ui.horizontal_centered(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                icons::paint(ui.painter(), rect, Icon::Search, c.faint_text, 1.5);
                let hint = RichText::new("Search settings").color(c.faint_text);
                let clear = 14.0 + ui.spacing().item_spacing.x;
                let width = if query.is_empty() {
                    f32::INFINITY
                } else {
                    ui.available_width() - clear
                };
                let field = TextEdit::singleline(query)
                    .hint_text(hint)
                    .font(theme::regular(12.0))
                    .frame(eframe::egui::Frame::NONE)
                    .align(Align2::LEFT_CENTER)
                    .desired_width(width)
                    .show(ui)
                    .response;
                field.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Search settings"));
                if field.lost_focus() && ui.input(|i| i.key_pressed(Key::Escape)) {
                    query.clear();
                }
                if !query.is_empty() && clear_button(ui, c).clicked() {
                    query.clear();
                }
            });
        });
}

/// The cross that clears the search.
fn clear_button(ui: &mut Ui, c: &Colors) -> eframe::egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Clear search"));
    let color = if response.hovered() {
        c.text
    } else {
        c.faint_text
    };
    icons::paint(ui.painter(), rect.shrink(2.0), Icon::Close, color, 1.5);
    response.on_hover_cursor(CursorIcon::PointingHand)
}
