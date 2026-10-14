//! Finding the monitor and workspace a window is on.

use crate::WindowId;

use super::{Plan, Tiler};

impl Tiler {
    pub(super) fn monitor_index_for(&self, plan: &Plan, window: WindowId) -> Option<usize> {
        let id = plan.monitor_of(window);
        self.monitors.iter().position(|m| m.id == id)
    }

    pub(super) fn is_managed_on_active_workspace(&self, window: WindowId) -> bool {
        self.monitors
            .get(self.focused_monitor)
            .is_some_and(|mon| mon.active_ws().contains(window))
    }

    pub(super) fn owning_monitor(&self, window: WindowId) -> Option<usize> {
        self.find_window(window).map(|(mi, _)| mi)
    }

    /// Finds the `(monitor, workspace)` indices holding a window.
    pub(super) fn find_window(&self, window: WindowId) -> Option<(usize, usize)> {
        self.monitors.iter().enumerate().find_map(|(mi, mon)| {
            mon.workspaces
                .iter()
                .position(|ws| ws.contains(window))
                .map(|wi| (mi, wi))
        })
    }

    /// Whether the window is on the active workspace of its monitor.
    pub(super) fn is_on_active_workspace(&self, window: WindowId) -> bool {
        self.find_window(window)
            .is_some_and(|(mi, wi)| self.monitors[mi].active_workspace == wi)
    }
}
