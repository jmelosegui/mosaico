//! What ends a drag the tiler follows before its `Moved`, and what
//! interrupts it.
//!
//! Every action that reaches the tiler while the user drags a window,
//! from a key, a command or the pointer on a bar, and a display change,
//! as soon as it is noted, interrupt the drag: where the window lands is
//! no longer the tiler's to decide, the preview is hidden, and its
//! `Moved` is handled as before drops moved windows. `Action::TogglePause`
//! is the daemon's, which never hands it to the tiler, so pausing the
//! hotkeys leaves the drag alone. The user keeps the window all the same:
//! no layout pass places it until then. The drag ends earlier when its
//! window is no longer tiled on the workspace on screen, as one that
//! closed, hid, was minimized or is no longer managed, or whose workspace
//! went out of view, when the tiler stops, and when another window takes
//! focus other than by the tiler's doing: the one it gave focus, or a
//! dialog that window owns, which the system gives focus in its place, is
//! its doing. Its `Moved`, if it comes, is then handled as for a drag the
//! tiler did not follow.
//!
//! The platform may not report the end of every drag. One whose end is
//! never reported is over once another window takes focus, the window
//! leaves the layout or the user drags another window. Until then, as
//! when the user goes on with the keyboard alone, the window is not laid
//! out and its border stays hidden.

use crate::WindowId;

use super::super::{Effect, Plan, Tiler};
use super::Drag;

impl Tiler {
    /// Stops following the drag, if the tiler follows one, hides its
    /// preview, and returns it.
    pub(in crate::tiler) fn end_drag(&mut self, plan: &mut Plan) -> Option<Drag> {
        let drag = self.mouse.drag.take()?;
        crate::log_debug!("drag {} ends", drag.window);
        if drag.preview.is_some() {
            plan.emit(Effect::HidePreview);
        }
        Some(drag)
    }

    /// Whether the tiler follows the drag of `window`: until it ends,
    /// nothing places the window.
    pub(in crate::tiler) fn is_dragged(&self, window: WindowId) -> bool {
        self.mouse.drag.as_ref().is_some_and(|d| d.window == window)
    }

    /// Interrupts the drag followed, if any: the window stays the user's,
    /// its preview is hidden, and its end is handled as before drops moved
    /// windows.
    pub(in crate::tiler) fn interrupt_drag(&mut self, plan: &mut Plan) {
        if let Some(drag) = self.mouse.drag.as_mut().filter(|d| d.decides) {
            crate::log_debug!("drag {} interrupted", drag.window);
            drag.decides = false;
            if drag.preview.take().is_some() {
                plan.emit(Effect::HidePreview);
            }
        }
    }

    /// Ends the drag followed of a window that is no longer tiled on the
    /// workspace on screen of its monitor, as one that closed, was
    /// minimized or is no longer managed. A display change that takes it
    /// to another monitor leaves it to the user.
    pub(in crate::tiler) fn end_drag_of_gone(&mut self, plan: &mut Plan) {
        let Some(drag) = &self.mouse.drag else {
            return;
        };
        let tiled = self
            .find_window(drag.window)
            .is_some_and(|(mon, ws)| self.monitors[mon].active_workspace == ws);
        if !tiled {
            self.end_drag(plan);
        }
    }

    /// Ends the drag followed as `window` takes focus, when that is not
    /// the tiler's doing: a window other than the one dragged, the one the
    /// tiler gave focus, and a window that one owns, as a dialog of it the
    /// system gives focus in its place. The user has moved on, whether or
    /// not the end of the drag was reported. The dragged window taking
    /// focus ends nothing, and its drop owes it focus no more.
    pub(in crate::tiler) fn end_drag_for_focus(&mut self, plan: &mut Plan, window: WindowId) {
        let Some(drag) = self.mouse.drag.as_mut() else {
            return;
        };
        if drag.window == window {
            drag.focus_owed = false;
            return;
        }
        let focused = self.focused_window;
        let ours = focused.is_some_and(|f| f == window || plan.owner(window) == Some(f));
        if !ours {
            crate::log_debug!("drag {} given up for {}", drag.window, window);
            self.end_drag(plan);
        }
    }
}

#[cfg(test)]
#[path = "tests/drag_lost_tests.rs"]
mod tests;
