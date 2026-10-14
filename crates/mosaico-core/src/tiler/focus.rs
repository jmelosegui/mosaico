//! Keyboard focus.

use crate::{WindowId, Workspace};

use super::{Effect, Plan, Tiler};

/// The window to focus when a workspace comes into view, or takes focus
/// back: its monocle window, then the one it remembers, then its first.
pub(super) fn focus_target(ws: &Workspace) -> Option<WindowId> {
    let present = |w: &WindowId| ws.contains(*w);
    let remembered = ws.last_focused().filter(present);
    if ws.monocle() {
        ws.monocle_window().filter(present).or(remembered)
    } else {
        remembered
    }
    .or_else(|| ws.windows().first().copied())
}

impl Tiler {
    /// Focuses a window: the border and the bar follow right away, and
    /// the platform is asked to move focus.
    ///
    /// `from_mouse` is set when the user pointed at the window, in which
    /// case the pointer is already there and is not moved.
    pub(super) fn focus_window(&mut self, plan: &mut Plan, window: WindowId, from_mouse: bool) {
        self.take_focus(plan, window, from_mouse);
        self.update_border(plan);
    }

    /// [`Self::focus_window`] without the borders, for a caller that
    /// draws them itself once it is done.
    fn take_focus(&mut self, plan: &mut Plan, window: WindowId, from_mouse: bool) {
        self.focused_window = Some(window);
        self.focused_maximized = plan.is_maximized(window);
        plan.emit(Effect::Focus {
            window,
            move_cursor: self.mouse_follows_focus && !from_mouse,
        });
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

    /// Settles focus after the monitors changed, before the windows in
    /// `hiding` are hidden. The caller draws the borders afterwards.
    ///
    /// A focused window still in view keeps focus, wherever it moved,
    /// unless the change has it wait behind a monocle window: then it
    /// hands focus to the workspace's window ([`focus_target`]), the one
    /// in front, so focus is on what is on screen (#132). One about to be
    /// hidden hands focus to the focused monitor's window, as in a
    /// workspace switch, and with nothing there to take it, focus is
    /// parked on that monitor. Either way the system's own pick, once the
    /// focused window is hidden, is not followed: the echo filter drops it
    /// while the focus change is held, and the empty claim while focus is
    /// parked.
    pub(super) fn settle_focus(&mut self, plan: &mut Plan, hiding: &[WindowId]) {
        if let Some((mi, _)) = self
            .focused_window
            .filter(|&w| self.is_on_active_workspace(w))
            .and_then(|w| self.find_window(w))
        {
            self.focused_monitor = mi;
            let ws = self.monitors[mi].active_ws();
            if let Some(front) = self
                .focused_window
                .filter(|&w| ws.waits_behind(w))
                .and_then(|w| focus_target(ws).filter(|&front| front != w))
            {
                self.take_focus(plan, front, false);
            }
        } else if self.focused_window.is_some_and(|w| hiding.contains(&w)) {
            match focus_target(self.monitors[self.focused_monitor].active_ws()) {
                Some(window) => self.take_focus(plan, window, false),
                None => {
                    self.focused_window = None;
                    self.park_focus_if_empty(plan);
                }
            }
        }
    }
}
