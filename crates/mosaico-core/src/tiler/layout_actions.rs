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

    /// Exits monocle mode on the active workspace of `mon_idx` and lays
    /// the workspace out again.
    pub(super) fn exit_monocle(&mut self, plan: &mut Plan, mon_idx: usize) {
        let ws = self.monitors[mon_idx].active_ws_mut();
        ws.set_monocle(false);
        ws.set_monocle_window(None);
        self.apply_layout_on(plan, mon_idx);
    }

    /// Lays out every monitor again.
    ///
    /// Each border keeps what the platform drew of it when it still looks
    /// the same. Only the user's retile and a display change that changes
    /// anything have every border drawn from scratch, see
    /// [`Effect::RedrawBorders`]: a work area or the bar read again, which
    /// a taskbar change reports several times, draws nothing again.
    pub(super) fn retile_all(&mut self, plan: &mut Plan) {
        for i in 0..self.monitors.len() {
            self.apply_layout_on(plan, i);
        }
    }

    /// The user's retile: the workspace on screen of every monitor forgets
    /// where the user moved its splits, in all its layouts, so each split
    /// follows `[layout] ratio` again, and every monitor is laid out again.
    /// The workspaces out of view keep theirs.
    pub(super) fn retile_from_ratio(&mut self, plan: &mut Plan) {
        for mon in &mut self.monitors {
            mon.active_ws_mut().forget_split_ratios();
        }
        self.retile_all(plan);
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

#[cfg(test)]
#[path = "tests/split_ratio_tests.rs"]
mod split_tests;

#[cfg(test)]
#[path = "tests/split_retile_tests.rs"]
mod retile_tests;
