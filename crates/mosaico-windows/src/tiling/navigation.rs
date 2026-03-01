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
        let monocle = self.monitors[self.focused_monitor].active_ws().monocle();
        match dir {
            Direction::Left | Direction::Right => {
                if monocle {
                    // Monocle: skip same-monitor neighbors, jump to
                    // adjacent monitor as if this were the only window.
                    self.focus_adjacent_monitor(dir);
                    return;
                }
                match self.resolve_horizontal_target(dir) {
                    Some(SpatialTarget::Neighbor(hwnd)) => {
                        self.focus_and_update_border(hwnd);
                    }
                    Some(SpatialTarget::AdjacentMonitor(idx)) => {
                        self.focus_adjacent_monitor_idx(idx, dir);
                    }
                    None if self.focused_window.is_none() => {
                        // No focused window (empty workspace) — jump to
                        // the adjacent monitor directly.
                        self.focus_adjacent_monitor(dir);
                    }
                    None => {}
                }
            }
            Direction::Up | Direction::Down => {
                // Monocle: no vertical navigation (only one "window").
                if monocle {
                    return;
                }
                if let Some(neighbor) = self.find_same_monitor_neighbor(dir) {
                    self.focus_and_update_border(neighbor);
                }
            }
        }
    }

    /// Focuses the entry window on the nearest monitor in the given direction.
    fn focus_adjacent_monitor(&mut self, dir: Direction) {
        if let Some(idx) = self.find_adjacent_monitor(dir) {
            self.focus_adjacent_monitor_idx(idx, dir);
        }
    }

    /// Focuses the entry window on a specific monitor.
    ///
    /// When the target monitor is in monocle mode, the first window in
    /// the workspace is focused and expanded to fill the screen.
    fn focus_adjacent_monitor_idx(&mut self, idx: usize, dir: Direction) {
        self.focused_monitor = idx;
        let mon = &self.monitors[idx];
        let entry = if mon.active_ws().monocle() {
            // Return to the remembered monocle window, or fall back
            // to the first window in the workspace.
            mon.active_ws()
                .monocle_window()
                .or_else(|| mon.active_ws().handles().first().copied())
        } else {
            self.find_entry_window(idx, dir)
        };
        if let Some(hwnd) = entry {
            self.focus_and_update_border(hwnd);
            if self.monitors[idx].active_ws().monocle() {
                self.apply_layout_on(idx);
            }
        } else {
            self.focused_window = None;
            self.update_border();
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
        // Monocle: window is stuck until monocle is removed.
        if self.monitors[self.focused_monitor].active_ws().monocle() {
            return;
        }
        match dir {
            Direction::Left | Direction::Right => match self.resolve_horizontal_target(dir) {
                Some(SpatialTarget::Neighbor(neighbor)) => {
                    let ws = self.monitors[self.focused_monitor].active_ws();
                    let Some(a) = ws.index_of(hwnd) else { return };
                    let Some(b) = ws.index_of(neighbor) else {
                        return;
                    };
                    self.swap_and_retile(a, b);
                }
                Some(SpatialTarget::AdjacentMonitor(target)) => {
                    self.move_to_monitor(hwnd, target, dir);
                }
                None => {}
            },
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

    /// Moves the focused window to a specific target monitor.
    fn move_to_monitor(&mut self, hwnd: usize, target: usize, dir: Direction) {
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

    /// Swaps two windows by workspace index, re-tiles, and updates the border.
    fn swap_and_retile(&mut self, a: usize, b: usize) {
        self.monitors[self.focused_monitor]
            .active_ws_mut()
            .swap(a, b);
        self.apply_layout_on(self.focused_monitor);
        self.update_border();
    }

    // helper methods live in navigation_helpers.rs
}
