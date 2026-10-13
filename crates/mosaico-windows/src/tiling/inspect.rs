//! The `debug state` dump: the tiler's model plus what Win32 reports
//! for each managed window.

use std::fmt::Write;

use mosaico_core::tiler::Tiler;
use mosaico_core::window::Window as _;

use crate::window::Window;

/// Formats every monitor, workspace and window the tiler manages.
pub(super) fn describe(tiler: &Tiler, self_elevated: bool) -> String {
    let mut out = String::new();

    for (mi, mon) in tiler.monitors().iter().enumerate() {
        let area = mon.work_area();
        let _ = writeln!(
            out,
            "Monitor {} (id={}, key={}, work_area={}x{}+{}+{}, active_ws={}):",
            mi,
            mon.id(),
            mon.key(),
            area.width,
            area.height,
            area.x,
            area.y,
            mon.active_workspace() + 1,
        );

        for (wi, ws) in mon.workspaces().iter().enumerate() {
            if ws.is_empty() {
                continue;
            }
            let _ = writeln!(
                out,
                "  Workspace {} ({} window{}):",
                wi + 1,
                ws.len(),
                if ws.len() == 1 { "" } else { "s" },
            );
            for &hwnd in ws.handles() {
                let _ = writeln!(out, "    {}", describe_window(hwnd));
            }
        }
    }

    let _ = writeln!(out, "{}", tiler.describe_reconnect());
    let _ = writeln!(out, "Self elevated: {}", self_elevated);
    let _ = writeln!(out, "Focused monitor: {}", tiler.focused_monitor());
    let _ = match tiler.focused_window() {
        Some(h) => writeln!(out, "Focused window: 0x{:X}", h),
        None => writeln!(out, "Focused window: None"),
    };

    out
}

fn describe_window(hwnd: usize) -> String {
    let win = Window::from_raw(hwnd);
    let rect = match win.rect() {
        Ok(r) => format!("{}x{}+{}+{}", r.width, r.height, r.x, r.y),
        Err(_) => "N/A".to_string(),
    };
    format!(
        "0x{:X}  visible={}  minimized={}  cloaked={}  elevated={}  rect={}  class={:?}  title={:?}",
        hwnd,
        win.is_visible(),
        win.is_minimized(),
        win.is_cloaked(),
        win.is_elevated(),
        rect,
        win.class().unwrap_or_default(),
        win.title().unwrap_or_default(),
    )
}
