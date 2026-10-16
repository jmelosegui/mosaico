//! Layout application: tiled layouts and monocle mode.

use crate::{BspLayout, LayoutKind, Rect, ThreeColumnLayout, VerticalStackLayout};

use crate::WindowId;
use crate::config::HidingBehaviour;

use super::{Effect, Plan, Tiler, Timing};

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

    /// Layout positions for the active workspace on the given monitor.
    pub(super) fn compute_positions(&self, monitor_idx: usize) -> Vec<(WindowId, Rect)> {
        let Some(state) = self.monitors.get(monitor_idx) else {
            return Vec::new();
        };
        let gap = self.layout.gap;
        let ratio = self.layout.ratio;
        let ws = state.active_ws();
        match ws.layout_kind() {
            LayoutKind::Bsp => ws.compute_layout(&BspLayout { gap, ratio }, &state.work_area),
            LayoutKind::VerticalStack => {
                ws.compute_layout(&VerticalStackLayout { gap, ratio }, &state.work_area)
            }
            LayoutKind::ThreeColumn => {
                ws.compute_layout(&ThreeColumnLayout { gap, ratio }, &state.work_area)
            }
        }
    }

    pub(super) fn cycle_layout(&mut self, plan: &mut Plan) {
        let idx = self.focused_monitor;
        let Some(mon) = self.monitors.get_mut(idx) else {
            return;
        };
        let ws = mon.active_ws_mut();
        ws.set_layout_kind(ws.layout_kind().next());
        self.apply_layout_on(plan, idx);
    }

    pub(super) fn toggle_monocle(&mut self, plan: &mut Plan) {
        let idx = self.focused_monitor;
        let focused = self.focused_window;
        let Some(mon) = self.monitors.get_mut(idx) else {
            return;
        };
        let ws = mon.active_ws_mut();
        let enabling = !ws.monocle();
        ws.set_monocle(enabling);
        ws.set_monocle_window(if enabling { focused } else { None });
        self.apply_layout_on(plan, idx);
    }

    pub(super) fn retile_all(&mut self, plan: &mut Plan) {
        for i in 0..self.monitors.len() {
            self.apply_layout_on(plan, i);
        }
    }

    /// Repaints every window on an active workspace, including the ones
    /// the layout skipped because they were already in place.
    pub(super) fn repaint_visible(&self, plan: &mut Plan) {
        for mon in &self.monitors {
            for &window in mon.active_ws().windows() {
                plan.emit(Effect::Repaint { window });
            }
        }
    }

    /// Lays out the monitor now and refreshes every border.
    pub(super) fn apply_layout_on(&mut self, plan: &mut Plan, monitor_idx: usize) {
        self.apply_layout_positions(plan, monitor_idx, Timing::Now);
        self.update_border(plan);
    }

    /// Emits a placement for every window on the monitor's active
    /// workspace, each followed by its border so both land together.
    ///
    /// Every window gets a placement, even one already in place: the
    /// platform skips those when it applies them, against the window's
    /// real position at that moment. Deciding here instead would go
    /// wrong for a window with a coalesced placement still pending.
    pub(super) fn apply_layout_positions(
        &mut self,
        plan: &mut Plan,
        monitor_idx: usize,
        timing: Timing,
    ) {
        self.prune_stale_handles(plan, monitor_idx);

        let Some(state) = self.monitors.get(monitor_idx) else {
            return;
        };
        if state.active_ws().monocle() {
            // The monocle window fills the work area. Fall back to the
            // focused window when monocle was toggled without a target.
            let target = state.active_ws().monocle_window().or(self.focused_window);
            if let Some(window) = target
                && state.active_ws().contains(window)
            {
                let rect = self.monocle_rect(monitor_idx);
                plan.emit(Effect::Place {
                    window,
                    rect,
                    timing,
                });
                self.show_border_for(plan, window, rect, monitor_idx);
                return;
            }
        }
        for (window, rect) in self.compute_positions(monitor_idx) {
            plan.emit(Effect::Place {
                window,
                rect,
                timing,
            });
            self.show_border_for(plan, window, rect, monitor_idx);
        }
    }

    /// Removes windows that are gone or minimized from the monitor's
    /// active workspace.
    ///
    /// Some applications close without reporting it, and a window can
    /// be minimized without a report, for example before it was
    /// adopted. Either would leave an empty slot in the layout.
    ///
    /// A window that is no longer shown is also forgotten as decorated,
    /// so a later window given its id does not get the system's corners
    /// back for it. A minimized one keeps the tiler's corners until it
    /// hides or closes.
    ///
    /// A window the tiler hid with "hide" is kept while it still exists
    /// and is still hidden: it is out of view by the tiler's doing, and
    /// the display change that brings it into view, by moving it or by
    /// switching its monitor's workspace, shows it after this retile
    /// (#119, #124).
    fn prune_stale_handles(&mut self, plan: &Plan, monitor_idx: usize) {
        let Some(state) = self.monitors.get_mut(monitor_idx) else {
            return;
        };
        let stale: Vec<WindowId> = state
            .active_ws()
            .windows()
            .iter()
            .copied()
            .filter(|&w| self.hidden_how.still_hidden(plan, w) != Some(HidingBehaviour::Hide))
            .filter(|&w| !plan.is_visible(w) || plan.is_minimized(w))
            .collect();
        for window in stale {
            state.active_ws_mut().remove(window);
            if !plan.is_visible(window) {
                self.decorated.remove(&window);
            }
            crate::log_info!("-prune {} (stale or minimized)", window);
        }
    }
}
