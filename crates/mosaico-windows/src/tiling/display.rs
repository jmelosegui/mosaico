//! Monitor display change handling for the tiling manager.

use mosaico_core::config::{HidingBehaviour, WorkspaceMode};

use crate::monitor::MonitorInfo;

use super::TilingManager;
use super::display_rebuild::rebuild_monitors;

impl TilingManager {
    /// Rebuilds internal monitor state after a display configuration change.
    ///
    /// Monitors that are still connected keep their workspaces. They are
    /// recognised by [`mosaico_core::MonitorKey`], because Windows hands out new
    /// HMONITOR values on every change, including for monitors that never
    /// went away. Windows on removed monitors move to the same workspace
    /// number on the primary monitor, and go back when the monitor
    /// reconnects (see [`super::reconnect`]).
    pub fn handle_display_change(
        &mut self,
        new_monitors: Vec<MonitorInfo>,
        bar_height: i32,
        bar_monitor_indices: &[usize],
    ) {
        if new_monitors.is_empty() {
            return;
        }

        let old_count = self.monitors.len();
        let new_count = new_monitors.len();

        // Check if monitors actually changed (debounce rapid WM_DISPLAYCHANGE).
        if old_count == new_count {
            let ids_match = new_monitors
                .iter()
                .zip(self.monitors.iter())
                .all(|(new, old)| new.id == old.id && new.work_area == old.os_work_area);
            if ids_match {
                return;
            }
        }

        mosaico_core::log_info!("Display change: {} -> {} monitors", old_count, new_count);

        let focused_key = self
            .monitors
            .get(self.focused_monitor)
            .map(|m| m.key.clone());
        let old = std::mem::take(&mut self.monitors);
        let rebuilt = rebuild_monitors(
            old,
            &new_monitors,
            &self.workspace_layouts,
            &mut self.reconnect,
        );
        self.monitors = rebuilt.monitors;

        // Monitor indices were just rebuilt, so any parked focus claim
        // refers to a monitor that may no longer be the same one.
        self.clear_focus_claim();

        // Stay on the same physical monitor if it is still connected,
        // otherwise follow the windows to the primary monitor.
        let primary = new_monitors.iter().position(|m| m.primary).unwrap_or(0);
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

        // In global mode, every monitor must share the same active
        // workspace. New monitors default to workspace 0; align them to
        // the focused monitor's workspace so the lockstep invariant holds.
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
        let (shown, hidden): (Vec<usize>, Vec<usize>) = rebuilt
            .moved
            .into_iter()
            .partition(|&hwnd| self.is_on_active_workspace(hwnd));
        for &hwnd in &hidden {
            if self.hiding != HidingBehaviour::Cloak {
                self.hidden_by_switch.insert(hwnd);
            }
            self.hide_window(hwnd);
        }

        // Re-apply bar offsets and retile.
        self.adjust_work_areas_for_bar(bar_height, bar_monitor_indices);

        for &hwnd in &shown {
            if self.hiding != HidingBehaviour::Cloak {
                self.hidden_by_switch.remove(&hwnd);
            }
            self.show_window(hwnd);
        }

        // Follow the focused window if it moved to another monitor and is
        // still visible there; otherwise it may now sit on a hidden
        // workspace and focus falls back to the focused monitor.
        if let Some((mi, _)) = self
            .focused_window
            .filter(|&h| self.is_on_active_workspace(h))
            .and_then(|h| self.find_window(h))
        {
            self.focused_monitor = mi;
        } else if self.focused_window.is_some_and(|h| hidden.contains(&h)) {
            let ws = self.monitors[self.focused_monitor].active_ws();
            self.focused_window = ws
                .last_focused()
                .filter(|&h| ws.contains(h))
                .or_else(|| ws.handles().first().copied());
        }
        self.update_border();
    }

    /// Whether the window is on the active workspace of its monitor.
    fn is_on_active_workspace(&self, hwnd: usize) -> bool {
        self.find_window(hwnd)
            .is_some_and(|(mi, wi)| self.monitors[mi].active_workspace == wi)
    }

    /// Returns the current monitor IDs and work areas for debounce comparison.
    pub fn monitor_ids(&self) -> Vec<usize> {
        self.monitors.iter().map(|m| m.id).collect()
    }
}
