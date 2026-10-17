//! What ends a drag the tiler follows before its `Moved`.
//!
//! The user keeps the window they drag while they go on with a key, a
//! command or the pointer on a bar, and through a display change: no
//! layout pass places it, and its `Moved` places it as the end of any
//! drag does. The drag ends earlier when its window is no longer tiled
//! on the workspace on screen, as one that closed, hid, was minimized or
//! is no longer managed, or whose workspace went out of view, and when
//! another window takes focus other than by the tiler's doing: the one it
//! gave focus, or a dialog that window owns, which the system gives focus
//! in its place, is its doing. Its `Moved`, if it comes, is then handled
//! as for a drag the tiler did not follow.
//!
//! The platform may not report the end of every drag. One whose end is
//! never reported is over once another window takes focus, the window
//! leaves the layout or the user drags another window. Until then, as
//! when the user goes on with the keyboard alone, the window is not laid
//! out and its border stays hidden.

use crate::WindowId;

use super::super::{Plan, Tiler};

impl Tiler {
    /// Ends the drag followed of a window that is no longer tiled on the
    /// workspace on screen of its monitor, as one that closed, was
    /// minimized or is no longer managed. A display change that takes it
    /// to another monitor leaves it to the user.
    pub(in crate::tiler) fn end_drag_of_gone(&mut self) {
        let Some(drag) = &self.mouse.drag else {
            return;
        };
        let tiled = self
            .find_window(drag.window)
            .is_some_and(|(mon, ws)| self.monitors[mon].active_workspace == ws);
        if !tiled {
            self.end_drag();
        }
    }

    /// Ends the drag followed as `window` takes focus, when that is not
    /// the tiler's doing: a window other than the one dragged, the one the
    /// tiler gave focus, and a window that one owns, as a dialog of it the
    /// system gives focus in its place. The user has moved on, whether or
    /// not the end of the drag was reported.
    pub(in crate::tiler) fn end_drag_for_focus(&mut self, plan: &Plan, window: WindowId) {
        let Some(drag) = &self.mouse.drag else {
            return;
        };
        let focused = self.focused_window;
        let ours = focused.is_some_and(|f| f == window || plan.owner(window) == Some(f));
        if drag.window != window && !ours {
            crate::log_debug!("drag {} given up for {}", drag.window, window);
            self.end_drag();
        }
    }
}

#[cfg(test)]
#[path = "tests/drag_lost_tests.rs"]
mod tests;
