use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, CellAlignment, ContentArrangement, Table};

use mosaico_core::Window;

pub fn execute() {
    let windows = mosaico_windows::enumerate_windows().expect("failed to enumerate windows");

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("HWND"),
            Cell::new("Title"),
            Cell::new("Class"),
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

        table.add_row(vec![
            Cell::new(format!("0x{:X}", window.hwnd().0 as usize)),
            Cell::new(title),
            Cell::new(class),
            Cell::new(rect.width).set_alignment(CellAlignment::Right),
            Cell::new(rect.height).set_alignment(CellAlignment::Right),
        ]);
        count += 1;
    }

    println!("{table}");
    println!("\n{count} windows found");
}
