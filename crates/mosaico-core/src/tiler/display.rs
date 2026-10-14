//! Monitors connecting and disconnecting.

use crate::Monitor;
use crate::config::WorkspaceMode;

use crate::WindowId;

use super::display_rebuild::rebuild_monitors;
use super::{BarReservation, Display, Plan, Tiler};

impl Tiler {
    /// Rebuilds the monitor state after the set of monitors changed.
    ///
    /// Monitors still connected keep their workspaces. They are
    /// recognized by [`crate::MonitorKey`], since platforms may hand out
    /// new session ids on every change, including for monitors that
    /// never went away. Windows on removed monitors move to the same
    /// workspace number on the primary monitor, and go back when the
    /// monitor reconnects (see [`super::reconnect`]).
    pub(super) fn handle_display_change(
        &mut self,
        plan: &mut Plan,
        displays: Vec<Display>,
        bar: &BarReservation,
    ) {
        if displays.is_empty() {
            return;
        }

        // Several change notifications can describe the same layout. The
        // bars may still have changed with it, when one could not be made
        // again or was made for the first time, so their space follows.
        let unchanged = displays.len() == self.monitors.len()
            && displays
                .iter()
                .zip(&self.monitors)
                .all(|(new, old)| new.id == old.id && new.work_area == old.os_work_area);
        if unchanged {
            self.reserve_bar_if_moved(plan, bar);
            return;
        }

        crate::log_info!(
            "Display change: {} -> {} monitors",
            self.monitors.len(),
            displays.len()
        );

        let focused_key = self
            .monitors
            .get(self.focused_monitor)
            .map(|m| m.key.clone());
        let old = std::mem::take(&mut self.monitors);
        let rebuilt =
            rebuild_monitors(old, &displays, &self.workspace_layouts, &mut self.reconnect);
        self.monitors = rebuilt.monitors;

        // Monitor indices were just rebuilt, so a parked focus claim may
        // refer to a different monitor now.
        self.empty_focus_claim = None;

        // Stay on the same physical monitor if it is still connected,
        // otherwise follow the windows to the primary monitor.
        let primary = displays.iter().position(Display::is_primary).unwrap_or(0);
        self.focused_monitor = focused_key
            .and_then(|key| {
                self.monitors
                    .iter()
                    .position(|m| m.key == key && !key.is_unknown())
            })
            .or_else(|| {
                (self.focused_monitor < self.monitors.len()).then_some(self.focused_monitor)
            })
            .unwrap_or(primary);

        // In global mode every monitor shows the same workspace number.
        // New monitors start on the first one, so line them up.
        if self.workspace_mode == WorkspaceMode::Global
            && let Some(target) = self
                .monitors
                .get(self.focused_monitor)
                .map(|m| m.active_workspace)
        {
            for mon in &mut self.monitors {
                mon.active_workspace = target;
            }
        }

        // Windows that landed on an inactive workspace are hidden before
        // the retile; the ones on an active workspace are shown after it,
        // so they appear in their final place.
        let (shown, hidden): (Vec<WindowId>, Vec<WindowId>) = rebuilt
            .moved
            .into_iter()
            .partition(|&w| self.is_on_active_workspace(w));
        let how = self.layout.hiding;
        for &window in &hidden {
            self.hidden_how.hide(plan, window, how);
        }

        self.reserve_bar(plan, bar);

        for &window in &shown {
            self.hidden_how.show(plan, window, how);
        }
        self.repaint_visible(plan);

        // Follow the focused window if it is still on screen on another
        // monitor; if it ended up on a hidden workspace, focus falls back
        // to the focused monitor.
        if let Some((mi, _)) = self
            .focused_window
            .filter(|&w| self.is_on_active_workspace(w))
            .and_then(|w| self.find_window(w))
        {
            self.focused_monitor = mi;
        } else if self.focused_window.is_some_and(|w| hidden.contains(&w)) {
            let ws = self.monitors[self.focused_monitor].active_ws();
            self.focused_window = ws
                .last_focused()
                .filter(|&w| ws.contains(w))
                .or_else(|| ws.windows().first().copied());
        }
        self.update_border(plan);
    }
}

#[cfg(test)]
#[path = "tests/display_bar_tests.rs"]
mod tests;
