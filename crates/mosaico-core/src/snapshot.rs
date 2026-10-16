//! The daemon's view of the desktop, which `mosaico debug state` prints.
//!
//! The tiler builds it and the `Inspect` command carries it, so
//! [`crate::ipc`] re-exports these types as its payload.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{MonitorId, MonitorKey, Rect, WindowId};

/// Every monitor, workspace and managed window, and what has focus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesktopSnapshot {
    /// Every monitor, in the platform's order.
    pub monitors: Vec<MonitorSnapshot>,
    /// Monitors that disconnected and are remembered so their windows
    /// can go back, in the order of their printed keys.
    pub disconnected: Vec<MonitorKey>,
    /// Windows moved off a disconnected monitor that go back when it
    /// returns.
    pub displaced_windows: usize,
    /// Whether the daemon runs with administrator rights, which it needs
    /// to move the windows of programs that run with them.
    pub daemon_elevated: bool,
    /// Index into `monitors` of the monitor the user is working on.
    pub focused_monitor: usize,
    /// The window that has focus, as far as the daemon is concerned.
    pub focused_window: Option<WindowId>,
}

/// One monitor of a [`DesktopSnapshot`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitorSnapshot {
    /// Session id of the monitor.
    pub id: MonitorId,
    /// Identity that survives reconnects.
    pub key: MonitorKey,
    /// Area windows are tiled into, after the bar is reserved.
    pub work_area: Rect,
    /// Index of the workspace on screen.
    pub active_workspace: usize,
    /// Every workspace on the monitor, in order.
    pub workspaces: Vec<WorkspaceSnapshot>,
}

/// One workspace of a [`MonitorSnapshot`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    /// The managed windows on the workspace, in layout order.
    pub windows: Vec<WindowSnapshot>,
}

/// One managed window, as the platform sees it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowSnapshot {
    /// The window.
    pub id: WindowId,
    /// Whether the window is shown.
    pub visible: bool,
    /// Whether the window is minimized.
    pub minimized: bool,
    /// Whether the window is out of view but keeps its taskbar entry, as
    /// the `cloak` hiding behaviour leaves it.
    pub cloaked: bool,
    /// Whether the window's program runs with administrator rights.
    pub elevated: bool,
    /// Where the window is, when that could be read.
    pub rect: Option<Rect>,
    /// Window class or the platform's nearest equivalent.
    pub class: String,
    /// Window title.
    pub title: String,
}

/// The `debug state` text: each monitor with its workspaces that have
/// windows, then the disconnected monitors and what has focus.
impl fmt::Display for DesktopSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (mi, mon) in self.monitors.iter().enumerate() {
            writeln!(
                f,
                "Monitor {mi} (id={}, key={}, work_area={}, active_ws={}):",
                mon.id,
                mon.key,
                geometry(&mon.work_area),
                mon.active_workspace + 1,
            )?;
            for (wi, ws) in mon.workspaces.iter().enumerate() {
                if ws.windows.is_empty() {
                    continue;
                }
                let count = ws.windows.len();
                let plural = if count == 1 { "" } else { "s" };
                writeln!(f, "  Workspace {} ({count} window{plural}):", wi + 1)?;
                for window in &ws.windows {
                    writeln!(f, "    {window}")?;
                }
            }
        }

        let disconnected = if self.disconnected.is_empty() {
            "none".to_string()
        } else {
            let keys: Vec<String> = self.disconnected.iter().map(ToString::to_string).collect();
            keys.join("; ")
        };
        writeln!(
            f,
            "Disconnected monitors: {disconnected}, displaced windows: {}",
            self.displaced_windows
        )?;
        writeln!(f, "Self elevated: {}", self.daemon_elevated)?;
        writeln!(f, "Focused monitor: {}", self.focused_monitor)?;
        match self.focused_window {
            Some(window) => writeln!(f, "Focused window: {window}"),
            None => writeln!(f, "Focused window: None"),
        }
    }
}

impl fmt::Display for WindowSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rect = self.rect.as_ref().map_or_else(|| "N/A".into(), geometry);
        write!(
            f,
            "{}  visible={}  minimized={}  cloaked={}  elevated={}  rect={rect}  class={:?}  title={:?}",
            self.id,
            self.visible,
            self.minimized,
            self.cloaked,
            self.elevated,
            self.class,
            self.title,
        )
    }
}

/// An area as `debug state` prints it: `WIDTHxHEIGHT+X+Y`.
fn geometry(area: &Rect) -> String {
    format!("{}x{}+{}+{}", area.width, area.height, area.x, area.y)
}

#[cfg(test)]
#[path = "snapshot_tests.rs"]
mod tests;
