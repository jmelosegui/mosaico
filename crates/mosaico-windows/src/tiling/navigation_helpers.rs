//! Helper methods for spatial navigation.

use mosaico_core::action::Direction;

use super::{SpatialTarget, TilingManager};

impl TilingManager {
    /// Finds the spatial neighbor for the focused window on the current
    /// monitor. Returns `None` at boundaries.
    pub(super) fn find_same_monitor_neighbor(&self, dir: Direction) -> Option<usize> {
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
    pub(super) fn resolve_horizontal_target(&self, dir: Direction) -> Option<SpatialTarget> {
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
