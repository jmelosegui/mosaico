//! Bringing the window forward when another launch asks for it.

use std::sync::{Arc, OnceLock};

use eframe::egui::{Context, ViewportCommand};

/// Brings the window forward, from any thread, once it is shown.
///
/// The single-window lock is taken before the window exists, so a
/// summons that arrives before [`attach`](Self::attach) is dropped: the
/// window comes up in front anyway.
#[derive(Debug, Clone, Default)]
pub struct Forward(Arc<OnceLock<Context>>);

impl Forward {
    /// Brings forward the window that `ctx` draws, from now on.
    pub fn attach(&self, ctx: &Context) {
        let _ = self.0.set(ctx.clone());
    }

    /// Restores the window if it is minimized and gives it the focus.
    pub fn bring(&self) {
        if let Some(ctx) = self.0.get() {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
            ctx.request_repaint();
        }
    }
}
