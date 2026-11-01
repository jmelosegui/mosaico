//! Pages that edit a file of their own rather than `config.toml`.

use eframe::egui::Ui;

use super::Page;
use crate::theme::Colors;

/// A page that reads, edits and saves one config file by itself. The
/// window counts its edits with the others, saves them on Apply, drops
/// them on Revert and, while it has none, has it follow the file.
pub trait FilePage {
    /// The page of the navigation it draws.
    fn page(&self) -> Page;

    /// How many edits are not saved yet.
    fn changes(&self) -> usize;

    /// Why the file cannot be edited, when it cannot. The page is shown
    /// disabled under the problem, so it has no edits then.
    fn problem(&self) -> Option<&str>;

    /// Draws the rows matching `query`, editing them.
    fn show(&mut self, ui: &mut Ui, c: &Colors, query: &str);

    /// Writes the edits to the file. On failure the edits are kept.
    fn apply(&mut self) -> Result<(), String>;

    /// Drops every edit.
    fn revert(&mut self);

    /// Reads the file again. Called only while there are no edits.
    fn reload(&mut self);
}

/// The pages that edit files of their own.
#[derive(Default)]
pub struct FilePages(Vec<Box<dyn FilePage>>);

impl FilePages {
    pub fn new(pages: Vec<Box<dyn FilePage>>) -> Self {
        Self(pages)
    }

    /// The one drawing `page`, if a file page does.
    pub fn get(&mut self, page: Page) -> Option<&mut dyn FilePage> {
        self.0
            .iter_mut()
            .find(|p| p.page() == page)
            .map(|p| &mut **p as &mut dyn FilePage)
    }

    /// The edits of every page, together.
    pub fn changes(&self) -> usize {
        self.0.iter().map(|p| p.changes()).sum()
    }

    /// Saves the pages with edits, stopping at the first that fails.
    pub fn apply(&mut self) -> Result<(), String> {
        for page in self.0.iter_mut().filter(|p| p.changes() > 0) {
            page.apply()?;
        }
        Ok(())
    }

    pub fn revert(&mut self) {
        self.0.iter_mut().for_each(|p| p.revert());
    }

    /// Has the pages without edits follow their files.
    pub fn reload_unedited(&mut self) {
        for page in self.0.iter_mut().filter(|p| p.changes() == 0) {
            page.reload();
        }
    }
}

#[cfg(test)]
#[path = "file_page_tests.rs"]
mod tests;
