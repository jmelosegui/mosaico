//! Layout application: tiled layouts and monocle mode.

use crate::layout::SplitRatios;
use crate::{Rect, WindowId};

use super::{Plan, Tiler, Timing};

/// The rect a monocle window fills, given a work area and a gap.
pub(super) fn monocle_rect_for(work_area: &Rect, gap: i32) -> Rect {
    Rect::new(
        work_area.x + gap,
        work_area.y + gap,
        (work_area.width - gap * 2).max(1),
        (work_area.height - gap * 2).max(1),
    )
}

impl Tiler {
    /// The rect a monocle window fills on the given monitor.
    ///
    /// Monocle ignores the layout: the monocle target covers the work
    /// area inset by the gap, so the destination is known without
    /// consulting the workspace. Empty for a monitor that does not exist.
    pub(super) fn monocle_rect(&self, monitor_idx: usize) -> Rect {
        let Some(state) = self.monitors.get(monitor_idx) else {
            return Rect::new(0, 0, 0, 0);
        };
        monocle_rect_for(&state.work_area, self.layout.gap)
    }

    /// Layout positions for the active workspace on the given monitor,
    /// each split where the user moved it.
    pub(super) fn compute_positions(&self, monitor_idx: usize) -> Vec<(WindowId, Rect)> {
        let Some(state) = self.monitors.get(monitor_idx) else {
            return Vec::new();
        };
        let ws = state.active_ws();
        self.positions_of(monitor_idx, ws.windows(), ws.split_ratios())
    }

    /// Where the layout of the active workspace on monitor `idx` puts
    /// `windows`, in order, with its splits at `splits`, and those nobody
    /// moved at the configured ratio, or an equal share down a stack.
    pub(super) fn positions_of(
        &self,
        idx: usize,
        windows: &[WindowId],
        splits: &SplitRatios,
    ) -> Vec<(WindowId, Rect)> {
        let Some(state) = self.monitors.get(idx) else {
            return Vec::new();
        };
        let kind = state.active_ws().layout_kind();
        let (gap, ratio) = (self.layout.gap, self.layout.ratio);
        splits.apply(kind, gap, ratio, windows, &state.work_area)
    }

    /// The window a workspace in monocle on screen of monitor `idx` shows
    /// in front: its monocle window, or the focused window when monocle
    /// was toggled without one, when that is on the workspace. `None` for a
    /// tiled workspace, and for one whose monocle window is away, which is
    /// tiled until it comes back.
    pub(super) fn monocle_front(&self, idx: usize) -> Option<WindowId> {
        let ws = self.monitors.get(idx)?.active_ws();
        let front = ws.monocle_window().or(self.focused_window);
        front.filter(|&w| ws.monocle() && ws.contains(w))
    }

    /// Lays out the monitor now and refreshes every border.
    pub(super) fn apply_layout_on(&mut self, plan: &mut Plan, monitor_idx: usize) {
        self.apply_layout_positions(plan, monitor_idx, Timing::Now);
        self.update_border(plan);
    }

    /// Emits a placement for every window on the monitor's active
    /// workspace, each followed by its border so both land together. A
    /// border decided again later in the step is drawn in that place.
    ///
    /// Every window gets a placement, even one already in place: those
    /// are skipped when they are applied, against the window's real
    /// position at that moment. Deciding here instead would go
    /// wrong for a window with a coalesced placement still pending. The
    /// window the user drags keeps its slot but gets nothing until they
    /// drop it, and what its drag laid out live there is forgotten, see
    /// [`forget_live`](Self::forget_live).
    pub(super) fn apply_layout_positions(
        &mut self,
        plan: &mut Plan,
        monitor_idx: usize,
        timing: Timing,
    ) {
        self.forget_live(plan, monitor_idx);
        self.prune_stale_handles(plan, monitor_idx);

        let Some(state) = self.monitors.get(monitor_idx) else {
            return;
        };
        let monitor = state.id;
        // The monocle window fills the work area.
        if let Some(front) = self.monocle_front(monitor_idx) {
            if self.is_dragged(front) {
                return;
            }
            let rect = self.monocle_rect(monitor_idx);
            plan.place(front, rect, timing, monitor);
            self.show_border_for(plan, front, rect, monitor_idx);
            self.place_waiting_behind(plan, monitor_idx, front, rect, timing);
            return;
        }
        for (window, rect) in self.compute_positions(monitor_idx) {
            if !self.is_dragged(window) {
                plan.place(window, rect, timing, monitor);
                self.show_border_for(plan, window, rect, monitor_idx);
            }
        }
    }
}
