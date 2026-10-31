//! The tiler's model as a [`DesktopSnapshot`], for `debug state`.

use crate::WindowId;
use crate::snapshot::{DesktopSnapshot, MonitorSnapshot, WindowSnapshot, WorkspaceSnapshot};

use super::Tiler;

impl Tiler {
    /// Every monitor, workspace and window the tiler manages, and what
    /// has focus.
    ///
    /// The tiler only knows its own model: `window` describes one managed
    /// window as the platform sees it, and `daemon_elevated` is passed
    /// through as the platform reports it.
    pub fn snapshot(
        &self,
        daemon_elevated: bool,
        mut window: impl FnMut(WindowId) -> WindowSnapshot,
    ) -> DesktopSnapshot {
        let mut monitors = Vec::with_capacity(self.monitors.len());
        for mon in &self.monitors {
            let workspaces = mon
                .workspaces
                .iter()
                .map(|ws| WorkspaceSnapshot {
                    windows: ws.windows().iter().map(|&w| window(w)).collect(),
                    floating: ws.floating().iter().map(|&w| window(w)).collect(),
                })
                .collect();
            monitors.push(MonitorSnapshot {
                id: mon.id,
                key: mon.key.clone(),
                work_area: mon.work_area,
                active_workspace: mon.active_workspace,
                workspaces,
            });
        }
        DesktopSnapshot {
            monitors,
            disconnected: self.reconnect.disconnected(),
            displaced_windows: self.reconnect.displaced_count(),
            daemon_elevated,
            focused_monitor: self.focused_monitor,
            focused_window: self.focused_window,
        }
    }
}

#[cfg(test)]
#[path = "tests/snapshot_tests.rs"]
mod tests;
