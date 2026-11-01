//! The page the window shows: its rows under what is wrong with its file.

use eframe::egui::{Frame, Margin, ScrollArea, Ui};

use super::SettingsApp;
use crate::pages::{self, Page, PageContext};
use crate::theme::Colors;

impl SettingsApp {
    /// The page picked in the navigation or, while searching, what matches
    /// on every page.
    pub(super) fn page(&mut self, ui: &mut Ui, c: &Colors) {
        let mut area = ScrollArea::vertical().auto_shrink(false);
        if std::mem::take(&mut self.scroll_top) {
            area = area.vertical_scroll_offset(0.0);
        }
        area.show(ui, |ui| {
            Frame::new()
                .inner_margin(Margin::symmetric(32, 24))
                .show(ui, |ui| {
                    if self.searching() {
                        self.results(ui, c);
                    } else {
                        pages::header(ui, c, self.page, self.page.subtitle());
                        self.body(ui, c, self.page, "");
                    }
                });
        });
    }

    /// The rows of `page` matching `query`, under what is wrong with its
    /// file, if anything is.
    pub(super) fn body(&mut self, ui: &mut Ui, c: &Colors, page: Page, query: &str) {
        let file = page.file();
        if let Some(shown) = self.files.get(page) {
            if let Some(problem) = shown.problem() {
                pages::problem(ui, c, file, problem);
            }
            let enabled = shown.problem().is_none();
            ui.add_enabled_ui(enabled, |ui| shown.show(ui, c, query));
            return;
        }
        if let Some(problem) = &self.problem {
            pages::problem(ui, c, file, problem);
        }
        let saved = self.draft.saved().clone();
        let cx = PageContext {
            c,
            saved: &saved,
            defaults: &self.defaults,
            log_file: &self.log_file,
        };
        ui.add_enabled_ui(self.problem.is_none(), |ui| {
            pages::show(ui, &cx, page, query, &mut self.draft.edit);
        });
    }
}
