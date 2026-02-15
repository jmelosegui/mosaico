use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, CellAlignment, ContentArrangement, Table};

use mosaico_core::Window;
use mosaico_windows::monitor;

pub fn execute() {
    let windows = mosaico_windows::enumerate_windows().expect("failed to enumerate windows");
    let monitors = monitor::enumerate_monitors().unwrap_or_default();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("HWND"),
            Cell::new("Monitor").set_alignment(CellAlignment::Right),
            Cell::new("Title"),
            Cell::new("Class"),
            Cell::new("X").set_alignment(CellAlignment::Right),
            Cell::new("Y").set_alignment(CellAlignment::Right),
            Cell::new("Width").set_alignment(CellAlignment::Right),
            Cell::new("Height").set_alignment(CellAlignment::Right),
        ]);

    let mut count = 0;
    for window in &windows {
        let title = window.title().unwrap_or_default();
        if title.is_empty() {
            continue;
        }

        let class = window.class().unwrap_or_default();
        let rect = window.rect().unwrap_or(mosaico_core::Rect::new(0, 0, 0, 0));
        let hwnd = window.hwnd().0 as usize;
        let monitor_num = monitor_index_for(&monitors, hwnd);

        table.add_row(vec![
            Cell::new(format!("0x{hwnd:X}")),
            Cell::new(monitor_num).set_alignment(CellAlignment::Right),
            Cell::new(title),
            Cell::new(class),
            Cell::new(rect.x).set_alignment(CellAlignment::Right),
            Cell::new(rect.y).set_alignment(CellAlignment::Right),
            Cell::new(rect.width).set_alignment(CellAlignment::Right),
            Cell::new(rect.height).set_alignment(CellAlignment::Right),
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
