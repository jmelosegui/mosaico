use crate::{LayoutKind, MonitorId, MonitorKey, Rect, StableMonitorId, WindowId, Workspace};

use super::Display;

/// Per-monitor state: its workspaces and the area they tile into.
pub struct MonitorState {
    /// Session id. Replaced on every display change.
    pub(super) id: MonitorId,
    /// Identity that survives reconnects, used to match monitors across
    /// display changes.
    pub(super) key: MonitorKey,
    /// The id programs outside mosaico name the monitor by, kept through
    /// display changes and parked with the workspaces.
    pub(super) stable_id: StableMonitorId,
    /// The whole monitor, as the platform reports it.
    pub(super) bounds: Rect,
    /// What the system calls the display, and the name of its output.
    pub(super) name: String,
    pub(super) device: String,
    /// Scale factor, 1.0 at 100%.
    pub(super) scale: f64,
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
    /// State for a monitor seen for the first time, or for the first time
    /// since it was parked, with `stable_id`: empty workspaces with the
    /// configured layouts.
    pub(super) fn new(
        display: &Display,
        layouts: &[LayoutKind],
        stable_id: StableMonitorId,
    ) -> Self {
        Self {
            id: display.id,
            key: display.key.clone(),
            stable_id,
            bounds: display.bounds,
            name: display.name.clone(),
            device: display.device.clone(),
            scale: display.scale,
            os_work_area: display.work_area,
            work_area: display.work_area,
            workspaces: layouts.iter().map(|&k| Workspace::with_layout(k)).collect(),
            active_workspace: 0,
        }
    }

    /// Takes what the platform reports of the monitor now, its session
    /// id, bounds, names, scale and work area, keeping its tiling area
    /// until the bar is reserved again.
    pub(super) fn sync(&mut self, display: &Display) {
        self.id = display.id;
        self.bounds = display.bounds;
        self.name.clone_from(&display.name);
        self.device.clone_from(&display.device);
        self.scale = display.scale;
        self.os_work_area = display.work_area;
    }

    pub(super) fn active_ws(&self) -> &Workspace {
        &self.workspaces[self.active_workspace]
    }

    pub(super) fn active_ws_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.active_workspace]
    }

    /// Puts workspace `idx` on screen and returns the windows of the
    /// workspace left and of the one now on screen, for the caller to
    /// hide and show. Returns none, and changes nothing, when `idx` is on
    /// screen already or is not a workspace of this monitor.
    pub(super) fn switch_to(&mut self, idx: usize) -> Vec<WindowId> {
        if idx == self.active_workspace || idx >= self.workspaces.len() {
            return Vec::new();
        }
        let left = std::mem::replace(&mut self.active_workspace, idx);
        let mut windows = self.workspaces[left].shown();
        windows.extend(self.workspaces[idx].shown());
        windows
    }

    /// Session id of the monitor.
    pub fn id(&self) -> MonitorId {
        self.id
    }

    /// Identity that survives reconnects.
    pub fn key(&self) -> &MonitorKey {
        &self.key
    }

    /// The id programs outside mosaico name the monitor by.
    pub fn stable_id(&self) -> &StableMonitorId {
        &self.stable_id
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
    Neighbor(WindowId),
    /// The adjacent monitor when no same-monitor neighbor exists.
    AdjacentMonitor(usize),
}
