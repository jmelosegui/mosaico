//! What the window redoes each frame before drawing: its style, the
//! daemon's status and the files it shows.

use eframe::egui::Context;

use super::SettingsApp;
use crate::theme::{self, Colors};

/// How often the daemon is asked whether it runs, in seconds.
const POLL_SECS: f64 = 2.0;

impl SettingsApp {
    /// Styles the window for the theme being edited, which previews it.
    pub(super) fn style(&mut self, ctx: &Context) -> Colors {
        let theme = self.draft.edit.theme();
        let colors = Colors::of(theme);
        if self.styled != Some(theme) {
            if self.styled.is_none() {
                theme::install_fonts(ctx);
            }
            theme::apply(ctx, &colors);
            self.styled = Some(theme);
        }
        colors
    }

    pub(super) fn poll(&mut self, ctx: &Context) {
        let now = ctx.input(|i| i.time);
        if self.polled_at.is_none_or(|at| now - at >= POLL_SECS) {
            self.daemon_state = (self.daemon)();
            self.polled_at = Some(now);
            // Without edits, the window follows the file, so what an
            // editor writes, or a fix to it, shows up.
            if self.draft.changes() == 0 {
                self.reload(self.draft.saved().start_at_sign_in);
            }
            self.files.reload_unedited();
        }
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(POLL_SECS));
    }
}
