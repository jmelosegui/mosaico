use mosaico_core::config::{load_merged_rules, matching_rule, should_manage};
use mosaico_core::{Window, WindowId};
use mosaico_windows::{frame, monitor};

pub fn execute() {
    mosaico_windows::dpi::enable_dpi_awareness();
    let windows = mosaico_windows::enumerate_windows().expect("failed to enumerate windows");
    let monitors = monitor::enumerate_monitors().unwrap_or_default();
    let rules = load_merged_rules();

    for window in &windows {
        let title = window.title().unwrap_or_default();
        let class = window.class().unwrap_or_default();
        let rect = window.rect().unwrap_or(mosaico_core::Rect::new(0, 0, 0, 0));
        let id = window.id();
        let monitor_num = monitor_index_for(&monitors, id);
        let path = window.exe_path();
        let has_caption = window.app_window_caption() == Some(true);
        let managed = if has_caption {
            should_manage(&class, &title, &path, &rules)
        } else {
            matching_rule(&class, &title, &path, &rules) == Some(true)
        };
        let managed = if managed { "yes" } else { "no" };
        let display_title = if title.is_empty() {
            "<untitled>"
        } else {
            &title
        };
        let border = frame::border_offset(window.hwnd())
            .map(|b| format!("L:{} T:{} R:{} B:{}", b.left, b.top, b.right, b.bottom))
            .unwrap_or_else(|_| "?".into());
        let state = if window.is_maximized() {
            "maximized"
        } else if window.is_minimized() {
            "minimized"
        } else {
            "normal"
        };

        println!("  {id}  {display_title}");
        println!("       Class: {class}");
        println!("       Path: {path}");
        println!("       Managed: {managed}  Monitor: {monitor_num}  State: {state}");
        println!(
            "       Rect: {}x{} at ({}, {})  Border: {border}",
            rect.width, rect.height, rect.x, rect.y
        );
        println!();
    }

    println!("{} windows found", windows.len());
    println!();

    for (i, mon) in monitors.iter().enumerate() {
        let r = mon.work_area;
        println!(
            "  Monitor {}{}: {}x{} at ({}, {})",
            i + 1,
            if mon.primary { " (primary)" } else { "" },
            r.width,
            r.height,
            r.x,
            r.y
        );
        println!("       Key: {}", mon.key);
    }
}

/// Returns the 1-based monitor number for a window, or "?" if unknown.
fn monitor_index_for(monitors: &[monitor::MonitorInfo], window: WindowId) -> String {
    let id = monitor::monitor_id_for_window(window);
    monitors
        .iter()
        .position(|m| m.id == id)
        .map(|i| (i + 1).to_string())
        .unwrap_or_else(|| "?".into())
}
