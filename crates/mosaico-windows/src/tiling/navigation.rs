//! Spatial navigation: focus and move in cardinal directions.
//!
//! Handles same-monitor neighbor lookup, cross-monitor overflow,
//! and spatial entry-point selection when jumping between monitors.

use mosaico_core::action::Direction;

use super::{SpatialTarget, TilingManager};

impl TilingManager {
    /// Focuses a window in the given spatial direction.
    ///
    /// Left/Right: horizontal neighbor, overflows to adjacent monitor.
    /// Up/Down: vertical neighbor, stops at boundary.
    pub(super) fn focus_direction(&mut self, dir: Direction) {
        if self.monitors.is_empty() {
            return;
        }
        match dir {
            Direction::Left | Direction::Right => match self.resolve_horizontal_target(dir) {
                Some(SpatialTarget::Neighbor(hwnd)) => {
                    self.focus_and_update_border(hwnd);
                }
                Some(SpatialTarget::AdjacentMonitor(idx)) => {
                    self.focused_monitor = idx;
                    if let Some(hwnd) = self.find_entry_window(idx, dir) {
                        self.focus_and_update_border(hwnd);
                    } else {
                        // Target monitor has no windows — clear focus
                        // so the user sees the selection leave.
                        self.focused_window = None;
                        self.update_border();
                    }
                }
                None if self.focused_window.is_none() => {
                    // No focused window (empty workspace) — jump to
                    // the adjacent monitor directly.
                    if let Some(idx) = self.find_adjacent_monitor(dir) {
                        self.focused_monitor = idx;
                        if let Some(hwnd) = self.find_entry_window(idx, dir) {
                            self.focus_and_update_border(hwnd);
                        }
                    }
                }
                None => {}
            },
            Direction::Up | Direction::Down => {
                if let Some(neighbor) = self.find_same_monitor_neighbor(dir) {
                    self.focus_and_update_border(neighbor);
                }
            }
        }
    }

    /// Moves (swaps) the focused window in the given spatial direction.
    ///
    /// Left/Right: horizontal swap, overflows to adjacent monitor.
    /// Up/Down: vertical swap, stops at boundary.
    pub(super) fn move_direction(&mut self, dir: Direction) {
        if self.monitors.is_empty() {
            return;
        }
        let Some(hwnd) = self.focused_window else {
            return;
        };
        match dir {
            Direction::Left | Direction::Right => {
                match self.resolve_horizontal_target(dir) {
                    Some(SpatialTarget::Neighbor(neighbor)) => {
                        let ws = self.monitors[self.focused_monitor].active_ws();
                        let Some(a) = ws.index_of(hwnd) else { return };
                        let Some(b) = ws.index_of(neighbor) else {
                            return;
                        };
                        self.swap_and_retile(a, b);
                    }
                    Some(SpatialTarget::AdjacentMonitor(target)) => {
                        // Insert at position 0 when entering from the left
                        // so the window takes the leftmost BSP slot; append
                        // when entering from the right.
                        let source = self.focused_monitor;
                        self.monitors[source].active_ws_mut().remove(hwnd);
                        if dir == Direction::Right {
                            self.monitors[target].active_ws_mut().insert(0, hwnd);
                        } else {
                            self.monitors[target].active_ws_mut().add(hwnd);
                        }
                        self.apply_layout_on(source);
                        self.apply_layout_on(target);
                        self.focused_monitor = target;
                        self.update_border();
                    }
                    None => {}
                }
            }
            Direction::Up | Direction::Down => {
                let ws = self.monitors[self.focused_monitor].active_ws();
                let Some(idx) = ws.index_of(hwnd) else {
                    return;
                };
                if let Some(neighbor) = self.find_same_monitor_neighbor(dir)
                    && let Some(other) = self.monitors[self.focused_monitor]
                        .active_ws()
                        .index_of(neighbor)
                {
                    self.swap_and_retile(idx, other);
                }
            }
        }
    }

    /// Swaps two windows by workspace index, re-tiles, and updates the border.
    fn swap_and_retile(&mut self, a: usize, b: usize) {
        self.monitors[self.focused_monitor]
            .active_ws_mut()
            .swap(a, b);
        self.apply_layout_on(self.focused_monitor);
        self.update_border();
    }

    /// Finds the spatial neighbor for the focused window on the current
    /// monitor. Returns `None` at boundaries.
    fn find_same_monitor_neighbor(&self, dir: Direction) -> Option<usize> {
        let focused_hwnd = self.focused_window?;
        let state = &self.monitors[self.focused_monitor];
        let positions = state
            .active_ws()
            .compute_layout(&self.layout, &state.work_area);
        let focused_rect = positions
            .iter()
            .find(|(h, _)| *h == focused_hwnd)
            .map(|(_, r)| *r)?;
        mosaico_core::spatial::find_neighbor(&positions, &focused_rect, dir)
    }

    /// Finds the nearest monitor in the given horizontal direction.
    pub(super) fn find_adjacent_monitor(&self, dir: Direction) -> Option<usize> {
        let positive = matches!(dir, Direction::Right | Direction::Down);
        let current_cx = self.monitors[self.focused_monitor].work_area.center_x();
        self.monitors
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != self.focused_monitor)
            .filter(|(_, m)| {
                if positive {
                    m.work_area.center_x() > current_cx
                } else {
                    m.work_area.center_x() < current_cx
                }
            })
            .min_by_key(|(_, m)| (m.work_area.center_x() - current_cx).abs())
            .map(|(i, _)| i)
    }

    /// Resolves the spatial target for a left/right action.
    ///
    /// Looks for a neighbor on the same monitor first. If none exists,
    /// looks for a monitor physically in the requested direction (no
    /// wrapping).
    fn resolve_horizontal_target(&self, dir: Direction) -> Option<SpatialTarget> {
        let focused_hwnd = self.focused_window?;

        let state = &self.monitors[self.focused_monitor];
        let positions = state
            .active_ws()
            .compute_layout(&self.layout, &state.work_area);

        mosaico_core::log_debug!(
            "resolve_horizontal dir={} mon={} focused=0x{:X} windows={}",
            dir,
            self.focused_monitor,
            focused_hwnd,
            positions.len()
        );
        for (h, r) in &positions {
            mosaico_core::log_debug!(
                "  pos 0x{:X}: ({},{} {}x{}) center_x={}",
                h,
                r.x,
                r.y,
                r.width,
                r.height,
                r.center_x()
            );
        }

        let focused_rect = positions
            .iter()
            .find(|(h, _)| *h == focused_hwnd)
            .map(|(_, r)| *r)?;

        mosaico_core::log_debug!(
            "  focused_rect: ({},{} {}x{}) center_x={}",
            focused_rect.x,
            focused_rect.y,
            focused_rect.width,
            focused_rect.height,
            focused_rect.center_x()
        );

        if let Some(neighbor) = mosaico_core::spatial::find_neighbor(&positions, &focused_rect, dir)
        {
            mosaico_core::log_debug!("  -> Neighbor 0x{:X}", neighbor);
            return Some(SpatialTarget::Neighbor(neighbor));
        }

        // No horizontal neighbor — check if a monitor exists in the
        // requested direction (no wrapping).
        match self.find_adjacent_monitor(dir) {
            Some(idx) => {
                mosaico_core::log_debug!("  -> AdjacentMonitor {}", idx);
                Some(SpatialTarget::AdjacentMonitor(idx))
            }
            None => {
                mosaico_core::log_debug!("  -> None (no monitor in dir {})", dir);
                None
            }
        }
    }

    pub(super) fn find_entry_window(&self, monitor_idx: usize, dir: Direction) -> Option<usize> {
        let state = &self.monitors[monitor_idx];
        let positions = state
            .active_ws()
            .compute_layout(&self.layout, &state.work_area);
        mosaico_core::spatial::find_entry(&positions, dir)
    }
}
