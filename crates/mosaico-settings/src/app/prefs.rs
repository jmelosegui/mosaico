//! What the window remembers between runs, in `settings-app.toml`: its
//! size, whether it filled the screen and the page it showed. Not its
//! place, which could be on a monitor no longer connected.

use eframe::egui::Context;
use mosaico_core::config::{ConfigStore, SettingsAppConfig};

use super::SettingsApp;
use crate::pages::Page;

/// What the window remembers in `store`, or the defaults without one.
pub fn remembered(store: &Result<ConfigStore, String>) -> SettingsAppConfig {
    store
        .as_ref()
        .map(ConfigStore::load_settings_app)
        .unwrap_or_default()
}

/// The page named `name` in the navigation, or the first page.
pub(super) fn page_named(name: &str) -> Page {
    Page::ALL
        .into_iter()
        .find(|page| page.label() == name)
        .unwrap_or(Page::General)
}

impl SettingsApp {
    /// What the window would remember if it closed now.
    pub fn remembered(&self) -> &SettingsAppConfig {
        &self.remembered
    }

    /// Writes what the window remembers, for its next run. A failure is
    /// not worth a word: the next run opens as a first one does.
    pub fn remember(&self) {
        if let Ok(store) = &self.store {
            let _ = store.save_settings_app(&self.remembered);
        }
    }

    /// Follows the page and the window's size, once a frame.
    pub(super) fn track(&mut self, ctx: &Context) {
        self.remembered.page = self.page.label().to_owned();
        let (rect, maximized, minimized) = ctx.input(|i| {
            let viewport = i.viewport();
            (viewport.inner_rect, viewport.maximized, viewport.minimized)
        });
        if minimized == Some(true) {
            return;
        }
        if let Some(maximized) = maximized {
            self.remembered.maximized = maximized;
        }
        // A window that fills the screen keeps the size to go back to.
        if let (false, Some(rect)) = (self.remembered.maximized, rect) {
            self.remembered.width = rect.width();
            self.remembered.height = rect.height();
        }
    }
}
