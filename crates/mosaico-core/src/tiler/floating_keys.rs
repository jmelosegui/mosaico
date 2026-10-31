//! The keys acting on the focused floating window: focus leaves it for
//! the nearest tile, move nudges it, and resize grows or shrinks it.

use crate::action::Direction;
use crate::{WindowId, spatial};

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

    /// Focuses the nearest tile in direction `dir` from the focused
    /// floating window, overflowing left and right to the adjacent
    /// monitor as from a tile. Returns `false`, doing nothing, when the
    /// focused window does not float.
    pub(in crate::tiler) fn focus_from_float(&mut self, plan: &mut Plan, dir: Direction) -> bool {
        let Some((window, mi)) = self.focused_float() else {
            return false;
        };
        self.focused_monitor = mi;
        let positions = self.compute_positions(mi);
        let neighbor = plan
            .rect(window)
            .and_then(|rect| spatial::find_neighbor(&positions, &rect, dir));
        match neighbor {
            Some(tile) => self.focus_window(plan, tile, false),
            None if matches!(dir, Direction::Left | Direction::Right) => {
                if let Some(idx) = self.find_adjacent_monitor(dir) {
                    self.enter_monitor(plan, idx);
                }
            }
            None => {}
        }
        true
    }

    /// Moves the focused floating window one step towards `dir`, within
    /// its work area. Already at the left or right edge, it goes to the
    /// adjacent monitor instead. Returns `false`, doing nothing, when the
    /// focused window does not float.
    pub(in crate::tiler) fn nudge_float(&mut self, plan: &mut Plan, dir: Direction) -> bool {
        let Some((window, mi)) = self.focused_float() else {
            return false;
        };
        let area = self.monitors[mi].work_area;
        let Some(rect) = plan.rect(window) else {
            return true;
        };
        let moved = place::nudge(rect, area, dir);
        let sideways = matches!(dir, Direction::Left | Direction::Right);
        self.focused_monitor = mi;
        match self.find_adjacent_monitor(dir) {
            Some(target) if moved == rect && sideways => {
                self.move_float_to_monitor(plan, window, mi, target, rect);
            }
            _ => self.place_float(plan, window, mi, moved),
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

    /// Moves floating `window` from monitor `from` to the workspace on
    /// screen of monitor `to`, at the same place relative to its work
    /// area, and keeps focus on it there.
    pub(in crate::tiler) fn move_float_to_monitor(
        &mut self,
        plan: &mut Plan,
        window: WindowId,
        from: usize,
        to: usize,
        rect: crate::Rect,
    ) {
        if let Some((mi, wi)) = self.find_floating(window) {
            self.monitors[mi].workspaces[wi].remove_floating(window);
        }
        if self.monitors[to].active_ws().monocle() {
            self.exit_monocle(plan, to);
        }
        self.monitors[to].active_ws_mut().add_floating(window);
        let (src, dst) = (self.monitors[from].work_area, self.monitors[to].work_area);
        self.focused_monitor = to;
        self.place_float(plan, window, to, place::relative_move(rect, src, dst));
        self.focus_window(plan, window, false);
        crate::log_info!("float {} mon {} -> mon {}", window, from, to);
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
