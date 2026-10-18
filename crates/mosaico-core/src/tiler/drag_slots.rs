//! Where the tiles a drop is decided against are laid out: those of each
//! monitor, the slot of a window, the layout another monitor would have
//! with it and the slot it would get there, and whether a window lies on a
//! monitor.

use crate::{Rect, WindowId};

use super::super::Tiler;
use super::target::{DropArea, contains};

impl Tiler {
    /// What a drop target is worked out from, monitor by monitor, for a
    /// window dragged from monitor `from`. A workspace in monocle counts as
    /// tiled, but on that monitor, where the window goes back.
    pub(in crate::tiler) fn drop_areas(&self, from: usize) -> Vec<DropArea> {
        let area = |idx: usize| {
            let mon = &self.monitors[idx];
            let back = idx == from && mon.active_ws().monocle();
            DropArea {
                area: mon.os_work_area,
                tiles: (!back).then(|| self.compute_positions(idx)),
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

    /// Where the windows of the workspace on screen of monitor `idx` would
    /// be laid out with `window` put before `before` there, or at the end,
    /// with a split of its own at the configured ratio, or an equal share
    /// down a stack. A workspace in monocle is laid out as if it were tiled.
    pub(in crate::tiler) fn positions_joining(
        &self,
        idx: usize,
        before: Option<WindowId>,
        window: WindowId,
    ) -> Option<Vec<(WindowId, Rect)>> {
        let ws = self.monitors.get(idx)?.active_ws();
        let mut windows = ws.windows().to_vec();
        let mut splits = ws.split_ratios().clone();
        let at = before.and_then(|w| ws.index_of(w)).unwrap_or(windows.len());
        splits.insert(at, windows.len());
        windows.insert(at, window);
        Some(self.positions_of(idx, &windows, &splits))
    }

    /// The slot `window` would get on the workspace on screen of monitor
    /// `idx`, put before `before` there or at the end, as
    /// [`positions_joining`](Self::positions_joining) lays it out.
    pub(in crate::tiler) fn slot_joining(
        &self,
        idx: usize,
        before: Option<WindowId>,
        window: WindowId,
    ) -> Option<Rect> {
        let positions = self.positions_joining(idx, before, window)?;
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
