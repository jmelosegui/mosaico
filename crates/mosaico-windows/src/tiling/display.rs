//! Monitor display change handling for the tiling manager.

use mosaico_core::config::WorkspaceMode;

use crate::monitor::MonitorInfo;

use super::TilingManager;
use super::display_rebuild::rebuild_monitors;

impl TilingManager {
    /// Rebuilds internal monitor state after a display configuration change.
    ///
    /// Monitors that are still connected keep their workspaces. They are
    /// recognised by [`mosaico_core::MonitorKey`], because Windows hands out new
    /// HMONITOR values on every change, including for monitors that never
    /// went away. Windows on removed monitors are migrated to the primary
    /// monitor.
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

        let old = std::mem::take(&mut self.monitors);
        self.monitors = rebuild_monitors(old, &new_monitors, &self.workspace_layouts);

        // Monitor indices were just rebuilt, so any parked focus claim
        // refers to a monitor that may no longer be the same one.
        self.clear_focus_claim();

        // Clamp focused monitor.
        if self.focused_monitor >= self.monitors.len() {
            self.focused_monitor = 0;
        }

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

        // Re-apply bar offsets and retile.
        self.adjust_work_areas_for_bar(bar_height, bar_monitor_indices);
        self.update_border();
    }

    /// Returns the current monitor IDs and work areas for debounce comparison.
    pub fn monitor_ids(&self) -> Vec<usize> {
        self.monitors.iter().map(|m| m.id).collect()
    }
}
