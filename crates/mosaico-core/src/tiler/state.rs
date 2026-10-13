use crate::{LayoutKind, MonitorKey, Rect, Workspace};

use super::Display;

/// Per-monitor state: its workspaces and the area they tile into.
pub struct MonitorState {
    /// Session id. Replaced on every display change.
    pub(super) id: usize,
    /// Identity that survives reconnects, used to match monitors across
    /// display changes.
    pub(super) key: MonitorKey,
    /// Work area as reported by the OS, before the bar is reserved.
    ///
    /// Kept so the bar-adjusted `work_area` can always be re-derived
    /// from it instead of offset in place, which would accumulate.
    pub(super) os_work_area: Rect,
    /// Space available for tiling: `os_work_area` minus the bar.
    pub(super) work_area: Rect,
    pub(super) workspaces: Vec<Workspace>,
    pub(super) active_workspace: usize,
}

impl MonitorState {
    /// State for a monitor seen for the first time: empty workspaces
    /// with the configured layouts.
    pub(super) fn new(display: &Display, layouts: &[LayoutKind]) -> Self {
        Self {
            id: display.id,
            key: display.key.clone(),
            os_work_area: display.work_area,
            work_area: display.work_area,
            workspaces: layouts.iter().map(|&k| Workspace::with_layout(k)).collect(),
            active_workspace: 0,
        }
    }

    pub(super) fn active_ws(&self) -> &Workspace {
        &self.workspaces[self.active_workspace]
    }

    pub(super) fn active_ws_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.active_workspace]
    }

    /// Session id of the monitor.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Identity that survives reconnects.
    pub fn key(&self) -> &MonitorKey {
        &self.key
    }

    /// Area windows are tiled into, after the bar is reserved.
    pub fn work_area(&self) -> Rect {
        self.work_area
    }

    /// Every workspace on the monitor, in order.
    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }

    /// Index of the workspace on screen.
    pub fn active_workspace(&self) -> usize {
        self.active_workspace
    }

    /// The workspace on screen.
    pub fn active(&self) -> &Workspace {
        self.active_ws()
    }
}

/// Result of resolving a left or right action.
///
/// Shared by focus and move so they agree on neighbors and on when to
/// overflow to the next monitor.
pub(super) enum SpatialTarget {
    /// A neighbor window on the same monitor.
    Neighbor(usize),
    /// The adjacent monitor when no same-monitor neighbor exists.
    AdjacentMonitor(usize),
}
