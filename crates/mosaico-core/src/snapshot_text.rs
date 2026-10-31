//! The `debug state` text of a [`DesktopSnapshot`].

use std::fmt;

use crate::Rect;

use super::{DesktopSnapshot, WindowSnapshot};

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
                if ws.windows.is_empty() && ws.floating.is_empty() {
                    continue;
                }
                let count = ws.windows.len();
                let plural = if count == 1 { "" } else { "s" };
                let floating = match ws.floating.len() {
                    0 => String::new(),
                    n => format!(", {n} floating"),
                };
                writeln!(
                    f,
                    "  Workspace {} ({count} window{plural}{floating}):",
                    wi + 1
                )?;
                for window in &ws.windows {
                    writeln!(f, "    {window}")?;
                }
                for window in &ws.floating {
                    writeln!(f, "    {window}  floating")?;
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
