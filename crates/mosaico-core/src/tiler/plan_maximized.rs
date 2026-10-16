//! Whether a window is maximized once the step's effects are applied.
//!
//! A placement takes a window out of its maximized state, as
//! [`WindowControl::place`](crate::WindowControl::place) promises. One
//! posted behind what is on its way to a window that is out of view, or is
//! to be, may not: the platform may leave the window maximized then, and a
//! later show brings it back as if it had not been placed (see
//! [`FrameControl`](crate::frames::FrameControl)). So a window placed this
//! step counts as no longer maximized when it is in view after the step,
//! or when its changes are made at once. A step that shows a window it
//! places shows it first, so the placement lands behind the show.

use crate::WindowId;

use super::Plan;

impl Plan<'_> {
    /// Whether `window` is maximized once the step's effects are applied.
    pub(in crate::tiler) fn is_maximized(&self, window: WindowId) -> bool {
        let placed = self.planned(window).rect.is_some();
        let in_view = self.is_visible(window) && !self.is_minimized(window);
        if placed && (in_view || !self.changes_posted(window)) {
            return false;
        }
        self.desktop.is_maximized(window)
    }

    /// Whether a change made to `window` now is posted behind what was
    /// posted to it, so that it lands only once the window gets to it.
    pub(in crate::tiler) fn changes_posted(&self, window: WindowId) -> bool {
        self.desktop.changes_posted(window)
    }
}

#[cfg(test)]
#[path = "tests/plan_maximized_tests.rs"]
mod tests;
