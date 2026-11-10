//! What the bar API shows of the tiler's state, and whether a window
//! under a bar takes the pointer.

use crate::api;
use crate::config::WorkspaceMode;
use crate::{Rect, WindowId, Workspace};

use super::{Desktop, MonitorState, Tiler};

#[path = "bar_filling.rs"]
mod filling;

pub(super) use filling::Filling;

impl Tiler {
    /// Every monitor as the bar API shows it, in monitor order, each
    /// marked as `maximized` says, in the same order (see
    /// [`maximized_monitors`](Self::maximized_monitors)).
    pub fn api_monitors(&self, maximized: &[bool]) -> Vec<api::Monitor> {
        (self.monitors.iter().enumerate())
            .map(|(index, mon)| api::Monitor {
                id: mon.stable_id.clone(),
                index: index as u32,
                name: mon.name.clone(),
                device: mon.device.clone(),
                scale: mon.scale,
                bounds: mon.bounds.into(),
                work_area: mon.os_work_area.into(),
                tiling_area: mon.work_area.into(),
                maximized: maximized.get(index).copied().unwrap_or(false),
                active_workspace: mon.active_workspace as u32 + 1,
                workspaces: (mon.workspaces.iter().enumerate())
                    .map(|(i, ws)| api_workspace(i, ws))
                    .collect(),
            })
            .collect()
    }

    /// Where the user works, as the bar API shows it, with `window` for
    /// the focused window when it is the one the tiler has focused.
    pub fn api_focus(&self, window: Option<api::FocusedWindow>) -> api::Focus {
        let mon = self.monitors.get(self.focused_monitor);
        api::Focus {
            monitor: mon.map(|mon| mon.stable_id.clone()),
            workspace: mon.map(|mon| mon.active_workspace as u32 + 1),
            window: window.filter(|w| Some(w.id) == self.focused_window),
        }
    }

    /// `daemon` with the workspace settings the tiler runs with:
    /// `workspace_mode` and `wrap`.
    pub fn api_daemon(&self, daemon: &api::Daemon) -> api::Daemon {
        let mode = match self.workspace_mode {
            WorkspaceMode::PerMonitor => "per-monitor",
            WorkspaceMode::Global => "global",
        };
        api::Daemon {
            workspace_mode: mode.to_string(),
            wrap: self.workspace_wrap,
            ..daemon.clone()
        }
    }

    /// Whether each monitor, in monitor order, has a window under its bar
    /// that is maximized or fullscreen, as `desktop` has it now: any
    /// window on screen there, managed or not, but not one the tiler
    /// manages on a workspace that is not on screen, however the desktop
    /// reports it.
    pub fn maximized_monitors(&self, desktop: &dyn Desktop) -> Vec<bool> {
        let mut maximized = vec![false; self.monitors.len()];
        for (_, at) in self.filling_windows(desktop) {
            maximized[at] = true;
        }
        maximized
    }

    /// The windows that [`maximized_monitors`](Self::maximized_monitors) counts,
    /// each with the index of the monitor it fills, as `desktop` has them
    /// now: every one, not only the first a monitor has.
    fn filling_windows(&self, desktop: &dyn Desktop) -> Vec<(WindowId, usize)> {
        let monitor = |window| {
            let id = desktop.monitor_of(window);
            self.monitors.iter().position(|mon| mon.id == id)
        };
        let fills_it = |&(window, at): &(WindowId, usize)| {
            let (maximized, rect) = (desktop.is_maximized(window), desktop.rect(window));
            fills(&self.monitors[at], maximized, rect)
        };
        (desktop.windows_on_screen().into_iter())
            .filter(|&window| self.shows(window))
            .filter_map(|window| Some((window, monitor(window)?)))
            .filter(fills_it)
            .collect()
    }

    /// Whether `window` is on screen as far as the tiler is concerned:
    /// one it does not manage, or one of a workspace on screen.
    fn shows(&self, window: WindowId) -> bool {
        self.find_tiled(window)
            .is_none_or(|(mon, ws)| self.monitors[mon].active_workspace == ws)
    }
}

/// Whether a window on `mon`, `maximized` or not, at `rect`, wants the
/// pointer under the monitor's bar: maximized, or over the whole of the
/// monitor's work area, as a fullscreen window is.
pub(in crate::tiler) fn fills(mon: &MonitorState, maximized: bool, rect: Option<Rect>) -> bool {
    maximized || rect.is_some_and(|rect| covers(&rect, &mon.os_work_area))
}

/// Workspace `index` of a monitor, as the bar API shows it: its tiled
/// windows in layout order, then its floating ones.
fn api_workspace(index: usize, ws: &Workspace) -> api::Workspace {
    api::Workspace {
        number: index as u32 + 1,
        name: (index + 1).to_string(),
        layout: ws.layout_kind().key().to_string(),
        monocle: ws.monocle(),
        windows: ws.windows().iter().chain(ws.floating()).copied().collect(),
        focused: ws.last_focused(),
    }
}

/// Whether `outer` holds all of `inner`.
fn covers(outer: &Rect, inner: &Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.width >= inner.x + inner.width
        && outer.y + outer.height >= inner.y + inner.height
}

#[cfg(test)]
#[path = "tests/bar_state_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/bar_maximized_tests.rs"]
mod maximized_tests;

#[cfg(test)]
#[path = "tests/bar_maximized_windows_tests.rs"]
mod maximized_windows_tests;

#[cfg(test)]
#[path = "tests/bar_filling_tests.rs"]
mod filling_tests;

#[cfg(test)]
#[path = "tests/bar_seed_tests.rs"]
mod seed_tests;
