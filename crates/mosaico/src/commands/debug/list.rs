use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, CellAlignment, ContentArrangement, Table};

use mosaico_core::Window;
use mosaico_core::config::{load_rules, should_manage};
use mosaico_windows::{frame, monitor};

pub fn execute() {
    mosaico_windows::dpi::enable_dpi_awareness();
    let windows = mosaico_windows::enumerate_windows().expect("failed to enumerate windows");
    let monitors = monitor::enumerate_monitors().unwrap_or_default();
    let rules = load_rules();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("HWND"),
            Cell::new("Managed"),
            Cell::new("Monitor").set_alignment(CellAlignment::Right),
            Cell::new("Title"),
            Cell::new("Class"),
            Cell::new("X").set_alignment(CellAlignment::Right),
            Cell::new("Y").set_alignment(CellAlignment::Right),
            Cell::new("Width").set_alignment(CellAlignment::Right),
            Cell::new("Height").set_alignment(CellAlignment::Right),
            Cell::new("Border Offset"),
        ]);

    let mut count = 0;
    for window in &windows {
        let title = window.title().unwrap_or_default();
        let title = if title.is_empty() {
            "<untitled>".to_string()
        } else {
            title
        };

        let class = window.class().unwrap_or_default();
        let rect = window.rect().unwrap_or(mosaico_core::Rect::new(0, 0, 0, 0));
        let hwnd = window.hwnd().0 as usize;
        let monitor_num = monitor_index_for(&monitors, hwnd);
        let managed = if should_manage(&class, &title, &rules) {
            "yes"
        } else {
            "no"
        };
        let border = frame::border_offset(window.hwnd())
            .map(|b| format!("L:{} T:{} R:{} B:{}", b.left, b.top, b.right, b.bottom))
            .unwrap_or_else(|_| "?".into());

        table.add_row(vec![
            Cell::new(format!("0x{hwnd:X}")),
            Cell::new(managed),
            Cell::new(monitor_num).set_alignment(CellAlignment::Right),
            Cell::new(&title),
            Cell::new(class),
            Cell::new(rect.x).set_alignment(CellAlignment::Right),
            Cell::new(rect.y).set_alignment(CellAlignment::Right),
            Cell::new(rect.width).set_alignment(CellAlignment::Right),
            Cell::new(rect.height).set_alignment(CellAlignment::Right),
            Cell::new(border),
        ]);
        count += 1;
    }

    println!("{table}");
    println!("\n{count} windows found");
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
