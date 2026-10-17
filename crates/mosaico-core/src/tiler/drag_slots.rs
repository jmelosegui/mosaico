//! Where the tiles a drop is decided against are laid out: those of each
//! monitor, and the slot of a window, and whether a window lies on a
//! monitor.

use crate::{Rect, WindowId};

use super::super::Tiler;
use super::target::{DropArea, contains};

impl Tiler {
    /// What a drop target is worked out from, monitor by monitor.
    pub(in crate::tiler) fn drop_areas(&self) -> Vec<DropArea> {
        let area = |idx: usize| {
            let mon = &self.monitors[idx];
            let monocle = mon.active_ws().monocle();
            DropArea {
                area: mon.os_work_area,
                tiles: (!monocle).then(|| self.compute_positions(idx)),
            }
        };
        (0..self.monitors.len()).map(area).collect()
    }

    /// Where `window` is laid out on the workspace on screen of monitor
    /// `idx`: in its slot, or, in monocle, in the monocle rect when it is
    /// the window monocle shows.
    pub(in crate::tiler) fn slot_of(&self, idx: usize, window: WindowId) -> Option<Rect> {
        let ws = self.monitors.get(idx)?.active_ws();
        if ws.monocle() {
            let front = ws.monocle_window().or(self.focused_window);
            return (front == Some(window)).then(|| self.monocle_rect(idx));
        }
        let positions = self.compute_positions(idx);
        positions
            .into_iter()
            .find(|&(w, _)| w == window)
            .map(|(_, rect)| rect)
    }

    /// Whether a window at `rect` lies on monitor `idx`: its center is in
    /// the area a drop counts as on the monitor.
    pub(in crate::tiler) fn lies_on(&self, idx: usize, rect: Rect) -> bool {
        let center = (rect.center_x(), rect.center_y());
        let mon = self.monitors.get(idx);
        mon.is_some_and(|m| contains(m.os_work_area, center))
    }
}
