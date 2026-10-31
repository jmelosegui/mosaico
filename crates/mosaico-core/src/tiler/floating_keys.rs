//! The keys acting on the focused floating window: move nudges it within
//! its monitor, and resize grows or shrinks it.

use crate::WindowId;
use crate::action::Direction;

use super::super::{Plan, Tiler, Timing};
use super::place::{self, Axis};

impl Tiler {
    /// The focused window and its monitor, when it floats on the workspace
    /// on screen there, outside monocle.
    fn focused_float(&self) -> Option<(WindowId, usize)> {
        let window = self.focused_window?;
        let (mi, wi) = self.find_floating(window)?;
        let mon = &self.monitors[mi];
        (mon.active_workspace == wi && !mon.active_ws().monocle()).then_some((window, mi))
    }

    /// Moves the focused floating window one step towards `dir`, within
    /// its work area: a floating window does not leave its monitor.
    /// Returns `false`, doing nothing, when the focused window does not
    /// float.
    pub(in crate::tiler) fn nudge_float(&mut self, plan: &mut Plan, dir: Direction) -> bool {
        let Some((window, mi)) = self.focused_float() else {
            return false;
        };
        self.focused_monitor = mi;
        if let Some(rect) = plan.rect(window) {
            let moved = place::nudge(rect, self.monitors[mi].work_area, dir);
            self.place_float(plan, window, mi, moved);
        }
        true
    }

    /// Grows or shrinks the focused floating window along `axis` by one
    /// step, around its center. Does nothing on a tile.
    pub(in crate::tiler) fn resize_float(&mut self, plan: &mut Plan, axis: Axis, grow: bool) {
        let Some((window, mi)) = self.focused_float() else {
            return;
        };
        let Some(rect) = plan.rect(window) else {
            return;
        };
        let min = plan.min_size(window).unwrap_or_default();
        let resized = place::resize(rect, self.monitors[mi].work_area, axis, grow, min);
        self.place_float(plan, window, mi, resized);
    }

    /// Places floating `window` at `rect` on monitor `mi` and moves its
    /// border along.
    pub(in crate::tiler) fn place_float(
        &mut self,
        plan: &mut Plan,
        window: WindowId,
        mi: usize,
        rect: crate::Rect,
    ) {
        plan.place(window, rect, Timing::Now, self.monitors[mi].id);
        self.update_border(plan);
    }
}
