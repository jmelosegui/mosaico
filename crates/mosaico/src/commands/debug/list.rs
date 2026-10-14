use mosaico_core::config::{WindowRule, load_merged_rules, matching_rule, should_manage};
use mosaico_core::tiler::{Display, WindowDetails};
use mosaico_core::{MonitorId, Rect};

use crate::commands::context::{Context, Exit, complain, say};

/// Lists the windows and monitors on the desktop, and whether the
/// window rules would tile each window.
pub fn execute(cx: &mut Context) -> Exit {
    list(cx, &load_merged_rules())
}

/// [`execute`], judging windows against `rules`.
fn list(cx: &mut Context, rules: &[WindowRule]) -> Exit {
    let windows = match cx.desktop.windows() {
        Ok(windows) => windows,
        Err(e) => {
            complain!(cx, "Error: {e}");
            return Exit::Failure;
        }
    };
    let monitors = cx.desktop.monitors().unwrap_or_default();

    for &id in &windows {
        let window = cx.desktop.details(id);
        let facts = &window.facts;
        let managed = if facts.has_caption {
            should_manage(&facts.class, &facts.title, &facts.path, rules)
        } else {
            matching_rule(&facts.class, &facts.title, &facts.path, rules) == Some(true)
        };
        let managed = if managed { "yes" } else { "no" };
        let display_title = if facts.title.is_empty() {
            "<untitled>"
        } else {
            &facts.title
        };
        let rect = window.rect.unwrap_or(Rect::new(0, 0, 0, 0));
        let monitor_num = monitor_number(&monitors, window.monitor);
        let state = state(&window);
        let border = window
            .frame
            .map(|b| format!("L:{} T:{} R:{} B:{}", b.left, b.top, b.right, b.bottom))
            .unwrap_or_else(|| "?".into());

        say!(cx, "  {id}  {display_title}");
        say!(cx, "       Class: {}", facts.class);
        say!(cx, "       Path: {}", facts.path);
        say!(
            cx,
            "       Managed: {managed}  Monitor: {monitor_num}  State: {state}"
        );
        say!(
            cx,
            "       Rect: {}x{} at ({}, {})  Border: {border}",
            rect.width,
            rect.height,
            rect.x,
            rect.y
        );
        say!(cx, "");
    }

    say!(cx, "{} windows found", windows.len());
    say!(cx, "");

    for (i, mon) in monitors.iter().enumerate() {
        let r = mon.work_area;
        say!(
            cx,
            "  Monitor {}{}: {}x{} at ({}, {})",
            i + 1,
            if mon.primary { " (primary)" } else { "" },
            r.width,
            r.height,
            r.x,
            r.y
        );
        say!(cx, "       Key: {}", mon.key);
    }
    Exit::Success
}

/// Returns the 1-based monitor number for a monitor id, or "?" if unknown.
fn monitor_number(monitors: &[Display], id: MonitorId) -> String {
    monitors
        .iter()
        .position(|m| m.id == id)
        .map(|i| (i + 1).to_string())
        .unwrap_or_else(|| "?".into())
}

/// How the window is shown: maximized, minimized or normal.
fn state(window: &WindowDetails) -> &'static str {
    if window.maximized {
        "maximized"
    } else if window.minimized {
        "minimized"
    } else {
        "normal"
    }
}

#[cfg(test)]
#[path = "tests/list_tests.rs"]
mod tests;
