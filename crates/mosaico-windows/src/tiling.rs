use mosaico_core::window::Window as WindowTrait;
use mosaico_core::{BspLayout, Rect, WindowEvent, WindowResult, Workspace};

use crate::enumerate;
use crate::monitor;
use crate::window::Window;

/// Manages tiled windows on the primary monitor.
///
/// Holds the workspace (window list), layout algorithm, and cached
/// work area. Processes window events and re-tiles as needed.
pub struct TilingManager {
    workspace: Workspace,
    layout: BspLayout,
    work_area: Rect,
}

impl TilingManager {
    /// Creates a new tiling manager and populates it with existing windows.
    pub fn new() -> WindowResult<Self> {
        let work_area = monitor::primary_work_area()?;
        let mut manager = Self {
            workspace: Workspace::new(),
            layout: BspLayout::default(),
            work_area,
        };

        // Add all currently visible windows.
        for win in enumerate::enumerate_windows()? {
            manager.workspace.add(win.hwnd().0 as usize);
        }

        manager.apply_layout();
        Ok(manager)
    }

    /// Handles a window event and re-tiles if needed.
    pub fn handle_event(&mut self, event: &WindowEvent) {
        let changed = match event {
            WindowEvent::Created { hwnd } => {
                let window = Window::from_raw(*hwnd);
                if is_tileable(&window) {
                    self.workspace.add(*hwnd)
                } else {
                    false
                }
            }
            WindowEvent::Destroyed { hwnd } => self.workspace.remove(*hwnd),
            WindowEvent::Minimized { hwnd } => self.workspace.remove(*hwnd),
            WindowEvent::Restored { hwnd } => {
                let window = Window::from_raw(*hwnd);
                if is_tileable(&window) {
                    self.workspace.add(*hwnd)
                } else {
                    false
                }
            }
            // Focus, move, and title changes don't affect layout.
            _ => false,
        };

        if changed {
            self.apply_layout();
        }
    }

    /// Applies the current layout to all managed windows.
    fn apply_layout(&self) {
        let positions = self.workspace.compute_layout(&self.layout, &self.work_area);

        for (hwnd, rect) in &positions {
            let window = Window::from_raw(*hwnd);
            if let Err(e) = window.set_rect(rect) {
                eprintln!("Failed to position window 0x{hwnd:X}: {e}");
            }
        }
    }

    /// Returns the number of managed windows.
    pub fn window_count(&self) -> usize {
        self.workspace.len()
    }
}

/// Determines whether a window should be tiled.
///
/// Reuses the same filtering logic as `enumerate_windows` — only real
/// application windows with a caption bar get tiled.
fn is_tileable(window: &Window) -> bool {
    window.is_visible()
}
