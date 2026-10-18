//! The focus and the border of the window the user drags.
//!
//! The window the user drags is the tiler's focused window from the start
//! of its drag: when it was not, the tiler takes focus for it, on its
//! monitor, and the borders follow, the one it leaves in the unfocused
//! style and its own in the focused one, or the monocle one. The platform
//! is not asked to move focus then, as the system gave focus to the window
//! it drags, and asking during its move or size loop could end it. When by
//! the drop no report says the system did, nor was the platform asked to,
//! the drop focuses the window through the platform, the pointer left where
//! the user let go.
//!
//! Its border stays on it through the drag, whatever ends or interrupts
//! it: each report of where the window is draws the border there, which a
//! move only puts in place, and a resize draws again as its size changes.
//! Without reports, as with window contents hidden while dragging, the
//! border stays where it was until the drop.

use crate::WindowId;

use super::super::{Plan, Tiler};

impl Tiler {
    /// Makes the window whose drag the tiler just started to follow its
    /// focused window, on its monitor, when it was not, and draws the
    /// borders again, without asking the platform to move focus. The drop
    /// owes it focus until a report says the system gave it.
    pub(in crate::tiler) fn focus_dragged(&mut self, plan: &mut Plan) {
        let Some(drag) = self.mouse.drag.as_mut() else {
            return;
        };
        if self.focused_window == Some(drag.window) {
            return;
        }
        drag.focus_owed = true;
        crate::log_debug!("drag {} takes focus", drag.window);
        self.focused_window = Some(drag.window);
        self.focused_monitor = drag.monitor;
        self.update_border(plan);
    }

    /// Takes a report of where the window the user drags is: its border is
    /// drawn there.
    pub(in crate::tiler) fn drag_reported(&mut self, plan: &mut Plan) {
        let Some(drag) = self.mouse.drag.as_ref() else {
            return;
        };
        let (window, monitor) = (drag.window, drag.monitor);
        if let Some(rect) = plan.rect(window) {
            self.show_border_for(plan, window, rect, monitor);
        }
    }

    /// Whether the drop of `window`, whose drag the tiler follows, owes it
    /// focus: the tiler took focus for it as the drag started, and neither
    /// a report said since that the system gave it focus, nor was the
    /// platform asked to.
    pub(in crate::tiler) fn owes_focus(&self, window: WindowId) -> bool {
        let drag = self.mouse.drag.as_ref();
        drag.is_some_and(|d| d.window == window && d.focus_owed)
    }

    /// Notes that the platform is asked to focus `window`: when the user
    /// drags it, its drop owes it focus no more.
    pub(in crate::tiler) fn focus_asked(&mut self, window: WindowId) {
        if let Some(drag) = self.mouse.drag.as_mut().filter(|d| d.window == window) {
            drag.focus_owed = false;
        }
    }

    /// Focuses `window` through the platform as its drop is handled, when
    /// it was `owed` focus and still has it in the tiler, so that the
    /// desktop agrees. The pointer stays where the user let go.
    pub(in crate::tiler) fn give_owed_focus(
        &mut self,
        plan: &mut Plan,
        window: WindowId,
        owed: bool,
    ) {
        if owed && self.focused_window == Some(window) {
            self.focus_window(plan, window, true);
        }
    }
}

#[cfg(test)]
#[path = "tests/drag_border_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/drag_border_end_tests.rs"]
mod end_tests;

#[cfg(test)]
#[path = "tests/drag_focus_tests.rs"]
mod focus_tests;

#[cfg(test)]
#[path = "tests/drag_focus_drop_tests.rs"]
mod focus_drop_tests;
