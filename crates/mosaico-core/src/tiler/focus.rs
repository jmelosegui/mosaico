//! Keyboard focus.

use crate::{WindowId, Workspace};

use super::{Effect, Plan, Tiler};

/// The window to focus when a workspace comes into view, or takes focus
/// back: its monocle window, then the one it remembers, floating or not
/// outside monocle, then its first tile.
pub(super) fn focus_target(ws: &Workspace) -> Option<WindowId> {
    let present = |w: &WindowId| ws.contains(*w);
    if ws.monocle() {
        let remembered = ws.last_focused().filter(present);
        ws.monocle_window().filter(present).or(remembered)
    } else {
        ws.last_focused().filter(|&w| ws.holds(w))
    }
    .or_else(|| ws.windows().first().copied())
}

impl Tiler {
    /// Focuses a window: the border and the bar follow right away, and
    /// the platform is asked to move focus.
    ///
    /// `from_mouse` is set when the user pointed at the window, in which
    /// case the pointer is already there and is not moved. Nor is it in a
    /// step that answers the pointer elsewhere, such as a click on a bar.
    pub(super) fn focus_window(&mut self, plan: &mut Plan, window: WindowId, from_mouse: bool) {
        self.take_focus(plan, window, from_mouse);
        self.update_border(plan);
    }

    /// [`Self::focus_window`] without the borders, for a caller that
    /// draws them itself once it is done.
    fn take_focus(&mut self, plan: &mut Plan, window: WindowId, from_mouse: bool) {
        self.focused_window = Some(window);
        self.focused_maximized = plan.is_maximized(window);
        self.focus_asked(window);
        plan.emit(Effect::Focus {
            window,
            move_cursor: self.mouse.follows_focus && !from_mouse && !plan.from_pointer,
        });
        self.keep_floats_in_front(plan, window);
    }

    /// Puts the pointer back on the focused window after it moved, when
    /// the pointer follows focus and the step does not answer the pointer.
    pub(super) fn center_cursor_on_focused(&self, plan: &mut Plan) {
        if self.mouse.follows_focus
            && !plan.from_pointer
            && let Some(window) = self.focused_window
        {
            plan.emit(Effect::CenterCursor { window });
        }
    }

    /// Moves focus to monitor `mon`, to the window its workspace on screen
    /// takes focus back with ([`focus_target`]), or parks focus there when
    /// the workspace is empty. The workspace focus leaves remembers its
    /// focused window, for coming back.
    pub(super) fn enter_monitor(&mut self, plan: &mut Plan, mon: usize) {
        if mon >= self.monitors.len() {
            return;
        }
        let focused = self.focused_window;
        if let Some(ws) = self
            .monitors
            .get_mut(self.focused_monitor)
            .map(|m| m.active_ws_mut())
            && focused.is_some_and(|w| ws.holds(w))
        {
            ws.set_last_focused(focused);
        }
        self.focused_monitor = mon;
        match focus_target(self.monitors[mon].active_ws()) {
            Some(window) => self.focus_window(plan, window, false),
            None => {
                self.focused_window = None;
                self.update_border(plan);
            }
        }
        self.park_focus_if_empty(plan);
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
    /// unless it is one of the windows the change has `waiting` behind a
    /// monocle window: then it hands focus to the workspace's window
    /// ([`focus_target`]), the one in front, so focus is on what is on
    /// screen (#132), but for the window the user drags, which waits with
    /// focus until they drop it. One about to be hidden hands focus to the
    /// focused monitor's window, as in a workspace switch, and with nothing
    /// there to take it, focus is parked on that monitor. Either way the
    /// system's own pick, once the focused window is hidden, is not
    /// followed: the echo filter drops it while the focus change is held,
    /// and the empty claim while focus is parked.
    pub(super) fn settle_focus(
        &mut self,
        plan: &mut Plan,
        hiding: &[WindowId],
        waiting: &[WindowId],
    ) {
        if let Some((mi, _)) = self
            .focused_window
            .filter(|&w| self.is_in_view(w))
            .and_then(|w| self.find_managed(w))
        {
            self.focused_monitor = mi;
            let ws = self.monitors[mi].active_ws();
            if let Some(front) = self
                .focused_window
                .filter(|&w| waiting.contains(&w) && !self.is_dragged(w))
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
