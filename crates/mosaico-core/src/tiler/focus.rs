//! Keyboard focus.

use crate::WindowId;

use super::{Effect, Plan, Tiler};

impl Tiler {
    /// Focuses a window: the border and the bar follow right away, and
    /// the platform is asked to move focus.
    ///
    /// `from_mouse` is set when the user pointed at the window, in which
    /// case the pointer is already there and is not moved.
    pub(super) fn focus_window(&mut self, plan: &mut Plan, window: WindowId, from_mouse: bool) {
        self.focused_window = Some(window);
        self.focused_maximized = plan.is_maximized(window);
        plan.emit(Effect::Focus {
            window,
            move_cursor: self.mouse_follows_focus && !from_mouse,
        });
        self.update_border(plan);
    }

    /// Puts the pointer back on the focused window after it moved, when
    /// the pointer follows focus.
    pub(super) fn center_cursor_on_focused(&self, plan: &mut Plan) {
        if self.mouse_follows_focus
            && let Some(window) = self.focused_window
        {
            plan.emit(Effect::CenterCursor { window });
        }
    }

    /// Focuses the first window on the first monitor at startup.
    pub(super) fn focus_initial(&mut self, plan: &mut Plan) {
        if let Some(&window) = self
            .monitors
            .first()
            .and_then(|mon| mon.active_ws().windows().first())
        {
            self.focus_window(plan, window, false);
        }
    }
}
