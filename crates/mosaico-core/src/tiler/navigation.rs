//! Spatial navigation: focus and move in cardinal directions.
//!
//! Handles same-monitor neighbor lookup, cross-monitor overflow, and
//! choosing where to enter a monitor when jumping between monitors.

use crate::WindowId;
use crate::action::Direction;

use super::{Plan, SpatialTarget, Tiler, Timing};

impl Tiler {
    /// Focuses a window in the given direction.
    ///
    /// Left and right go to the horizontal neighbor and overflow to the
    /// adjacent monitor. Up and down stop at the monitor's edge. With the
    /// tiles out of reach only left and right do, to the adjacent monitor.
    pub(super) fn focus_direction(&mut self, plan: &mut Plan, dir: Direction) {
        let Some(mon) = self.monitors.get(self.focused_monitor) else {
            return;
        };
        let out_of_reach = mon.active_ws().tiles_out_of_reach();
        match dir {
            Direction::Left | Direction::Right => {
                if out_of_reach {
                    // No other tile is in reach, so go straight to the
                    // adjacent monitor.
                    self.focus_adjacent_monitor(plan, dir);
                    return;
                }
                match self.resolve_horizontal_target(dir) {
                    Some(SpatialTarget::Neighbor(window)) => self.focus_window(plan, window, false),
                    Some(SpatialTarget::AdjacentMonitor(idx)) => {
                        self.focus_adjacent_monitor_idx(plan, idx, dir);
                    }
                    // Nothing focused, as on an empty workspace: go to
                    // the adjacent monitor directly.
                    None if self.focused_window.is_none() => self.focus_adjacent_monitor(plan, dir),
                    None => {}
                }
            }
            Direction::Up | Direction::Down => {
                if out_of_reach {
                    return;
                }
                if let Some(neighbor) = self.find_same_monitor_neighbor(dir) {
                    self.focus_window(plan, neighbor, false);
                }
            }
        }
    }

    fn focus_adjacent_monitor(&mut self, plan: &mut Plan, dir: Direction) {
        if let Some(idx) = self.find_adjacent_monitor(dir) {
            self.focus_adjacent_monitor_idx(plan, idx, dir);
        }
    }

    /// Focuses the window to enter monitor `idx` through, coming from
    /// direction `dir`: the monocle window, laid out again to fill the
    /// screen, else the frontmost floating window, else a tile.
    fn focus_adjacent_monitor_idx(&mut self, plan: &mut Plan, idx: usize, dir: Direction) {
        self.focused_monitor = idx;
        let ws = self.monitors[idx].active_ws();
        let monocle = ws.monocle();
        let entry = if monocle {
            ws.monocle_window()
                .or_else(|| ws.windows().first().copied())
        } else {
            let front = ws.floating().last().copied();
            front.or_else(|| self.find_entry_window(idx, dir))
        };
        if let Some(window) = entry {
            self.focus_window(plan, window, false);
            if monocle {
                self.apply_layout_on(plan, idx);
            }
        } else {
            self.focused_window = None;
            self.update_border(plan);
        }
        self.park_focus_if_empty(plan);
    }

    /// Swaps the focused window with its neighbor in the given direction.
    ///
    /// Left and right overflow to the adjacent monitor. Up and down stop
    /// at the monitor's edge. Nothing moves while the tiles are out of
    /// reach, in monocle or behind floating windows.
    pub(super) fn move_direction(&mut self, plan: &mut Plan, dir: Direction) {
        let mon = self.monitors.get(self.focused_monitor);
        let Some(window) = self.focused_window else {
            return;
        };
        if mon.is_none_or(|mon| mon.active_ws().tiles_out_of_reach()) {
            return;
        }
        let target = match dir {
            Direction::Left | Direction::Right => self.resolve_horizontal_target(dir),
            Direction::Up | Direction::Down => self
                .find_same_monitor_neighbor(dir)
                .map(SpatialTarget::Neighbor),
        };
        match target {
            Some(SpatialTarget::Neighbor(neighbor)) => {
                let ws = self.monitors[self.focused_monitor].active_ws();
                if let (Some(a), Some(b)) = (ws.index_of(window), ws.index_of(neighbor)) {
                    self.swap_and_retile(plan, a, b);
                }
            }
            Some(SpatialTarget::AdjacentMonitor(idx)) => {
                self.move_to_monitor(plan, window, idx, dir)
            }
            None => {}
        }
    }

    /// Moves the focused window to another monitor, entering on the side
    /// it came from.
    fn move_to_monitor(
        &mut self,
        plan: &mut Plan,
        window: WindowId,
        target: usize,
        dir: Direction,
    ) {
        let source = self.focused_monitor;
        self.monitors[source].active_ws_mut().remove(window);
        if dir == Direction::Right {
            self.monitors[target].active_ws_mut().insert(0, window);
        } else {
            self.monitors[target].active_ws_mut().add(window);
        }
        if self.monitors[target].active_ws().monocle() {
            let ws = self.monitors[target].active_ws_mut();
            ws.set_monocle(false);
            ws.set_monocle_window(None);
        }
        self.focused_monitor = target;
        // Both monitors are laid out before any border is drawn, so the
        // moved window's border never shows at its old slot.
        self.apply_layout_positions(plan, source, Timing::Coalesced);
        self.apply_layout_positions(plan, target, Timing::Coalesced);
        self.update_border(plan);
        self.center_cursor_on_focused(plan);
    }

    fn swap_and_retile(&mut self, plan: &mut Plan, a: usize, b: usize) {
        let idx = self.focused_monitor;
        self.monitors[idx].active_ws_mut().swap(a, b);
        self.apply_layout_positions(plan, idx, Timing::Coalesced);
        self.update_border(plan);
        self.center_cursor_on_focused(plan);
    }
}
