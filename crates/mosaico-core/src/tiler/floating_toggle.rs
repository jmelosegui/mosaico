//! Floating the focused window, tiling it again, and cycling focus through
//! the floating windows.

use crate::WindowId;

use super::super::removal::untile;
use super::super::{Plan, Tiler};
use super::place;

impl Tiler {
    /// Floats the focused window when it is tiled, and tiles it again when
    /// it floats. Only a window on the workspace on screen of its monitor.
    pub(in crate::tiler) fn toggle_float(&mut self, plan: &mut Plan) {
        let Some(window) = self.focused_window else {
            return;
        };
        let Some((mi, wi)) = self.find_managed(window) else {
            return;
        };
        if wi != self.monitors[mi].active_workspace {
            return;
        }
        self.focused_monitor = mi;
        if self.monitors[mi].active_ws().is_floating(window) {
            self.tile_again(plan, mi, window);
        } else {
            self.float_tile(plan, mi, window);
        }
    }

    /// Takes a tile out of the layout, which closes the gap, and floats it
    /// where it last floated, or centered at two-thirds of the work area.
    fn float_tile(&mut self, plan: &mut Plan, mi: usize, window: WindowId) {
        if self.monitors[mi].active_ws().monocle() {
            self.exit_monocle(plan, mi);
        }
        let area = self.monitors[mi].work_area;
        let ws = self.monitors[mi].active_ws_mut();
        untile(plan, ws, window);
        ws.add_floating(window);
        let rect = self
            .floats
            .remembered(window)
            .map_or_else(|| place::first(area), |rect| place::clamp_into(rect, area));
        crate::log_info!("float {} on mon {}", window, mi);
        self.apply_layout_on(plan, mi);
        self.place_float(plan, window, mi, rect);
        self.focus_window(plan, window, false);
    }

    /// Tiles a floating window again at the end of the layout, remembering
    /// where it floated for the next time.
    fn tile_again(&mut self, plan: &mut Plan, mi: usize, window: WindowId) {
        if let Some(rect) = plan.rect(window) {
            self.floats.remember(window, rect);
        }
        let ws = self.monitors[mi].active_ws_mut();
        ws.remove_floating(window);
        ws.add(window);
        crate::log_info!("tile {} on mon {}", window, mi);
        self.apply_layout_on(plan, mi);
        self.focus_window(plan, window, false);
    }

    /// Focuses the next floating window of the workspace on screen of the
    /// focused monitor: from a floating window the backmost, which comes
    /// to the front, so repeating it visits every one, and from a tile the
    /// frontmost.
    pub(in crate::tiler) fn cycle_float(&mut self, plan: &mut Plan) {
        let Some(mon) = self.monitors.get(self.focused_monitor) else {
            return;
        };
        let ws = mon.active_ws();
        if ws.monocle() {
            return;
        }
        let from_float = self.focused_window.is_some_and(|w| ws.is_floating(w));
        let floating = ws.floating();
        let target = if from_float {
            floating.first()
        } else {
            floating.last()
        };
        if let Some(&window) = target {
            self.focus_window(plan, window, false);
        }
    }
}
