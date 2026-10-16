//! The `debug state` snapshot: the tiler's model plus what Win32 reports
//! for each managed window.

use mosaico_core::WindowId;
use mosaico_core::snapshot::{DesktopSnapshot, WindowSnapshot};
use mosaico_core::tiler::Tiler;

use crate::window::Window;

/// Every monitor, workspace and window the tiler manages.
pub(super) fn snapshot(tiler: &Tiler, self_elevated: bool) -> DesktopSnapshot {
    tiler.snapshot(self_elevated, describe_window)
}

fn describe_window(window: WindowId) -> WindowSnapshot {
    let win = Window::from_id(window);
    WindowSnapshot {
        id: window,
        visible: win.is_visible(),
        minimized: win.is_minimized(),
        cloaked: win.is_cloaked(),
        elevated: win.is_elevated(),
        rect: win.rect().ok(),
        class: win.class().unwrap_or_default(),
        title: win.title().unwrap_or_default(),
    }
}
