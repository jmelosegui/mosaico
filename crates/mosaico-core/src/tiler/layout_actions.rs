//! The layout actions: cycling layouts, monocle and retiling.

use super::{Effect, Plan, Tiler};

impl Tiler {
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
}
