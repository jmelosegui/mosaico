//! Monitor display change handling for the tiling manager.

use mosaico_core::Workspace;
use mosaico_core::action::MAX_WORKSPACES;

use crate::monitor::MonitorInfo;

use super::{MonitorState, TilingManager};

impl TilingManager {
    /// Adjusts monitor work areas by subtracting bar height from the top.
    ///
    /// Only monitors whose index appears in `bar_monitors` are adjusted.
    /// An empty slice means all monitors.
    pub fn adjust_work_areas_for_bar(&mut self, bar_height: i32, bar_monitors: &[usize]) {
        for (i, mon) in self.monitors.iter_mut().enumerate() {
            if bar_monitors.is_empty() || bar_monitors.contains(&i) {
                mon.work_area.y += bar_height;
                mon.work_area.height -= bar_height;
            }
        }
        self.retile_all();
    }

    /// Resets work areas to the OS values, then applies the bar offset.
    ///
    /// Used when bar config changes to avoid accumulating offsets.
    pub fn reset_and_adjust_work_areas(&mut self, bar_height: i32, bar_monitors: &[usize]) {
        if let Ok(os_monitors) = crate::monitor::enumerate_monitors() {
            for mon in &mut self.monitors {
                if let Some(os) = os_monitors.iter().find(|m| m.id == mon.id) {
                    mon.work_area = os.work_area;
                }
            }
        }
        self.adjust_work_areas_for_bar(bar_height, bar_monitors);
    }

    /// Rebuilds internal monitor state after a display configuration change.
    ///
    /// Preserves workspaces for monitors that still exist (matched by ID,
    /// with position/resolution fallback). Windows on removed monitors are
    /// migrated to the nearest remaining monitor's active workspace.
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
                .all(|(new, old)| new.id == old.id && new.work_area == old.work_area);
            if ids_match {
                return;
            }
        }

        mosaico_core::log_info!("Display change: {} -> {} monitors", old_count, new_count);

        let mut new_states: Vec<MonitorState> = Vec::with_capacity(new_count);

        for info in &new_monitors {
            // Try to find a matching old monitor by ID first, then by position.
            let old_idx = self
                .monitors
                .iter()
                .position(|m| m.id == info.id)
                .or_else(|| {
                    self.monitors.iter().position(|m| {
                        m.work_area.x == info.work_area.x && m.work_area.y == info.work_area.y
                    })
                });

            if let Some(idx) = old_idx {
                // Reuse workspaces from the matching old monitor.
                let old = &mut self.monitors[idx];
                new_states.push(MonitorState {
                    id: info.id,
                    work_area: info.work_area,
                    workspaces: std::mem::take(&mut old.workspaces),
                    active_workspace: old.active_workspace,
                    monocle: old.monocle,
                    monocle_window: old.monocle_window,
                });
            } else {
                // Brand new monitor — create fresh workspaces.
                new_states.push(MonitorState {
                    id: info.id,
                    work_area: info.work_area,
                    workspaces: (0..MAX_WORKSPACES).map(|_| Workspace::new()).collect(),
                    active_workspace: 0,
                    monocle: false,
                    monocle_window: None,
                });
            }
        }

        // Migrate windows from removed monitors (those whose workspaces
        // were not claimed by any new monitor).
        let fallback_idx = 0; // primary monitor
        for old_mon in &self.monitors {
            // If workspaces are empty, they were moved via std::mem::take.
            if old_mon.workspaces.is_empty() {
                continue;
            }
            // This old monitor was removed — migrate its windows.
            for ws in &old_mon.workspaces {
                for &hwnd in ws.handles() {
                    mosaico_core::log_info!(
                        "Migrating window 0x{:X} from removed monitor {} to monitor {}",
                        hwnd,
                        old_mon.id,
                        new_states[fallback_idx].id
                    );
                    new_states[fallback_idx].active_ws_mut().add(hwnd);
                }
            }
        }

        self.monitors = new_states;

        // Clamp focused monitor.
        if self.focused_monitor >= self.monitors.len() {
            self.focused_monitor = 0;
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
