//! Floating windows when the displays change: where they settle.

use crate::Rect;

use super::super::{Plan, Tiler, Timing};
use super::place;

impl Tiler {
    /// Settles the floating windows in view after the displays changed,
    /// each inside the work area of its monitor (see [`place::settle`]),
    /// then updates the border.
    pub(in crate::tiler) fn settle_floats(&mut self, plan: &mut Plan) {
        let areas: Vec<Rect> = self.monitors.iter().map(|m| m.work_area).collect();
        for mon in &self.monitors {
            let ws = mon.active_ws();
            if ws.monocle() {
                continue;
            }
            for &window in ws.floating() {
                let Some(rect) = plan.rect(window) else {
                    continue;
                };
                let settled = place::settle(rect, mon.work_area, &areas);
                if settled != rect {
                    plan.place(window, settled, Timing::Now, mon.id);
                }
            }
        }
        self.update_border(plan);
    }
}

#[cfg(test)]
#[path = "tests/floating_display_tests.rs"]
mod tests;
