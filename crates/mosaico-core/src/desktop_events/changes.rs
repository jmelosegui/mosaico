//! Deciding what changed between two views of the desktop.

use std::collections::{HashMap, HashSet};

use crate::snapshot::{DesktopSnapshot, MonitorSnapshot, WindowSnapshot};
use crate::{MonitorId, WindowId};

use super::DesktopEvent;

/// Where a managed window is, and what a subscriber is told about it.
struct Place<'a> {
    monitor: MonitorId,
    workspace: usize,
    floating: bool,
    window: &'a WindowSnapshot,
}

/// Every managed window of `desktop`, in the order the snapshot lists
/// them: by monitor, workspace, then tiled before floating.
fn places(desktop: &DesktopSnapshot) -> Vec<Place<'_>> {
    let mut out = Vec::new();
    for mon in &desktop.monitors {
        for (index, ws) in mon.workspaces.iter().enumerate() {
            let tiled = ws.windows.iter().map(|w| (w, false));
            let floating = ws.floating.iter().map(|w| (w, true));
            for (window, floating) in tiled.chain(floating) {
                out.push(Place {
                    monitor: mon.id,
                    workspace: index,
                    floating,
                    window,
                });
            }
        }
    }
    out
}

fn focused_monitor(desktop: &DesktopSnapshot) -> Option<MonitorId> {
    desktop.monitors.get(desktop.focused_monitor).map(|m| m.id)
}

/// What changed from `before` to `after`, in the order a subscriber
/// should hear it: monitors added, windows managed, moved, floated and
/// renamed, windows unmanaged, monitors removed, then the workspaces and
/// layouts each monitor shows, and focus last.
///
/// Workspaces and layouts are only compared on monitors both views have,
/// so a monitor that appears reports itself, not each of its workspaces.
pub fn changes(before: &DesktopSnapshot, after: &DesktopSnapshot) -> Vec<DesktopEvent> {
    let mut out = Vec::new();
    let old_monitors: HashMap<MonitorId, &MonitorSnapshot> =
        before.monitors.iter().map(|m| (m.id, m)).collect();
    let new_ids: Vec<MonitorId> = after.monitors.iter().map(|m| m.id).collect();

    for mon in &after.monitors {
        if !old_monitors.contains_key(&mon.id) {
            out.push(DesktopEvent::MonitorAdded { monitor: mon.id });
        }
    }

    windows(before, after, &mut out);

    for mon in &before.monitors {
        if !new_ids.contains(&mon.id) {
            out.push(DesktopEvent::MonitorRemoved { monitor: mon.id });
        }
    }

    for mon in &after.monitors {
        let Some(old) = old_monitors.get(&mon.id) else {
            continue;
        };
        if old.active_workspace != mon.active_workspace {
            out.push(DesktopEvent::WorkspaceChanged {
                monitor: mon.id,
                workspace: mon.active_workspace,
            });
        }
        for (index, ws) in mon.workspaces.iter().enumerate() {
            let same = old
                .workspaces
                .get(index)
                .is_some_and(|o| o.layout == ws.layout && o.monocle == ws.monocle);
            if !same {
                out.push(DesktopEvent::LayoutChanged {
                    monitor: mon.id,
                    workspace: index,
                    layout: ws.layout,
                    monocle: ws.monocle,
                });
            }
        }
    }

    let monitor = focused_monitor(after);
    let moved = monitor != focused_monitor(before) || after.focused_window != before.focused_window;
    if let (true, Some(monitor)) = (moved, monitor) {
        out.push(DesktopEvent::FocusChanged {
            window: after.focused_window,
            monitor,
        });
    }
    out
}

/// The events about managed windows: managed, moved, floated, renamed,
/// then unmanaged.
fn windows(before: &DesktopSnapshot, after: &DesktopSnapshot, out: &mut Vec<DesktopEvent>) {
    let old: HashMap<WindowId, Place<'_>> = places(before)
        .into_iter()
        .map(|p| (p.window.id, p))
        .collect();
    let new = places(after);

    for place in &new {
        let id = place.window.id;
        let Some(was) = old.get(&id) else {
            out.push(DesktopEvent::WindowManaged {
                window: id,
                monitor: place.monitor,
                workspace: place.workspace,
                class: place.window.class.clone(),
                title: place.window.title.clone(),
            });
            continue;
        };
        if (was.monitor, was.workspace) != (place.monitor, place.workspace) {
            out.push(DesktopEvent::WindowMoved {
                window: id,
                monitor: place.monitor,
                workspace: place.workspace,
            });
        }
        if was.floating != place.floating {
            out.push(DesktopEvent::WindowFloatingChanged {
                window: id,
                floating: place.floating,
            });
        }
        if was.window.title != place.window.title {
            out.push(DesktopEvent::WindowTitleChanged {
                window: id,
                title: place.window.title.clone(),
            });
        }
    }

    let still: HashSet<WindowId> = new.iter().map(|p| p.window.id).collect();
    for place in places(before) {
        if !still.contains(&place.window.id) {
            out.push(DesktopEvent::WindowUnmanaged {
                window: place.window.id,
            });
        }
    }
}
