//! Neighbor and monitor lookups for spatial navigation.

use crate::action::Direction;
use crate::spatial;

use super::{SpatialTarget, Tiler};

impl Tiler {
    /// The focused window's neighbor on its own monitor, or `None` at
    /// the monitor's edge.
    pub(super) fn find_same_monitor_neighbor(&self, dir: Direction) -> Option<usize> {
        let focused = self.focused_window?;
        let positions = self.compute_positions(self.focused_monitor);
        let rect = positions
            .iter()
            .find(|(w, _)| *w == focused)
            .map(|(_, r)| *r)?;
        spatial::find_neighbor(&positions, &rect, dir)
    }

    /// The nearest monitor in the given horizontal direction.
    pub(super) fn find_adjacent_monitor(&self, dir: Direction) -> Option<usize> {
        let positive = matches!(dir, Direction::Right | Direction::Down);
        let current_cx = self
            .monitors
            .get(self.focused_monitor)?
            .work_area
            .center_x();
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

    /// Resolves the target of a left or right action: a neighbor on the
    /// same monitor first, then the monitor in that direction. Never
    /// wraps around.
    pub(super) fn resolve_horizontal_target(&self, dir: Direction) -> Option<SpatialTarget> {
        let focused = self.focused_window?;
        let positions = self.compute_positions(self.focused_monitor);
        let rect = positions
            .iter()
            .find(|(w, _)| *w == focused)
            .map(|(_, r)| *r)?;

        crate::log_debug!(
            "resolve_horizontal dir={} mon={} focused=0x{:X} ({},{} {}x{}) windows={}",
            dir,
            self.focused_monitor,
            focused,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            positions.len()
        );

        if let Some(neighbor) = spatial::find_neighbor(&positions, &rect, dir) {
            crate::log_debug!("  -> Neighbor 0x{:X}", neighbor);
            return Some(SpatialTarget::Neighbor(neighbor));
        }
        let adjacent = self.find_adjacent_monitor(dir);
        crate::log_debug!("  -> AdjacentMonitor {:?}", adjacent);
        adjacent.map(SpatialTarget::AdjacentMonitor)
    }

    /// The window to enter monitor `monitor_idx` through, coming from
    /// direction `dir`.
    pub(super) fn find_entry_window(&self, monitor_idx: usize, dir: Direction) -> Option<usize> {
        spatial::find_entry(&self.compute_positions(monitor_idx), dir)
    }
}
