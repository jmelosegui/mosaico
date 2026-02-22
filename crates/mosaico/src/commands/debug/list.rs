use mosaico_core::Window;
use mosaico_core::config::{load_rules, should_manage};
use mosaico_windows::{frame, monitor};

pub fn execute() {
    mosaico_windows::dpi::enable_dpi_awareness();
    let windows = mosaico_windows::enumerate_windows().expect("failed to enumerate windows");
    let monitors = monitor::enumerate_monitors().unwrap_or_default();
    let rules = load_rules();

    for window in &windows {
        let title = window.title().unwrap_or_default();
        let title = if title.is_empty() {
            "<untitled>"
        } else {
            &title
        };
        let class = window.class().unwrap_or_default();
        let rect = window.rect().unwrap_or(mosaico_core::Rect::new(0, 0, 0, 0));
        let hwnd = window.hwnd().0 as usize;
        let monitor_num = monitor_index_for(&monitors, hwnd);
        let managed = if should_manage(&class, title, &rules) {
            "yes"
        } else {
            "no"
        };
        let border = frame::border_offset(window.hwnd())
            .map(|b| format!("L:{} T:{} R:{} B:{}", b.left, b.top, b.right, b.bottom))
            .unwrap_or_else(|_| "?".into());

        println!("  0x{hwnd:X}  {title}");
        println!("       Class: {class}");
        println!("       Managed: {managed}  Monitor: {monitor_num}");
        println!(
            "       Rect: {}x{} at ({}, {})  Border: {border}",
            rect.width, rect.height, rect.x, rect.y
        );
        println!();
    }

    println!("{} windows found", windows.len());
}

/// Returns the 1-based monitor number for a window, or "?" if unknown.
fn monitor_index_for(monitors: &[monitor::MonitorInfo], hwnd: usize) -> String {
    let id = monitor::monitor_id_for_window(hwnd);
    monitors
        .iter()
        .position(|m| m.id == id)
        .map(|i| (i + 1).to_string())
        .unwrap_or_else(|| "?".into())
}
