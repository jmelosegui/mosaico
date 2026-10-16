//! The corners the tiler gives the windows it manages, and gives back.
//!
//! Every window the tiler takes in gets the configured corner style, and
//! the tiler remembers which windows it gave it to. Only those get the
//! system's corners back, so menus and windows the rules leave alone
//! keep the corners they chose.

use crate::WindowId;

use super::{Effect, Plan, Tiler};

impl Tiler {
    /// Applies the configured corners to a window that became managed.
    pub(super) fn decorate(&mut self, plan: &mut Plan, window: WindowId) {
        self.decorated.insert(window);
        plan.emit(Effect::Decorate {
            window,
            corners: self.border_config.corner_style,
        });
    }

    /// Gives a window the system's corners back, if the tiler gave it
    /// the configured ones. Other windows, such as menus or windows the
    /// rules leave alone, keep the corners they chose.
    pub(super) fn undecorate(&mut self, plan: &mut Plan, window: WindowId) {
        if self.decorated.remove(&window) {
            plan.emit(Effect::Undecorate { window });
        }
    }
}
