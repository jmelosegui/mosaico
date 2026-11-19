//! The desktop and its events as scripts see them.
//!
//! The daemon counts workspaces from 0. Scripts count them from 1, as the
//! bar shows them and the `goto-workspace-N` actions name them, so every
//! workspace number is turned here, and only here.

use serde_json::{Value, json};

use mosaico_core::desktop_events::DesktopEvent;
use mosaico_core::ipc::{DesktopSnapshot, WindowSnapshot};

/// The desktop as a script's `mosaico.state()` returns it.
pub fn state(desktop: &DesktopSnapshot) -> Value {
    let mut windows = Vec::new();
    let mut monitors = Vec::new();
    for mon in &desktop.monitors {
        let mut workspaces = Vec::new();
        for (index, ws) in mon.workspaces.iter().enumerate() {
            let tiled = ws.windows.iter().map(|w| (w, false));
            let floating = ws.floating.iter().map(|w| (w, true));
            for (w, floats) in tiled.chain(floating) {
                windows.push(window(w, mon.id.raw(), index + 1, floats));
            }
            workspaces.push(json!({
                "index": index + 1,
                "layout": ws.layout,
                "monocle": ws.monocle,
                "windows": ws.windows.iter().map(|w| w.id.raw()).collect::<Vec<_>>(),
                "floating": ws.floating.iter().map(|w| w.id.raw()).collect::<Vec<_>>(),
            }));
        }
        let area = mon.work_area;
        monitors.push(json!({
            "id": mon.id.raw(),
            "workspace": mon.active_workspace + 1,
            "work_area": {
                "x": area.x, "y": area.y, "width": area.width, "height": area.height,
            },
            "workspaces": workspaces,
        }));
    }
    let focused_monitor = desktop
        .monitors
        .get(desktop.focused_monitor)
        .map(|m| m.id.raw());
    json!({
        "focused_window": desktop.focused_window.map(|w| w.raw()),
        "focused_monitor": focused_monitor,
        "monitors": monitors,
        "windows": windows,
    })
}

fn window(w: &WindowSnapshot, monitor: u64, workspace: usize, floating: bool) -> Value {
    json!({
        "id": w.id.raw(),
        "class": w.class,
        "title": w.title,
        "monitor": monitor,
        "workspace": workspace,
        "floating": floating,
        "visible": w.visible,
        "minimized": w.minimized,
    })
}

/// An event as a handler receives it: its fields, `event` naming it,
/// and workspaces counted from 1.
pub fn event(event: &DesktopEvent) -> Value {
    let mut value = serde_json::to_value(event).unwrap_or(Value::Null);
    if let Some(Value::Number(n)) = value.get("workspace")
        && let Some(index) = n.as_u64()
    {
        value["workspace"] = json!(index + 1);
    }
    value
}

/// Whether every field of `filter` equals the one of `event`.
pub fn matches(event: &Value, filter: &Value) -> bool {
    let Some(filter) = filter.as_object() else {
        return true;
    };
    filter
        .iter()
        .all(|(key, wanted)| event.get(key) == Some(wanted))
}
